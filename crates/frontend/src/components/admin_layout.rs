use leptos::prelude::*;
use leptos_router::hooks::use_location;

// ── SVG Icons ──────────────────────────────────────────────────────────────

fn icon_grid() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect>
            <rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect>
        </svg>
    }
}

fn icon_shopping_bag() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M6 2L3 6v14a2 2 0 002 2h14a2 2 0 002-2V6l-3-4z"></path>
            <line x1="3" y1="6" x2="21" y2="6"></line>
            <path d="M16 10a4 4 0 01-8 0"></path>
        </svg>
    }
}

fn icon_truck() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="1" y="3" width="15" height="13"></rect>
            <polygon points="16 8 20 8 23 11 23 16 16 16 16 8"></polygon>
            <circle cx="5.5" cy="18.5" r="2.5"></circle>
            <circle cx="18.5" cy="18.5" r="2.5"></circle>
        </svg>
    }
}

fn icon_users() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 00-3-3.87"></path>
            <path d="M16 3.13a4 4 0 010 7.75"></path>
        </svg>
    }
}

fn icon_credit_card() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="1" y="4" width="22" height="16" rx="2" ry="2"></rect>
            <line x1="1" y1="10" x2="23" y2="10"></line>
        </svg>
    }
}

fn icon_tag() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z"></path>
            <line x1="7" y1="7" x2="7.01" y2="7"></line>
        </svg>
    }
}

fn icon_star() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
        </svg>
    }
}

fn icon_bar_chart() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="20" x2="18" y2="10"></line>
            <line x1="12" y1="20" x2="12" y2="4"></line>
            <line x1="6" y1="20" x2="6" y2="14"></line>
        </svg>
    }
}

fn icon_settings() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.07 4.93l-1.41 1.41M4.93 4.93l1.41 1.41M4.93 19.07l1.41-1.41M19.07 19.07l-1.41-1.41"></path>
            <line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line>
            <line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line>
        </svg>
    }
}

fn icon_search() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
    }
}

fn icon_bell() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
            <path d="M13.73 21a2 2 0 01-3.46 0"></path>
        </svg>
    }
}

fn icon_chevron_down() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
    }
}

// ── Nav item data ──────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct NavItem {
    label: &'static str,
    href:  &'static str,
    icon:  u8,
}

fn nav_icon(id: u8) -> impl IntoView {
    match id {
        0 => icon_grid().into_any(),
        1 => icon_shopping_bag().into_any(),
        2 => icon_truck().into_any(),
        3 => icon_users().into_any(),
        4 => icon_credit_card().into_any(),
        5 => icon_tag().into_any(),
        6 => icon_star().into_any(),
        7 => icon_bar_chart().into_any(),
        8 => icon_settings().into_any(),
        _ => icon_grid().into_any(),
    }
}

// ── Admin Layout ───────────────────────────────────────────────────────────

