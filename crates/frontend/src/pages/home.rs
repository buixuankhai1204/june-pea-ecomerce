use leptos::prelude::*;

// ─── Home page — Yame.vn inspired storefront ──────────────────────────────
// Sections:
//   1. Hero banner (full-width, dark overlay, CTA)
//   2. Trust strip (3 USPs)
//   3. Category grid (4 cards)
//   4. Featured products row (4 mock cards)
//   5. Promo / brand story strip
//   6. Newsletter bar

// ─── Icon helpers ──────────────────────────────────────────────────────────

fn icon_truck() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <rect x="1" y="3" width="15" height="13"></rect>
            <polygon points="16 8 20 8 23 11 23 16 16 16 16 8"></polygon>
            <circle cx="5.5" cy="18.5" r="2.5"></circle>
            <circle cx="18.5" cy="18.5" r="2.5"></circle>
        </svg>
    }
}

fn icon_shield() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
        </svg>
    }
}

fn icon_refresh() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="23 4 23 10 17 10"></polyline>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"></path>
        </svg>
    }
}

fn icon_arrow_right() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <line x1="5" y1="12" x2="19" y2="12"></line>
            <polyline points="12 5 19 12 12 19"></polyline>
        </svg>
    }
}

fn icon_star() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24"
            fill="currentColor" stroke="none">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
        </svg>
    }
}

// ─── Page ──────────────────────────────────────────────────────────────────

