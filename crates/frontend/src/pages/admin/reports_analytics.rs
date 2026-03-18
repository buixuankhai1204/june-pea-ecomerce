use leptos::prelude::*;
use crate::api::client::analytics::*;

fn sparkline(points: &[f64], color: &str) -> impl IntoView {
    let w = 200.0_f64;
    let h = 60.0_f64;
    let min = points.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1.0);
    let n = points.len();

    let path: String = points
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = i as f64 / (n - 1).max(1) as f64 * w;
            let y = h - (v - min) / range * (h - 8.0) - 4.0;
            if i == 0 {
                format!("M {:.1} {:.1}", x, y)
            } else {
                format!(" L {:.1} {:.1}", x, y)
            }
        })
        .collect();

    let area = format!("{} L {:.1} {:.1} L 0 {:.1} Z", path, w, h, h);
    let color = color.to_string();
    view! {
        <svg viewBox={format!("0 0 {} {}", w, h)} class="w-full h-full" preserveAspectRatio="none">
            <defs>
                <linearGradient id="grad1" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color={color.clone()} stop-opacity="0.3"/>
                    <stop offset="100%" stop-color={color.clone()} stop-opacity="0.0"/>
                </linearGradient>
            </defs>
            <path d={area} fill="url(#grad1)"/>
            <path d={path} fill="none" stroke={color} stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

fn icon_download() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
    }
}

