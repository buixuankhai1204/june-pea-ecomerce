use leptos::prelude::*;

use crate::api::client;
use crate::api::types::PaginatedProducts;
use leptos_router::hooks::use_query_map;

// ─── Icons ─────────────────────────────────────────────────────────────────

fn icon_chevron_down_sm() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
    }
}

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

fn icon_x_sm() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
    }
}

fn icon_check_sm() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
    }
}

// ─── Filter option data ─────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
struct FilterOpt {
    value: &'static str,
    label: &'static str,
}

fn opts_type() -> Vec<FilterOpt> {
    vec![
        FilterOpt {
            value: "ao-thun",
            label: "Áo thun",
        },
        FilterOpt {
            value: "ao-polo",
            label: "Áo polo",
        },
        FilterOpt {
            value: "ao-so-mi",
            label: "Áo sơ mi",
        },
        FilterOpt {
            value: "ao-khoac",
            label: "Áo khoác",
        },
    ]
}

fn opts_collection() -> Vec<FilterOpt> {
    vec![
        FilterOpt {
            value: "non-branded",
            label: "Non Branded",
        },
        FilterOpt {
            value: "the-worker",
            label: "The Worker",
        },
        FilterOpt {
            value: "the-no-style",
            label: "The No Style",
        },
    ]
}

fn opts_color() -> Vec<FilterOpt> {
    vec![
        FilterOpt {
            value: "trang",
            label: "Trắng",
        },
        FilterOpt {
            value: "den",
            label: "Đen",
        },
        FilterOpt {
            value: "xam",
            label: "Xám",
        },
        FilterOpt {
            value: "xanh-duong",
            label: "Xanh dương",
        },
        FilterOpt {
            value: "xanh-la",
            label: "Xanh lá",
        },
        FilterOpt {
            value: "be",
            label: "Be",
        },
    ]
}

fn opts_fit() -> Vec<FilterOpt> {
    vec![
        FilterOpt {
            value: "regular",
            label: "Regular Fit",
        },
        FilterOpt {
            value: "slim",
            label: "Slim Fit",
        },
        FilterOpt {
            value: "boxy",
            label: "Boxy / Rộng",
        },
        FilterOpt {
            value: "oversize",
            label: "Oversize",
        },
    ]
}

fn opts_sort() -> Vec<FilterOpt> {
    vec![
        FilterOpt {
            value: "newest",
            label: "Ngày (từ mới đến cũ)",
        },
        FilterOpt {
            value: "oldest",
            label: "Ngày (từ cũ đến mới)",
        },
        FilterOpt {
            value: "price-asc",
            label: "Giá: Thấp đến cao",
        },
        FilterOpt {
            value: "price-desc",
            label: "Giá: Cao đến thấp",
        },
    ]
}

// ─── FilterDropdown component ───────────────────────────────────────────────
// Uses `StoredValue` instead of raw strings to avoid move-after-use errors.

