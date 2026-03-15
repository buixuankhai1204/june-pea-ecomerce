use crate::api::client::catalog as catalog_api;
use crate::api::types::CreateCategoryRequest;
use leptos::either::Either;
use leptos::prelude::*;
use uuid::Uuid;

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

fn icon_folder() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-indigo-500" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
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
pub fn AdminCategoriesPage() -> impl IntoView {
    let categories_resource: LocalResource<Vec<crate::api::types::Category>> =
        LocalResource::new(move || async move {
            catalog_api::list_categories().await.unwrap_or_default()
        });

    let (name, set_name) = signal(String::new());
    let (parent_id, set_parent_id) = signal(None::<Uuid>);

    let create_category_action = Action::new_local(move |req: &CreateCategoryRequest| {
        let req = req.clone();
        async move { catalog_api::create_category(req).await }
    });

    let delete_category_action = Action::new_local(|id: &Uuid| {
        let id = *id;
        async move { catalog_api::delete_category(id).await }
    });

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        let req = CreateCategoryRequest {
            name: name.get(),
            slug: None,
            parent_id: parent_id.get(),
        };
        create_category_action.dispatch(req);
    };

    Effect::new(move |_| {
        if create_category_action.value().get().is_some() {
            set_name.set(String::new());
            categories_resource.refetch();
        }
    });

    view! {
        <div class="p-6 space-y-6 animate-in fade-in duration-500">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-black text-gray-900">"Categories"</h1>
                    <p class="text-sm text-gray-400 mt-1">"Organize your products into a hierarchical structure"</p>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left: Create Form
                <div class="lg:col-span-1">
                    <div class="bg-white rounded-3xl p-6 border border-gray-100 shadow-xl">
                        <h2 class="text-lg font-bold text-gray-800 mb-6">"New Category"</h2>
                        <form on:submit=on_submit class="space-y-4">
                            <div>
                                <label class="block text-xs font-bold text-gray-400 uppercase tracking-wider mb-1">"Category Name"</label>
                                <input
                                    type="text"
                                    prop:value=name
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                    placeholder="e.g. Menswear"
                                    class="w-full bg-gray-50 border-none rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-[#FCE300] transition-all outline-none"
                                    required
                                />
                            </div>

                            <div>
                                <label class="block text-xs font-bold text-gray-400 uppercase tracking-wider mb-1">"Parent Category (Optional)"</label>
                                <select
                                    class="w-full bg-gray-50 border-none rounded-xl px-4 py-3 text-sm focus:ring-2 focus:ring-[#FCE300] transition-all outline-none"
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        if val.is_empty() {
                                            set_parent_id.set(None);
                                        } else {
                                            set_parent_id.set(Uuid::parse_str(&val).ok());
                                        }
                                    }
                                >
                                    <option value="">"No Parent"</option>
                                    <Suspense fallback=|| view! { <option>"Loading..."</option> }>
                                        {move || categories_resource.get().map(|list| {
                                            list.iter().map(|cat| view! {
                                                <option value=cat.id.to_string()>{cat.name.clone()}</option>
                                            }).collect_view()
                                        })}
                                    </Suspense>
                                </select>
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
                <div class="lg:col-span-2">
                    <div class="bg-white rounded-3xl border border-gray-100 shadow-xl overflow-hidden">
                        <div class="p-6 border-b border-gray-100 flex items-center justify-between">
                            <h2 class="text-lg font-bold text-gray-800">"All Categories"</h2>
                            <span class="bg-indigo-50 text-indigo-600 text-[10px] font-black px-2.5 py-1 rounded-full uppercase tracking-tighter">
                                {move || categories_resource.get().map(|c| c.len()).unwrap_or(0)} " total"
                            </span>
                        </div>
                        <div class="divide-y divide-gray-50 max-h-[600px] overflow-y-auto">
                            <Suspense fallback=|| view! { <div class="p-10 text-center text-gray-400">"Loading categories..."</div> }>
                                {move || categories_resource.get().map(|list| {
                                    if list.is_empty() {
                                        Either::Left(view! { <div class="p-10 text-center text-gray-400">"No categories found yet."</div> })
                                    } else {
                                        Either::Right(list.iter().map(|cat| {
                                            let cat_id = cat.id;
                                            view! {
                                                <div class="p-4 hover:bg-gray-50/50 flex items-center justify-between group transition-colors">
                                                    <div class="flex items-center gap-4">
                                                        <div class="w-10 h-10 bg-indigo-50 rounded-xl flex items-center justify-center">
                                                            {icon_folder()}
                                                        </div>
                                                        <div>
                                                            <p class="text-sm font-bold text-gray-900">{cat.name.clone()}</p>
                                                            <p class="text-[10px] text-gray-400 font-mono mt-0.5">{cat.slug.clone()}</p>
                                                        </div>
                                                    </div>
                                                    <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                        <button
                                                            class="p-2 text-red-500 hover:bg-red-50 rounded-lg transition-colors cursor-pointer"
                                                            on:click=move |_| {
                                                                delete_category_action.dispatch(cat_id);
                                                                categories_resource.refetch();
                                                            }
                                                        >
                                                            {icon_trash()}
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view())
                                    }
                                })}
                            </Suspense>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
