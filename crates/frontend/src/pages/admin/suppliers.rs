use leptos::prelude::*;

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

fn icon_search() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
    }
}

struct Supplier {
    name: &'static str,
    contact: &'static str,
    location: &'static str,
    products: u32,
    deliveries: u32,
    status: &'static str,
}

#[component]
pub fn AdminSuppliersPage() -> impl IntoView {
    let suppliers = vec![
        Supplier {
            name: "Vải Đẹp Việt Nam",
            contact: "contact@vaidepc.vn",
            location: "TP. Hồ Chí Minh",
            products: 128,
            deliveries: 14,
            status: "Active",
        },
        Supplier {
            name: "Textile World Co.",
            contact: "info@textileworld.com",
            location: "Bình Dương",
            products: 84,
            deliveries: 8,
            status: "Active",
        },
        Supplier {
            name: "Sợi Bông Miền Nam",
            contact: "sales@soibong.vn",
            location: "Đồng Nai",
            products: 56,
            deliveries: 3,
            status: "Pending",
        },
        Supplier {
            name: "Korean Fashion Supply",
            contact: "kr@fashionsupply.kr",
            location: "Seoul, Korea",
            products: 212,
            deliveries: 22,
            status: "Active",
        },
        Supplier {
            name: "Premium Fabric Ltd.",
            contact: "orders@premiumfabric.com",
            location: "Hà Nội",
            products: 67,
            deliveries: 5,
            status: "Inactive",
        },
        Supplier {
            name: "Eco Yarn Traders",
            contact: "hello@ecoyarn.io",
            location: "Đà Nẵng",
            products: 38,
            deliveries: 2,
            status: "Active",
        },
    ];

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Suppliers"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Manage your product suppliers and deliveries"</p>
                </div>
                <button class="flex items-center gap-1.5 bg-[#FCE300] hover:bg-yellow-400 text-gray-900 text-xs font-bold px-4 py-2.5 rounded-xl transition-colors cursor-pointer shadow-sm">
                    {icon_plus()} "Add Supplier"
                </button>
            </div>

            // KPI cards
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                {[
                    ("Active Suppliers", "4", "#10B981", "bg-emerald-50 border-emerald-100"),
                    ("Total Products", "585", "#6366F1", "bg-indigo-50 border-indigo-100"),
                    ("Pending Deliveries", "3", "#F59E0B", "bg-amber-50 border-amber-100"),
                ].iter().map(|&(label, val, color, bg)| view! {
                    <div class=format!("rounded-2xl p-5 border shadow-sm {}", bg)>
                        <p class="text-xs text-gray-500 font-medium">{label}</p>
                        <p class="text-3xl font-black mt-1.5" style={format!("color:{}", color)}>{val}</p>
                    </div>
                }).collect_view()}
            </div>

            // Table
            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900">"All Suppliers"</h2>
                    <div class="flex items-center gap-2 bg-gray-50 border border-gray-200 rounded-xl px-3 py-2 w-48">
                        <span class="text-gray-400">{icon_search()}</span>
                        <input type="text" placeholder="Search suppliers..." class="bg-transparent text-xs text-gray-700 outline-none w-full"/>
                    </div>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-xs">
                        <thead>
                            <tr class="border-b border-gray-100 text-gray-400 uppercase tracking-wider text-[10px]">
                                <th class="px-5 py-3 text-left font-medium">"Supplier"</th>
                                <th class="px-4 py-3 text-left font-medium hidden md:table-cell">"Contact"</th>
                                <th class="px-4 py-3 text-left font-medium hidden sm:table-cell">"Location"</th>
                                <th class="px-4 py-3 text-center font-medium">"Products"</th>
                                <th class="px-4 py-3 text-center font-medium hidden lg:table-cell">"Deliveries"</th>
                                <th class="px-4 py-3 text-center font-medium">"Status"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-50">
                            {suppliers.into_iter().map(|s| {
                                let (badge_bg, badge_text) = match s.status {
                                    "Active"   => ("bg-emerald-50 text-emerald-600 border-emerald-100", "Active"),
                                    "Pending"  => ("bg-amber-50 text-amber-600 border-amber-100",       "Pending"),
                                    _          => ("bg-gray-100 text-gray-400 border-gray-200",          "Inactive"),
                                };
                                view! {
                                    <tr class="hover:bg-gray-50 transition-colors duration-100">
                                        <td class="px-5 py-3.5">
                                            <div class="flex items-center gap-3">
                                                <div class="w-8 h-8 rounded-xl bg-gradient-to-br from-indigo-400 to-violet-500 flex items-center justify-center text-white text-xs font-bold flex-shrink-0">
                                                    {s.name.chars().next().unwrap_or('?').to_string()}
                                                </div>
                                                <span class="font-semibold text-gray-900">{s.name}</span>
                                            </div>
                                        </td>
                                        <td class="px-4 py-3.5 text-gray-400 hidden md:table-cell">{s.contact}</td>
                                        <td class="px-4 py-3.5 text-gray-600 hidden sm:table-cell">{s.location}</td>
                                        <td class="px-4 py-3.5 text-center text-gray-900 font-semibold">{s.products}</td>
                                        <td class="px-4 py-3.5 text-center text-gray-600 hidden lg:table-cell">{s.deliveries}</td>
                                        <td class="px-4 py-3.5 text-center">
                                            <span class=format!("text-[10px] font-semibold px-2.5 py-1 rounded-full border {}", badge_bg)>
                                                {badge_text}
                                            </span>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