#[component]
fn FilterDropdown(
    label: &'static str,
    options: Vec<FilterOpt>,
    selected: RwSignal<Option<String>>,
    active_dropdown: RwSignal<Option<String>>,
    key: &'static str,
) -> impl IntoView {
    let is_open = move || active_dropdown.get().as_deref() == Some(key);
    let toggle = move |_: web_sys::MouseEvent| {
        active_dropdown.update(|cur| {
            *cur = if cur.as_deref() == Some(key) {
                None
            } else {
                Some(key.to_string())
            };
        });
    };
    let close = move |_: web_sys::MouseEvent| {
        active_dropdown.update(|cur| {
            if cur.as_deref() == Some(key) {
                *cur = None;
            }
        });
    };

    view! {
        <div class="relative">
            <button
                class=move || format!(
                    "flex items-center gap-1.5 text-sm px-3 py-1.5 rounded-sm border transition-colors duration-150 cursor-pointer focus:outline-none {}",
                    if is_open() || selected.get().is_some() {
                        "border-black text-black bg-gray-50"
                    } else {
                        "border-transparent text-gray-700 hover:text-black"
                    }
                )
                on:click=toggle
            >
                {move || if selected.get().is_some() {
                    view! { <span class="font-semibold">{label}</span> }.into_any()
                } else {
                    view! { <span>{label}</span> }.into_any()
                }}
                <span class=move || format!(
                    "transition-transform duration-200 {}",
                    if is_open() { "rotate-180" } else { "" }
                )>
                    {icon_chevron_down_sm()}
                </span>
                {move || if selected.get().is_some() {
                    view! {
                        <span class="ml-0.5 text-gray-400 hover:text-black cursor-pointer"
                            on:click=move |e| { e.stop_propagation(); selected.set(None); }>
                            {icon_x_sm()}
                        </span>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </button>

            {move || if is_open() {
                let opts = options.clone();
                view! {
                    // invisible overlay to catch outside-clicks
                    <div class="fixed inset-0 z-10" on:click=close></div>
                    // panel
                    <div class="absolute top-full left-0 mt-1 z-20 bg-white border border-gray-200 rounded-sm shadow-lg min-w-[180px] py-1">
                        {opts.into_iter().map(|opt| {
                            let val = opt.value;
                            let is_checked = move || selected.get().as_deref() == Some(val);
                            view! {
                                <button
                                    class="w-full text-left flex items-center justify-between gap-4 px-4 py-2.5 text-sm text-gray-700 hover:bg-gray-50 hover:text-black transition-colors cursor-pointer"
                                    on:click=move |_| {
                                        selected.set(if is_checked() { None } else { Some(val.to_string()) });
                                        active_dropdown.set(None);
                                    }
                                >
                                    <span>{opt.label}</span>
                                    {move || if is_checked() {
                                        view! { <span class="text-black">{icon_check_sm()}</span> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
        </div>
    }
}

// ─── Sub-category struct ────────────────────────────────────────────────────

struct SubCat {
    label: &'static str,
    desc: &'static str,
}

// ─── Page ──────────────────────────────────────────────────────────────────

#[component]
pub fn ProductsPage() -> impl IntoView {
    let page = RwSignal::new(1i64);
    let page_size = 12i64;

    // Filter & sort state
    let filter_type = RwSignal::new(Option::<String>::None);
    let filter_collection = RwSignal::new(Option::<String>::None);
    let filter_color = RwSignal::new(Option::<String>::None);
    let filter_fit = RwSignal::new(Option::<String>::None);
    let sort_by = RwSignal::new("newest".to_string());
    // Only one dropdown open at a time
    let active_dropdown = RwSignal::new(Option::<String>::None);


    let query_map = use_query_map();
    let search_query = move || query_map.get().get("q").map(|s| s.to_string());

    let products = LocalResource::new(move || {
        let p = page.get();
        let q = search_query();
        async move {
            if let Some(q_val) = q {
                match client::catalog::search_products(&q_val).await {
                    Ok(items) => {
                        let count = items.len() as i64;
                        Ok(PaginatedProducts {
                            total: count,
                            items,
                            page: 1,
                            page_size: count,
                        })
                    }
                    Err(e) => Err(e),
                }
            } else {
                client::catalog::list_products(p, page_size).await
            }
        }
    });

    let categories = LocalResource::new(|| async move { client::catalog::get_category_tree().await });

    // Number of active filters (for "clear all" button)
    let active_count = move || {
        [
            filter_type.get().is_some(),
            filter_collection.get().is_some(),
            filter_color.get().is_some(),
            filter_fit.get().is_some(),
        ]
        .iter()
        .filter(|&&b| b)
        .count()
    };

    view! {
        <div class="bg-white min-h-screen font-[Montserrat,system-ui,sans-serif]">
            <div class="max-w-screen-xl mx-auto px-4 sm:px-6">

                // ── Breadcrumb ─────────────────────────────────────────────
                <nav class="text-xs text-gray-500 flex items-center gap-1.5 py-3">
                    <a href="/" class="hover:text-black transition-colors cursor-pointer">"Trang chủ"</a>
                    <span class="text-gray-300">"›"</span>
                    <span class="text-gray-700">"Áo"</span>
                </nav>

                // ── Page title ────────────────────────────────────────────
                <h1 class="text-3xl font-light text-black tracking-wide mb-6">"ÁO"</h1>

                // ── Sub-category banners (Dynamic from API) ─────────────────────────
                <Suspense fallback=|| view! { <div class="h-32 bg-gray-50 animate-pulse rounded-2xl mb-8"></div> }>
                    {move || categories.get().map(|res| match res.as_ref() {
                        Ok(cats) => {
                            view! {
                                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-8">
                                    {cats.iter().take(4).map(|cat| {
                                        let name = cat.category.name.clone();
                                        view! {
                                            <a href="/products" class="relative block aspect-[3/4] bg-gray-100 overflow-hidden group cursor-pointer">
                                                <div class="absolute inset-0 bg-gradient-to-b from-gray-100 to-gray-200 group-hover:scale-105 transition-transform duration-500 ease-out"></div>
                                                <div class="absolute bottom-0 left-0 right-0 p-4 space-y-1">
                                                    <span class="inline-block bg-[#2D3748] text-white text-[11px] font-semibold px-2.5 py-1 rounded-full tracking-wider">
                                                        {name}
                                                    </span>
                                                    <p class="text-xs text-gray-700 font-medium leading-tight">"Explore collection"</p>
                                                </div>
                                            </a>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        },
                        Err(_) => ().into_any()
                    })}
                </Suspense>

                // ── Filter bar ────────────────────────────────────────────
                <div class="flex items-center justify-between border-t border-b border-gray-200 py-2.5 mb-2 gap-4 flex-wrap">
                    // Left: filter dropdowns
                    <div class="flex items-center gap-1 flex-wrap">
                        <span class="text-sm text-gray-500 font-medium mr-2">"Bộ lọc:"</span>

                        <FilterDropdown label="Kiểu sản phẩm" options=opts_type()       selected=filter_type       active_dropdown=active_dropdown key="type"       />
                        <FilterDropdown label="Bộ sưu tập"    options=opts_collection()  selected=filter_collection active_dropdown=active_dropdown key="collection" />
                        <FilterDropdown label="Màu sắc"        options=opts_color()       selected=filter_color      active_dropdown=active_dropdown key="color"      />
                        <FilterDropdown label="Phom dáng"      options=opts_fit()         selected=filter_fit        active_dropdown=active_dropdown key="fit"        />

                        // Clear all
                        {move || if active_count() > 0 {
                            view! {
                                <button
                                    class="ml-2 text-xs text-gray-400 hover:text-black underline underline-offset-2 transition-colors cursor-pointer"
                                    on:click=move |_| {
                                        filter_type.set(None);
                                        filter_collection.set(None);
                                        filter_color.set(None);
                                        filter_fit.set(None);
                                    }
                                >
                                    {move || format!("Xóa bộ lọc ({})", active_count())}
                                </button>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>

                    // Right: sort dropdown
                    <div class="relative flex-shrink-0">
                        <div class="flex items-center gap-2 text-sm text-gray-500">
                            <span>"Sắp xếp theo:"</span>
                            <div class="relative">
                                <button
                                    class="flex items-center gap-1 text-gray-800 font-medium hover:text-black cursor-pointer focus:outline-none"
                                    on:click=move |_| {
                                        active_dropdown.update(|cur| {
                                            *cur = if cur.as_deref() == Some("sort") { None } else { Some("sort".to_string()) };
                                        });
                                    }
                                >
                                    {move || {
                                        let cur = sort_by.get();
                                        opts_sort().into_iter().find(|o| o.value == cur).map(|o| o.label).unwrap_or("Ngày (từ mới đến cũ)")
                                    }}
                                    <span class=move || format!(
                                        "transition-transform duration-200 {}",
                                        if active_dropdown.get().as_deref() == Some("sort") { "rotate-180" } else { "" }
                                    )>
                                        {icon_chevron_down_sm()}
                                    </span>
                                </button>
                                {move || if active_dropdown.get().as_deref() == Some("sort") {
                                    view! {
                                        <div class="fixed inset-0 z-10"
                                            on:click=move |_| {
                                                active_dropdown.update(|cur| {
                                                    if cur.as_deref() == Some("sort") { *cur = None; }
                                                });
                                            }>
                                        </div>
                                        <div class="absolute top-full right-0 mt-1 z-20 bg-white border border-gray-200 rounded-sm shadow-lg min-w-[230px] py-1">
                                            {opts_sort().into_iter().map(|opt| {
                                                let val = opt.value;
                                                view! {
                                                    <button
                                                        class="w-full text-left flex items-center justify-between gap-4 px-4 py-2.5 text-sm text-gray-700 hover:bg-gray-50 hover:text-black transition-colors cursor-pointer"
                                                        on:click=move |_| {
                                                            sort_by.set(val.to_string());
                                                            active_dropdown.set(None);
                                                        }
                                                    >
                                                        <span>{opt.label}</span>
                                                        {move || if sort_by.get() == val {
                                                            view! { <span class="text-black">{icon_check_sm()}</span> }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                // ── Active filter chips ───────────────────────────────────
                {move || {
                    struct Chip { prefix: &'static str, label: &'static str, clear: Box<dyn Fn()> }
                    let mut chips: Vec<Chip> = vec![];
                    if let Some(v) = filter_type.get() {
                        let lbl = opts_type().into_iter().find(|o| o.value == v).map(|o| o.label).unwrap_or("?");
                        chips.push(Chip { prefix: "Kiểu", label: lbl, clear: Box::new(move || filter_type.set(None)) });
                    }
                    if let Some(v) = filter_collection.get() {
                        let lbl = opts_collection().into_iter().find(|o| o.value == v).map(|o| o.label).unwrap_or("?");
                        chips.push(Chip { prefix: "Bộ sưu tập", label: lbl, clear: Box::new(move || filter_collection.set(None)) });
                    }
                    if let Some(v) = filter_color.get() {
                        let lbl = opts_color().into_iter().find(|o| o.value == v).map(|o| o.label).unwrap_or("?");
                        chips.push(Chip { prefix: "Màu", label: lbl, clear: Box::new(move || filter_color.set(None)) });
                    }
                    if let Some(v) = filter_fit.get() {
                        let lbl = opts_fit().into_iter().find(|o| o.value == v).map(|o| o.label).unwrap_or("?");
                        chips.push(Chip { prefix: "Phom", label: lbl, clear: Box::new(move || filter_fit.set(None)) });
                    }

                    if chips.is_empty() {
                        view! { <div></div> }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-wrap gap-2 py-3">
                                {chips.into_iter().map(|chip| view! {
                                    <span class="inline-flex items-center gap-1.5 bg-black text-white text-xs font-medium px-3 py-1.5 rounded-full">
                                        <span class="text-gray-400">{chip.prefix}": "</span>
                                        {chip.label}
                                        <button class="ml-0.5 text-gray-300 hover:text-white cursor-pointer"
                                            on:click=move |_| (chip.clear)()>
                                            {icon_x_sm()}
                                        </button>
                                    </span>
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}

                // ── Product grid ──────────────────────────────────────────
                <Suspense fallback=move || view! { <ProductGridSkeleton /> }>
                    {move || Suspend::new(async move {
                        match products.await {
                            Ok(data) => {
                                let total_pages = (data.total + page_size - 1) / page_size;
                                let total = data.total;
                                view! {
                                    <p class="text-xs text-gray-400 mb-4">{format!("{} sản phẩm", total)}</p>

                                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-x-4 gap-y-8">
                                        {data.items.into_iter().map(|product| {
                                            let slug = product.product.slug.clone();
                                            view! {
                                                <a href=format!("/products/{}", slug) class="group block cursor-pointer">
                                                    <div class="relative bg-gray-100 aspect-[3/4] overflow-hidden mb-3">
                                                        // "MỚI" badge
                                                        <div class="absolute top-3 left-3 z-10 w-10 h-10 bg-[#2D3748] rounded-full flex items-center justify-center">
                                                            <span class="text-white text-[10px] font-bold">"MỚI"</span>
                                                        </div>
                                                        // Placeholder
                                                        <div class="w-full h-full flex items-center justify-center group-hover:scale-105 transition-transform duration-500 ease-out">
                                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="0.75">
                                                                <rect x="3" y="3" width="18" height="18" rx="2"></rect>
                                                                <circle cx="8.5" cy="8.5" r="1.5"></circle>
                                                                <polyline points="21 15 16 10 5 21"></polyline>
                                                            </svg>
                                                        </div>
                                                        // "Miễn Phí Ship" pill
                                                        <div class="absolute bottom-3 left-3">
                                                            <span class="bg-white border border-gray-200 text-gray-700 text-[11px] font-medium px-2 py-0.5 rounded-sm">
                                                                "Miễn Phí Ship"
                                                            </span>
                                                        </div>
                                                    </div>
                                                    <div class="space-y-1">
                                                        <h3 class="text-sm text-gray-900 leading-snug line-clamp-2 group-hover:text-gray-600 transition-colors">
                                                            {product.product.name.clone()}
                                                        </h3>
                                                        <div class="flex items-center gap-2 flex-wrap">
                                                            <span class="text-gray-400 line-through text-xs">"297.000 VND"</span>
                                                            <span class="text-black font-semibold text-sm">"263.150 VND"</span>
                                                        </div>
                                                    </div>
                                                </a>
                                            }
                                        }).collect_view()}
                                    </div>

                                    // ── Pagination ────────────────────────
                                    <div class="mt-12 mb-10 flex items-center justify-center gap-2">
                                        <button
                                            class="w-9 h-9 flex items-center justify-center border border-gray-200 rounded-sm text-gray-600 hover:border-black hover:text-black transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                            disabled=move || {page.get() <= 1}
                                            on:click=move |_| page.update(|p| *p -= 1)
                                        >
                                            {icon_chevron_left()}
                                        </button>
                                        {move || {
                                            let cur = page.get();
                                            let tp  = total_pages.max(1);
                                            (1..=tp).map(|i| view! {
                                                <button
                                                    class=move || format!(
                                                        "w-9 h-9 flex items-center justify-center border rounded-sm text-sm transition-colors cursor-pointer {}",
                                                        if cur == i { "border-black bg-black text-white" }
                                                        else { "border-gray-200 text-gray-700 hover:border-black hover:text-black" }
                                                    )
                                                    on:click=move |_| page.set(i)
                                                >
                                                    {i.to_string()}
                                                </button>
                                            }).collect_view()
                                        }}
                                        <button
                                            class="w-9 h-9 flex items-center justify-center border border-gray-200 rounded-sm text-gray-600 hover:border-black hover:text-black transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                            disabled=move || {page.get() >= total_pages}
                                            on:click=move |_| page.update(|p| *p += 1)
                                        >
                                            {icon_chevron_right()}
                                        </button>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! {
                                <div class="py-16 text-center text-sm text-gray-500">
                                    {e.user_message().to_string()}
                                </div>
                            }.into_any(),
                        }
                    })}
                </Suspense>

            </div>
        </div>
    }
}

// ─── Skeleton ──────────────────────────────────────────────────────────────

#[component]
fn ProductGridSkeleton() -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-x-4 gap-y-8 animate-pulse">
            {(0..8usize).map(|_| view! {
                <div>
                    <div class="bg-gray-100 aspect-[3/4] mb-3"></div>
                    <div class="space-y-2">
                        <div class="h-3.5 bg-gray-100 rounded w-3/4"></div>
                        <div class="h-3.5 bg-gray-100 rounded w-1/2"></div>
                        <div class="h-4 bg-gray-100 rounded w-2/3"></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}
