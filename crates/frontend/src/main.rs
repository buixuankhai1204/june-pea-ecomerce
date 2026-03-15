#![allow(dead_code)]
mod api;
mod components;
mod pages;
mod state;

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use components::admin_layout::AdminLayout;
use components::auth_guard::AuthGuard;
use components::layout::MainLayout;
use pages::admin::categories::AdminCategoriesPage;
use pages::admin::dashboard::AdminDashboardPage;
use pages::admin::inventory::AdminInventoryPage;
use pages::admin::memberships::AdminMembershipsPage;
use pages::admin::orders::AdminOrdersPage;
use pages::admin::payment_invoices::AdminPaymentInvoicesPage;
use pages::admin::products::AdminProductsPage;
use pages::admin::promotions::AdminPromotionsPage;
use pages::admin::reports_analytics::AdminReportsAnalyticsPage;
use pages::admin::staff::AdminStaffPage;
use pages::admin::suppliers::AdminSuppliersPage;
use pages::cart::CartPage;
use pages::checkout::CheckoutPage;
use pages::dashboard::DashboardPage;
use pages::home::HomePage;
use pages::login::LoginPage;
use pages::orders::OrdersPage;
use pages::product_detail::ProductDetailPage;
use pages::products::ProductsPage;
use pages::profile::ProfilePage;
use pages::register::RegisterPage;
use state::auth::AuthState;
use state::cart::CartState;

fn main() {
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    // Provide global state
    let auth_state = AuthState::new();
    let cart_state = CartState::new();

    provide_context(auth_state);
    provide_context(cart_state);

    view! {
        <Router>
            <MainLayout>
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=path!("/") view=HomePage />
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/register") view=RegisterPage />
                    <Route path=path!("/profile") view=|| view! {
                        <AuthGuard>
                            <ProfilePage />
                        </AuthGuard>
                    } />
                    <Route path=path!("/dashboard") view=DashboardPage />
                    <Route path=path!("/products") view=ProductsPage />
                    <Route path=path!("/products/:slug") view=ProductDetailPage />
                    <Route path=path!("/cart") view=CartPage />
                    <Route path=path!("/checkout") view=|| view! {
                        <AuthGuard>
                            <CheckoutPage />
                        </AuthGuard>
                    } />
                    <Route path=path!("/orders") view=|| view! {
                        <AuthGuard>
                            <OrdersPage />
                        </AuthGuard>
                    } />

                    // Admin routes
                    <Route path=path!("/admin") view=|| view! {
                        <AdminLayout>
                            <AdminDashboardPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/orders") view=|| view! {
                        <AdminLayout>
                            <AdminOrdersPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/suppliers") view=|| view! {
                        <AdminLayout>
                            <AdminSuppliersPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/staff") view=|| view! {
                        <AdminLayout>
                            <AdminStaffPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/payments") view=|| view! {
                        <AdminLayout>
                            <AdminPaymentInvoicesPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/promotions") view=|| view! {
                        <AdminLayout>
                            <AdminPromotionsPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/memberships") view=|| view! {
                        <AdminLayout>
                            <AdminMembershipsPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/reports") view=|| view! {
                        <AdminLayout>
                            <AdminReportsAnalyticsPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/categories") view=|| view! {
                        <AdminLayout>
                            <AdminCategoriesPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/catalog") view=|| view! {
                        <AdminLayout>
                            <AdminProductsPage />
                        </AdminLayout>
                    } />
                    <Route path=path!("/admin/stock") view=|| view! {
                        <AdminLayout>
                            <AdminInventoryPage />
                        </AdminLayout>
                    } />
                </Routes>
            </MainLayout>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-[60vh] flex items-center justify-center">
            <div class="text-center">
                <h1 class="text-6xl font-bold text-gray-300">"404"</h1>
                <p class="mt-4 text-xl text-gray-500">"Page not found"</p>
                <a href="/" class="mt-6 inline-block text-indigo-600 hover:underline">"Go Home"</a>
            </div>
        </div>
    }
}
