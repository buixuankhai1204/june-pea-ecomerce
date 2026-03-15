use leptos::ev::SubmitEvent;
use leptos::html;
use leptos::prelude::*;
use web_sys::HtmlInputElement;

use crate::api::client;
use crate::api::types::{LoginRequest, LoginResponse};
use crate::state::auth::AuthState;

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let error = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let navigate = leptos_router::hooks::use_navigate();

    let email_ref = NodeRef::<html::Input>::new();
    let password_ref = NodeRef::<html::Input>::new();

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

        let auth = auth.clone();
        let navigate = navigate.clone();

        loading.set(true);
        error.set(String::new());

        wasm_bindgen_futures::spawn_local(async move {
            let req = LoginRequest { email, password };
            match client::post::<LoginResponse, _>("/api/v1/auth/login", &req).await {
                Ok(resp) => {
                    auth.login(&resp.token);
                    navigate("/", Default::default());
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
                <div>
                    <h2 class="text-center text-3xl font-bold text-gray-900">"Sign in"</h2>
                </div>

                {move || {
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
                    <button
                        type="submit"
                        class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Signing in..." } else { "Sign in" }}
                    </button>
                </form>
                <p class="text-center text-sm text-gray-600">
                    "Don't have an account? "
                    <a href="/register" class="font-medium text-indigo-600 hover:text-indigo-500">"Register"</a>
                </p>
            </div>
        </div>
    }
}
