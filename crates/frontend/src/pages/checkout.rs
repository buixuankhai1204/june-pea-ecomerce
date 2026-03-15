use leptos::prelude::*;

use crate::api::client;
use crate::api::types::{NewOrderItem, PlaceOrderRequest, PlaceOrderResponse};
use crate::state::auth::AuthState;
use crate::state::cart::CartState;

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let cart = expect_context::<CartState>();
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let success_order_id = RwSignal::new(Option::<String>::None);
    let coupon_input = RwSignal::new(String::new());
    let applied_coupon = RwSignal::new(Option::<crate::api::types::ValidateCouponResponse>::None);
    let coupon_error = RwSignal::new(String::new());
    let coupon_loading = RwSignal::new(false);

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

        wasm_bindgen_futures::spawn_local(async move {
            match client::post::<PlaceOrderResponse, _>("/api/v1/ordering/orders", &req).await {
                Ok(resp) => {
                    cart.clear();
                    success_order_id.set(Some(resp.order_id.to_string()));
                }
                Err(e) => {
                    error.set(e.user_message().to_string());
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="max-w-screen-xl mx-auto px-4 py-12 font-[Montserrat,system-ui,sans-serif]">
            <h1 class="text-3xl font-light text-gray-900 mb-10 tracking-tight">"Thanh toán"</h1>

            <div class="grid grid-cols-1 lg:grid-cols-[1fr_400px] gap-12">
                // ── Left Column: Form (Simplified for now) ──
                <div class="space-y-8">
                    {move || {
                        if let Some(oid) = success_order_id.get() {
                            view! {
                                <div class="bg-emerald-50 border border-emerald-100 text-emerald-800 px-8 py-10 rounded-sm text-center animate-fadeInUp">
                                    <div class="w-16 h-16 bg-emerald-100 rounded-full flex items-center justify-center mx-auto mb-6">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-emerald-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                                            <polyline points="20 6 9 17 4 12"></polyline>
                                        </svg>
                                    </div>
                                    <h2 class="text-2xl font-bold mb-3">"Đặt hàng thành công!"</h2>
                                    <p class="text-emerald-600/80 mb-6">"Cảm ơn bạn đã lựa chọn June Pea. Đơn hàng của bạn đang được xử lý."</p>
                                    <div class="bg-white/50 p-4 rounded-sm border border-emerald-100 inline-block mb-8">
                                        <span class="text-xs uppercase tracking-widest font-bold text-emerald-600 block mb-1">"Mã đơn hàng"</span>
                                        <span class="font-mono text-lg font-bold">{oid}</span>
                                    </div>
                                    <div>
                                        <a href="/dashboard" class="px-6 py-3 bg-emerald-600 text-white text-sm font-bold tracking-wide rounded-sm hover:bg-emerald-700 transition-colors">
                                            "Xem lịch sử đơn hàng"
                                        </a>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="bg-white p-8 border border-gray-100 rounded-sm space-y-6">
                                    <h3 class="text-lg font-bold text-gray-900 uppercase tracking-wide">"Thông tin giao hàng"</h3>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="space-y-2">
                                            <label class="text-xs font-bold text-gray-400 uppercase tracking-tighter">"Họ và tên"</label>
                                            <input type="text" class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:border-black focus:bg-white transition-all outline-none rounded-sm text-sm" placeholder="Nguyễn Văn A" />
                                        </div>
                                        <div class="space-y-2">
                                            <label class="text-xs font-bold text-gray-400 uppercase tracking-tighter">"Số điện thoại"</label>
                                            <input type="text" class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:border-black focus:bg-white transition-all outline-none rounded-sm text-sm" placeholder="0901234567" />
                                        </div>
                                    </div>
                                    <div class="space-y-2">
                                        <label class="text-xs font-bold text-gray-400 uppercase tracking-tighter">"Địa chỉ nhận hàng"</label>
                                        <input type="text" class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:border-black focus:bg-white transition-all outline-none rounded-sm text-sm" placeholder="Số nhà, tên đường, phường/xã..." />
                                    </div>
                                    <div class="space-y-2">
                                        <label class="text-xs font-bold text-gray-400 uppercase tracking-tighter">"Ghi chú (Tùy chọn)"</label>
                                        <textarea class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:border-black focus:bg-white transition-all outline-none rounded-sm text-sm h-24 resize-none" placeholder="Lời nhắn cho shipper..."></textarea>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // ── Right Column: Summary ──
                <div class="space-y-6">
                    <div class="bg-gray-50 p-8 rounded-sm space-y-6">
                        <h3 class="text-lg font-bold text-gray-900 uppercase tracking-wide">"Đơn hàng của bạn"</h3>

                        <div class="max-h-[300px] overflow-y-auto pr-2 space-y-4">
                            {move || {
                                let items = cart.items.get();
                                items.into_iter().map(|item| {
                                    view! {
                                        <div class="flex justify-between items-start gap-4 text-sm">
                                            <div class="flex-1">
                                                <p class="font-bold text-gray-900 truncate">{item.product_name.clone()}</p>
                                                <p class="text-xs text-gray-400 mt-0.5">{item.variant_name.clone()} " × " {item.quantity}</p>
                                            </div>
                                            <span class="font-bold text-gray-900">{format!("{:.0} VND", item.line_total() as f64)}</span>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>

                        <hr class="border-gray-200" />

                        // Coupon Section
                        <div class="space-y-3">
                            <label class="text-[10px] font-bold text-gray-400 uppercase tracking-widest">"Mã giảm giá"</label>
                            <div class="flex gap-2">
                                <input
                                    type="text"
                                    class="flex-1 px-4 py-2 bg-white border border-gray-200 outline-none focus:border-black rounded-sm text-sm"
                                    placeholder="JUNEPEA10"
                                    prop:value=move || coupon_input.get()
                                    on:input=move |e| coupon_input.set(event_target_value(&e))
                                />
                                <button
                                    class="px-4 py-2 bg-black text-white text-xs font-bold uppercase tracking-wider hover:bg-gray-800 transition-colors disabled:opacity-50 cursor-pointer"
                                    disabled=move || coupon_loading.get()
                                    on:click=on_apply_coupon
                                >
                                    {move || if coupon_loading.get() { "..." } else { "Áp dụng" }}
                                </button>
                            </div>
                            {move || {
                                let err = coupon_error.get();
                                if !err.is_empty() {
                                    view! { <p class="text-xs text-red-500 font-medium italic">{err}</p> }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                            {move || {
                                let c = applied_coupon.get();
                                if let Some(c) = c {
                                    view! {
                                        <div class="flex items-center justify-between text-xs text-emerald-600 bg-emerald-50 px-3 py-2 rounded-sm border border-emerald-100">
                                            <div class="flex items-center gap-1.5 font-bold">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path><line x1="7" y1="7" x2="7.01" y2="7"></line></svg>
                                                {format!("Đã áp dụng: {}", c.code)}
                                            </div>
                                            <button
                                                class="text-gray-400 hover:text-red-500 cursor-pointer"
                                                on:click=move |_| applied_coupon.set(None)
                                            >
                                                "Gỡ bỏ"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }
                            }}
                        </div>

                        <div class="space-y-2 pt-2 text-sm text-gray-500">
                            <div class="flex justify-between">
                                <span>"Tạm tính"</span>
                                <span class="font-medium text-gray-900">{move || format!("{:.0} VND", cart.items.get().iter().map(|i| i.line_total()).sum::<i64>() as f64)}</span>
                            </div>
                            <div class="flex justify-between">
                                <span>"Giảm giá"</span>
                                <span class="font-medium text-emerald-600">
                                    {move || {
                                        let disc = applied_coupon.get().map(|c| c.discount_amount).unwrap_or(0);
                                        format!("- {:.0} VND", disc as f64)
                                    }}
                                </span>
                            </div>
                            <div class="flex justify-between">
                                <span>"Phí vận chuyển"</span>
                                <span class="font-medium text-gray-900">"0 VND"</span>
                            </div>
                        </div>

                        <hr class="border-gray-200" />

                        <div class="flex justify-between items-baseline font-bold">
                            <span class="text-gray-900 uppercase tracking-widest text-xs">"Tổng cộng"</span>
                            <span class="text-2xl text-black">
                                {move || {
                                    let subtotal: i64 = cart.items.get().iter().map(|i| i.line_total()).sum();
                                    let discount = applied_coupon.get().map(|c| c.discount_amount).unwrap_or(0);
                                    let total = (subtotal - discount).max(0);
                                    format!("{:.0} VND", total as f64)
                                }}
                            </span>
                        </div>

                        {move || {
                            let err = error.get();
                            if !err.is_empty() {
                                view! {
                                    <div class="p-3 bg-red-50 border border-red-100 text-red-600 text-xs font-bold tracking-tight rounded-sm">
                                        {err}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }
                        }}

                        <button
                            class="w-full py-4 bg-black text-white text-sm font-bold uppercase tracking-widest hover:bg-gray-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed rounded-sm shadow-xl shadow-black/10"
                            disabled=move || loading.get() || success_order_id.get().is_some() || cart.items.get().is_empty()
                            on:click=on_place_order
                        >
                            {move || if loading.get() { "Đang xử lý..." } else { "Hoàn tất đặt hàng" }}
                        </button>
                    </div>

                    <p class="text-[11px] text-gray-400 text-center leading-relaxed px-4">
                        "Bằng cách nhấn nút Hoàn tất đặt hàng, bạn đồng ý với các Điều khoản & Điều kiện của June Pea."
                    </p>
                </div>
            </div>
        </div>
    }
}
