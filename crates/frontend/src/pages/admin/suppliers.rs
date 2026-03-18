use leptos::prelude::*;
use crate::api::client::suppliers::*;
use crate::api::types::CreateSupplierRequest;

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


#[component]
pub fn AdminSuppliersPage() -> impl IntoView {
    let suppliers_resource = LocalResource::new(|| async move { list_suppliers().await });

    let add_supplier_action = Action::new_local(|req: &CreateSupplierRequest| {
        let req = req.clone();
        async move { create_supplier(req).await }
    });

    Effect::new(move |_| {
        if add_supplier_action.value().get().is_some() {
            suppliers_resource.refetch();
        }
    });

    let add_supplier = move |name: String, contact: String, location: String| {
        let req = CreateSupplierRequest { name, contact, location };
        add_supplier_action.dispatch(req);
    };

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Suppliers"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Manage your product suppliers and deliveries"</p>
                </div>
                <button 
                    on:click=move |_| add_supplier("New Supplier".to_string(), "contact@new.com".to_string(), "Hanoi".to_string())
                    class="flex items-center gap-1.5 bg-[#FCE300] hover:bg-yellow-400 text-gray-900 text-xs font-bold px-4 py-2.5 rounded-xl transition-colors cursor-pointer shadow-sm">
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
                            <Suspense fallback=move || view! { <p>"Loading suppliers..."</p> }>
                                {move || match suppliers_resource.get() {
                                    Some(res) => match &*res {
                                        Ok(items) => items.into_iter().map(|s| {
                                            let (badge_bg, badge_text) = match s.status.as_str() {
                                                "Active"   => ("bg-emerald-50 text-emerald-600 border-emerald-100", "Active"),
                                                "Pending"  => ("bg-amber-50 text-amber-600 border-amber-100",       "Pending"),
                                                _          => ("bg-gray-100 text-gray-400 border-gray-200",          "Inactive"),
                                            };
                                            view! {
                                                <tr class="hover:bg-gray-50 transition-colors duration-100 group">
                                                    <td class="px-5 py-3.5">
                                                        <div class="flex items-center gap-3">
                                                            <div class="w-8 h-8 rounded-xl bg-gradient-to-br from-indigo-400 to-violet-500 flex items-center justify-center text-white text-xs font-bold flex-shrink-0">
                                                                {s.name.chars().next().unwrap_or('?').to_string().to_uppercase()}
                                                            </div>
                                                            <div>
                                                                <div class="flex items-center gap-2">
                                                                    <p class="font-bold text-gray-900 leading-tight">{s.name.clone()}</p>
                                                                    <span class="px-1.5 py-0.5 rounded-md bg-indigo-50 text-indigo-500 text-[10px] font-bold uppercase tracking-wider">"Official"</span>
                                                                </div>
                                                                <p class="text-[10px] text-gray-400">"ID: " {s.id.to_string().chars().take(8).collect::<String>()}</p>
                                                            </div>
                                                        </div>
                                                    </td>
                                                    <td class="px-4 py-3.5 text-gray-600 hidden md:table-cell">{s.contact.clone()}</td>
                                                    <td class="px-4 py-3.5 text-gray-400 hidden sm:table-cell">{s.location.clone()}</td>
                                                    <td class="px-4 py-3.5 text-center font-bold text-gray-900">"42"</td>
                                                    <td class="px-4 py-3.5 text-center hidden lg:table-cell text-gray-500">"186"</td>
                                                    <td class="px-4 py-3.5 text-center">
                                                        <span class=format!("text-[10px] font-semibold px-2.5 py-1 rounded-full border {}", badge_bg)>
                                                            {badge_text}
                                                        </span>
                                                    </td>
                                                </tr>
                                            }
                                        }).collect_view().into_any(),
                                        _ => view! { <tr><td colspan="6" class="p-10 text-center text-gray-400">"No data"</td></tr> }.into_any()
                                    },
                                    None => view! { <tr><td colspan="6" class="text-center py-4 text-rose-500">"Error loading suppliers"</td></tr> }.into_any(),
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