#[component]
pub fn HomePage() -> impl IntoView {
    // Email signal for newsletter
    let email = RwSignal::new(String::new());
    let subscribed = RwSignal::new(false);

    let categories = vec![
        (
            "ÁO THUN",
            "Dẫn đầu công nghệ AirDry™",
            "from-slate-800 to-slate-700",
            "/products",
        ),
        (
            "ÁO SƠ MI",
            "Non-iron ít nhăn dễ ủi",
            "from-stone-800 to-stone-700",
            "/products",
        ),
        (
            "ÁO POLO",
            "Bền phom mặc bền quanh năm",
            "from-zinc-800 to-zinc-700",
            "/products",
        ),
        (
            "ÁO KHOÁC",
            "Đa năng cho mọi hành trình",
            "from-neutral-800 to-neutral-700",
            "/products",
        ),
    ];

    struct FeaturedProduct {
        name: &'static str,
        price: &'static str,
        orig: &'static str,
        badge: &'static str,
        slug: &'static str,
    }
    let featured = vec![
        FeaturedProduct {
            name: "Áo Thun Modal AirDry™ Trắng",
            price: "167.450 VND",
            orig: "197.000 VND",
            badge: "MỚI",
            slug: "ao-thun",
        },
        FeaturedProduct {
            name: "Áo Sơ Mi Non-Iron Non Branded Xanh",
            price: "234.650 VND",
            orig: "277.000 VND",
            badge: "SALE",
            slug: "ao-so-mi",
        },
        FeaturedProduct {
            name: "Áo Polo Raglan FlexFit™ Xám Nhạt",
            price: "263.150 VND",
            orig: "297.000 VND",
            badge: "MỚI",
            slug: "ao-polo",
        },
        FeaturedProduct {
            name: "Áo Khoác Kaki DurableTex™ Xám",
            price: "757.150 VND",
            orig: "797.000 VND",
            badge: "HOT",
            slug: "ao-khoac",
        },
    ];

    view! {
        <div class="bg-white font-[Montserrat,system-ui,sans-serif]">

            // ── 1. HERO ──────────────────────────────────────────────────
            <section class="relative overflow-hidden bg-[#111] min-h-[560px] sm:min-h-[640px] flex items-center">
                // Background texture gradient
                <div class="absolute inset-0 bg-gradient-to-br from-[#0a0a0a] via-[#181818] to-[#222]"></div>
                // Decorative blobs
                <div class="absolute -top-40 -right-40 w-[600px] h-[600px] rounded-full bg-blue-600/10 blur-3xl pointer-events-none"></div>
                <div class="absolute -bottom-40 -left-20 w-[400px] h-[400px] rounded-full bg-amber-500/8 blur-3xl pointer-events-none"></div>
                // Grid lines overlay
                <div class="absolute inset-0 opacity-[0.03]"
                    style="background-image: linear-gradient(#fff 1px, transparent 1px), linear-gradient(90deg, #fff 1px, transparent 1px); background-size: 60px 60px;">
                </div>

                <div class="relative z-10 max-w-screen-xl mx-auto px-4 sm:px-6 py-20 sm:py-28">
                    <div class="max-w-2xl">
                        // Pill badge
                        <div class="inline-flex items-center gap-2 bg-white/10 border border-white/20 rounded-full px-4 py-1.5 mb-8">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                            <span class="text-xs text-white/80 font-medium tracking-wide">"Bộ sưu tập mới — Spring 2026"</span>
                        </div>
                        <h1 class="text-4xl sm:text-5xl lg:text-6xl font-bold text-white leading-tight tracking-tight">
                            "Mặc Tốt,"
                            <br/>
                            <span class="text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-cyan-300">
                                "Sống Chuẩn"
                            </span>
                        </h1>
                        <p class="mt-6 text-base sm:text-lg text-white/60 leading-relaxed max-w-lg">
                            "Thời trang nam chất lượng cao — bền, mềm, thoáng khí. Được thiết kế riêng cho vóc dáng người Việt."
                        </p>
                        <div class="mt-10 flex flex-wrap gap-3">
                            <a href="/products"
                                class="inline-flex items-center gap-2 bg-white text-black font-semibold px-7 py-3.5 rounded-full hover:bg-gray-100 transition-colors duration-200 shadow-lg shadow-white/10 text-sm">
                                "Khám phá ngay"
                                {icon_arrow_right()}
                            </a>
                            <a href="/products"
                                class="inline-flex items-center gap-2 border border-white/25 text-white font-medium px-7 py-3.5 rounded-full hover:border-white/50 hover:bg-white/5 transition-all duration-200 text-sm">
                                "Xem ưu đãi SALE"
                            </a>
                        </div>
                        // Social proof
                        <div class="mt-12 flex items-center gap-6 flex-wrap">
                            <div class="flex -space-x-2">
                                {(0..5u8).map(|i| {
                                    let colors = ["bg-blue-500","bg-emerald-500","bg-amber-500","bg-rose-500","bg-violet-500"];
                                    view! {
                                        <div class=format!("w-8 h-8 rounded-full border-2 border-[#111] {} flex items-center justify-center text-white text-xs font-bold", colors[i as usize])>
                                            {(b'A' + i) as char}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="text-sm text-white/60">
                                <div class="flex items-center gap-1 text-amber-400 mb-0.5">
                                    {icon_star()}{icon_star()}{icon_star()}{icon_star()}{icon_star()}
                                </div>
                                <span>"Hơn 50,000 khách hàng hài lòng"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            // ── 2. TRUST STRIP ───────────────────────────────────────────
            <section class="border-b border-gray-100 bg-gray-50">
                <div class="max-w-screen-xl mx-auto px-4 sm:px-6">
                    <div class="grid grid-cols-1 sm:grid-cols-3 divide-y sm:divide-y-0 sm:divide-x divide-gray-200">
                        <div class="flex items-center gap-4 px-8 py-5">
                            <div class="w-11 h-11 rounded-full bg-black flex items-center justify-center text-white flex-shrink-0">
                                {icon_truck()}
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-black">"Miễn Phí Vận Chuyển"</p>
                                <p class="text-xs text-gray-500 mt-0.5">"Cho đơn hàng từ 300.000 VND"</p>
                            </div>
                        </div>
                        <div class="flex items-center gap-4 px-8 py-5">
                            <div class="w-11 h-11 rounded-full bg-black flex items-center justify-center text-white flex-shrink-0">
                                {icon_shield()}
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-black">"Bảo Hành 365 Ngày"</p>
                                <p class="text-xs text-gray-500 mt-0.5">"Đổi trả dễ dàng, no-question"</p>
                            </div>
                        </div>
                        <div class="flex items-center gap-4 px-8 py-5">
                            <div class="w-11 h-11 rounded-full bg-black flex items-center justify-center text-white flex-shrink-0">
                                {icon_refresh()}
                            </div>
                            <div>
                                <p class="text-sm font-semibold text-black">"30 Ngày Hoàn Hàng"</p>
                                <p class="text-xs text-gray-500 mt-0.5">"Không hài lòng — hoàn tiền"</p>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            // ── 3. CATEGORY GRID ─────────────────────────────────────────
            <section class="py-14 sm:py-20">
                <div class="max-w-screen-xl mx-auto px-4 sm:px-6">
                    <div class="flex items-end justify-between mb-8">
                        <div>
                            <p class="text-xs font-semibold text-gray-400 uppercase tracking-widest mb-1">"Danh mục"</p>
                            <h2 class="text-2xl sm:text-3xl font-bold text-black">"Tất cả dòng áo"</h2>
                        </div>
                        <a href="/products" class="hidden sm:flex items-center gap-1.5 text-sm font-medium text-gray-500 hover:text-black transition-colors">
                            "Xem tất cả"
                            {icon_arrow_right()}
                        </a>
                    </div>

                    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
                        {categories.into_iter().map(|(label, desc, gradient, href)| {
                            view! {
                                <a href=href class="group relative block rounded-2xl overflow-hidden aspect-[3/4] cursor-pointer">
                                    // Dark gradient bg simulating image
                                    <div class=format!("absolute inset-0 bg-gradient-to-br {} group-hover:scale-105 transition-transform duration-700 ease-out", gradient)></div>
                                    // Grain texture overlay
                                    <div class="absolute inset-0 opacity-30"
                                        style="background-image: url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='0.4'/%3E%3C/svg%3E\"); background-size: 128px;">
                                    </div>
                                    // Bottom card info
                                    <div class="absolute inset-x-0 bottom-0 p-5 bg-gradient-to-t from-black/80 to-transparent">
                                        <span class="inline-block bg-white/15 border border-white/25 text-white text-[11px] font-bold px-3 py-1 rounded-full tracking-widest mb-2">
                                            {label}
                                        </span>
                                        <p class="text-white/70 text-xs leading-snug">{desc}</p>
                                    </div>
                                    // Hover arrow
                                    <div class="absolute top-4 right-4 w-8 h-8 rounded-full bg-white/0 border border-white/0 flex items-center justify-center text-white opacity-0 group-hover:opacity-100 group-hover:bg-white/15 group-hover:border-white/30 transition-all duration-300">
                                        {icon_arrow_right()}
                                    </div>
                                </a>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // ── 4. FEATURED PRODUCTS ──────────────────────────────────────
            <section class="pb-14 sm:pb-20 bg-[#FAFAF9]">
                <div class="max-w-screen-xl mx-auto px-4 sm:px-6">
                    <div class="flex items-end justify-between py-10">
                        <div>
                            <p class="text-xs font-semibold text-gray-400 uppercase tracking-widest mb-1">"Nổi bật"</p>
                            <h2 class="text-2xl sm:text-3xl font-bold text-black">"Sản phẩm bán chạy"</h2>
                        </div>
                        <a href="/products" class="hidden sm:flex items-center gap-1.5 text-sm font-medium text-gray-500 hover:text-black transition-colors">
                            "Xem tất cả"
                            {icon_arrow_right()}
                        </a>
                    </div>

                    <div class="grid grid-cols-2 lg:grid-cols-4 gap-x-4 gap-y-8">
                        {featured.into_iter().map(|p| {
                            let badge_class = match p.badge {
                                "SALE" => "bg-red-500 text-white",
                                "HOT"  => "bg-amber-500 text-white",
                                _      => "bg-[#2D3748] text-white",
                            };
                            view! {
                                <a href=format!("/products/{}", p.slug) class="group block cursor-pointer">
                                    // Image area
                                    <div class="relative bg-gray-100 aspect-[3/4] rounded-xl overflow-hidden mb-3">
                                        // Badge
                                        <div class=format!("absolute top-3 left-3 z-10 text-[10px] font-bold px-2.5 py-1 rounded-full {}", badge_class)>
                                            {p.badge}
                                        </div>
                                        // Placeholder illustration
                                        <div class="w-full h-full flex items-center justify-center bg-gradient-to-b from-gray-50 to-gray-100 group-hover:scale-[1.03] transition-transform duration-500 ease-out">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-20 h-20 text-gray-200" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="0.6">
                                                <path d="M20.38 3.46 16 2a4 4 0 0 1-8 0L3.62 3.46a2 2 0 0 0-1.34 2.23l.58 3.57a1 1 0 0 0 .99.84H6v10c0 1.1.9 2 2 2h8a2 2 0 0 0 2-2V10h2.15a1 1 0 0 0 .99-.84l.58-3.57a2 2 0 0 0-1.34-2.23z"></path>
                                            </svg>
                                        </div>
                                        // "Miễn Phí Ship" pill at bottom
                                        <div class="absolute bottom-3 left-3">
                                            <span class="bg-white border border-gray-200 text-gray-600 text-[10px] font-medium px-2 py-0.5 rounded-sm shadow-sm">
                                                "Miễn Phí Ship"
                                            </span>
                                        </div>
                                        // Quick-view on hover
                                        <div class="absolute inset-x-3 bottom-10 opacity-0 group-hover:opacity-100 transition-all duration-300 translate-y-2 group-hover:translate-y-0">
                                            <button class="w-full bg-black text-white text-xs font-semibold py-2 rounded-lg text-center cursor-pointer">
                                                "Xem nhanh"
                                            </button>
                                        </div>
                                    </div>
                                    // Info
                                    <div>
                                        <h3 class="text-sm text-gray-800 leading-snug line-clamp-2 mb-1.5 group-hover:text-gray-500 transition-colors">
                                            {p.name}
                                        </h3>
                                        <div class="flex items-center gap-2 flex-wrap">
                                            <span class="text-gray-400 line-through text-xs">{p.orig}</span>
                                            <span class="text-black font-bold text-sm">{p.price}</span>
                                        </div>
                                        // Stars
                                        <div class="flex items-center gap-0.5 mt-1.5 text-amber-400">
                                            {icon_star()}{icon_star()}{icon_star()}{icon_star()}{icon_star()}
                                            <span class="text-[10px] text-gray-400 ml-1">"(128)"</span>
                                        </div>
                                    </div>
                                </a>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // ── 5. BRAND STORY / MINI PROMO ──────────────────────────────
            <section class="bg-black text-white py-16 sm:py-24 overflow-hidden relative">
                <div class="absolute inset-0 bg-gradient-to-r from-black via-black to-blue-950/40 pointer-events-none"></div>
                <div class="relative max-w-screen-xl mx-auto px-4 sm:px-6 grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
                    <div>
                        <p class="text-xs font-semibold text-blue-400 uppercase tracking-widest mb-4">"Công nghệ vải"</p>
                        <h2 class="text-3xl sm:text-4xl font-bold leading-tight">
                            "AirDry™ — Thoáng Khí,"
                            <br/>
                            "Mặc Cả Ngày"
                        </h2>
                        <p class="mt-5 text-white/60 leading-relaxed text-sm max-w-lg">
                            "Công nghệ vải AirDry™ độc quyền giúp thoát nhiệt nhanh, không thấm mồ hôi, giữ form dáng bền bỉ sau hàng trăm lần giặt. Đây là lý do hơn 50,000 khách hàng chọn YAME."
                        </p>
                        <ul class="mt-8 space-y-3">
                            {["Thoáng khí 3× so với cotton thường", "Chống nhăn — ít phải ủi", "Bền màu đến 300 lần giặt"].iter().map(|feat| {
                                view! {
                                    <li class="flex items-center gap-3 text-sm text-white/80">
                                        <span class="w-5 h-5 rounded-full bg-blue-500/20 border border-blue-500/40 flex items-center justify-center flex-shrink-0">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                        </span>
                                        {*feat}
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                        <a href="/products"
                            class="inline-flex items-center gap-2 mt-10 bg-white text-black font-semibold px-7 py-3.5 rounded-full hover:bg-gray-100 transition-colors text-sm shadow-lg">
                            "Khám phá AirDry™"
                            {icon_arrow_right()}
                        </a>
                    </div>
                    // Stats column
                    <div class="grid grid-cols-2 gap-4">
                        {[
                            ("50K+", "Khách hàng tin dùng"),
                            ("4.9★", "Điểm đánh giá trung bình"),
                            ("365",  "Ngày bảo hành"),
                            ("<2h",  "Xử lý đơn trong giờ hành chính"),
                        ].iter().map(|(num, label)| {
                            view! {
                                <div class="bg-white/5 border border-white/10 rounded-2xl p-6">
                                    <p class="text-3xl font-bold text-white mb-1">{*num}</p>
                                    <p class="text-xs text-white/50 leading-snug">{*label}</p>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // ── 6. NEWSLETTER BAR ─────────────────────────────────────────
            <section class="bg-[#F5F5F0] py-12 sm:py-16">
                <div class="max-w-screen-xl mx-auto px-4 sm:px-6 text-center">
                    <h2 class="text-xl sm:text-2xl font-bold text-black">"Nhận ưu đãi độc quyền"</h2>
                    <p class="text-sm text-gray-500 mt-2 mb-8">
                        "Đăng ký nhận email — giảm ngay 10% đơn đầu tiên + thông báo sale sớm nhất."
                    </p>
                    {move || if subscribed.get() {
                        view! {
                            <div class="inline-flex items-center gap-2 bg-emerald-500/10 border border-emerald-500/30 text-emerald-700 text-sm font-medium px-6 py-3 rounded-full">
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
                                "Cảm ơn bạn đã đăng ký! Kiểm tra hộp thư nhé 🎉"
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <form
                                class="flex flex-col sm:flex-row gap-3 max-w-md mx-auto"
                                on:submit=move |e| {
                                    e.prevent_default();
                                    if !email.get().is_empty() { subscribed.set(true); }
                                }
                            >
                                <input
                                    type="email"
                                    placeholder="email@example.com"
                                    class="flex-1 border border-gray-300 rounded-full px-5 py-3 text-sm text-black placeholder-gray-400 focus:outline-none focus:border-black transition-colors"
                                    on:input=move |e| email.set(event_target_value(&e))
                                    prop:value=move || email.get()
                                />
                                <button
                                    type="submit"
                                    class="bg-black text-white font-semibold px-7 py-3 rounded-full text-sm hover:bg-gray-800 transition-colors cursor-pointer whitespace-nowrap"
                                >
                                    "Đăng ký ngay"
                                </button>
                            </form>
                        }.into_any()
                    }}
                    <p class="text-[11px] text-gray-400 mt-4">"Không spam. Huỷ đăng ký bất cứ lúc nào."</p>
                </div>
            </section>

        </div>
    }
}
