use leptos::prelude::*;

// ─── Colour tokens (dark theme) ────────────────────────────────────────────
// bg-[#0B1120]  slate-950-ish
// bg-[#0F172A]  slate-900
// bg-[#1E293B]  slate-800
// text-[#94A3B8]
// accent-blue   #3B82F6   (blue-500)
// accent-amber  #F59E0B   (amber-500)
// accent-emerald#10B981
// accent-rose   #F43F5E

// ─── Icon helpers ──────────────────────────────────────────────────────────

fn icon_trending_up() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 6 13.5 15.5 8.5 10.5 1 18"></polyline>
            <polyline points="17 6 23 6 23 12"></polyline>
        </svg>
    }
}

fn icon_trending_down() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 18 13.5 8.5 8.5 13.5 1 6"></polyline>
            <polyline points="17 18 23 18 23 12"></polyline>
        </svg>
    }
}

fn icon_cart() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="9" cy="21" r="1"></circle>
            <circle cx="20" cy="21" r="1"></circle>
            <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path>
        </svg>
    }
}

fn icon_users() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
    }
}

fn icon_dollar() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="1" x2="12" y2="23"></line>
            <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
        </svg>
    }
}

fn icon_package() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <line x1="16.5" y1="9.4" x2="7.5" y2="4.21"></line>
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
            <line x1="12" y1="22.08" x2="12" y2="12"></line>
        </svg>
    }
}

fn icon_activity() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
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
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="5" y1="12" x2="19" y2="12"></line>
            <polyline points="12 5 19 12 12 19"></polyline>
        </svg>
    }
}

// ─── Sparkline SVG ─────────────────────────────────────────────────────────

fn sparkline(points: &[f64], color: &str, fill: &str) -> impl IntoView {
    let w = 120.0_f64;
    let h = 40.0_f64;
    let min = points.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1.0);
    let n = points.len();

    let path: String = points
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = i as f64 / (n - 1) as f64 * w;
            let y = h - (v - min) / range * (h - 6.0) - 3.0;
            if i == 0 {
                format!("M {:.1} {:.1}", x, y)
            } else {
                format!(" L {:.1} {:.1}", x, y)
            }
        })
        .collect();

    let area = format!("{} L {:.1} {:.1} L 0 {:.1} Z", path, w, h, h);

    let color = color.to_string();
    let fill = fill.to_string();

    view! {
        <svg viewBox=format!("0 0 {} {}", w, h) class="w-full h-10" preserveAspectRatio="none">
            <path d=area fill=fill opacity="0.25" />
            <path d=path fill="none" stroke=color stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
    }
}

// ─── Donut chart ────────────────────────────────────────────────────────────

