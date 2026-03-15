use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api::client;
use crate::api::types::RegisterRequest;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let error = RwSignal::new(String::new());
    let success = RwSignal::new(false);
    let loading = RwSignal::new(false);

    let email_ref = NodeRef::<html::Input>::new();
    let password_ref = NodeRef::<html::Input>::new();
    let confirm_ref = NodeRef::<html::Input>::new();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let email = email_ref
            .get()
            .map(|el| {
                let el: &HtmlInputElement = &el;
                el.value()
            })
            .unwrap_or_default();
        let password = password_ref
            .get()
            .map(|el| {
                let el: &HtmlInputElement = &el;
                el.value()
            })
            .unwrap_or_default();
        let password_confirm = confirm_ref
            .get()
            .map(|el| {
                let el: &HtmlInputElement = &el;
                el.value()
            })
            .unwrap_or_default();

        loading.set(true);
        error.set(String::new());

        wasm_bindgen_futures::spawn_local(async move {
            let req = RegisterRequest {
                email,
                password,
                password_confirm,
            };
            match client::post::<serde_json::Value, _>("/api/v1/auth/register", &req).await {
                Ok(_) => {
                    success.set(true);
                }
                Err(e) => {
                    error.set(e.user_message().to_string());
                }
            }
            loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="max-w-md w-full space-y-8 p-8 bg-white rounded-xl shadow-lg">
                <h2 class="text-center text-3xl font-bold text-gray-900">"Create Account"</h2>

                {move || {
                    if success.get() {
                        view! {
                            <div class="bg-green-50 border border-green-300 text-green-700 px-4 py-3 rounded">
                                "Registration successful! "
                                <a href="/login" class="font-medium underline">"Sign in here"</a>
                            </div>
                        }.into_any()
                    } else {
                        let err = error.get();
                        if err.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            view! {
                                <div class="bg-red-50 border border-red-300 text-red-700 px-4 py-3 rounded">
                                    {err}
                                </div>
                            }.into_any()
                        }
                    }
                }}

                <form class="space-y-6" on:submit=on_submit>
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"Email"</label>
                        <input
                            node_ref=email_ref
                            type="email"
                            required=true
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                            placeholder="you@example.com"
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"Password"</label>
                        <input
                            node_ref=password_ref
                            type="password"
                            required=true
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700">"Confirm Password"</label>
                        <input
                            node_ref=confirm_ref
                            type="password"
                            required=true
                            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                        />
                    </div>
                    <button
                        type="submit"
                        class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Creating..." } else { "Create Account" }}
                    </button>
                </form>
                <p class="text-center text-sm text-gray-600">
                    "Already have an account? "
                    <a href="/login" class="font-medium text-indigo-600 hover:text-indigo-500">"Sign in"</a>
                </p>
            </div>
        </div>
    }
}
