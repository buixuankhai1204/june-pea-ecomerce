use crate::api::client::ordering::{list_all_orders, update_order_status};
use crate::api::types::OrderStatus;
use leptos::either::Either;
use leptos::prelude::*;
use uuid::Uuid;

fn icon_search() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
    }
}

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

fn icon_filter() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
        </svg>
    }
}

#[component]
pub fn AdminOrdersPage() -> impl IntoView {
    let active_tab = RwSignal::new("All");
    let search_query = RwSignal::new("".to_string());
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (customer_id_str, set_customer_id_str) = signal(String::new());
    let (variant_id_str, set_variant_id_str) = signal(String::new());
    let (quantity, set_quantity) = signal(1);
    let (price, set_price) = signal(0i64);

    let orders_resource: LocalResource<
        Result<Vec<crate::api::types::Order>, crate::api::types::ApiError>,
    > = LocalResource::new(move || async move { list_all_orders().await });

    let update_status_action = Action::new_local(|(id, status): &(Uuid, OrderStatus)| {
        let id = *id;
        let status = status.clone();
        async move { update_order_status(id, status).await }
    });

    let create_order_action = Action::new_local(|req: &crate::api::types::PlaceOrderRequest| {
        let req = req.clone();
        async move { crate::api::client::ordering::place_order(req).await }
    });

    let update_note_action = Action::new_local(|(id, note): &(Uuid, String)| {
        let id = *id;
        let note = note.clone();
        async move { crate::api::client::ordering::update_order_note(id, &note).await }
    });

    let refund_action = Action::new_local(|id: &Uuid| {
        let id = *id;
        async move { crate::api::client::payment::refund_payment(id).await }
    });

    let on_create_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let customer_id = Uuid::parse_str(&customer_id_str.get()).ok();
        let variant_id = Uuid::parse_str(&variant_id_str.get()).ok();

        if let Some(vid) = variant_id {
            create_order_action.dispatch(crate::api::types::PlaceOrderRequest {
                customer_id,
                items: vec![crate::api::types::NewOrderItem {
                    variant_id: vid,
                    quantity: quantity.get(),
                    unit_price: price.get(),
                }],
            });
        }
    };

    Effect::new(move |_| {
        if update_status_action.value().get().is_some()
            || create_order_action.value().get().is_some()
            || update_note_action.value().get().is_some()
            || refund_action.value().get().is_some()
        {
            if create_order_action.value().get().is_some() {
                set_show_create_modal.set(false);
            }
            orders_resource.refetch();
        }
    });

    let filtered_orders = move || {
        orders_resource.get().and_then(|sw| {
            let res = &*sw;
            res.as_ref().ok().map(|orders| {
                orders
                    .iter()
                    .filter(|o| {
                        let status_match =
                            active_tab.get() == "All" || o.status.to_string() == active_tab.get();
                        let search_match = search_query.get().is_empty()
                            || o.id.to_string().contains(&search_query.get());
                        status_match && search_match
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    };

    let tabs = ["All", "Pending", "Completed", "Cancelled"];

    view! {
        <div class="p-6 space-y-6 animate-in fade-in duration-500">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Orders"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Manage and track all customer orders"</p>
                </div>
                <button
                    on:click=move |_| set_show_create_modal.set(true)
                    class="flex items-center gap-1.5 bg-[#FCE300] hover:bg-yellow-400 text-gray-900 text-xs font-bold px-4 py-2.5 rounded-xl transition-colors cursor-pointer shadow-sm"
                >
                    {icon_plus()} "New Order"
                </button>
            </div>

            // Table card
            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                // Toolbar
                <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100 flex-wrap gap-3">
                    // Tabs
                    <div class="flex items-center gap-1">
                        {tabs.into_iter().map(|t| {
                            view! {
                                <button
                                    class=move || format!("px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 cursor-pointer {}",
                                        if active_tab.get() == t { "bg-[#FCE300] text-gray-900" } else { "text-gray-500 hover:text-gray-800 hover:bg-gray-50" })
                                    on:click=move |_| active_tab.set(t)
                                >{t}</button>
                            }
                        }).collect_view()}
                    </div>
                    // Search + filter
                    <div class="flex items-center gap-2">
                        <div class="flex items-center gap-2 bg-gray-50 border border-gray-200 rounded-xl px-3 py-2 w-44">
                            <span class="text-gray-400">{icon_search()}</span>
                            <input
                                type="text"
                                placeholder="Search ID..."
                                class="bg-transparent text-xs text-gray-700 outline-none w-full"
                                on:input=move |ev| search_query.set(event_target_value(&ev))
                                prop:value=search_query
                            />
                        </div>
                        <button class="flex items-center gap-1.5 border border-gray-200 text-gray-500 rounded-xl px-3 py-2 text-xs font-medium hover:bg-gray-50 transition-colors cursor-pointer">
                            {icon_filter()} "Filter"
                        </button>
                    </div>
                </div>

                // Table
                <div class="overflow-x-auto">
                    <table class="w-full text-xs">
                        <thead>
                            <tr class="border-b border-gray-100 text-gray-400 uppercase tracking-wider text-[10px]">
                                <th class="px-5 py-3 text-left font-medium">"Order ID"</th>
                                <th class="px-4 py-3 text-left font-medium">"Customer ID"</th>
                                <th class="px-4 py-3 text-left font-medium hidden sm:table-cell">"Date"</th>
                                <th class="px-4 py-3 text-right font-medium">"Amount"</th>
                                <th class="px-4 py-3 text-center font-medium">"Status"</th>
                                <th class="px-4 py-3 text-right font-medium">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-50">
                            <Suspense fallback=move || view! { <tr><td colspan="6" class="px-6 py-4 text-center">"Loading orders..."</td></tr> }>
                                {move || match filtered_orders() {
                                    None => Either::Left(view! { <tr><td colspan="6" class="px-6 py-4 text-center">"Fetching data..."</td></tr> }),
                                    Some(orders) => Either::Right(orders.into_iter().map(|o| {
                                        let (bg, text) = match o.status {
                                            OrderStatus::Completed  => ("bg-emerald-50 text-emerald-600 border-emerald-100", "●  Completed"),
                                            OrderStatus::Cancelled  => ("bg-rose-50 text-rose-600 border-rose-100",           "●  Cancelled"),
                                            OrderStatus::Pending => ("bg-amber-50 text-amber-600 border-amber-100",        "●  Pending"),
                                        };
                                        let o_id = o.id;

                                        view! {
                                            <tr class="hover:bg-gray-50 transition-colors duration-100 group">
                                                <td class="px-5 py-3.5 font-mono text-indigo-600 font-semibold whitespace-nowrap">
                                                    {format!("#{}", o.id.to_string().chars().take(8).collect::<String>())}
                                                </td>
                                                <td class="px-4 py-3.5 text-gray-800 font-medium whitespace-nowrap">
                                                    {o.customer_id.map(|id| id.to_string()).unwrap_or_else(|| "Guest".to_string())}
                                                </td>
                                                <td class="px-4 py-3.5 text-gray-400 hidden sm:table-cell whitespace-nowrap">
                                                    {o.created_at.format("%d/%m/%Y").to_string()}
                                                </td>
                                                <td class="px-4 py-3.5 text-right text-gray-900 font-bold whitespace-nowrap">
                                                    {format!("₫{}k", o.total / 1000)}
                                                </td>
                                                <td class="px-4 py-3.5 text-center">
                                                    <span class=format!("text-[10px] font-semibold px-2.5 py-1 rounded-full border {}", bg)>
                                                        {text}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3.5 text-right">
                                                    <div class="flex justify-end space-x-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                        <button
                                                            class="text-blue-600 hover:text-blue-900 font-bold cursor-pointer"
                                                            on:click=move |_| {
                                                                if let Some(note) = web_sys::window().and_then(|w| w.prompt_with_message("Enter order note:").ok().flatten()) {
                                                                    update_note_action.dispatch((o_id, note));
                                                                }
                                                            }
                                                        >
                                                            "Note"
                                                        </button>
                                                        <button
                                                            class="text-orange-600 hover:text-orange-900 font-bold cursor-pointer"
                                                            on:click=move |_| {
                                                                if web_sys::window().map(|w| w.confirm_with_message("Refund this order?").unwrap_or(false)).unwrap_or(false) {
                                                                    refund_action.dispatch(o_id);
                                                                }
                                                            }
                                                        >
                                                            "Refund"
                                                        </button>
                                                        <button
                                                            class="text-emerald-600 hover:text-emerald-900 font-bold cursor-pointer"
                                                            on:click=move |_| {
                                                                update_status_action.dispatch((o_id, OrderStatus::Completed));
                                                            }
                                                        >
                                                            "Complete"
                                                        </button>
                                                        <button
                                                            class="text-rose-600 hover:text-rose-900 font-bold cursor-pointer"
                                                            on:click=move |_| {
                                                                update_status_action.dispatch((o_id, OrderStatus::Cancelled));
                                                            }
                                                        >
                                                            "Cancel"
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view())
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            // Create Modal
            {move || show_create_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm">
                    <div class="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden">
                        <div class="p-6 border-b border-gray-50">
                            <h3 class="text-xl font-black text-gray-900">"New Manual Order"</h3>
                        </div>
                        <form on:submit=on_create_submit class="p-6 space-y-4">
                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Customer ID (Optional)"</label>
                                <input
                                    type="text"
                                    placeholder="UUID..."
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:input=move |ev| set_customer_id_str.set(event_target_value(&ev))
                                    prop:value=customer_id_str
                                />
                            </div>
                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Variant ID"</label>
                                <input
                                    type="text"
                                    required
                                    placeholder="UUID..."
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:input=move |ev| set_variant_id_str.set(event_target_value(&ev))
                                    prop:value=variant_id_str
                                />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div class="space-y-1">
                                    <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Quantity"</label>
                                    <input
                                        type="number"
                                        required
                                        min="1"
                                        class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                        on:input=move |ev| set_quantity.set(event_target_value(&ev).parse().unwrap_or(1))
                                        prop:value=quantity
                                    />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Price (₫)"</label>
                                    <input
                                        type="number"
                                        required
                                        class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                        on:input=move |ev| set_price.set(event_target_value(&ev).parse().unwrap_or(0))
                                        prop:value=price
                                    />
                                </div>
                            </div>
                            <div class="flex gap-3 pt-4">
                                <button
                                    type="button"
                                    class="flex-1 px-6 py-3 rounded-2xl font-bold text-gray-500 hover:bg-gray-50 transition-all cursor-pointer"
                                    on:click=move |_| set_show_create_modal.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="submit"
                                    class="flex-1 px-6 py-3 rounded-2xl bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold transition-all shadow-lg cursor-pointer"
                                >
                                    "Place Order"
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}
        </div>
    }
}
