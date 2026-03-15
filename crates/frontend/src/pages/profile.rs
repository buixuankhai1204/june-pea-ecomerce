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

    let update_error = RwSignal::new(Option::<String>::None);
    let update_success = RwSignal::new(false);
    let (email, set_email) = signal(String::new());

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
                </div>
            </div>
        </div>
    }
}
