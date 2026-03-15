use leptos::prelude::*;

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

    let revenue_points = vec![
        42.0, 58.0, 51.0, 73.0, 64.0, 88.0, 72.0, 95.0, 84.0, 110.0, 98.0, 128.0,
    ];
    let order_points = vec![
        840.0, 1020.0, 920.0, 1150.0, 1080.0, 1320.0, 1240.0, 1580.0, 1420.0, 1780.0, 1640.0,
        1920.0,
    ];

    let top_products = [
        ("Áo Thun Modal AirDry", 420, "#6366F1"),
        ("Áo Sơ Mi Non-Iron", 310, "#F59E0B"),
        ("Áo Polo Raglan Flex", 268, "#10B981"),
        ("Áo Khoác Worker", 145, "#F43F5E"),
        ("Áo Thun Boxy Oversize", 98, "#8B5CF6"),
    ];

    let traffic_sources = [
        ("Direct", 38, "#6366F1"),
        ("Social", 27, "#F59E0B"),
        ("Search", 22, "#10B981"),
        ("Referral", 13, "#F43F5E"),
    ];

    let monthly_data = [
        ("Jan", 42.0_f64),
        ("Feb", 58.0),
        ("Mar", 51.0),
        ("Apr", 73.0),
        ("May", 64.0),
        ("Jun", 88.0),
        ("Jul", 72.0),
        ("Aug", 95.0),
        ("Sep", 84.0),
        ("Oct", 110.0),
        ("Nov", 98.0),
        ("Dec", 128.0),
    ];
    let max_val = monthly_data.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);

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
                {[
                    ("Total Revenue",    "₫128.4M", "+12.5%", "#6366F1", "bg-indigo-50 border-indigo-100"),
                    ("Total Orders",     "3,247",   "+8.1%",  "#10B981", "bg-emerald-50 border-emerald-100"),
                    ("Avg. Order Value", "₫284k",   "+3.2%",  "#F59E0B", "bg-amber-50 border-amber-100"),
                    ("Return Rate",      "2.4%",    "-0.8%",  "#F43F5E", "bg-rose-50 border-rose-100"),
                ].iter().map(|&(label, val, delta, color, bg)| view! {
                    <div class=format!("rounded-2xl p-5 border shadow-sm {}", bg)>
                        <p class="text-xs text-gray-500 font-medium">{label}</p>
                        <p class="text-2xl font-black mt-1.5" style={format!("color:{}", color)}>{val}</p>
                        <p class="text-xs mt-1 font-semibold" style={format!("color:{}", if delta.starts_with('+') { "#10B981" } else { "#F43F5E" })}>
                            {delta} " vs last period"
                        </p>
                    </div>
                }).collect_view()}
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
                    <div class="flex items-end gap-1.5 h-36">
                        {monthly_data.iter().map(|&(month, val)| {
                            let pct = (val / max_val * 100.0) as u32;
                            let is_max = val == max_val;
                            view! {
                                <div class="flex-1 flex flex-col items-center gap-1">
                                    <div class="w-full bg-gray-100 rounded-t-lg overflow-hidden" style="height:120px;">
                                        <div class=move || format!("w-full rounded-t-lg transition-all duration-700 {}",
                                            if is_max { "bg-[#FCE300]" } else { "bg-indigo-200 hover:bg-indigo-400" })
                                            style={format!("height:{}%; margin-top:{}%", pct, 100 - pct)}>
                                        </div>
                                    </div>
                                    <span class="text-[9px] text-gray-400">{month}</span>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    // Area chart below
                    <div class="mt-4 h-16 bg-gray-50 rounded-xl overflow-hidden">
                        {sparkline(&revenue_points, "#6366F1")}
                    </div>
                </div>

                // Traffic sources (1/3)
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900 mb-4">"Traffic Sources"</h2>

                    // Visual ring (simple)
                    <div class="h-4 rounded-full overflow-hidden flex mb-5">
                        {traffic_sources.iter().map(|&(_, pct, color)| view! {
                            <div class="h-full transition-all" style={format!("width:{}%; background:{}", pct, color)}></div>
                        }).collect_view()}
                    </div>

                    <div class="space-y-3.5">
                        {traffic_sources.iter().map(|&(name, pct, color)| view! {
                            <div class="flex items-center gap-3">
                                <div class="w-8 h-8 rounded-xl flex items-center justify-center flex-shrink-0"
                                    style={format!("background:{}20", color)}>
                                    <span class="w-2.5 h-2.5 rounded-full" style={format!("background:{}", color)}></span>
                                </div>
                                <div class="flex-1">
                                    <div class="flex items-center justify-between text-xs mb-1.5">
                                        <span class="font-semibold text-gray-700">{name}</span>
                                        <span class="text-gray-900 font-bold">{pct}"%"</span>
                                    </div>
                                    <div class="h-1.5 bg-gray-100 rounded-full overflow-hidden">
                                        <div class="h-full rounded-full" style={format!("width:{}%; background:{}", pct, color)}></div>
                                    </div>
                                </div>
                            </div>
                        }).collect_view()}
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