#[component]
pub fn AdminLayout(children: Children) -> impl IntoView {
    let main_nav: Vec<NavItem> = vec![
        NavItem { label: "Dashboard",          href: "/admin",          icon: 0 },
        NavItem { label: "Product Catalog",    href: "/admin/catalog",  icon: 1 },
        NavItem { label: "Categories",         href: "/admin/categories", icon: 5 },
        NavItem { label: "Inventory & Stock",  href: "/admin/stock",    icon: 2 },
        NavItem { label: "Orders",             href: "/admin/orders",   icon: 1 },
        NavItem { label: "Staff Management",   href: "/admin/staff",    icon: 3 },
        NavItem { label: "Payment & Invoices", href: "/admin/payments", icon: 4 },
    ];
    let engage_nav: Vec<NavItem> = vec![
        NavItem { label: "Promotions",          href: "/admin/promotions",   icon: 5 },
        NavItem { label: "Memberships",         href: "/admin/memberships",  icon: 6 },
        NavItem { label: "Reports & Analytics", href: "/admin/reports",      icon: 7 },
    ];
    let other_nav: Vec<NavItem> = vec![
        NavItem { label: "Settings", href: "/admin/settings", icon: 8 },
    ];

    let location = use_location();

    let is_active = move |href: &'static str| {
        let p = location.pathname.get();
        if href == "/admin" {
            p == "/admin" || p == "/admin/"
        } else {
            p.starts_with(href)
        }
    };

    view! {
        <div class="flex min-h-screen bg-[#F7F8FA] font-[Inter,system-ui,sans-serif]">

            // ── Sidebar ────────────────────────────────────────────────────
            <aside class="w-[220px] flex-shrink-0 bg-white border-r border-gray-100 flex flex-col fixed top-0 left-0 h-screen z-40 shadow-sm">

                // Logo + Team
                <div class="px-5 pt-6 pb-4">
                    <div class="flex items-center gap-2.5 mb-5">
                        <div class="w-8 h-8 bg-black rounded-lg flex items-center justify-center">
                            <span class="text-white text-xs font-black">"JP"</span>
                        </div>
                        <span class="font-bold text-gray-900 text-sm">"JunePea"</span>
                    </div>

                    // Team pill
                    <button class="w-full flex items-center justify-between px-3 py-2 rounded-xl bg-gray-50 hover:bg-gray-100 transition-colors duration-150 cursor-pointer group">
                        <div class="flex items-center gap-2">
                            <div class="w-5 h-5 rounded-md bg-gradient-to-br from-amber-400 to-orange-500 flex-shrink-0"></div>
                            <span class="text-xs font-semibold text-gray-700">"Store Team"</span>
                        </div>
                        <span class="text-gray-400 group-hover:text-gray-600 transition-colors">{icon_chevron_down()}</span>
                    </button>
                </div>

                <nav class="flex-1 overflow-y-auto px-3 space-y-5 pb-4">
                    // Main section
                    <div>
                        <p class="px-2 mb-1.5 text-[10px] font-bold text-gray-400 uppercase tracking-widest">"MAIN"</p>
                        <ul class="space-y-0.5">
                            {main_nav.into_iter().map(move |item| {
                                view! {
                                    <li>
                                        <a
                                            href={item.href}
                                            class=move || format!(
                                                "flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-medium transition-all duration-150 cursor-pointer group {}",
                                                if is_active(item.href) {
                                                    "bg-[#FCE300] text-gray-900 shadow-sm"
                                                } else {
                                                    "text-gray-500 hover:bg-gray-50 hover:text-gray-900"
                                                }
                                            )
                                        >
                                            <span class=move || if is_active(item.href) { "text-gray-900" } else { "text-gray-400 group-hover:text-gray-700 transition-colors" }>
                                                {nav_icon(item.icon)}
                                            </span>
                                            {item.label}
                                        </a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    </div>

                    // Engagement section
                    <div>
                        <p class="px-2 mb-1.5 text-[10px] font-bold text-gray-400 uppercase tracking-widest">"ENGAGEMENT"</p>
                        <ul class="space-y-0.5">
                            {engage_nav.into_iter().map(move |item| {
                                view! {
                                    <li>
                                        <a
                                            href={item.href}
                                            class=move || format!(
                                                "flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-medium transition-all duration-150 cursor-pointer group {}",
                                                if is_active(item.href) {
                                                    "bg-[#FCE300] text-gray-900 shadow-sm"
                                                } else {
                                                    "text-gray-500 hover:bg-gray-50 hover:text-gray-900"
                                                }
                                            )
                                        >
                                            <span class=move || if is_active(item.href) { "text-gray-900" } else { "text-gray-400 group-hover:text-gray-700 transition-colors" }>
                                                {nav_icon(item.icon)}
                                            </span>
                                            {item.label}
                                        </a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    </div>

                    // Others section
                    <div>
                        <p class="px-2 mb-1.5 text-[10px] font-bold text-gray-400 uppercase tracking-widest">"OTHERS"</p>
                        <ul class="space-y-0.5">
                            {other_nav.into_iter().map(move |item| {
                                view! {
                                    <li>
                                        <a
                                            href={item.href}
                                            class=move || format!(
                                                "flex items-center gap-3 px-3 py-2 rounded-xl text-xs font-medium transition-all duration-150 cursor-pointer group {}",
                                                if is_active(item.href) {
                                                    "bg-[#FCE300] text-gray-900 shadow-sm"
                                                } else {
                                                    "text-gray-500 hover:bg-gray-50 hover:text-gray-900"
                                                }
                                            )
                                        >
                                            <span class=move || if is_active(item.href) { "text-gray-900" } else { "text-gray-400 group-hover:text-gray-700 transition-colors" }>
                                                {nav_icon(item.icon)}
                                            </span>
                                            {item.label}
                                        </a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    </div>
                </nav>

                // User profile at bottom
                <div class="px-3 py-4 border-t border-gray-100">
                    <div class="flex items-center gap-2.5 px-2">
                        <div class="w-8 h-8 rounded-full bg-gradient-to-br from-violet-400 to-indigo-600 flex items-center justify-center text-white text-xs font-bold flex-shrink-0">
                            "A"
                        </div>
                        <div class="flex-1 min-w-0">
                            <p class="text-xs font-semibold text-gray-800 truncate">"Admin User"</p>
                            <p class="text-[10px] text-gray-400 truncate">"admin@junepea.com"</p>
                        </div>
                    </div>
                </div>
            </aside>

            // ── Main content ───────────────────────────────────────────────
            <div class="flex-1 ml-[220px] flex flex-col min-h-screen">

                // Top header
                <header class="h-14 bg-white border-b border-gray-100 flex items-center justify-between px-6 flex-shrink-0 sticky top-0 z-30">
                    <div class="flex items-center gap-3">
                        <p class="text-sm text-gray-500">
                            "Good morning, "
                            <span class="font-semibold text-gray-800">"Admin"</span>
                            " 👋"
                        </p>
                    </div>

                    <div class="flex items-center gap-3">
                        // Search bar
                        <div class="flex items-center gap-2 bg-gray-50 border border-gray-200 rounded-xl px-3 py-2 w-56 hover:border-gray-300 transition-colors">
                            <span class="text-gray-400 flex-shrink-0">{icon_search()}</span>
                            <input
                                type="text"
                                placeholder="Search..."
                                class="bg-transparent text-xs text-gray-600 placeholder-gray-400 outline-none w-full"
                            />
                            <span class="text-[10px] text-gray-300 font-mono bg-gray-200 px-1.5 py-0.5 rounded flex-shrink-0">"⌘K"</span>
                        </div>

                        // Notification bell
                        <button class="relative w-9 h-9 rounded-xl bg-gray-50 border border-gray-200 flex items-center justify-center text-gray-500 hover:bg-gray-100 hover:text-gray-700 transition-colors cursor-pointer">
                            {icon_bell()}
                            <span class="absolute -top-0.5 -right-0.5 w-2 h-2 bg-red-500 rounded-full border-2 border-white"></span>
                        </button>

                        // Status badge
                        <div class="flex items-center gap-1.5 px-3 py-1.5 bg-emerald-50 border border-emerald-100 rounded-xl">
                            <span class="relative flex h-1.5 w-1.5">
                                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                <span class="relative inline-flex rounded-full h-1.5 w-1.5 bg-emerald-500"></span>
                            </span>
                            <span class="text-[11px] text-emerald-600 font-semibold">"Opened"</span>
                        </div>

                        // Avatar
                        <div class="w-8 h-8 rounded-full bg-gradient-to-br from-violet-400 to-indigo-600 flex items-center justify-center text-white text-xs font-bold cursor-pointer">
                            "A"
                        </div>
                    </div>
                </header>

                // Page content
                <main class="flex-1 overflow-auto">
                    {children()}
                </main>
            </div>
        </div>
    }
}
