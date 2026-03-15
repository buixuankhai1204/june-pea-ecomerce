use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::client;
use crate::api::client::payment;
use crate::api::types::{
    CreateVnPayQrRequest, NewOrderItem, PaymentIntentView, PaymentStatus, PlaceOrderRequest,
    PlaceOrderResponse,
};
use crate::state::auth::AuthState;
use crate::state::cart::CartState;

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let cart = expect_context::<CartState>();
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let success_order_id = RwSignal::new(Option::<Uuid>::None);
    let order_paid = RwSignal::new(false);
    let coupon_input = RwSignal::new(String::new());
    let applied_coupon = RwSignal::new(Option::<crate::api::types::ValidateCouponResponse>::None);
    let coupon_error = RwSignal::new(String::new());
    let coupon_loading = RwSignal::new(false);
    let payment_intent = RwSignal::new(Option::<PaymentIntentView>::None);
    let payment_message = RwSignal::new(String::new());
    let payment_polling = RwSignal::new(false);

    let on_apply_coupon = move |_: web_sys::MouseEvent| {
        let code = coupon_input.get_untracked();
        if code.is_empty() {
            return;
        }

        coupon_loading.set(true);
        coupon_error.set(String::new());

        let req = crate::api::types::ValidateCouponRequest { code: code.clone() };
        wasm_bindgen_futures::spawn_local(async move {
            match client::post::<crate::api::types::ValidateCouponResponse, _>(
                "/api/v1/marketing/coupons/validate",
                &req,
            )
            .await
            {
                Ok(resp) => {
                    if resp.is_valid {
                        applied_coupon.set(Some(resp));
                    } else {
                        coupon_error.set("Mã giảm giá không hợp lệ hoặc đã hết hạn.".into());
                    }
                }
                Err(e) => {
                    coupon_error.set(e.user_message().to_string());
                }
            }
            coupon_loading.set(false);
        });
    };

    let payment_requester: Arc<dyn Fn(Uuid) + Send + Sync> = {
         let payment_intent = payment_intent;
         let payment_message = payment_message;
         let payment_polling = payment_polling;
         let order_paid = order_paid;
         let success_order_id = success_order_id;
         Arc::new(move |order_id: Uuid| {
             payment_message.set("Đang tạo mã QR của VNPay...".into());
             payment_intent.set(None);
             order_paid.set(false);
             wasm_bindgen_futures::spawn_local(async move {
                 match payment::create_vnpay_qr(CreateVnPayQrRequest { order_id }).await {
                     Ok(intent) => {
                         payment_message
                             .set("Quét mã QR bằng ứng dụng ngân hàng trước khi hết hạn.".into());
                         payment_intent.set(Some(intent.clone()));
                         spawn_payment_polling(
                             order_id,
                             payment_intent,
                             payment_message,
                             payment_polling,
                             order_paid,
                         );
                     }
                     Err(err) => {
                         payment_message.set(err.user_message().to_string());
                         payment_intent.set(None);
                     }
                 }
                 success_order_id.set(Some(order_id));
             });
         })
     };
    let payment_requester = RwSignal::new(payment_requester);

    let on_place_order = move |_: web_sys::MouseEvent| {
        let user_id = auth.current_user_id();
        let items = cart.items.get_untracked();
        if items.is_empty() {
            error.set("Giỏ hàng đang trống.".into());
            return;
        }

        let order_items: Vec<NewOrderItem> = items
            .iter()
            .map(|i| NewOrderItem {
                variant_id: i.variant_id,
                quantity: i.quantity,
                unit_price: i.unit_price,
            })
            .collect();

        let req = PlaceOrderRequest {
            customer_id: user_id,
            items: order_items,
            coupon_code: applied_coupon.get_untracked().map(|c| c.code),
        };

        loading.set(true);
        error.set(String::new());
        let cart = cart;
        let payment_requester = payment_requester;

        wasm_bindgen_futures::spawn_local(async move {
            match client::post::<PlaceOrderResponse, _>("/api/v1/ordering/orders", &req).await {
                Ok(resp) => {
                    cart.clear();
                    let handler = payment_requester.get_untracked();
                    handler(resp.order_id);
                }
                Err(e) => error.set(e.user_message().to_string()),
            }
            loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen bg-[#0F172A] text-[#F8FAFC] py-12 px-4 font-['Rubik','Nunito Sans',system-ui]">
            <div class="max-w-6xl mx-auto space-y-10">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm uppercase tracking-[0.3em] text-white/60">"June Pea"</p>
                        <h1 class="text-4xl font-semibold mt-2">"Hoàn tất thanh toán"</h1>
                    </div>
                    <div class="text-right">
                        <p class="text-white/60 text-sm">"Bước 3/3"</p>
                        <p class="text-lg font-medium">"Thanh toán VNPay"</p>
                    </div>
                </div>

                <div class="grid gap-8 lg:grid-cols-12">
                    <div class="lg:col-span-7 space-y-6">
                        <div class="bg-white/10 border border-white/15 rounded-2xl p-8 backdrop-blur-xl shadow-2xl space-y-6">
                            <div class="flex items-center justify-between">
                                <h2 class="text-xl font-semibold">"Thông tin giao hàng"</h2>
                                <span class="text-xs uppercase tracking-[0.3em] text-white/60">"Bảo mật SSL"</span>
                            </div>

                            <div class="grid md:grid-cols-2 gap-4">
                                <div class="space-y-2">
                                    <label class="text-xs uppercase tracking-[0.3em] text-white/60">"Họ và tên"</label>
                                    <input type="text" class="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:border-[#FBBF24] outline-none transition" placeholder="Nguyễn Văn A" />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs uppercase tracking-[0.3em] text-white/60">"Số điện thoại"</label>
                                    <input type="text" class="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:border-[#FBBF24] outline-none transition" placeholder="0901 234 567" />
                                </div>
                            </div>

                            <div class="space-y-2">
                                <label class="text-xs uppercase tracking-[0.3em] text-white/60">"Địa chỉ nhận hàng"</label>
                                <input type="text" class="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:border-[#FBBF24] outline-none transition" placeholder="Số nhà, tên đường, phường/xã..." />
                            </div>
                            <div class="space-y-2">
                                <label class="text-xs uppercase tracking-[0.3em] text-white/60">"Ghi chú"</label>
                                <textarea class="w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:border-[#FBBF24] outline-none transition h-24 resize-none" placeholder="Lời nhắn cho shipper..."></textarea>
                            </div>
                        </div>

                        {move || {
                            if order_paid.get() {
                                if let Some(id) = success_order_id.get() {
                                    view! {
                                        <div class="bg-emerald-500/20 border border-emerald-400/30 rounded-2xl p-8 text-emerald-100 shadow-lg">
                                            <div class="flex items-center gap-4">
                                                <div class="w-12 h-12 rounded-full bg-emerald-400/40 flex items-center justify-center">
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                                </div>
                                                <div>
                                                    <p class="text-sm uppercase tracking-[0.3em]">"Thanh toán thành công"</p>
                                                    <p class="text-xl font-semibold">{format!("Mã đơn hàng {id}")}</p>
                                                </div>
                                            </div>
                                            <a href="/dashboard" class="inline-flex items-center gap-2 mt-6 text-sm font-semibold text-[#FBBF24] underline">
                                                "Xem lịch sử đơn hàng"
                                            </a>
                                        </div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }
                            } else {
                                ().into_any()
                            }
                        }}
                    </div>

                    <div class="lg:col-span-5 space-y-6">
                        <div class="bg-white/10 border border-white/15 rounded-2xl p-8 backdrop-blur-xl shadow-2xl space-y-6">
                            <div class="flex items-center justify-between">
                                <h3 class="text-lg font-semibold">"Đơn hàng của bạn"</h3>
                                <span class="text-xs uppercase tracking-[0.3em] text-white/60">"VNPay"</span>
                            </div>

                            <div class="max-h-[280px] overflow-y-auto space-y-4 pr-2">
                                {move || {
                                    let items = cart.items.get();
                                    items.into_iter().map(|item| {
                                        view! {
                                            <div class="flex justify-between items-start gap-4 text-sm">
                                                <div>
                                                    <p class="font-semibold">{item.product_name.clone()}</p>
                                                    <p class="text-white/60 text-xs">{item.variant_name.clone()} " × " {item.quantity}</p>
                                                </div>
                                                <span class="font-semibold">{format!("{:.0} VND", item.line_total() as f64)}</span>
                                            </div>
                                        }
                                    }).collect_view()
                                }}
                            </div>

                            <div class="space-y-3 text-sm">
                                <div class="flex justify-between text-white/70">
                                    <span>"Tạm tính"</span>
                                    <span class="font-semibold">{move || format!("{:.0} VND", cart.items.get().iter().map(|i| i.line_total()).sum::<i64>() as f64)}</span>
                                </div>
                                <div class="flex justify-between text-emerald-300">
                                    <span>"Giảm giá"</span>
                                    <span>{move || {
                                        let disc = applied_coupon.get().map(|c| c.discount_amount).unwrap_or(0);
                                        format!("- {:.0} VND", disc as f64)
                                    }}</span>
                                </div>
                                <div class="flex justify-between text-white/80">
                                    <span>"Phí vận chuyển"</span>
                                    <span>"0 VND"</span>
                                </div>
                            </div>

                            <div class="flex items-center justify-between border-t border-white/10 pt-4">
                                <div>
                                    <p class="text-xs uppercase tracking-[0.3em] text-white/60">"Tổng cộng"</p>
                                    <p class="text-3xl font-semibold text-[#FBBF24]">{move || {
                                        let subtotal: i64 = cart.items.get().iter().map(|i| i.line_total()).sum();
                                        let discount = applied_coupon.get().map(|c| c.discount_amount).unwrap_or(0);
                                        let total = (subtotal - discount).max(0);
                                        format!("{:.0} VND", total as f64)
                                    }}</p>
                                </div>
                                <button
                                    class="px-6 py-3 bg-[#F97316] text-white font-semibold rounded-full shadow-lg shadow-[#F97316]/40 hover:opacity-90 transition disabled:opacity-40"
                                    disabled=move || loading.get() || cart.items.get().is_empty()
                                    on:click=on_place_order
                                >
                                    {move || if loading.get() { "Đang xử lý" } else { "Thanh toán VNPay" }}
                                </button>
                            </div>

                            {move || {
                                let err = error.get();
                                if !err.is_empty() {
                                    view! { <p class="text-sm text-red-300">{err}</p> }.into_any()
                                } else {
                                    ().into_any()
                                }
                            }}

                            <div class="space-y-2">
                                <label class="text-xs uppercase tracking-[0.3em] text-white/60">"Mã giảm giá"</label>
                                <div class="flex gap-3">
                                    <input
                                        type="text"
                                        class="flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded-xl focus:border-[#FBBF24] outline-none text-sm"
                                        placeholder="JUNEPEA10"
                                        prop:value=move || coupon_input.get()
                                        on:input=move |e| coupon_input.set(event_target_value(&e))
                                    />
                                    <button
                                        class="px-4 py-2 border border-white/20 rounded-xl text-sm font-semibold hover:border-[#F97316] transition disabled:opacity-40"
                                        disabled=move || coupon_loading.get()
                                        on:click=on_apply_coupon
                                    >
                                        {move || if coupon_loading.get() { "..." } else { "Áp dụng" }}
                                    </button>
                                </div>
                                {move || {
                                    let err = coupon_error.get();
                                    if !err.is_empty() {
                                        view! { <p class="text-xs text-red-300">{err}</p> }.into_any()
                                    } else if let Some(c) = applied_coupon.get() {
                                        view! {
                                            <div class="text-xs text-emerald-200 flex items-center justify-between">
                                                <span>{format!("Đã áp dụng: {}", c.code)}</span>
                                                <button class="underline" on:click=move |_| applied_coupon.set(None)>"Gỡ"</button>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}
                            </div>
                        </div>

                        {move || {
                            if let Some(intent) = payment_intent.get() {
                                let status_text = payment_status_text(&intent.status);
                                let show_retry = matches!(intent.status, PaymentStatus::Failed | PaymentStatus::Expired);
                                view! {
                                    <div class="bg-[#0B1220] border border-white/10 rounded-2xl p-6 space-y-4 shadow-2xl">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-xs uppercase tracking-[0.3em] text-white/50">"Trạng thái"</p>
                                                <p class="text-lg font-semibold">{status_text}</p>
                                            </div>
                                            <span class="text-xs text-white/60">{format!("Hết hạn {}", intent.expires_at.format("%H:%M"))}</span>
                                        </div>
                                        <div class="grid md:grid-cols-[256px,1fr] gap-4 items-center">
                                            <div class="rounded-2xl bg-white p-3" inner_html=move || {
                                                intent.qr_svg.clone().unwrap_or_default()
                                            }></div>
                                            <div class="space-y-3 text-sm text-white/80">
                                                <p>"Dùng ứng dụng ngân hàng hoặc VNPay để quét QR."</p>
                                                <a href={intent.payment_url.clone()} target="_blank" class="inline-flex items-center gap-2 text-[#FBBF24] font-semibold underline">
                                                    "Mở cổng thanh toán"
                                                </a>
                                                <p class="text-xs text-white/60">{payment_message.get()}</p>
                                                {move || if payment_polling.get() {
                                                    view! { <p class="text-xs text-[#FBBF24]">"Đang chờ VNPay phản hồi..."</p> }.into_any()
                                                } else {
                                                    ().into_any()
                                                 }}
                                                {move || if show_retry {
                                                    if let Some(order_id) = success_order_id.get() {
                                                        let handler = payment_requester.get_untracked();
                                                        view! {
                                                            <button class="px-4 py-2 border border-white/20 rounded-xl text-sm font-semibold hover:border-[#F97316]"
                                                                on:click=move |_| handler(order_id)
                                                            >"Tạo lại mã QR"</button>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }
                                                } else {
                                                    ().into_any()
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

fn payment_status_text(status: &PaymentStatus) -> &'static str {
    match status {
        PaymentStatus::Pending => "Đang chờ thanh toán",
        PaymentStatus::Paid => "Đã thanh toán",
        PaymentStatus::Failed => "Thanh toán thất bại",
        PaymentStatus::Expired => "QR đã hết hạn",
    }
}

fn spawn_payment_polling(
    order_id: Uuid,
    payment_intent: RwSignal<Option<PaymentIntentView>>,
    payment_message: RwSignal<String>,
    payment_polling: RwSignal<bool>,
    order_paid: RwSignal<bool>,
) {
    if order_paid.get_untracked() {
        return;
    }
    payment_polling.set(true);
    wasm_bindgen_futures::spawn_local(async move {
        for _ in 0..20 {
            match payment::get_payment_status(order_id).await {
                Ok(intent) => {
                    order_paid.set(matches!(intent.status, PaymentStatus::Paid));
                    payment_intent.set(Some(intent.clone()));
                    match intent.status {
                        PaymentStatus::Pending => {
                            payment_message.set("VNPay đang chờ xác nhận từ ngân hàng.".into());
                        }
                        PaymentStatus::Paid => {
                            payment_message.set("Thanh toán thành công! Cảm ơn bạn.".into());
                            payment_polling.set(false);
                            return;
                        }
                        PaymentStatus::Failed => {
                            payment_message.set("Thanh toán bị từ chối. Vui lòng thử lại.".into());
                            payment_polling.set(false);
                            return;
                        }
                        PaymentStatus::Expired => {
                            payment_message.set("Mã QR đã hết hạn. Tạo lại để tiếp tục.".into());
                            payment_polling.set(false);
                            return;
                        }
                    }
                }
                Err(err) => {
                    payment_message.set(err.user_message().to_string());
                }
            }
            TimeoutFuture::new(3000).await;
        }
        payment_polling.set(false);
        payment_message.set("Không nhận được phản hồi từ VNPay. Vui lòng thử lại.".into());
    });
}
