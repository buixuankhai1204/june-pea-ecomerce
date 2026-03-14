use leptos::prelude::*;
use crate::api::client::catalog as catalog_api;
use crate::api::types::{CreateProductRequest, UpdateProductRequest, Product, ProductWithVariants, CreateVariantRequest};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::str::FromStr;

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

fn icon_edit() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
        </svg>
    }
}

fn icon_box() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line>
        </svg>
    }
}

#[component]
pub fn AdminProductsPage() -> impl IntoView {
    let (show_modal, set_show_modal) = signal(false);
    let (show_variants_modal, set_show_variants_modal) = signal(false);
    let (editing_product, set_editing_product) = signal::<Option<Product>>(None);
    let (selected_product_for_variants, set_selected_product_for_variants) = signal::<Option<Product>>(None);
    
    // Product form fields
    let (name, set_name) = signal("".to_string());
    let (slug, set_slug) = signal("".to_string());
    let (description, set_description) = signal("".to_string());
    let (category_id, set_category_id) = signal("".to_string());

    // Variant form fields
    let (v_sku, set_v_sku) = signal("".to_string());
    let (v_name, set_v_name) = signal("".to_string());
    let (v_price, set_v_price) = signal("".to_string());

    let products_resource: LocalResource<Vec<crate::api::types::Product>> = LocalResource::new(move || {
        async move { catalog_api::list_products(1, 100).await.map(|p| p.items).unwrap_or_default() }
    });

    let categories_resource: LocalResource<Vec<crate::api::types::Category>> = LocalResource::new(move || {
        async move { catalog_api::list_categories().await.unwrap_or_default() }
    });

    let variants_resource: LocalResource<Option<ProductWithVariants>> = LocalResource::new(move || {
        let p = selected_product_for_variants.get();
        async move {
            match p {
                Some(prod) => catalog_api::get_product_by_id(prod.id).await.ok(),
                None => None
            }
        }
    });

    let create_action = Action::new_local(|req: &CreateProductRequest| {
        let req = req.clone();
        async move { catalog_api::create_product(req).await }
    });

    let update_action = Action::new_local(|(id, req): &(Uuid, UpdateProductRequest)| {
        let id = *id;
        let req = req.clone();
        async move { catalog_api::update_product(id, req).await }
    });

    let delete_action = Action::new_local(|id: &Uuid| {
        let id = *id;
        async move { catalog_api::delete_product(id).await }
    });

    let create_variant_action = Action::new_local(|req: &CreateVariantRequest| {
        let req = req.clone();
        async move { catalog_api::create_variant(req).await }
    });

    let delete_variant_action = Action::new_local(|id: &Uuid| {
        let id = *id;
        async move { catalog_api::delete_variant(id).await }
    });

    let on_submit_product = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        if let Ok(c_id) = Uuid::parse_str(&category_id.get()) {
            let s = slug.get();
            let d = description.get();
            
            if let Some(p) = editing_product.get() {
                update_action.dispatch((p.id, UpdateProductRequest {
                    name: name.get(),
                    slug: if s.is_empty() { None } else { Some(s) },
                    description: if d.is_empty() { None } else { Some(d) },
                    category_id: c_id,
                }));
            } else {
                create_action.dispatch(CreateProductRequest {
                    name: name.get(),
                    slug: if s.is_empty() { None } else { Some(s) },
                    description: if d.is_empty() { None } else { Some(d) },
                    category_id: c_id,
                });
            }
        }
    };

    let on_submit_variant = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        if let Some(p) = selected_product_for_variants.get() {
            if let Ok(price) = Decimal::from_str(&v_price.get()) {
                create_variant_action.dispatch(CreateVariantRequest {
                    product_id: p.id,
                    sku: v_sku.get(),
                    name: v_name.get(),
                    base_price: price,
                    sale_price: None,
                    attributes: serde_json::json!({}),
                });
            }
        }
    };

    Effect::new(move |_| {
        if let Some(Ok(_)) = create_variant_action.value().get() {
            set_v_sku.set("".to_string());
            set_v_name.set("".to_string());
            set_v_price.set("".to_string());
            variants_resource.refetch();
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(true)) = create_action.value().get() {
            set_show_modal.set(false);
            products_resource.refetch();
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(true)) = update_action.value().get() {
            set_show_modal.set(false);
            set_editing_product.set(None);
            products_resource.refetch();
        }
    });

    Effect::new(move |_| {
        if let Some(p) = editing_product.get() {
            set_name.set(p.name);
            set_slug.set(p.slug);
            set_description.set(p.description.unwrap_or_default());
            set_category_id.set(p.category_id.to_string());
            set_show_modal.set(true);
        }
    });

    view! {
        <div class="p-6 space-y-6 animate-in fade-in duration-500">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-black text-gray-900">"Products"</h1>
                    <p class="text-sm text-gray-400 mt-1">"Manage your product catalog and variants"</p>
                </div>
                <button 
                    class="bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold px-4 py-2.5 rounded-xl transition-all shadow-lg shadow-yellow-200/50 flex items-center gap-2 cursor-pointer"
                    on:click=move |_| {
                        set_editing_product.set(None);
                        set_name.set("".to_string());
                        set_slug.set("".to_string());
                        set_description.set("".to_string());
                        set_category_id.set("".to_string());
                        set_show_modal.set(true);
                    }
                >
                    {icon_plus()} "New Product"
                </button>
            </div>

            // Products Table
            <div class="bg-white rounded-3xl border border-gray-100 shadow-xl overflow-hidden">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-gray-50 text-[10px] font-black text-gray-400 uppercase tracking-widest">
                            <th class="px-6 py-4">"Product"</th>
                            <th class="px-6 py-4">"Slug"</th>
                            <th class="px-6 py-4">"Category"</th>
                            <th class="px-6 py-4 text-right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-50">
                        <Suspense fallback=|| view! { <tr><td colspan="4" class="p-10 text-center text-gray-400">"Loading products..."</td></tr> }>
                            {move || products_resource.get().map(|list| {
                                let categories = categories_resource.get().map(|s| (*s).clone()).unwrap_or_default();
                                
                                (*list).iter().map(|p| {
                                    let p_cloned = p.clone();
                                    let p_cloned_variant = p.clone();
                                    let p_cloned_edit = p.clone();
                                    let p_id = p.id;
                                    let cat_name = categories.iter()
                                        .find(|c| c.id == p.category_id)
                                        .map(|c| c.name.clone())
                                        .unwrap_or_else(|| "Unknown".to_string());

                                    view! {
                                        <tr class="hover:bg-gray-50/50 transition-colors group">
                                            <td class="px-6 py-4 min-w-[200px]">
                                                <div class="font-bold text-gray-900">{p_id.to_string()}</div>
                                                <div class="text-[10px] text-gray-400 truncate max-w-[200px]">{p_cloned.name.clone()}</div>
                                            </td>
                                            <td class="px-6 py-4 text-xs font-mono text-gray-400">{p_cloned.slug.clone()}</td>
                                            <td class="px-6 py-4">
                                                <span class="px-2.5 py-1 rounded-lg bg-gray-50 text-gray-600 text-[10px] font-bold uppercase tracking-wider">
                                                    {cat_name}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 text-right">
                                                <div class="flex justify-end gap-2">
                                                    <button 
                                                        class="p-2 rounded-lg text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 transition-all cursor-pointer flex items-center gap-1 text-[10px] font-black"
                                                        on:click=move |_| {
                                                            set_selected_product_for_variants.set(Some(p_cloned_variant.clone()));
                                                            set_show_variants_modal.set(true);
                                                        }
                                                    >
                                                        {icon_box()} "VAR"
                                                    </button>
                                                    <button 
                                                        class="p-2 rounded-lg text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 transition-all cursor-pointer"
                                                        on:click=move |_| set_editing_product.set(Some(p_cloned_edit.clone()))
                                                    >
                                                        {icon_edit()}
                                                    </button>
                                                    <button 
                                                        class="p-2 rounded-lg text-gray-400 hover:text-red-600 hover:bg-red-50 transition-all cursor-pointer"
                                                        on:click=move |_| {
                                                            delete_action.dispatch(p_id);
                                                            products_resource.refetch();
                                                        }
                                                    >
                                                        {icon_trash()}
                                                    </button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>

            // Create/Edit Product Modal
            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm animate-in fade-in duration-200">
                    <div class="bg-white rounded-3xl shadow-2xl w-full max-w-lg overflow-hidden animate-in zoom-in-95 duration-200">
                        <div class="p-6 border-b border-gray-50">
                            <h3 class="text-xl font-black text-gray-900">
                                {move || if editing_product.get().is_some() { "Edit Product" } else { "Add New Product" }}
                            </h3>
                        </div>
                        <form on:submit=on_submit_product class="p-6 space-y-4">
                            <div class="grid grid-cols-2 gap-4">
                                <div class="space-y-1">
                                    <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Product Name"</label>
                                    <input 
                                        type="text" 
                                        placeholder="e.g. Áo Thun Modal" 
                                        required
                                        class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                        on:input=move |ev| set_name.set(event_target_value(&ev))
                                        prop:value=name
                                    />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Slug (Optional)"</label>
                                    <input 
                                        type="text" 
                                        placeholder="e.g. ao-thun-modal" 
                                        class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                        on:input=move |ev| set_slug.set(event_target_value(&ev))
                                        prop:value=slug
                                    />
                                </div>
                            </div>

                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Category"</label>
                                <select 
                                    required
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:change=move |ev| set_category_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select Category"</option>
                                    <Suspense fallback=|| view! { <option>"Loading..."</option> }>
                                        {move || categories_resource.get().map(|list| {
                                            (*list).iter().map(|c| {
                                                let is_selected = category_id.get() == c.id.to_string();
                                                view! {
                                                    <option value=c.id.to_string() selected=is_selected>
                                                        {c.name.clone()}
                                                    </option>
                                                }
                                            }).collect_view()
                                        })}
                                    </Suspense>
                                </select>
                            </div>

                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Description (Optional)"</label>
                                <textarea 
                                    placeholder="Tell more about this product..." 
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm min-h-24"
                                    on:input=move |ev| set_description.set(event_target_value(&ev))
                                    prop:value=description
                                ></textarea>
                            </div>

                            <div class="flex gap-3 pt-4">
                                <button 
                                    type="button"
                                    class="flex-1 px-6 py-3 rounded-2xl font-bold text-gray-500 hover:bg-gray-50 transition-all cursor-pointer"
                                    on:click=move |_| {
                                        set_show_modal.set(false);
                                        set_editing_product.set(None);
                                    }
                                >
                                    "Cancel"
                                </button>
                                <button 
                                    type="submit"
                                    disabled=move || create_action.pending().get() || update_action.pending().get()
                                    class="flex-1 px-6 py-3 rounded-2xl bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold transition-all shadow-lg shadow-yellow-200/50 cursor-pointer disabled:opacity-50"
                                >
                                    {move || if create_action.pending().get() || update_action.pending().get() { 
                                        "Saving..." 
                                    } else if editing_product.get().is_some() {
                                        "Save Changes"
                                    } else {
                                        "Create Product" 
                                    }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}

            // Variants Modal
            {move || show_variants_modal.get().then(|| {
                let p = selected_product_for_variants.get().unwrap();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm animate-in fade-in duration-200">
                        <div class="bg-white rounded-3xl shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col animate-in zoom-in-95 duration-200">
                            <div class="p-6 border-b border-gray-50 flex items-center justify-between">
                                <div>
                                    <h3 class="text-xl font-black text-gray-900">{format!("Variants for {}", p.name)}</h3>
                                    <p class="text-xs text-gray-400">"Add and manage SKUs and pricing"</p>
                                </div>
                                <button on:click=move |_| set_show_variants_modal.set(false) class="text-gray-400 hover:text-gray-600 cursor-pointer">"✕"</button>
                            </div>

                            <div class="flex-1 overflow-auto p-6 flex gap-6">
                                // New Variant Form
                                <div class="w-1/3 border-r border-gray-100 pr-6 space-y-4">
                                    <h4 class="text-sm font-bold text-gray-800">"Add New Variant"</h4>
                                    <form on:submit=on_submit_variant class="space-y-4">
                                        <div class="space-y-1">
                                            <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"SKU"</label>
                                            <input type="text" prop:value=v_sku on:input=move |ev| set_v_sku.set(event_target_value(&ev)) class="w-full px-4 py-2.5 rounded-xl bg-gray-50 border border-gray-100 text-sm outline-none" placeholder="AT-MODAL-M-RED" required />
                                        </div>
                                        <div class="space-y-1">
                                            <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Display Name"</label>
                                            <input type="text" prop:value=v_name on:input=move |ev| set_v_name.set(event_target_value(&ev)) class="w-full px-4 py-2.5 rounded-xl bg-gray-50 border border-gray-100 text-sm outline-none" placeholder="Red, Size M" required />
                                        </div>
                                        <div class="space-y-1">
                                            <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Price"</label>
                                            <input type="text" prop:value=v_price on:input=move |ev| set_v_price.set(event_target_value(&ev)) class="w-full px-4 py-2.5 rounded-xl bg-gray-50 border border-gray-100 text-sm outline-none" placeholder="19.99" required />
                                        </div>
                                        <button type="submit" class="w-full bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold py-2.5 rounded-xl transition-all cursor-pointer">"Add Variant"</button>
                                    </form>
                                </div>

                                // Variants List
                                <div class="flex-1">
                                    <Suspense fallback=|| view! { <div class="text-center p-10 text-gray-400">"Loading variants..."</div> }>
                                        {move || variants_resource.get().map(|v_data| match *v_data {
                                            Some(ref data) => {
                                                if data.variants.is_empty() {
                                                    view! { <div class="text-center p-10 text-gray-400 italic">"No variants yet."</div> }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="space-y-3">
                                                            {data.variants.iter().map(|v| {
                                                                let v_id = v.id;
                                                                view! {
                                                                    <div class="p-4 bg-gray-50 rounded-2xl flex items-center justify-between group">
                                                                        <div>
                                                                            <p class="font-bold text-gray-900 text-sm">{v.name.clone()}</p>
                                                                            <p class="text-[10px] text-gray-400 font-mono uppercase tracking-wider">{v.sku.clone()}</p>
                                                                        </div>
                                                                        <div class="flex items-center gap-4">
                                                                            <div class="text-sm font-black text-gray-900">{"$"}{v.base_price.to_string()}</div>
                                                                            <button 
                                                                                class="p-2 text-red-500 hover:bg-red-100 rounded-lg transition-colors cursor-pointer"
                                                                                on:click=move |_| {
                                                                                    delete_variant_action.dispatch(v_id);
                                                                                    variants_resource.refetch();
                                                                                }
                                                                            >
                                                                                {icon_trash()}
                                                                            </button>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            },
                                            None => view! { <div class="text-red-500">"Failed to load variants"</div> }.into_any()
                                        })}
                                    </Suspense>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
