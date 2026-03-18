use leptos::prelude::*;
use crate::api::client;
use crate::api::client::analytics::*;

// ── Shared icon helpers ────────────────────────────────────────────────────

fn icon_trending_up() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"></polyline>
            <polyline points="17 6 23 6 23 12"></polyline>
        </svg>
    }
}

fn icon_more() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="1"></circle>
            <circle cx="19" cy="12" r="1"></circle>
            <circle cx="5" cy="12" r="1"></circle>
        </svg>
    }
}

fn icon_arrow_right() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="5" y1="12" x2="19" y2="12"></line>
            <polyline points="12 5 19 12 12 19"></polyline>
        </svg>
    }
}

// ── Sparkline ─────────────────────────────────────────────────────────────

fn sparkline(points: &[f64], color: &str) -> impl IntoView {
    let w = 80.0_f64;
    let h = 32.0_f64;
    let min = points.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1.0);
    let n = points.len();

    let path: String = points
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = i as f64 / (n - 1).max(1) as f64 * w;
            let y = h - (v - min) / range * (h - 4.0) - 2.0;
            if i == 0 {
                format!("M {:.1} {:.1}", x, y)
            } else {
                format!(" L {:.1} {:.1}", x, y)
            }
        })
        .collect();

    let color = color.to_string();
    view! {
        <svg viewBox={format!("0 0 {} {}", w, h)} class="w-20 h-8" preserveAspectRatio="none">
            <path d={path} fill="none" stroke={color} stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

// ── Gauge SVG ─────────────────────────────────────────────────────────────

fn gauge(pct: f64, color: &str) -> impl IntoView {
    let r = 54.0_f64;
    let circumference = std::f64::consts::PI * r; // half circle
    let dash = pct / 100.0 * circumference;
    let gap = circumference - dash;
    let color = color.to_string();
    view! {
        <div class="relative flex items-center justify-center" style="width:128px;height:72px;overflow:hidden;">
            <svg viewBox="0 0 128 72" style="width:128px;height:72px;">
                // Track
                <path d="M 10 64 A 54 54 0 0 1 118 64" fill="none" stroke="#F3F4F6" stroke-width="12" stroke-linecap="round"/>
                // Fill
                <path d="M 10 64 A 54 54 0 0 1 118 64" fill="none" stroke={color} stroke-width="12" stroke-linecap="round"
                    stroke-dasharray={format!("{:.2} {:.2}", dash, gap)}/>
            </svg>
            <div class="absolute bottom-0 inset-x-0 text-center pb-1">
                <span class="text-xl font-black text-gray-900">{format!("{:.0}%", pct)}</span>
            </div>
        </div>
    }
}

// ── Product card ──────────────────────────────────────────────────────────

struct Product {
    name: &'static str,
    category: &'static str,
    stock: u32,
    sold: u32,
    price: &'static str,
    color: &'static str,
}

struct StaffAttendance {
    name: &'static str,
    role: &'static str,
    time: &'static str,
    status: &'static str,
}

struct TopSeller {
    name: &'static str,
    pct: u32,
    color: &'static str,
}

// ── Page ──────────────────────────────────────────────────────────────────

#[component]
pub fn AdminDashboardPage() -> impl IntoView {
    let revenue_toggle = RwSignal::new("Daily");

    let summary_resource = LocalResource::new(|| async move { get_dashboard_summary().await });
    let revenue_resource = LocalResource::new(|| async move { get_revenue_analytics().await });
    let low_stock_resource = LocalResource::new(|| async move { get_low_stock_items().await });
    let categories_resource = LocalResource::new(|| async move { get_category_breakdown().await });
    let invoices_resource = LocalResource::new(|| async move { get_recent_invoices().await });

    view! {
        <div class="p-6 space-y-6">

            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Dashboard"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Welcome back, here's what's happening today"</p>
                </div>
                <button
                    on:click=move |_| {
                        let _ = Action::new_local(|_: &()| async move {
                             client::analytics::export_order_report().await
                        }).dispatch(());
                        if let Some(w) = web_sys::window() {
                            let _ = w.alert_with_message("Exporting order report to CSV...");
                        }
                    }
                    class="flex items-center gap-1.5 bg-gray-900 hover:bg-black text-white text-xs font-bold px-4 py-2.5 rounded-xl transition-colors cursor-pointer shadow-sm"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v4"></path>
                        <polyline points="7 10 12 15 17 10"></polyline>
                        <line x1="12" y1="15" x2="12" y2="3"></line>
                    </svg>
                    "Export Report"
                </button>
            </div>

            // ── KPI cards ──────────────────────────────────────────────────
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                // Stock Accuracy
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100 flex items-center justify-between">
                    <div>
                        <p class="text-xs text-gray-500 font-medium uppercase tracking-wider">"Stock Accuracy"</p>
                        {move || match summary_resource.get() {
                            Some(res) => match &*res {
                                Ok(s) => view! { <p class="text-3xl font-black text-gray-900 mt-1">{format!("{:.0}%", s.stock_accuracy)}</p> }.into_any(),
                                _ => view! { <p class="text-3xl font-black text-gray-900 mt-1">"..."</p> }.into_any()
                            },
                            _ => view! { <p class="text-3xl font-black text-gray-900 mt-1">"..."</p> }.into_any()
                        }}
                        <div class="flex items-center gap-1 mt-1">
                            <span class="text-emerald-500 text-xs font-semibold flex items-center gap-0.5">
                                {icon_trending_up()} "+1.2%"
                            </span>
                            <span class="text-xs text-gray-400">"vs last month"</span>
                        </div>
                    </div>
                    {sparkline(&[88.0, 90.0, 94.0, 92.0, 96.0, 97.0, 98.0], "#6366F1")}
                </div>

                // Today's Orders
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100 flex items-center justify-between">
                    <div>
                        <p class="text-xs text-gray-500 font-medium uppercase tracking-wider">"Today Orders"</p>
                        {move || match summary_resource.get() {
                            Some(res) => match &*res {
                                Ok(s) => view! { <p class="text-3xl font-black text-gray-900 mt-1">{s.today_orders}</p> }.into_any(),
                                _ => view! { <p class="text-3xl font-black text-gray-900 mt-1">"..."</p> }.into_any()
                            },
                            _ => view! { <p class="text-3xl font-black text-gray-900 mt-1">"..."</p> }.into_any()
                        }}
                        <div class="flex items-center gap-1 mt-1">
                            <span class="text-emerald-500 text-xs font-semibold flex items-center gap-0.5">
                                {icon_trending_up()} "+3.8%"
                            </span>
                            <span class="text-xs text-gray-400">"vs yesterday"</span>
                        </div>
                    </div>
                    {sparkline(&[42.0, 55.0, 48.0, 63.0, 72.0, 68.0, 80.0], "#F59E0B")}
                </div>

                // New Products
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100 flex items-center justify-between">
                    <div>
                        <p class="text-xs text-gray-500 font-medium uppercase tracking-wider">"New Products"</p>
                        {move || match summary_resource.get() {
                            Some(res) => match &*res {
                                Ok(s) => view! { <p class="text-3xl font-black text-gray-900 mt-1">{s.new_products_this_week}</p> }.into_any(),
                                _ => view! { <p class="text-3xl font-black text-gray-900 mt-1">"..."</p> }.into_any()
                            },
                            _ => view! { <p class="text-3xl font-black text-gray-900 mt-1">"..."</p> }.into_any()
                        }}
                        <div class="flex items-center gap-1 mt-1">
                            <span class="text-emerald-500 text-xs font-semibold flex items-center gap-0.5">
                                {icon_trending_up()} "+8"
                            </span>
                            <span class="text-xs text-gray-400">"this week"</span>
                        </div>
                    </div>
                    {sparkline(&[12.0, 18.0, 14.0, 22.0, 28.0, 35.0, 45.0], "#10B981")}
                </div>
            </div>

            // ── Middle row ─────────────────────────────────────────────────
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">

                // Product gallery (2/3)
                <div class="lg:col-span-2 bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <div class="flex items-center justify-between mb-4">
                        <div>
                            <h2 class="text-sm font-bold text-gray-900">"Total Product in Stock"</h2>
                            <p class="text-xs text-gray-400 mt-0.5">"4 categories · 834 units"</p>
                        </div>
                        <a href="/admin/suppliers" class="flex items-center gap-1 text-xs text-gray-500 hover:text-gray-900 transition-colors cursor-pointer font-medium">
                            "See more" {icon_arrow_right()}
                        </a>
                    </div>
                    <div class="space-y-3">
                        <Suspense fallback=move || view! { <p>"Loading products..."</p> }>
                            {move || match low_stock_resource.get() {
                                Some(sw) => {
                                    let res = &*sw;
                                    match res {
                                        Ok(items) => items.iter().map(|p| {
                                            let p_name = p.product_name.clone();
                                            let v_name = p.variant_name.clone();
                                            let initial = p_name.chars().next().unwrap_or('?').to_string();
                                            let stock = p.current_stock;
                                            view! {
                                                <div class="flex items-center gap-4 p-3 rounded-xl bg-gray-50 hover:bg-gray-100 transition-colors cursor-pointer group">
                                                    <div class="w-10 h-10 rounded-xl flex-shrink-0 flex items-center justify-center text-white text-xs font-bold bg-rose-500">
                                                        {initial}
                                                    </div>
                                                    <div class="flex-1 min-w-0">
                                                        <p class="text-xs font-semibold text-gray-900 truncate">{p_name}</p>
                                                        <div class="flex items-center gap-3 mt-1">
                                                            <span class="text-[11px] text-gray-400">{v_name}</span>
                                                            <span class="text-[11px] text-rose-500 font-bold">"Stock: " {stock}</span>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_any(),
                                        _ => view! { <p>"No low stock items found"</p> }.into_any()
                                    }
                                },
                                None => view! { <p>"Loading..."</p> }.into_any()
                            }}
                        </Suspense>
                    </div>
                </div>

                // Right column: Top Sellers + Live Revenue
                <div class="space-y-4">
                    // Top Selling
                    <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                        <div class="flex items-center justify-between mb-3">
                            <h2 class="text-sm font-bold text-gray-900">"Top Selling"</h2>
                            <div class="flex items-center gap-1">
                                <span class="w-1.5 h-1.5 rounded-full bg-[#FCE300] inline-block"></span>
                                <span class="text-[11px] text-amber-500 font-semibold">"142"</span>
                            </div>
                        </div>
                        <div class="space-y-3">
                            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                                {move || match categories_resource.get() {
                                    Some(sw) => {
                                        let res = &*sw;
                                        match res {
                                            Ok(items) => items.iter().map(|s| {
                                                let c_name = s.category_name.clone();
                                                let pct = s.sales_percentage;
                                                let bg = format!("width:{}%; background:{}", pct, s.color);
                                                view! {
                                                    <div class="space-y-1">
                                                        <div class="flex items-center justify-between text-xs">
                                                            <span class="text-gray-700 font-medium">{c_name}</span>
                                                            <span class="text-gray-900 font-bold">{pct}"%"</span>
                                                        </div>
                                                        <div class="h-1.5 bg-gray-100 rounded-full overflow-hidden">
                                                            <div class="h-full rounded-full" style=bg></div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view().into_any(),
                                            _ => view! { <p>"No category data"</p> }.into_any()
                                        }
                                    },
                                    None => view! { <p>"Loading..."</p> }.into_any()
                                }}
                            </Suspense>
                        </div>
                    </div>

                    // Live Revenue
                    <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                        <div class="flex items-center justify-between mb-2">
                            <h2 class="text-sm font-bold text-gray-900">"Live Revenue"</h2>
                            <div class="flex items-center bg-gray-100 rounded-lg p-0.5">
                                {["Daily", "Monthly"].iter().map(|&t| {
                                    let t_str = t;
                                    view! {
                                        <button
                                            class=move || format!("px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all duration-150 cursor-pointer {}",
                                                if revenue_toggle.get() == t_str { "bg-white text-gray-900 shadow-sm" } else { "text-gray-500" })
                                            on:click=move |_| revenue_toggle.set(t_str)
                                        >{t}</button>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                        <div class="flex items-baseline gap-2 mb-1">
                            {move || match summary_resource.get() {
                                Some(res) => match &*res {
                                    Ok(s) => view! { <span class="text-2xl font-black text-gray-900">{format!("₫{}k", s.total_revenue / 1000)}</span> }.into_any(),
                                    _ => view! { <span class="text-2xl font-black text-gray-900">"₫..."</span> }.into_any()
                                },
                                _ => view! { <span class="text-2xl font-black text-gray-900">"₫..."</span> }.into_any()
                            }}
                            <span class="text-xs text-emerald-500 font-semibold">"+5.4% vs yesterday"</span>
                        </div>
                        // Line chart
                        <div class="mt-3">
                            <Suspense fallback=move || view! { <p>"Loading chart..."</p> }>
                                {move || match revenue_resource.get() {
                                    Some(res) => match &*res {
                                        Ok(points) => {
                                            let data: Vec<f64> = points.iter().map(|p| p.value as f64).collect();
                                            view! { {sparkline(&data, "#6366F1")} }.into_any()
                                        },
                                        _ => view! { <p>"Chart error"</p> }.into_any()
                                    },
                                    None => view! { <p>"Loading..."</p> }.into_any()
                                }}
                            </Suspense>
                        </div>
                        // Legend
                        <div class="flex items-center gap-3 mt-2">
                            {[("Food", "#6366F1"), ("Toys", "#F59E0B"), ("Medicine", "#F43F5E")].iter().map(|&(name, color)| view! {
                                <div class="flex items-center gap-1">
                                    <span class="w-2 h-2 rounded-full flex-shrink-0" style={format!("background:{}", color)}></span>
                                    <span class="text-[10px] text-gray-500">{name}</span>
                                </div>
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </div>

            // ── Bottom row ─────────────────────────────────────────────────
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">

                // Staff attendance
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-sm font-bold text-gray-900">"Today's Attendance"</h2>
                        <a href="/admin/staff" class="flex items-center gap-1 text-xs text-gray-500 hover:text-gray-900 transition-colors cursor-pointer">
                            "See more" {icon_arrow_right()}
                        </a>
                    </div>
                    <div class="space-y-3">
                        <Suspense fallback=move || view! { <p>"Loading invoices..."</p> }>
                            {move || match invoices_resource.get() {
                                Some(sw) => {
                                    let res = &*sw;
                                    match res {
                                        Ok(items) => items.iter().map(|inv| {
                                            let email = inv.customer_email.clone();
                                            let initial = email.chars().next().unwrap_or('?').to_string();
                                            let order_id_short = format!("#{}", &inv.order_id.to_string()[..8]);
                                            let total_k = format!("₫{}k", inv.total/1000);
                                            let status = inv.status.clone();
                                            view! {
                                                <div class="flex items-center gap-3">
                                                    <div class="w-8 h-8 rounded-full bg-indigo-500 flex items-center justify-center text-white text-[10px] font-bold flex-shrink-0">
                                                        {initial}
                                                    </div>
                                                    <div class="flex-1 min-w-0">
                                                        <p class="text-xs font-semibold text-gray-900 truncate">{email}</p>
                                                        <p class="text-[11px] text-gray-400 truncate">{order_id_short}</p>
                                                    </div>
                                                    <span class="text-[11px] text-gray-900 font-bold flex-shrink-0">{total_k}</span>
                                                    <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full border bg-emerald-50 text-emerald-600 border-emerald-100 flex-shrink-0">
                                                        {status.to_string()}
                                                    </span>
                                                </div>
                                            }
                                        }).collect_view().into_any(),
                                        _ => view! { <p>"No recent invoices"</p> }.into_any()
                                    }
                                },
                                None => view! { <p>"Loading..."</p> }.into_any()
                            }}
                        </Suspense>
                    </div>
                </div>

                // Monthly target — performance gauge
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-sm font-bold text-gray-900">"Monthly Target — Staff Performance"</h2>
                        <button class="text-gray-400 hover:text-gray-700 transition-colors cursor-pointer">{icon_more()}</button>
                    </div>

                    <div class="flex items-center justify-between">
                        <div class="flex flex-col items-center">
                            {gauge(88.0, "#FCE300")}
                            <p class="text-xs text-gray-500 mt-1">"Team Average"</p>
                            <p class="text-[11px] text-emerald-500 font-semibold">"+4.5% vs last month"</p>
                        </div>

                        // Top performers
                        <div class="flex-1 pl-6 space-y-3">
                            <p class="text-xs font-bold text-gray-700 mb-2">"Top Performers"</p>
                            {[
                                ("Nguyễn Văn An", 1920, "+1658"),
                                ("Trần Thị Bình",  1710, "+899"),
                                ("Phạm Thu Dung",  1540, "+712"),
                            ].iter().map(|&(name, target, delta)| view! {
                                <div class="flex items-center gap-2">
                                    <div class="w-6 h-6 rounded-full bg-gradient-to-br from-amber-400 to-orange-500 flex items-center justify-center text-white text-[9px] font-bold flex-shrink-0">
                                        {name.chars().next().unwrap_or('?').to_string()}
                                    </div>
                                    <div class="flex-1">
                                        <p class="text-[11px] font-semibold text-gray-900">{name}</p>
                                        <p class="text-[10px] text-gray-400">"Target: " {target} " pcs"</p>
                                    </div>
                                    <span class="text-[11px] text-emerald-500 font-bold">{delta}</span>
                                </div>
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}
