use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::state::auth::AuthState;
use crate::state::cart::CartState;

#[component]
pub fn MainLayout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 flex flex-col">
            <Navbar />
            <main class="flex-1">
                {children()}
            </main>
            <footer class="bg-gray-800 text-gray-400 py-8 mt-auto">
                <div class="max-w-7xl mx-auto px-4 text-center text-sm">
                    "© 2026 Yame Store. All rights reserved."
                </div>
            </footer>
        </div>
    }
}

#[component]
fn Navbar() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let cart = expect_context::<CartState>();

    let is_logged_in = move || auth.user.get().is_some();
    let cart_count = move || {
        cart.items
            .get()
            .iter()
            .map(|i| i.quantity as usize)
            .sum::<usize>()
    };

    view! {
        <nav class="bg-white shadow-sm sticky top-0 z-50">
            <div class="max-w-7xl mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Logo
                    <div class="flex items-center gap-8">
                        <a href="/" class="text-xl font-bold text-indigo-600">"Yame"</a>

                        // Search Bar
                        <form
                            class="hidden md:flex items-center bg-gray-100 rounded-full px-4 py-1.5 w-64 lg:w-96"
                            on:submit=move |e| {
                                e.prevent_default();
                                let target = event_target::<web_sys::HtmlFormElement>(&e);
                                let form_data = web_sys::FormData::new_with_form(&target).unwrap();
                                if let Some(q) = form_data.get("q").as_string() {
                                    if !q.is_empty() {
                                        let _ = use_navigate()(&format!("/products?q={}", q), Default::default());
                                    }
                                }
                            }
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="11" cy="11" r="8"></circle>
                                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                            </svg>
                            <input
                                type="text"
                                name="q"
                                placeholder="Search products..."
                                class="bg-transparent border-none outline-none text-sm ml-2 w-full"
                            />
                        </form>
                    </div>

                    // Nav links
                    <div class="flex items-center gap-6">
                        <a href="/products" class="text-gray-700 hover:text-indigo-600 text-sm font-medium">
                            "Products"
                        </a>

                        // Cart
                        <a href="/cart" class="relative text-gray-700 hover:text-indigo-600">
                            <span class="text-sm font-medium">"Cart"</span>
                            {move || {
                                let count = cart_count();
                                if count > 0 {
                                    view! {
                                        <span class="absolute -top-2 -right-4 bg-indigo-600 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
                                            {count}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }
                            }}
                        </a>

                        // Auth section
                        {move || {
                            let auth = auth.clone();
                            if is_logged_in() {
                                view! {
                                    <a href="/orders" class="text-gray-700 hover:text-indigo-600 text-sm font-medium">
                                        "Orders"
                                    </a>
                                    <button
                                        class="text-sm text-red-500 hover:text-red-700 font-medium"
                                        on:click=move |_| {
                                            auth.logout();
                                            let _ = web_sys::window().and_then(|w| w.location().set_href("/").ok());
                                        }
                                    >
                                        "Logout"
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <a href="/login" class="text-sm font-medium text-white bg-indigo-600 px-4 py-2 rounded-md hover:bg-indigo-700">
                                        "Sign In"
                                    </a>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </nav>
    }
}
