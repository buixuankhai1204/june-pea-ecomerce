use leptos::prelude::*;
use crate::api::client::catalog::*;
use crate::api::types::CreateCategoryRequest;
use uuid::Uuid;

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" 
            stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

fn icon_trash() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
    }
}

#[component]
pub fn AdminCategoriesPage() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (slug, set_slug) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (parent_id, set_parent_id) = signal(String::new());

    let categories_resource = LocalResource::new(|| async move { list_categories().await });

    let create_action = Action::new_local(|req: &CreateCategoryRequest| {
        let req = req.clone();
        async move { create_category(req).await }
    });

    let delete_action = Action::new_local(|id: &Uuid| {
        let id = *id;
        async move { delete_category(id).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let p_id = parent_id.get();
        let parent = if p_id.is_empty() {
            None
        } else {
            Uuid::parse_str(&p_id).ok()
        };

        create_action.dispatch(CreateCategoryRequest {
            name: name.get(),
            slug: Some(slug.get()),
            parent_id: parent,
        });
    };

    Effect::new(move |_| {
        if create_action.value().get().is_some() || delete_action.value().get().is_some() {
            categories_resource.refetch();
            set_name.set(String::new());
            set_slug.set(String::new());
            set_description.set(String::new());
            set_parent_id.set(String::new());
        }
    });

    view! {
        <div class="p-6 space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Categories"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Organize your product catalog with multi-level categories"</p>
                </div>
                <div class="bg-indigo-50 border border-indigo-100 px-4 py-2 rounded-xl">
                    <span class="text-xs font-bold text-indigo-600">
                        {move || match categories_resource.get() {
                            Some(sw) => {
                                let res = &*sw;
                                match res {
                                    Ok(list) => format!("{} Total Categories", list.len()),
                                    _ => "... Categories".to_string()
                                }
                            },
                            _ => "... Categories".to_string()
                        }}
                    </span>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left: Create Form
                <div class="lg:col-span-1">
                    <div class="bg-white rounded-3xl p-6 shadow-sm border border-gray-100 sticky top-6">
                        <h2 class="text-sm font-bold text-gray-900 mb-6">"Create New Category"</h2>
                        
                        <form on:submit=on_submit class="space-y-4">
                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Category Name"</label>
                                <input 
                                    type="text" 
                                    required
                                    placeholder="e.g. Arabica Coffee"
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                    prop:value=name
                                />
                            </div>

                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Slug (URL path)"</label>
                                <input 
                                    type="text" 
                                    required
                                    placeholder="arabica-coffee"
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:input=move |ev| set_slug.set(event_target_value(&ev))
                                    prop:value=slug
                                />
                            </div>

                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Parent Category"</label>
                                <select 
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm appearance-none"
                                    on:change=move |ev| set_parent_id.set(event_target_value(&ev))
                                    prop:value=parent_id
                                >
                                    <option value="">"No Parent"</option>
                                    <Suspense fallback=|| view! { <option>"Loading..."</option> }.into_any()>
                                        {move || match categories_resource.get() {
                                            Some(sw) => {
                                                let res = &*sw;
                                                match res {
                                                    Ok(list) => list.iter().map(|cat| {
                                                        let c_name = cat.name.clone();
                                                        let c_id = cat.id.to_string();
                                                        view! { <option value=c_id>{c_name}</option> }
                                                    }).collect_view().into_any(),
                                                    _ => view! { <option value="">"Error loading"</option> }.into_any()
                                                }
                                            },
                                            None => view! { <option value="">"Loading..."</option> }.into_any()
                                        }}
                                    </Suspense>
                                </select>
                            </div>

                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Description"</label>
                                <textarea 
                                    rows="3"
                                    placeholder="Optional category summary..."
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm resize-none"
                                    on:input=move |ev| set_description.set(event_target_value(&ev))
                                    prop:value=description
                                ></textarea>
                            </div>

                            <button 
                                type="submit"
                                class="w-full bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold py-3 rounded-xl shadow-lg shadow-yellow-200/50 transition-all active:scale-[0.98] cursor-pointer"
                            >
                                {icon_plus()} "Create Category"
                            </button>
                        </form>
                    </div>
                </div>

                // Right: List
                <div class="lg:col-span-2 space-y-4">
                    <Suspense fallback=move || view! { <p class="text-center py-12 text-gray-400">"Loading catalog..."</p> }>
                        {move || match categories_resource.get() {
                            Some(sw) => {
                                let res = &*sw;
                                match res {
                                    Ok(list) => {
                                        if list.is_empty() {
                                            view! { <div class="bg-gray-50 border-2 border-dashed border-gray-200 rounded-3xl py-12 text-center text-gray-400 font-medium">"No categories found. Create one to get started."</div> }.into_any()
                                        } else {
                                            list.iter().map(|cat| {
                                                let c_id = cat.id;
                                                let c_name = cat.name.clone();
                                                let c_slug = cat.slug.clone();
                                                view! {
                                                    <div class="bg-white rounded-2xl p-4 shadow-sm border border-gray-100 flex items-center justify-between group hover:border-[#FCE300] transition-colors">
                                                        <div class="flex items-center gap-4">
                                                            <div class="w-10 h-10 rounded-xl bg-gray-50 flex items-center justify-center text-gray-400 font-black text-xs">
                                                                {c_name.chars().next().unwrap_or('?').to_string()}
                                                            </div>
                                                            <div>
                                                                <p class="text-sm font-bold text-gray-900">{c_name}</p>
                                                                <p class="text-[11px] text-gray-400">"slug: " {c_slug}</p>
                                                            </div>
                                                        </div>
                                                        <button 
                                                            on:click=move |_| { delete_action.dispatch(c_id); }
                                                            class="p-2 text-gray-300 hover:text-rose-500 hover:bg-rose-50 rounded-lg transition-all opacity-0 group-hover:opacity-100"
                                                        >
                                                            {icon_trash()}
                                                        </button>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    },
                                    _ => view! { <div class="text-rose-500">"Error loading categories"</div> }.into_any()
                                }
                            },
                            None => view! { <p>"Loading..."</p> }.into_any()
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
