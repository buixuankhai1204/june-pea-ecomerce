use leptos::prelude::*;

use crate::state::cart::CartState;

fn format_cents(cents: i64) -> String {
    // Simple VND formatter
    let s = cents.to_string();
    let mut result = String::new();
    for (count, c) in s.chars().rev().enumerate() {
        if count != 0 && count % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }
    format!("{} VND", result.chars().rev().collect::<String>())
}

#[component]
pub fn CartPage() -> impl IntoView {
    let cart = expect_context::<CartState>();

    view! {
        <div class="bg-white min-h-screen font-[Montserrat,system-ui,sans-serif]">
            // Breadcrumb
            <div class="max-w-screen-xl mx-auto px-4 sm:px-6 pt-3 pb-1">
                <nav class="text-xs text-gray-500 flex items-center gap-1.5">
                    <a href="/" class="hover:text-black transition-colors cursor-pointer">"Trang chủ"</a>
                    <span class="text-gray-300">"›"</span>
                    <span class="text-gray-700">"Giỏ hàng"</span>
                </nav>
            </div>

            <div class="max-w-screen-xl mx-auto px-4 sm:px-6 py-6">
                <h1 class="text-3xl font-light text-black tracking-wide mb-8">"GIỎ HÀNG"</h1>

                {move || {
                    let items = cart.items.get();
                    if items.is_empty() {
                        view! {
                            <div class="text-center py-20 bg-gray-50 rounded-sm border border-gray-200">
                                <svg xmlns="http://www.w3.org/2000/svg" class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M16 11V7a4 4 0 00-8 0v4M5 9h14l1 12H4L5 9z" />
                                </svg>
                                <p class="text-gray-500 text-sm mb-6">"Giỏ hàng của bạn đang trống"</p>
                                <a href="/products" class="inline-block bg-black text-white px-8 py-3 rounded-sm hover:bg-gray-800 transition font-medium text-sm">
                                    "Tiếp tục mua sắm"
                                </a>
                            </div>
                        }.into_any()
                    } else {
                        let subtotal: i64 = items.iter().map(|i| i.line_total()).sum();
                        let delivery = 0; // Free shipping in Yame inspired
                        let discount = 0;
                        let total = subtotal + delivery - discount;

                        view! {
                            <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 xl:gap-14 items-start">
                                <div class="lg:col-span-2 space-y-4">
                                    {items.into_iter().map(|item| {
                                        let vid = item.variant_id;
                                        let qty = item.quantity;
                                        let cart_inc = cart;
                                        let cart_dec = cart;
                                        let cart_rm = cart;

                                        view! {
                                            <div class="flex flex-row items-start gap-4 p-4 bg-white rounded-sm border border-gray-200 hover:border-gray-300 transition-colors">
                                                <div class="w-20 sm:w-24 aspect-[3/4] bg-gray-100 rounded-sm flex-shrink-0 relative overflow-hidden">
                                                    <div class="absolute inset-0 flex items-center justify-center text-gray-300">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <rect x="3" y="3" width="18" height="18" rx="2"></rect>
                                                            <polyline points="21 15 16 10 5 21"></polyline>
                                                        </svg>
                                                    </div>
                                                </div>

                                                <div class="flex-1 min-w-0 flex flex-col h-full justify-between">
                                                    <div class="flex justify-between items-start gap-4">
                                                        <div>
                                                            <h3 class="font-medium text-gray-900 text-sm leading-snug line-clamp-2">{item.product_name.clone()}</h3>
                                                            <p class="text-sm text-gray-500 mt-1">{item.variant_name.clone()}</p>
                                                        </div>
                                                        <p class="font-semibold text-black text-sm whitespace-nowrap">{format_cents(item.line_total())}</p>
                                                    </div>

                                                    <div class="flex items-end justify-between w-full mt-4">
                                                        <div class="inline-flex items-center border border-gray-300 rounded-sm">
                                                            <button
                                                                class="w-8 h-8 flex items-center justify-center text-gray-600 hover:bg-gray-50 transition-colors cursor-pointer text-lg leading-none"
                                                                on:click=move |_| cart_dec.update_quantity(vid, qty - 1)
                                                            >"−"</button>
                                                            <span class="w-10 h-8 flex items-center justify-center text-sm font-medium border-x border-gray-300">{qty}</span>
                                                            <button
                                                                class="w-8 h-8 flex items-center justify-center text-gray-600 hover:bg-gray-50 transition-colors cursor-pointer text-lg leading-none"
                                                                on:click=move |_| cart_inc.update_quantity(vid, qty + 1)
                                                            >"+"</button>
                                                        </div>

                                                        <div class="flex items-center gap-3">
                                                            <button class="text-gray-400 hover:text-black transition" title="Add to wishlist">
                                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                                                                </svg>
                                                            </button>
                                                            <button class="text-gray-400 hover:text-black transition" on:click=move |_| cart_rm.remove_item(vid) title="Remove item">
                                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                    <polyline points="3 6 5 6 21 6"></polyline>
                                                                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                                                </svg>
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>

                                <div class="lg:col-span-1">
                                    <div class="bg-gray-50 rounded-sm p-6 sticky top-4 border border-gray-200">
                                        <h2 class="text-lg font-medium text-black mb-5">"Tóm tắt đơn hàng"</h2>

                                        <div class="space-y-3 text-sm text-gray-600 mb-5 pb-5 border-b border-gray-200">
                                            <div class="flex justify-between items-center">
                                                <span>"Tạm tính"</span>
                                                <span class="font-medium text-black">{format_cents(subtotal)}</span>
                                            </div>
                                            <div class="flex justify-between items-center">
                                                <span>"Phí vận chuyển"</span>
                                                <span class="font-medium text-black">"Miễn phí"</span>
                                            </div>
                                        </div>

                                        <div class="flex justify-between items-center mb-6">
                                            <span class="font-medium text-black text-base">"Tổng cộng"</span>
                                            <span class="text-xl font-semibold text-black">{format_cents(total)}</span>
                                        </div>

                                        <a
                                            href="/checkout"
                                            class="block w-full text-center py-3.5 bg-black text-white rounded-sm hover:bg-gray-900 transition-colors font-semibold text-sm tracking-wide"
                                        >
                                            "Thanh Toán"
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
