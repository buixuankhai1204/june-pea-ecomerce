use crate::api::client;
use crate::api::types::User;
use crate::state::auth::AuthState;
use leptos::prelude::*;

fn icon_user() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-12 h-12 text-gray-300" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
            <circle cx="12" cy="7" r="4"></circle>
        </svg>
    }
}

fn icon_mail() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
            <polyline points="22,6 12,13 2,6"></polyline>
        </svg>
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let user_resource =
        LocalResource::new(move || async move { client::get::<User>("/api/v1/identity/me").await });

    let orders_resource = LocalResource::new(move || {
        let auth = auth.clone();
        async move {
            if let Some(user_id) = auth.current_user_id() {
                client::ordering::list_recent_orders(user_id).await
            } else {
                Ok(vec![])
            }
        }
    });

    let (email, set_email) = signal(String::new());
    let update_success = RwSignal::new(false);
    let update_error = RwSignal::new(Option::<String>::None);

    // Password Update signals
    let (old_password, set_old_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let pwd_error = RwSignal::new(Option::<String>::None);
    let pwd_success = RwSignal::new(false);
    let pwd_loading = RwSignal::new(false);

    let on_update = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        let email_val = email.get();
        if email_val.is_empty() {
            return;
        }

        let user_resource = user_resource.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({ "email": email_val });
            match client::patch::<serde_json::Value, _>("/api/v1/identity/profile", &body).await {
                Ok(_) => {
                    update_success.set(true);
                    update_error.set(None);
                    user_resource.refetch();
                }
                Err(e) => {
                    update_error.set(Some(e.user_message().to_string()));
                    update_success.set(false);
                }
            }
        });
    };

    let on_change_password = move |e: web_sys::SubmitEvent| {
        e.prevent_default();
        let old = old_password.get();
        let new = new_password.get();
        let confirm = confirm_password.get();

        if new != confirm {
            pwd_error.set(Some("New passwords do not match".to_string()));
            return;
        }

        if new.len() < 8 {
            pwd_error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }

        pwd_loading.set(true);
        pwd_error.set(None);
        pwd_success.set(false);

        wasm_bindgen_futures::spawn_local(async move {
            let req = crate::api::types::ChangePasswordRequest {
                old_password: old,
                new_password: new,
            };
            match client::identity::change_password(req).await {
                Ok(_) => {
                    pwd_success.set(true);
                    set_old_password.set(String::new());
                    set_new_password.set(String::new());
                    set_confirm_password.set(String::new());
                }
                Err(e) => {
                    pwd_error.set(Some(e.user_message().to_string()));
                }
            }
            pwd_loading.set(false);
        });
    };

    let on_delete_account = move |_| {
        let auth = auth.clone();
        if let Some(user_id) = auth.current_user_id() {
             wasm_bindgen_futures::spawn_local(async move {
                if let Ok(_) = client::identity::delete_user(user_id).await {
                    auth.logout();
                }
            });
        }
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-12">
            <div class="flex flex-col md:flex-row gap-8 items-start">
                // LEFT: Profile Header / Card
                <div class="w-full md:w-1/3 bg-white rounded-3xl p-8 border border-gray-100 shadow-sm text-center">
                    <div class="w-24 h-24 bg-gray-50 rounded-full mx-auto flex items-center justify-center mb-4 border border-gray-100">
                        {icon_user()}
                    </div>
                    <Suspense fallback=|| view! { <div class="h-6 bg-gray-50 rounded animate-pulse w-3/4 mx-auto"></div> }.into_any()>
                        {move || user_resource.get().map(|res| match res.as_ref() {
                            Ok(u) => {
                                let email = u.email.clone();
                                set_email.set(email.clone());
                                view! {
                                    <h2 class="text-xl font-bold text-gray-900">{email.clone()}</h2>
                                    <p class="text-sm text-gray-400 mt-1">{email}</p>
                                }.into_any()
                            },
                            Err(_) => view! { <p class="text-red-500">"Guest"</p> }.into_any(),
                        })}
                    </Suspense>

                    <div class="mt-8 pt-8 border-t border-gray-50 space-y-3 font-medium">
                        <a href="/orders" class="block py-2 text-sm text-gray-600 hover:text-black transition-colors">"My Orders"</a>
                        <button
                            on:click=move |_| auth.logout()
                            class="block w-full py-2 text-sm text-red-500 hover:text-red-700 transition-colors"
                        >
                            "Sign Out"
                        </button>
                    </div>
                </div>

                // RIGHT: Settings Form
                <div class="flex-1 bg-white rounded-3xl p-8 border border-gray-100 shadow-sm">
                    <h3 class="text-lg font-bold text-gray-900 mb-6">"Account Settings"</h3>

                    <form on:submit=on_update class="space-y-6">
                        <div class="space-y-2">
                            <label class="text-xs font-black text-gray-400 uppercase tracking-widest">"Email Address"</label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                    {icon_mail()}
                                </div>
                                <input
                                    type="email"
                                    prop:value=move || email.get()
                                    on:input=move |e| set_email.set(event_target_value(&e))
                                    class="w-full pl-10 pr-4 py-3 bg-gray-50 border border-transparent focus:bg-white focus:border-[#FCE300] rounded-xl outline-none transition-all text-sm"
                                />
                            </div>
                        </div>

                        {move || if let Some(err) = update_error.get() {
                            view! { <p class="text-xs text-red-500 font-bold">{err}</p> }.into_any()
                        } else if update_success.get() {
                            view! { <p class="text-xs text-emerald-600 font-bold">"Profile updated successfully!"</p> }.into_any()
                        } else {
                            view! { <div class="h-4"></div> }.into_any()
                        }}

                        <button
                            type="submit"
                            class="bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold px-8 py-3 rounded-xl transition-all shadow-lg shadow-yellow-200/50"
                        >
                            "Save Changes"
                        </button>
                    </form>

                    <div class="mt-12 pt-12 border-t border-gray-100">
                        <h3 class="text-lg font-bold text-gray-900 mb-6">"Change Password"</h3>
                        <form on:submit=on_change_password class="space-y-4">
                            <div class="grid md:grid-cols-2 gap-4">
                                <div class="space-y-2">
                                    <label class="text-xs font-black text-gray-400 uppercase tracking-widest">"Old Password"</label>
                                    <input
                                        type="password"
                                        prop:value=move || old_password.get()
                                        on:input=move |e| set_old_password.set(event_target_value(&e))
                                        class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:bg-white focus:border-[#FCE300] rounded-xl outline-none transition-all text-sm"
                                    />
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-black text-gray-400 uppercase tracking-widest">"New Password"</label>
                                    <input
                                        type="password"
                                        prop:value=move || new_password.get()
                                        on:input=move |e| set_new_password.set(event_target_value(&e))
                                        class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:bg-white focus:border-[#FCE300] rounded-xl outline-none transition-all text-sm"
                                    />
                                </div>
                            </div>
                            <div class="space-y-2">
                                <label class="text-xs font-black text-gray-400 uppercase tracking-widest">"Confirm New Password"</label>
                                <input
                                    type="password"
                                    prop:value=move || confirm_password.get()
                                    on:input=move |e| set_confirm_password.set(event_target_value(&e))
                                    class="w-full px-4 py-3 bg-gray-50 border border-transparent focus:bg-white focus:border-[#FCE300] rounded-xl outline-none transition-all text-sm"
                                />
                            </div>

                            {move || if let Some(err) = pwd_error.get() {
                                view! { <p class="text-xs text-red-500 font-bold">{err}</p> }.into_any()
                            } else if pwd_success.get() {
                                view! { <p class="text-xs text-emerald-600 font-bold">"Password updated successfully!"</p> }.into_any()
                            } else {
                                view! { <div class="h-4"></div> }.into_any()
                            }}

                            <button
                                type="submit"
                                disabled=move || pwd_loading.get()
                                class="bg-gray-900 hover:bg-black text-white font-bold px-8 py-3 rounded-xl transition-all shadow-lg"
                            >
                                {move || if pwd_loading.get() { "Updating..." } else { "Update Password" }}
                            </button>
                        </form>
                    </div>

                    <div class="mt-12 pt-8 border-t border-red-50">
                        <h3 class="text-lg font-bold text-red-600 mb-2">"Danger Zone"</h3>
                        <p class="text-sm text-gray-500 mb-6">"Once you delete your account, there is no going back. Please be certain."</p>
                        <button
                            on:click=on_delete_account
                            class="bg-white hover:bg-red-50 text-red-600 border border-red-200 font-bold px-8 py-3 rounded-xl transition-all"
                        >
                            "Delete Account"
                        </button>
                    </div>

                    // Recent Orders Component
                    <div class="mt-12 pt-12 border-t border-gray-100">
                        <h3 class="text-lg font-bold text-gray-900 mb-6">"Recent Orders"</h3>
                        <Suspense fallback=|| view! { <div class="space-y-4">{ (0..3).map(|_| view! { <div class="h-16 bg-gray-50 animate-pulse rounded-2xl"></div> }).collect_view() }</div> }>
                            {move || orders_resource.get().map(|res| match res.as_ref() {
                                Ok(orders) if !orders.is_empty() => {
                                    orders.iter().take(5).map(|o| {
                                        let date = o.created_at.format("%d/%m/%Y").to_string();
                                        let total = format!("₫{}k", o.total / 1000);
                                        let id_short = format!("#{}", &o.id.to_string()[..8]);
                                        let status_color = match o.status {
                                            crate::api::types::OrderStatus::Completed => "bg-emerald-50 text-emerald-600",
                                            crate::api::types::OrderStatus::Cancelled => "bg-rose-50 text-rose-600",
                                            crate::api::types::OrderStatus::Pending => "bg-amber-50 text-amber-600",
                                        };
                                        view! {
                                            <div class="flex items-center justify-between p-4 bg-gray-50 rounded-2xl mb-3 hover:bg-gray-100 transition-colors">
                                                <div>
                                                    <p class="text-sm font-bold text-gray-900">{id_short}</p>
                                                    <p class="text-[11px] text-gray-400 mt-0.5">{date}</p>
                                                </div>
                                                <div class="text-right">
                                                    <p class="text-sm font-black text-gray-900">{total}</p>
                                                    <span class=format!("text-[10px] font-bold px-2 py-0.5 rounded-full mt-1 inline-block {}", status_color)>
                                                        {o.status.to_string()}
                                                    </span>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view().into_any()
                                },
                                Ok(_) => view! { <p class="text-sm text-gray-400">"No orders yet. Time to shop!"</p> }.into_any(),
                                Err(e) => view! { <p class="text-sm text-red-500">{e.user_message().to_string()}</p> }.into_any(),
                            })}
                        </Suspense>
                        <a href="/orders" class="inline-block mt-4 text-sm text-indigo-600 font-bold hover:underline">"View all orders →"</a>
                    </div>
                </div>
            </div>
        </div>
    }
}