#[component]
pub fn AdminReportsAnalyticsPage() -> impl IntoView {
    let chart_toggle = RwSignal::new("Monthly");

    let revenue_resource = LocalResource::new(|| async move { get_revenue_analytics().await });
    let summary_resource = LocalResource::new(|| async move { get_dashboard_summary().await });
    let categories_resource = LocalResource::new(|| async move { get_category_breakdown().await });

    let top_products = [
        ("Cà Phê Muối", 420, "#FCE300"),
        ("Trà Đào Cam Sả", 380, "#FDBA74"),
        ("Bạc Xỉu", 290, "#94A3B8"),
        ("Trà Sữa Khoai Môn", 150, "#C084FC"),
    ];

    let order_points = [45.0, 52.0, 48.0, 70.0, 65.0, 85.0, 68.0, 92.0, 88.0, 105.0, 110.0, 125.0];

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Reports & Analytics"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Business performance overview and trends"</p>
                </div>
                <div class="flex items-center gap-2">
                    // Period toggle
                    <div class="flex items-center bg-gray-100 rounded-xl p-0.5">
                        {["Daily", "Monthly", "Yearly"].iter().map(|&t| {
                            let t_str = t;
                            view! {
                                <button
                                    class=move || format!("px-3 py-1.5 text-xs font-semibold rounded-lg transition-all duration-150 cursor-pointer {}",
                                        if chart_toggle.get() == t_str { "bg-white text-gray-900 shadow-sm" } else { "text-gray-500" })
                                    on:click=move |_| chart_toggle.set(t_str)
                                >{t}</button>
                            }
                        }).collect_view()}
                    </div>
                    <button class="flex items-center gap-1.5 border border-gray-200 text-gray-700 text-xs font-bold px-4 py-2.5 rounded-xl hover:bg-gray-50 cursor-pointer transition-colors shadow-sm">
                        {icon_download()} "Export"
                    </button>
                </div>
            </div>

            // KPI row
            <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                <div class="rounded-2xl p-5 border shadow-sm bg-indigo-50 border-indigo-100">
                    <p class="text-xs text-gray-500 font-medium">"Total Revenue"</p>
                    {move || match summary_resource.get() {
                        Some(res) => match &*res {
                            Ok(s) => view! { <p class="text-2xl font-black mt-1.5 text-indigo-600">{format!("₫{}k", s.total_revenue / 1000)}</p> }.into_any(),
                            _ => view! { <p class="text-2xl font-black mt-1.5 text-indigo-600">"₫..."</p> }.into_any()
                        },
                        _ => view! { <p class="text-2xl font-black mt-1.5 text-indigo-600">"₫..."</p> }.into_any()
                    }}
                </div>
                <div class="rounded-2xl p-5 border shadow-sm bg-emerald-50 border-emerald-100">
                    <p class="text-xs text-gray-500 font-medium">"Total Orders"</p>
                    {move || match summary_resource.get() {
                        Some(res) => match &*res {
                            Ok(s) => view! { <p class="text-2xl font-black mt-1.5 text-emerald-600">{s.today_orders}</p> }.into_any(),
                            _ => view! { <p class="text-2xl font-black mt-1.5 text-emerald-600">"..."</p> }.into_any()
                        },
                        _ => view! { <p class="text-2xl font-black mt-1.5 text-emerald-600">"..."</p> }.into_any()
                    }}
                </div>
                <div class="rounded-2xl p-5 border shadow-sm bg-amber-50 border-amber-100">
                    <p class="text-xs text-gray-500 font-medium">"Stock Accuracy"</p>
                    {move || match summary_resource.get() {
                        Some(res) => match &*res {
                            Ok(s) => view! { <p class="text-2xl font-black mt-1.5 text-amber-500">{format!("{:.0}%", s.stock_accuracy)}</p> }.into_any(),
                            _ => view! { <p class="text-2xl font-black mt-1.5 text-amber-500">"..."</p> }.into_any()
                        },
                        _ => view! { <p class="text-2xl font-black mt-1.5 text-amber-500">"..."</p> }.into_any()
                    }}
                </div>
                <div class="rounded-2xl p-5 border shadow-sm bg-rose-50 border-rose-100">
                    <p class="text-xs text-gray-500 font-medium">"Return Rate"</p>
                    <p class="text-2xl font-black mt-1.5 text-rose-600">"2.4%"</p>
                </div>
            </div>

            // Revenue chart + traffic source
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
                // Revenue line chart (2/3)
                <div class="lg:col-span-2 bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <div class="flex items-center justify-between mb-4">
                        <div>
                            <h2 class="text-sm font-bold text-gray-900">"Revenue Trend"</h2>
                            <p class="text-xs text-gray-400">"Monthly revenue in millions ₫"</p>
                        </div>
                        <div class="flex items-center gap-3 text-xs">
                            <div class="flex items-center gap-1.5">
                                <span class="w-3 h-3 rounded-full bg-indigo-500"></span>
                                <span class="text-gray-500">"Revenue"</span>
                            </div>
                        </div>
                    </div>

                    // Bar chart
                    // Area chart 
                    <div class="h-64 bg-gray-50 rounded-xl overflow-hidden">
                        <Suspense fallback=move || view! { <p>"Loading charts..."</p> }>
                            {move || match revenue_resource.get() {
                                Some(res) => match &*res {
                                    Ok(points) => {
                                        let data: Vec<f64> = points.iter().map(|p| p.value as f64).collect();
                                        view! { {sparkline(&data, "#6366F1")} }.into_any()
                                    },
                                    _ => view! { <p>"Chart error"</p> }.into_any()
                                },
                                None => view! { <p>"No data"</p> }.into_any()
                            }}
                        </Suspense>
                    </div>
                </div>

                // Traffic sources (1/3)
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900 mb-4">"Category Breakdown"</h2>

                    <div class="space-y-3.5">
                        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                            {move || match categories_resource.get() {
                                Some(sw) => {
                                    let res = &*sw;
                                    match res {
                                        Ok(items) => items.iter().map(|s| {
                                            view! {
                                                <div class="flex items-center gap-3">
                                                    <div class="w-8 h-8 rounded-xl flex items-center justify-center flex-shrink-0"
                                                        style={format!("background:{}20", s.color)}>
                                                        <span class="w-2.5 h-2.5 rounded-full" style={format!("background:{}", s.color)}></span>
                                                    </div>
                                                    <div class="flex-1">
                                                        <div class="flex items-center justify-between text-xs mb-1.5">
                                                            <span class="font-semibold text-gray-700">{s.category_name.clone()}</span>
                                                            <span class="text-gray-900 font-bold">{s.sales_percentage}"%"</span>
                                                        </div>
                                                        <div class="h-1.5 bg-gray-100 rounded-full overflow-hidden">
                                                            <div class="h-full rounded-full" style={format!("width:{}%; background:{}", s.sales_percentage, s.color)}></div>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_any(),
                                        _ => view! { <p>"No data"</p> }.into_any()
                                    }
                                },
                                None => view! { <p>"No data"</p> }.into_any()
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            // Top products + orders trend
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                // Top products
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900 mb-4">"Top Products by Sales"</h2>
                    <div class="space-y-4">
                        {top_products.iter().map(|&(name, sold, color)| {
                            let max_sold = 420;
                            let pct = (sold as f64 / max_sold as f64 * 100.0) as u32;
                            view! {
                                <div class="space-y-1.5">
                                    <div class="flex items-center justify-between text-xs">
                                        <span class="font-medium text-gray-700 truncate pr-4">{name}</span>
                                        <span class="font-bold text-gray-900 flex-shrink-0">{sold} " sold"</span>
                                    </div>
                                    <div class="h-2 bg-gray-100 rounded-full overflow-hidden">
                                        <div class="h-full rounded-full transition-all duration-700"
                                            style={format!("width:{}%; background:{}", pct, color)}>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // Orders trend area chart
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <div class="flex items-center justify-between mb-4">
                        <div>
                            <h2 class="text-sm font-bold text-gray-900">"Orders Trend"</h2>
                            <p class="text-xs text-gray-400">"12-month order volume"</p>
                        </div>
                        <p class="text-xl font-black text-gray-900">"3,247"</p>
                    </div>
                    <div class="h-32 bg-gray-50 rounded-xl overflow-hidden">
                        {sparkline(&order_points, "#10B981")}
                    </div>
                    // Month labels
                    <div class="flex justify-between text-[9px] text-gray-300 mt-2">
                        {["J","F","M","A","M","J","J","A","S","O","N","D"].iter().map(|&m| view! {
                            <span>{m}</span>
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}
