use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::client;
use crate::api::types::{ApiError, ProductVariant, ProductWithVariants};
use crate::state::cart::{CartItem, CartState};
use rust_decimal::prelude::ToPrimitive;

// ─── SVG Icons ─────────────────────────────────────────────────────────────

fn icon_chevron_left() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
    }
}

fn icon_chevron_right() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
    }
}

fn icon_chevron_down() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 flex-shrink-0 transition-transform duration-300" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
    }
}

fn product_detail_skeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse space-y-8">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div class="bg-gray-200 aspect-square rounded-lg"></div>
                <div class="space-y-4">
                    <div class="h-8 bg-gray-200 rounded w-3/4"></div>
                    <div class="h-6 bg-gray-200 rounded w-1/4"></div>
                    <div class="h-32 bg-gray-200 rounded"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ProductDetailPage() -> impl IntoView {
    let params = use_params_map();
    let cart = use_context::<CartState>().expect("CartState missing");

    let selected_color = RwSignal::new(Option::<String>::None);
    let selected_size = RwSignal::new(Option::<String>::None);
    let active_thumb = RwSignal::new(0usize);
    let quantity = RwSignal::new(1i32);

    let product: LocalResource<Result<ProductWithVariants, ApiError>> =
        LocalResource::new(move || {
            let slug = params.read().get("slug").unwrap_or_default().to_string();
            async move {
                let res: Result<ProductWithVariants, ApiError> =
                    client::get(&format!("/api/v1/catalog/products/slug/{}", slug)).await;
                res
            }
        });

    view! {
        <div class="bg-white min-h-screen font-[Montserrat,system-ui,sans-serif]">

            // ── Breadcrumb ───────────────────────────────────────────────
            <div class="max-w-screen-xl mx-auto px-4 sm:px-6 pt-3 pb-1">
                <nav class="text-xs text-gray-500 flex items-center gap-1.5">
                    <a href="/" class="hover:text-black transition-colors cursor-pointer">"Trang chủ"</a>
                    <span class="text-gray-300">"›"</span>
                    <a href="/products" class="hover:text-black transition-colors cursor-pointer">"Bộ sưu tập"</a>
                    <span class="text-gray-300">"›"</span>
                    <span class="text-gray-700">"Sản phẩm"</span>
                </nav>
            </div>

            // ── Main Content ─────────────────────────────────────────────
            <div class="max-w-screen-xl mx-auto px-4 sm:px-6 py-4">
                <Suspense fallback=move || view! { {product_detail_skeleton().into_any()} }>
                    {move || {
                        let cart = cart.clone();
                        Suspend::new(async move {
                            match product.await {
                                Ok(data) => {
                                    let mut colors = std::collections::HashSet::new();
                                    let mut sizes = std::collections::HashSet::new();

                                    for v in &data.variants {
                                        if let Some(c) = v.attributes.get("color").and_then(|val: &serde_json::Value| val.as_str()) {
                                            colors.insert(c.to_string());
                                        }
                                        if let Some(s) = v.attributes.get("size").and_then(|val: &serde_json::Value| val.as_str()) {
                                            sizes.insert(s.to_string());
                                        }
                                    }

                                    let mut colors: Vec<_> = colors.into_iter().collect();
                                    colors.sort();
                                    let mut sizes: Vec<_> = sizes.into_iter().collect();
                                    sizes.sort();

                                    if selected_color.get_untracked().is_none() && !colors.is_empty() {
                                        selected_color.set(Some(colors[0].clone()));
                                    }
                                    if selected_size.get_untracked().is_none() && !sizes.is_empty() {
                                        selected_size.set(Some(sizes[0].clone()));
                                    }

                                    let matching_variant: Memo<Option<ProductVariant>> = Memo::new({
                                        let variants = data.variants.clone();
                                        move |_| {
                                            let c = selected_color.get();
                                            let s = selected_size.get();
                                            if let (Some(c), Some(s)) = (c, s) {
                                                variants.iter().find(|v| {
                                                    let vc = v.attributes.get("color").and_then(|val: &serde_json::Value| val.as_str()).unwrap_or("");
                                                    let vs = v.attributes.get("size").and_then(|val: &serde_json::Value| val.as_str()).unwrap_or("");
                                                    vc == c && vs == s
                                                }).cloned()
                                            } else {
                                                None
                                            }
                                        }
                                    });

                                    let has_sale = Memo::new(move |_| {
                                        matching_variant.get().and_then(|v| v.sale_price).is_some()
                                    });

                                    view! {
                                        // ── Two-column grid (60 / 40) ──────────────────
                                        <div class="grid grid-cols-1 lg:grid-cols-[minmax(0,1fr)_420px] gap-8 xl:gap-14 items-start">

                                            // ──── LEFT: Gallery ────────────────────────
                                            <div class="flex flex-col gap-3">

                                                // Main image
                                                <div class="relative bg-gray-50 rounded-sm overflow-hidden aspect-[3/4]">
                                                    // Red badge (show only when sale)
                                                    {move || if has_sale.get() {
                                                        view! {
                                                            <div class="absolute top-4 left-4 z-10 w-16 h-16 bg-red-500 rounded-full flex items-center justify-center text-white text-xs font-bold text-center leading-tight">
                                                                "Giá Rất Tốt"
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }}
                                                    // Placeholder image
                                                    <div class="w-full h-full flex items-center justify-center">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-24 h-24 text-gray-200" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="0.75">
                                                            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                                                            <circle cx="8.5" cy="8.5" r="1.5"></circle>
                                                            <polyline points="21 15 16 10 5 21"></polyline>
                                                        </svg>
                                                    </div>
                                                </div>

                                                // Thumbnail strip
                                                <div class="flex items-start gap-2">
                                                    // Prev arrow
                                                    <button class="mt-2 p-1 text-gray-400 hover:text-black transition-colors cursor-pointer flex-shrink-0" on:click=move |_| {
                                                        active_thumb.update(|i| { if *i > 0 { *i -= 1; } });
                                                    }>
                                                        {icon_chevron_left().into_any()}
                                                    </button>
                                                    // Thumbnails
                                                    <div class="flex gap-2 flex-wrap">
                                                        {(0..5usize).map(|i| {
                                                            view! {
                                                                <button
                                                                    class=move || format!(
                                                                        "w-[76px] h-[100px] bg-gray-100 rounded-sm overflow-hidden flex-shrink-0 border-2 transition-all duration-150 cursor-pointer {}",
                                                                        if active_thumb.get() == i { "border-black" } else { "border-transparent hover:border-gray-300" }
                                                                    )
                                                                    on:click=move |_| active_thumb.set(i)
                                                                >
                                                                    <div class="w-full h-full flex items-center justify-center">
                                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
                                                                            <rect x="3" y="3" width="18" height="18" rx="2"></rect>
                                                                            <polyline points="21 15 16 10 5 21"></polyline>
                                                                        </svg>
                                                                    </div>
                                                                </button>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                    // Next arrow
                                                    <button class="mt-2 p-1 text-gray-400 hover:text-black transition-colors cursor-pointer flex-shrink-0" on:click=move |_| {
                                                        active_thumb.update(|i| { if *i < 4 { *i += 1; } });
                                                    }>
                                                        {icon_chevron_right().into_any()}
                                                    </button>
                                                </div>
                                            </div>

                                            // ──── RIGHT: Sticky info panel ─────────────
                                            <div class="lg:sticky lg:top-4 flex flex-col gap-5">

                                                // ── Title ─────────────────────────────
                                                <h1 class="text-2xl font-normal text-gray-900 leading-snug">
                                                    {data.product.name.clone()}
                                                </h1>

                                                // ── Price row ──────────────────────────
                                                <div class="flex items-center gap-3 flex-wrap">
                                                    {move || match matching_variant.get() {
                                                        Some(variant) => {
                                                            let base_dec = variant.base_price * rust_decimal::Decimal::from(1000);
                                                            let base = format!("{:.0} VND", base_dec);
                                                            match variant.sale_price {
                                                                Some(sp) => {
                                                                    let sale_dec = sp * rust_decimal::Decimal::from(1000);
                                                                    let sale = format!("{:.0} VND", sale_dec);
                                                                    view! {
                                                                        <span class="text-gray-400 line-through text-base">{base}</span>
                                                                        <span class="text-black font-semibold text-xl">{sale}</span>
                                                                        <span class="bg-blue-100 text-blue-700 text-xs font-medium px-2 py-0.5 rounded-sm">"Miễn Phí Ship"</span>
                                                                    }.into_any()
                                                                },
                                                                None => view! {
                                                                    <span class="text-black font-semibold text-xl">{base}</span>
                                                                    <span class="bg-blue-100 text-blue-700 text-xs font-medium px-2 py-0.5 rounded-sm">"Miễn Phí Ship"</span>
                                                                }.into_any(),
                                                            }
                                                        },
                                                        None => view! {
                                                            <span class="text-gray-400 text-sm">"Chọn biến thể để xem giá"</span>
                                                        }.into_any(),
                                                    }}
                                                </div>

                                                // ── Size Selector ─────────────────────
                                                {
                                                    let variants_for_sizes = data.variants.clone();
                                                    let sizes_clone = sizes.clone();
                                                    if !sizes_clone.is_empty() {
                                                        view! {
                                                            <div class="space-y-2.5">
                                                                <p class="text-sm text-gray-700">
                                                                    "Size: "
                                                                    <span class="font-semibold">{move || selected_size.get().unwrap_or_default()}</span>
                                                                </p>
                                                                <div class="flex flex-wrap gap-2">
                                                                    {sizes_clone.into_iter().map(move |s| {
                                                                        let s_sel = s.clone();
                                                                        let s_val = s.clone();
                                                                        let s_view = s.clone();
                                                                        let s_avail = s.clone();
                                                                        let is_selected = move || selected_size.get() == Some(s_sel.clone());
                                                                        let variants = variants_for_sizes.clone();
                                                                        let is_available = move || {
                                                                            let c = selected_color.get();
                                                                            if let Some(c) = c {
                                                                                let c_val = c.clone();
                                                                                variants.iter().any(|v| {
                                                                                    let vc = v.attributes.get("color").and_then(|val: &serde_json::Value| val.as_str()).unwrap_or_default();
                                                                                    let vs = v.attributes.get("size").and_then(|val: &serde_json::Value| val.as_str()).unwrap_or_default();
                                                                                    vc == c_val && vs == s_avail
                                                                                })
                                                                            } else { true }
                                                                        };
                                                                        view! {
                                                                            <button
                                                                                class=move || format!(
                                                                                    "min-w-[44px] h-[44px] px-3 border text-sm font-medium transition-all duration-150 cursor-pointer focus:outline-none rounded-sm {} {}",
                                                                                    if is_selected() { "bg-black border-black text-white" }
                                                                                    else { "bg-white border-gray-300 text-gray-800 hover:border-black" },
                                                                                    if !is_available() { "opacity-30 line-through cursor-not-allowed" } else { "" }
                                                                                )
                                                                                on:click=move |_| selected_size.set(Some(s_val.clone()))
                                                                            >
                                                                                {s_view}
                                                                            </button>
                                                                        }
                                                                    }).collect_view()}
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }
                                                }

                                                // ── Quantity & Stock ──────────────────
                                                <div class="space-y-4">
                                                    <div class="flex items-center justify-between">
                                                        <div class="space-y-2">
                                                            <p class="text-sm text-gray-700">"Số lượng"</p>
                                                            <div class="inline-flex items-center border border-gray-300 rounded-sm">
                                                                <button
                                                                    class="w-10 h-11 flex items-center justify-center text-gray-600 hover:bg-gray-50 transition-colors cursor-pointer text-lg leading-none"
                                                                    on:click=move |_| quantity.update(|q| { if *q > 1 { *q -= 1; } })
                                                                >"−"</button>
                                                                <span class="w-10 h-11 flex items-center justify-center text-sm font-medium border-x border-gray-300">{move || quantity.get()}</span>
                                                                <button
                                                                    class="w-10 h-11 flex items-center justify-center text-gray-600 hover:bg-gray-50 transition-colors cursor-pointer text-lg leading-none"
                                                                    on:click=move |_| quantity.update(|q| *q += 1)
                                                                >"+"</button>
                                                            </div>
                                                        </div>

                                                        {move || {
                                                            let v: Option<ProductVariant> = matching_variant.get();
                                                            view! {
                                                                <div class="text-right">
                                                                    <p class="text-[10px] text-gray-400 uppercase tracking-widest font-bold">"Tình trạng"</p>
                                                                    {match v {
                                                                        Some(v) => {
                                                                            let vid = v.id;
                                                                            let stock: LocalResource<Result<serde_json::Value, ApiError>> = LocalResource::new(move || {
                                                                                let vid = vid;
                                                                                async move { client::inventory::get_stock(vid).await }
                                                                            });
                                                                            view! {
                                                                                <Suspense fallback=|| view! { <span class="text-xs text-gray-400">"Đang kiểm tra..."</span> }.into_any()>
                                                                                     {move || {
                                                                                         match stock.get() {
                                                                                             Some(res) => {
                                                                                                 match res.as_ref() {
                                                                                                     Ok(s) => {
                                                                                                         let qty = s.get("quantity").and_then(|q: &serde_json::Value| q.as_i64()).unwrap_or(0);
                                                                                                         if qty > 0 {
                                                                                                             view! { <span class="text-xs text-emerald-600 font-bold">"Còn hàng (" {qty} ")"</span> }.into_any()
                                                                                                         } else {
                                                                                                             view! { <span class="text-xs text-red-500 font-bold">"Hết hàng"</span> }.into_any()
                                                                                                         }
                                                                                                     },
                                                                                                     Err(_) => view! { <span class="text-xs text-red-500 font-bold">"Lỗi"</span> }.into_any()
                                                                                                 }
                                                                                             },
                                                                                             None => view! { <span class="text-xs text-gray-400">"..."</span> }.into_any()
                                                                                         }
                                                                                     }}
                                                                                </Suspense>
                                                                            }.into_any()
                                                                        },
                                                                        None => view! { <span class="text-xs text-gray-400 italic">"Chưa chọn biến thể"</span> }.into_any()
                                                                    }}
                                                                </div>
                                                            }.into_any()
                                                        }}
                                                    </div>

                                                    <button
                                                        class="w-full bg-black text-white py-4 rounded-sm font-semibold tracking-wide hover:bg-gray-900 transition-all duration-200 active:scale-[0.98] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        disabled=move || matching_variant.get().is_none()
                                                        on:click={
                                                            let cart = cart;
                                                            let data = data.clone();
                                                            move |_| {
                                                                if let Some(v) = matching_variant.get() {
                                                                    let price_dec = v.sale_price.unwrap_or(v.base_price) * rust_decimal::Decimal::from(1000);
                                                                    cart.add_item(CartItem {
                                                                        variant_id: v.id,
                                                                        product_name: data.product.name.clone(),
                                                                        variant_name: v.name.clone(),
                                                                        unit_price: price_dec.to_i64().unwrap_or(0),
                                                                        quantity: quantity.get(),
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    >
                                                        "THÊM VÀO GIỎ HÀNG"
                                                    </button>
                                                </div>

                                                // ── Info Accordion ─────────────────────
                                                <div class="mt-4 border-t border-gray-100">
                                                    <div class="py-4 border-b border-gray-100">
                                                        <button class="w-full flex items-center justify-between text-left group cursor-pointer">
                                                            <span class="text-sm font-medium">"CHI TIẾT SẢN PHẨM"</span>
                                                            {icon_chevron_down().into_any()}
                                                        </button>
                                                        <div class="mt-3 text-sm text-gray-600 space-y-2 leading-relaxed">
                                                            <p>{"Mẫu sản phẩm cao cấp từ June Pea. Chất liệu thoáng mát, form dáng chuẩn."}</p>
                                                            <p>{data.product.description.clone().unwrap_or_default()}</p>
                                                        </div>
                                                    </div>
                                                    <div class="py-4 border-b border-gray-100">
                                                        <button class="w-full flex items-center justify-between text-left group cursor-pointer">
                                                            <span class="text-sm font-medium">"CHÍNH SÁCH ĐỔI TRẢ"</span>
                                                            {icon_chevron_down().into_any()}
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                },
                                Err(e) => view! {
                                    <div class="py-20 text-center">
                                        <h2 class="text-xl font-medium text-red-600">"Không tìm thấy sản phẩm"</h2>
                                        <p class="text-gray-500 mt-2">{e.to_string()}</p>
                                        <a href="/products" class="mt-6 inline-block underline">"Quay lại cửa hàng"</a>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
