use crate::api::client::marketing as marketing_api;
use leptos::prelude::*;

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

fn icon_trash() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
    }
}

#[component]
pub fn AdminPromotionsPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (code, set_code) = signal(String::new());
    let (discount, set_discount) = signal(0i64);
    let (max_uses, set_max_uses) = signal(100i32);

    let coupons: LocalResource<Vec<crate::api::types::Coupon>> =
        LocalResource::new(move || async move {
            marketing_api::list_coupons().await.unwrap_or_default()
        });

    let create_coupon = Action::new_local(|req: &crate::api::types::CreateCouponRequest| {
        let req = req.clone();
        async move { marketing_api::create_coupon(req).await }
    });

    let delete_coupon = Action::new_local(|code: &String| {
        let code = code.clone();
        async move { marketing_api::delete_coupon(&code).await }
    });

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        create_coupon.dispatch(crate::api::types::CreateCouponRequest {
            code: code.get(),
            discount_amount: discount.get(),
            max_uses: max_uses.get(),
        });
    };

    Effect::new(move |_| {
        if delete_coupon.value().get().is_some() || create_coupon.value().get().is_some() {
            if create_coupon.value().get().is_some() {
                set_show_create_modal.set(false);
                set_code.set(String::new());
                set_discount.set(0);
            }
            coupons.refetch();
        }
    });

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Promotions"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Create and manage discount codes and campaigns"</p>
                </div>
                <button
                    class="flex items-center gap-1.5 bg-[#FCE300] hover:bg-yellow-400 text-gray-900 text-xs font-bold px-4 py-2.5 rounded-xl transition-colors cursor-pointer shadow-sm"
                    on:click=move |_| set_show_create_modal.set(true)
                >
                    {icon_plus()} "New Promotion"
                </button>
            </div>

            // KPI cards
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <Suspense fallback=|| view! { <div class="col-span-4 text-center py-4">"Computing stats..."</div> }>
                    {move || coupons.get().map(|list| {
                        let active = list.iter().filter(|c| c.is_active).count();
                        let total_used: i32 = list.iter().map(|c| c.current_uses).sum();
                        view! {
                            <>
                                <div class="rounded-2xl p-4 border shadow-sm bg-emerald-50 border-emerald-100">
                                    <p class="text-xs text-gray-500 font-medium">"Active Promos"</p>
                                    <p class="text-2xl font-black mt-1 text-emerald-600">{active}</p>
                                </div>
                                <div class="rounded-2xl p-4 border shadow-sm bg-indigo-50 border-indigo-100">
                                    <p class="text-xs text-gray-500 font-medium">"Total Used"</p>
                                    <p class="text-2xl font-black mt-1 text-indigo-600">{total_used}</p>
                                </div>
                                <div class="rounded-2xl p-4 border shadow-sm bg-gray-50 border-gray-200">
                                    <p class="text-xs text-gray-500 font-medium">"Total Coupons"</p>
                                    <p class="text-2xl font-black mt-1 text-gray-600">{list.len()}</p>
                                </div>
                            </>
                        }
                    })}
                </Suspense>
            </div>

            // Promo cards grid
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                <Suspense fallback=|| view! { <div class="col-span-3 text-center py-10">"Loading tokens..."</div> }>
                    {move || coupons.get().map(|list| {
                        list.iter().map(|p| {
                            let fill_pct = if p.max_uses > 0 { p.current_uses as f64 / p.max_uses as f64 * 100.0 } else { 0.0 };
                            let (badge_bg, bar_color, status_text) = if p.is_active {
                                ("bg-emerald-50 text-emerald-600 border-emerald-100", "#10B981", "Active")
                            } else {
                                ("bg-gray-100 text-gray-400 border-gray-200", "#94A3B8", "Inactive")
                            };
                            let p_code = p.code.clone();
                            view! {
                                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100 hover:shadow-md transition-shadow cursor-pointer relative group">
                                    <div class="flex items-start justify-between mb-3">
                                        <div>
                                            <p class="font-black text-gray-900 text-base font-mono">{p.code.clone()}</p>
                                            <p class="text-xs text-gray-400 mt-0.5">"Discount Code"</p>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            <span class=format!("text-[10px] font-semibold px-2.5 py-1 rounded-full border flex-shrink-0 {}", badge_bg)>
                                                {status_text}
                                            </span>
                                            <button
                                                on:click=move |e| { e.stop_propagation(); delete_coupon.dispatch(p_code.clone()); }
                                                class="p-1.5 hover:bg-red-50 text-gray-300 hover:text-red-500 rounded-lg transition-colors opacity-0 group-hover:opacity-100"
                                            >
                                                {icon_trash()}
                                            </button>
                                        </div>
                                    </div>

                                    // Discount highlight
                                    <div class="bg-gray-50 rounded-xl p-3 text-center mb-3">
                                        <span class="text-2xl font-black text-[#FCE300]" style="text-shadow: 0 0 1px #0003;">"₫" {p.discount_amount}</span>
                                        <span class="text-xs text-gray-500 block">"Discount"</span>
                                    </div>

                                    // Usage bar
                                    <div class="space-y-1.5">
                                        <div class="flex items-center justify-between text-[11px] text-gray-500">
                                            <span>"Usage: " {p.current_uses} "/" {p.max_uses}</span>
                                            <span class="font-semibold text-gray-900">{format!("{:.0}%", fill_pct)}</span>
                                        </div>
                                        <div class="h-1.5 bg-gray-100 rounded-full overflow-hidden">
                                            <div class="h-full rounded-full transition-all duration-500"
                                                style={format!("width:{}%; background:{}", fill_pct, bar_color)}>
                                            </div>
                                        </div>
                                    </div>

                                    <p class="text-[11px] text-gray-400 mt-3">"Created: " <span class="text-gray-700 font-semibold">{p.created_at.format("%d/%m/%Y").to_string()}</span></p>
                                </div>
                            }
                        }).collect_view()
                    })}
                </Suspense>
            </div>

            // Create Modal
            {move || show_create_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm animate-in fade-in duration-200">
                    <div class="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200">
                        <div class="p-6 border-b border-gray-50">
                            <h3 class="text-xl font-black text-gray-900">"New Promotion"</h3>
                        </div>
                        <form on:submit=on_submit class="p-6 space-y-4">
                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Coupon Code"</label>
                                <input
                                    type="text"
                                    placeholder="e.g. TET2024"
                                    required
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm font-mono uppercase"
                                    on:input=move |ev| set_code.set(event_target_value(&ev).to_uppercase())
                                    prop:value=code
                                />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div class="space-y-1">
                                    <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Discount (₫)"</label>
                                    <input
                                        type="number"
                                        required
                                        class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                        on:input=move |ev| set_discount.set(event_target_value(&ev).parse().unwrap_or(0))
                                        prop:value=discount
                                    />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Max Uses"</label>
                                    <input
                                        type="number"
                                        required
                                        class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                        on:input=move |ev| set_max_uses.set(event_target_value(&ev).parse().unwrap_or(100))
                                        prop:value=max_uses
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
                                    class="flex-1 px-6 py-3 rounded-2xl bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold transition-all shadow-lg shadow-yellow-200/50 cursor-pointer"
                                >
                                    "Create"
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}
        </div>
    }
}
