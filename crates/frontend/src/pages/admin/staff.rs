use crate::api::client::identity as identity_api;
use crate::api::types::{RegisterRequest, User};
use leptos::prelude::*;

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

#[component]
pub fn AdminStaffPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let users_resource: LocalResource<Vec<User>> =
        LocalResource::new(
            move || async move { identity_api::list_users().await.unwrap_or_default() },
        );

    let register_action = Action::new_local(|req: &RegisterRequest| {
        let req = req.clone();
        async move { identity_api::register(req).await }
    });

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        register_action.dispatch(RegisterRequest {
            email: email.get(),
            password: password.get(),
            password_confirm: password.get(),
        });
    };

    Effect::new(move |_| {
        if register_action.value().get().is_some() {
            set_show_create_modal.set(false);
            set_email.set(String::new());
            set_password.set(String::new());
            users_resource.refetch();
        }
    });

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Staff Management"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Manage user accounts and administrative rolls"</p>
                </div>
                <button
                    on:click=move |_| set_show_create_modal.set(true)
                    class="flex items-center gap-1.5 bg-[#FCE300] hover:bg-yellow-400 text-gray-900 text-xs font-bold px-4 py-2.5 rounded-xl transition-colors cursor-pointer shadow-sm"
                >
                    {icon_plus()} "Add Staff"
                </button>
            </div>

            // Staff table
            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900">"User Overview"</h2>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-xs">
                        <thead>
                            <tr class="border-b border-gray-100 text-gray-400 uppercase tracking-wider text-[10px]">
                                <th class="px-5 py-3 text-left font-medium">"User ID"</th>
                                <th class="px-5 py-3 text-left font-medium">"Email"</th>
                                <th class="px-4 py-3 text-center font-medium">"Role"</th>
                                <th class="px-4 py-3 text-center font-medium">"Action"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-50">
                            <Suspense fallback=|| view! { <tr><td colspan="4" class="px-6 py-4 text-center">"Loading..."</td></tr> }>
                                {move || users_resource.get().map(|list| {
                                    list.iter().map(|u| {
                                        let role_color = if u.role == "Admin" { "bg-indigo-50 text-indigo-600 border-indigo-100" } else { "bg-emerald-50 text-emerald-600 border-emerald-100" };
                                        view! {
                                            <tr class="hover:bg-gray-50 transition-colors duration-100">
                                                <td class="px-5 py-3.5 font-mono text-gray-400">{u.id.to_string()[..8].to_string()}</td>
                                                <td class="px-5 py-3.5">
                                                    <div class="flex items-center gap-3">
                                                        <div class="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-500 text-[10px] font-bold">
                                                            {u.email.chars().next().unwrap_or('?').to_string().to_uppercase()}
                                                        </div>
                                                        <span class="font-semibold text-gray-900">{u.email.clone()}</span>
                                                    </div>
                                                </td>
                                                <td class="px-4 py-3.5 text-center">
                                                    <span class=format!("text-[10px] font-semibold px-2.5 py-1 rounded-full border {}", role_color)>
                                                        {u.role.clone()}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3.5 text-center">
                                                    <button class="text-gray-400 hover:text-rose-500 transition-colors">"Remove"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()
                                })}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            // Create Modal
            {move || show_create_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/20 backdrop-blur-sm animate-in fade-in duration-200">
                    <div class="bg-white rounded-3xl shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-200">
                        <div class="p-6 border-b border-gray-50">
                            <h3 class="text-xl font-black text-gray-900">"Add New Staff Member"</h3>
                        </div>
                        <form on:submit=on_submit class="p-6 space-y-4">
                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Email Address"</label>
                                <input
                                    type="email"
                                    required
                                    placeholder="staff@junepae.com"
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    prop:value=email
                                />
                            </div>
                            <div class="space-y-1">
                                <label class="text-[10px] font-black text-gray-400 uppercase tracking-widest px-1">"Initial Password"</label>
                                <input
                                    type="password"
                                    required
                                    placeholder="••••••••"
                                    class="w-full px-4 py-3 rounded-2xl bg-gray-50 border border-gray-100 focus:bg-white focus:ring-2 focus:ring-[#FCE300] outline-none transition-all text-sm"
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    prop:value=password
                                />
                            </div>
                            <div class="flex gap-3 pt-4">
                                <button
                                    type="button"
                                    class="flex-1 px-6 py-3 rounded-2xl font-bold text-gray-500 hover:bg-gray-50 transition-all cursor-pointer"
                                    on:click=move |_| set_show_create_modal.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="submit"
                                    class="flex-1 px-6 py-3 rounded-2xl bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold transition-all shadow-lg cursor-pointer"
                                >
                                    "Add Staff"
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}
        </div>
    }
}
