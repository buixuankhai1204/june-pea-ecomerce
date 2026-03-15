use leptos::prelude::*;

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
                    <a href="/" class="text-xl font-bold text-indigo-600">"Yame"</a>

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