fn donut_chart(segments: Vec<(&'static str, f64, &'static str)>) -> impl IntoView {
    let total: f64 = segments.iter().map(|(_, v, _)| v).sum();
    let _cx = 60.0_f64;
    let _cy = 60.0_f64;
    let r = 48.0_f64;
    let circumference = 2.0 * std::f64::consts::PI * r;

    let mut offset = 0.0_f64;
    let arcs: Vec<_> = segments
        .iter()
        .map(|(label, val, color)| {
            let fraction = val / total;
            let dash = fraction * circumference;
            let gap = circumference - dash;
            let rotate = offset / total * 360.0 - 90.0;
            offset += val;
            (*label, *val, *color, dash, gap, rotate)
        })
        .collect();

    view! {
        <div class="flex items-center gap-6">
            <div class="relative flex-shrink-0 w-[120px] h-[120px]">
                <svg viewBox="0 0 120 120" class="w-full h-full -rotate-90">
                    {arcs.iter().map(|(_, _, color, dash, gap, rotate)| {
                        let d = *dash;
                        let g = *gap;
                        let ro = *rotate;
                        let col = *color;
                        view! {
                            <circle
                                cx="60" cy="60" r="48"
                                fill="none"
                                stroke=col
                                stroke-width="14"
                                stroke-dasharray=format!("{:.2} {:.2}", d, g)
                                style=format!("transform-origin: 60px 60px; transform: rotate({:.1}deg);", ro)
                            />
                        }
                    }).collect_view()}
                    // Hole
                    <circle cx="60" cy="60" r="34" fill="#0F172A" />
                </svg>
                // Centre label
                <div class="absolute inset-0 flex flex-col items-center justify-center">
                    <span class="text-xs text-slate-400 font-medium">"Total"</span>
                    <span class="text-sm font-bold text-white font-[Fira_Code,monospace]">{format!("{:.0}", total)}</span>
                </div>
            </div>
            // Legend
            <ul class="space-y-2 flex-1">
                {arcs.iter().map(|(label, val, color, _, _, _)| {
                    let pct = val / total * 100.0;
                    let col = *color;
                    view! {
                        <li class="flex items-center justify-between gap-3 text-xs">
                            <span class="flex items-center gap-1.5 text-slate-400">
                                <span class="w-2 h-2 rounded-full flex-shrink-0" style=format!("background:{}", col)></span>
                                {*label}
                            </span>
                            <span class="text-white font-medium font-[Fira_Code,monospace]">{format!("{:.0}%", pct)}</span>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}

// ─── Bar chart ──────────────────────────────────────────────────────────────

fn bar_chart(data: Vec<(&'static str, f64, f64)>) -> impl IntoView {
    let max = data.iter().map(|(_, v, _)| *v).fold(0.0_f64, f64::max);
    view! {
        <div class="space-y-3">
            {data.into_iter().map(|(label, val, prev)| {
                let pct = (val / max * 100.0) as u32;
                let delta = val - prev;
                let up = delta >= 0.0;
                view! {
                    <div class="space-y-1">
                        <div class="flex items-center justify-between text-xs">
                            <span class="text-slate-400">{label}</span>
                            <div class="flex items-center gap-2">
                                <span class=move || format!("flex items-center gap-0.5 {}", if up { "text-emerald-400" } else { "text-rose-400" })>
                                    {if up { icon_trending_up().into_any() } else { icon_trending_down().into_any() }}
                                    {format!("{:+.0}", delta)}
                                </span>
                                <span class="text-white font-medium font-[Fira_Code,monospace]">{format!("{:.0}", val)}</span>
                            </div>
                        </div>
                        <div class="h-1.5 bg-slate-700 rounded-full overflow-hidden">
                            <div
                                class="h-full bg-gradient-to-r from-blue-500 to-blue-400 rounded-full transition-all duration-700"
                                style=format!("width: {}%", pct)
                            ></div>
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

// ─── Stat card ──────────────────────────────────────────────────────────────

struct StatCard {
    label: &'static str,
    value: &'static str,
    delta: f64,
    sub: &'static str,
    color: &'static str, // text-color class for Tailwind
    bg: &'static str,    // bg-color class
    points: Vec<f64>,
    spark_color: &'static str,
    spark_fill: &'static str,
}

// ─── Orders data ────────────────────────────────────────────────────────────

struct Order {
    id: &'static str,
    customer: &'static str,
    product: &'static str,
    amount: &'static str,
    status: &'static str,
    date: &'static str,
}

// ─── Page ──────────────────────────────────────────────────────────────────

#[component]
pub fn DashboardPage() -> impl IntoView {
    let active_period = RwSignal::new("7d");

    let stats = vec![
        StatCard {
            label: "Doanh thu",
            value: "₫128.4M",
            delta: 12.5,
            sub: "so với tháng trước",
            color: "text-blue-400",
            bg: "bg-blue-500/10",
            points: vec![
                40.0, 55.0, 42.0, 68.0, 58.0, 72.0, 85.0, 91.0, 78.0, 95.0, 88.0, 110.0,
            ],
            spark_color: "#3B82F6",
            spark_fill: "#3B82F6",
        },
        StatCard {
            label: "Đơn hàng",
            value: "2,847",
            delta: 8.1,
            sub: "so với tháng trước",
            color: "text-emerald-400",
            bg: "bg-emerald-500/10",
            points: vec![
                20.0, 35.0, 28.0, 45.0, 40.0, 55.0, 48.0, 62.0, 58.0, 70.0, 65.0, 80.0,
            ],
            spark_color: "#10B981",
            spark_fill: "#10B981",
        },
        StatCard {
            label: "Khách hàng",
            value: "14,920",
            delta: -3.2,
            sub: "so với tháng trước",
            color: "text-amber-400",
            bg: "bg-amber-500/10",
            points: vec![
                80.0, 75.0, 82.0, 78.0, 70.0, 65.0, 72.0, 68.0, 60.0, 58.0, 55.0, 52.0,
            ],
            spark_color: "#F59E0B",
            spark_fill: "#F59E0B",
        },
        StatCard {
            label: "Sản phẩm bán",
            value: "6,312",
            delta: 5.7,
            sub: "so với tháng trước",
            color: "text-violet-400",
            bg: "bg-violet-500/10",
            points: vec![
                30.0, 38.0, 33.0, 50.0, 44.0, 58.0, 52.0, 65.0, 60.0, 75.0, 68.0, 82.0,
            ],
            spark_color: "#8B5CF6",
            spark_fill: "#8B5CF6",
        },
    ];

    let orders = vec![
        Order {
            id: "#ORD-8821",
            customer: "Nguyễn Văn A",
            product: "Áo Thun Modal AirDry Trắng",
            amount: "₫167.450",
            status: "Đã giao",
            date: "11/03/2026",
        },
        Order {
            id: "#ORD-8820",
            customer: "Trần Thị B",
            product: "Áo Sơ Mi Non-Iron Xanh",
            amount: "₫234.650",
            status: "Đang giao",
            date: "11/03/2026",
        },
        Order {
            id: "#ORD-8819",
            customer: "Lê Văn C",
            product: "Áo Khoác Worker Xám Nhạt",
            amount: "₫757.150",
            status: "Xử lý",
            date: "10/03/2026",
        },
        Order {
            id: "#ORD-8818",
            customer: "Phạm Thị D",
            product: "Áo Polo Raglan FlexFit",
            amount: "₫263.150",
            status: "Đã giao",
            date: "10/03/2026",
        },
        Order {
            id: "#ORD-8817",
            customer: "Hoàng Văn E",
            product: "Áo Thun Boxy AirDry Đen",
            amount: "₫263.150",
            status: "Huỷ",
            date: "09/03/2026",
        },
        Order {
            id: "#ORD-8816",
            customer: "Đặng Thị F",
            product: "Áo Sơ Mi Modal Trắng",
            amount: "₫167.450",
            status: "Đã giao",
            date: "09/03/2026",
        },
    ];

    let top_cats = vec![
        ("Áo Thun", 420.0, 380.0),
        ("Áo Sơ Mi", 310.0, 295.0),
        ("Áo Polo", 268.0, 270.0),
        ("Áo Khoác", 145.0, 130.0),
    ];

    view! {
        // ── Dark shell ───────────────────────────────────────────────────
        <div class="min-h-screen bg-[#0B1120] text-slate-100 font-[Montserrat,system-ui,sans-serif]">
            <div class="max-w-screen-2xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">

                // ── Header ───────────────────────────────────────────────
                <div class="flex items-start justify-between gap-4 flex-wrap">
                    <div>
                        <div class="flex items-center gap-2 mb-1">
                            // Live indicator
                            <div class="relative flex items-center gap-1.5">
                                <span class="relative flex h-2 w-2">
                                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                                </span>
                                <span class="text-xs text-emerald-400 font-medium">"Live"</span>
                            </div>
                        </div>
                        <h1 class="text-2xl font-bold text-white">"Dashboard"</h1>
                        <p class="text-sm text-slate-400 mt-0.5">"Tổng quan hoạt động kinh doanh — cập nhật liên tục"</p>
                    </div>
                    // Period switcher
                    <div class="flex items-center gap-1 bg-slate-800/60 border border-slate-700/50 rounded-lg p-1">
                        {["24h", "7d", "30d", "90d"].iter().map(|&p| {
                            view! {
                                <button
                                    class=move || format!(
                                        "px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-150 cursor-pointer {}",
                                        if active_period.get() == p {
                                            "bg-blue-600 text-white shadow"
                                        } else {
                                            "text-slate-400 hover:text-white"
                                        }
                                    )
                                    on:click=move |_| active_period.set(p)
                                >
                                    {p}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>

                // ── Stat cards ───────────────────────────────────────────
                <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
                    {stats.into_iter().enumerate().map(|(i, card)| {
                        let up = card.delta >= 0.0;
                        let points = card.points.clone();
                        let spark_color = card.spark_color;
                        let spark_fill = card.spark_fill;
                        view! {
                            <div
                                class="bg-[#0F172A] border border-slate-700/50 rounded-xl p-5 space-y-3 hover:border-slate-600/80 transition-colors duration-200 animate-fadeInUp"
                                style=format!("animation-delay: {}ms", i * 80)
                            >
                                // Top row: label + icon
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="text-xs text-slate-400 font-medium uppercase tracking-wider">{card.label}</p>
                                        <p class="mt-1 text-2xl font-bold text-white font-[Fira_Code,monospace]">{card.value}</p>
                                    </div>
                                    <div class=format!("w-10 h-10 rounded-lg {} flex items-center justify-center flex-shrink-0 {}", card.bg, card.color)>
                                        {if i == 0 { icon_dollar().into_any() }
                                         else if i == 1 { icon_cart().into_any() }
                                         else if i == 2 { icon_users().into_any() }
                                         else { icon_package().into_any() }}
                                    </div>
                                </div>
                                // Delta
                                <div class="flex items-center gap-1.5">
                                    <span class=move || format!("flex items-center gap-0.5 text-xs font-semibold {}", if up { "text-emerald-400" } else { "text-rose-400" })>
                                        {if up { icon_trending_up().into_any() } else { icon_trending_down().into_any() }}
                                        {format!("{:+.1}%", card.delta)}
                                    </span>
                                    <span class="text-xs text-slate-500">{card.sub}</span>
                                </div>
                                // Sparkline
                                <div class="pt-1">
                                    {sparkline(&points, spark_color, spark_fill)}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>

                // ── Middle row: Revenue bar chart + Donut ─────────────────
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">

                    // Revenue by day (bar chart style) ──────────────────
                    <div class="lg:col-span-2 bg-[#0F172A] border border-slate-700/50 rounded-xl p-6">
                        <div class="flex items-center justify-between mb-6">
                            <div>
                                <h2 class="text-sm font-semibold text-white">"Doanh thu theo ngày"</h2>
                                <p class="text-xs text-slate-400 mt-0.5">"7 ngày gần nhất"</p>
                            </div>
                            <span class="flex items-center gap-1 text-xs text-slate-400">
                                {icon_activity()}
                                "Đơn vị: nghìn ₫"
                            </span>
                        </div>
                        // Manual bar chart — 7 bars
                        <div class="flex items-end gap-2 h-40">
                            {[
                                ("T2", 72.0_f64, 100.0), ("T3", 88.0, 100.0), ("T4", 65.0, 100.0),
                                ("T5", 95.0, 100.0),      ("T6", 110.0, 100.0),("T7", 82.0, 100.0),
                                ("CN", 128.0, 100.0),
                            ].iter().map(|&(day, val, max)| {
                                let pct = (val / max * 100.0) as u32;
                                let is_today = day == "CN";
                                view! {
                                    <div class="flex-1 flex flex-col items-center gap-2">
                                        <span class="text-[10px] text-slate-400 font-[Fira_Code,monospace]">
                                            {format!("{:.0}k", val)}
                                        </span>
                                        <div class="w-full bg-slate-800 rounded-t-sm overflow-hidden" style="height: 100px;">
                                            <div
                                                class=move || format!(
                                                    "w-full rounded-t-sm transition-all duration-700 {}",
                                                    if is_today { "bg-gradient-to-t from-blue-600 to-blue-400" }
                                                    else { "bg-slate-700 hover:bg-slate-600" }
                                                )
                                                style=format!("height: {}%", pct)
                                            ></div>
                                        </div>
                                        <span class="text-[10px] text-slate-500">{day}</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    // Donut — Category share ─────────────────────────────
                    <div class="bg-[#0F172A] border border-slate-700/50 rounded-xl p-6">
                        <div class="flex items-center justify-between mb-6">
                            <div>
                                <h2 class="text-sm font-semibold text-white">"Phân loại sản phẩm"</h2>
                                <p class="text-xs text-slate-400 mt-0.5">"Tỷ lệ theo đơn hàng"</p>
                            </div>
                            <button class="text-slate-500 hover:text-white transition-colors cursor-pointer">
                                {icon_more()}
                            </button>
                        </div>
                        {donut_chart(vec![
                            ("Áo Thun",  420.0, "#3B82F6"),
                            ("Áo Sơ Mi", 310.0, "#10B981"),
                            ("Áo Polo",  268.0, "#F59E0B"),
                            ("Áo Khoác", 145.0, "#8B5CF6"),
                        ])}
                    </div>
                </div>

                // ── Bottom row: Orders table + Top products ───────────────
                <div class="grid grid-cols-1 xl:grid-cols-3 gap-4">

                    // Recent orders ──────────────────────────────────────
                    <div class="xl:col-span-2 bg-[#0F172A] border border-slate-700/50 rounded-xl overflow-hidden">
                        <div class="flex items-center justify-between px-6 py-4 border-b border-slate-700/50">
                            <h2 class="text-sm font-semibold text-white">"Đơn hàng gần đây"</h2>
                            <a href="/orders" class="flex items-center gap-1 text-xs text-blue-400 hover:text-blue-300 transition-colors cursor-pointer">
                                "Xem tất cả"
                                {icon_arrow_right()}
                            </a>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-xs">
                                <thead>
                                    <tr class="border-b border-slate-700/50 text-slate-500 uppercase tracking-wider text-[10px]">
                                        <th class="px-6 py-3 text-left font-medium">"Mã đơn"</th>
                                        <th class="px-3 py-3 text-left font-medium">"Khách hàng"</th>
                                        <th class="px-3 py-3 text-left font-medium hidden md:table-cell">"Sản phẩm"</th>
                                        <th class="px-3 py-3 text-right font-medium">"Số tiền"</th>
                                        <th class="px-3 py-3 text-center font-medium">"Trạng thái"</th>
                                        <th class="px-6 py-3 text-right font-medium hidden sm:table-cell">"Ngày"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-slate-700/30">
                                    {orders.into_iter().map(|order| {
                                        let (status_class, dot_class) = match order.status {
                                            "Đã giao"   => ("bg-emerald-500/10 text-emerald-400 border border-emerald-500/20", "bg-emerald-400"),
                                            "Đang giao" => ("bg-blue-500/10 text-blue-400 border border-blue-500/20",           "bg-blue-400"),
                                            "Xử lý"     => ("bg-amber-500/10 text-amber-400 border border-amber-500/20",         "bg-amber-400"),
                                            "Huỷ"       => ("bg-rose-500/10 text-rose-400 border border-rose-500/20",            "bg-rose-400"),
                                            _           => ("bg-slate-500/10 text-slate-400 border border-slate-500/20",         "bg-slate-400"),
                                        };
                                        view! {
                                            <tr class="hover:bg-slate-800/30 transition-colors duration-150">
                                                <td class="px-6 py-3.5 font-[Fira_Code,monospace] text-blue-400 whitespace-nowrap">{order.id}</td>
                                                <td class="px-3 py-3.5 text-slate-300 whitespace-nowrap">{order.customer}</td>
                                                <td class="px-3 py-3.5 text-slate-500 truncate max-w-[180px] hidden md:table-cell">{order.product}</td>
                                                <td class="px-3 py-3.5 text-right text-white font-semibold font-[Fira_Code,monospace] whitespace-nowrap">{order.amount}</td>
                                                <td class="px-3 py-3.5 text-center">
                                                    <span class=format!("inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium {}", status_class)>
                                                        <span class=format!("w-1 h-1 rounded-full flex-shrink-0 {}", dot_class)></span>
                                                        {order.status}
                                                    </span>
                                                </td>
                                                <td class="px-6 py-3.5 text-right text-slate-500 whitespace-nowrap hidden sm:table-cell">{order.date}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    </div>

                    // Top categories bar chart ───────────────────────────
                    <div class="bg-[#0F172A] border border-slate-700/50 rounded-xl p-6 space-y-5">
                        <div class="flex items-center justify-between">
                            <div>
                                <h2 class="text-sm font-semibold text-white">"Top danh mục"</h2>
                                <p class="text-xs text-slate-400 mt-0.5">"Đơn hàng theo danh mục"</p>
                            </div>
                            <button class="text-slate-500 hover:text-white transition-colors cursor-pointer">
                                {icon_more()}
                            </button>
                        </div>
                        {bar_chart(top_cats)}

                        // Quick metrics
                        <div class="mt-4 pt-4 border-t border-slate-700/50 grid grid-cols-2 gap-3">
                            <div class="bg-slate-800/50 rounded-lg p-3 text-center">
                                <p class="text-[10px] text-slate-500 mb-1">"Tỷ lệ hoàn"</p>
                                <p class="text-base font-bold text-rose-400 font-[Fira_Code,monospace]">"2.4%"</p>
                            </div>
                            <div class="bg-slate-800/50 rounded-lg p-3 text-center">
                                <p class="text-[10px] text-slate-500 mb-1">"Avg. đơn"</p>
                                <p class="text-base font-bold text-blue-400 font-[Fira_Code,monospace]">"₫284k"</p>
                            </div>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    }
}
