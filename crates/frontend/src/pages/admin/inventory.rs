use crate::api::client::inventory as inventory_api;
use crate::api::types::StockUpdate;
use leptos::prelude::*;

#[component]
pub fn AdminInventoryPage() -> impl IntoView {
    let (variant_id_str, set_variant_id_str) = signal(String::new());
    let (quantity, set_quantity) = signal(0);

    let stocks: LocalResource<Vec<crate::api::types::StockResponse>> =
        LocalResource::new(move || async move {
            inventory_api::list_all_stocks().await.unwrap_or_default()
        });

    let update_stock = Action::new_local(|req: &StockUpdate| {
        let req = req.clone();
        async move { inventory_api::update_stock(req).await }
    });

    let on_apply = move |_| {
        if let Ok(id) = uuid::Uuid::parse_str(&variant_id_str.get()) {
            update_stock.dispatch(StockUpdate {
                variant_id: id,
                quantity: quantity.get(),
            });
        }
    };

    Effect::new(move |_| {
        if update_stock.value().get().is_some() {
            stocks.refetch();
        }
    });

    view! {
        <div class="p-6 space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-black text-gray-900">"Inventory Control"</h1>
                    <p class="text-sm text-gray-400 mt-1">"Monitor and adjust stock levels across all variants"</p>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Form
                <div class="lg:col-span-1">
                    <div class="bg-white rounded-3xl border border-gray-100 shadow-sm p-6 space-y-4 sticky top-6">
                        <div class="w-12 h-12 bg-amber-50 text-amber-500 rounded-2xl flex items-center justify-center mb-2">
                             <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"></path><path d="m3.3 7 8.7 5 8.7-5"></path><path d="M12 22V12"></path></svg>
                        </div>
                        <h2 class="text-lg font-bold text-gray-800">"Update Stock"</h2>
                        <p class="text-xs text-gray-500">"Set absolute stock levels for a specific variant ID."</p>

                        <div class="space-y-4 pt-2">
                            <div>
                                <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5 px-1">"Variant ID"</label>
                                <input
                                    type="text"
                                    prop:value=variant_id_str
                                    on:input=move |ev| set_variant_id_str.set(event_target_value(&ev))
                                    placeholder="UUID..."
                                    class="w-full bg-gray-50 border border-gray-100 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-[#FCE300] outline-none transition-all"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] font-black text-gray-400 uppercase tracking-widest mb-1.5 px-1">"New Level"</label>
                                <input
                                    type="number"
                                    prop:value=quantity
                                    on:input=move |ev| set_quantity.set(event_target_value(&ev).parse().unwrap_or(0))
                                    placeholder="0"
                                    class="w-full bg-gray-50 border border-gray-100 rounded-xl px-4 py-2.5 text-sm focus:ring-2 focus:ring-[#FCE300] outline-none transition-all"
                                />
                            </div>
                            <button
                                on:click=on_apply
                                class="w-full bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold py-3 rounded-xl transition-all shadow-md active:scale-95 cursor-pointer mt-2"
                            >
                                "Apply Update"
                            </button>

                            {move || update_stock.value().get().map(|res| match res {
                                Ok(_) => view! { <p class="text-xs text-emerald-600 font-bold bg-emerald-50 p-2 rounded-lg">"✓ Stock updated!"</p> }.into_any(),
                                Err(e) => view! { <p class="text-xs text-red-500 font-bold bg-red-50 p-2 rounded-lg">{format!("Error: {}", e)}</p> }.into_any(),
                            })}
                        </div>
                    </div>
                </div>

                // List
                <div class="lg:col-span-2">
                    <div class="bg-white rounded-3xl border border-gray-100 shadow-sm overflow-hidden">
                        <div class="px-6 py-4 border-b border-gray-50 flex items-center justify-between">
                            <h3 class="text-sm font-bold text-gray-800">"All Variant Stock Levels"</h3>
                            <button
                                on:click=move |_| stocks.refetch()
                                class="text-gray-400 hover:text-gray-600"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path><path d="M3 3v5h5"></path><path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"></path><path d="M16 16h5v5"></path></svg>
                            </button>
                        </div>
                        <div class="overflow-x-auto">
                            <table class="w-full text-xs text-left">
                                <thead class="bg-gray-50 text-gray-400 uppercase tracking-widest text-[10px]">
                                    <tr>
                                        <th class="px-6 py-3 font-black">"Variant ID"</th>
                                        <th class="px-6 py-3 font-black">"Quantity"</th>
                                        <th class="px-6 py-3 font-black text-right">"Status"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-50">
                                    <Suspense fallback=|| view! { <tr><td colspan="3" class="px-6 py-8 text-center text-gray-400">"Loading stock data..."</td></tr> }>
                                        {move || stocks.get().map(|list| {
                                            if list.is_empty() {
                                                view! { <tr><td colspan="3" class="px-6 py-8 text-center text-gray-400 font-medium">"No stock records found"</td></tr> }.into_any()
                                            } else {
                                                list.iter().map(|s| {
                                                    let vid = s.variant_id;
                                                    let (status, color) = if s.quantity == 0 {
                                                        ("Out of Stock", "text-red-500 bg-red-50")
                                                    } else if s.quantity < 10 {
                                                        ("Low Stock", "text-amber-500 bg-amber-50")
                                                    } else {
                                                        ("In Stock", "text-emerald-500 bg-emerald-50")
                                                    };
                                                    view! {
                                                        <tr class="hover:bg-gray-50/50 transition-colors group">
                                                            <td class="px-6 py-4 font-mono text-indigo-600 font-medium">
                                                                {vid.to_string()}
                                                            </td>
                                                            <td class="px-6 py-4 text-gray-900 font-black text-sm">
                                                                {s.quantity}
                                                            </td>
                                                            <td class="px-6 py-4 text-right">
                                                                <span class=format!("px-2 py-1 rounded-full text-[10px] font-bold {}", color)>
                                                                    {status}
                                                                </span>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view().into_any()
                                            }
                                        })}
                                    </Suspense>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
