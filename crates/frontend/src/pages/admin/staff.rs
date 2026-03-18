use leptos::prelude::*;
use crate::api::client::identity::*;

fn icon_plus() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" 
            stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
    }
}

#[component]
pub fn AdminStaffPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = signal(false);
    
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let staff_resource = LocalResource::new(|| async move { list_staff().await });
    let create_staff_action = Action::new_local(|(e, p): &(String, String)| {
        let e = e.clone();
        let p = p.clone();
        async move { create_staff(&e, &p).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        create_staff_action.dispatch((email.get(), password.get()));
    };

    Effect::new(move |_| {
        if create_staff_action.value().get().is_some() {
            set_show_create_modal.set(false);
            staff_resource.refetch();
        }
    });

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Staff Management"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Manage administrators and operational staff roles"</p>
                </div>
                <button 
                    on:click=move |_| set_show_create_modal.set(true)
                    class="bg-[#FCE300] hover:bg-yellow-400 text-gray-900 font-bold px-4 py-2.5 rounded-xl flex items-center gap-2 shadow-lg shadow-yellow-200/50 transition-all active:scale-95 cursor-pointer"
                >
                    {icon_plus()} "Add Staff"
                </button>
            </div>

            // Table
            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full text-left">
                        <thead>
                            <tr class="border-b border-gray-50 text-[10px] uppercase tracking-widest text-gray-400 font-bold">
                                <th class="px-6 py-4">"Full Name / Email"</th>
                                <th class="px-6 py-4">"Role"</th>
                                <th class="px-6 py-4">"Status"</th>
                                <th class="px-6 py-4 text-center">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-50">
                             <Suspense fallback=move || view! { <tr><td colspan="4" class="px-6 py-4 text-center">"Loading..."</td></tr> }>
                                 {move || match staff_resource.get() {
                                     Some(sw) => {
                                         let res = &*sw;
                                         match res {
                                             Ok(items) => items.iter().map(|s| {
                                                 let s_email = s.email.clone();
                                                 let s_role = s.role.to_string();
                                                 let initial = s_email.chars().next().unwrap_or('?').to_string();
                                                 view! {
                                                     <tr class="hover:bg-gray-50/50 transition-colors group">
                                                         <td class="px-6 py-4">
                                                             <div class="flex items-center gap-3">
                                                                 <div class="w-8 h-8 rounded-full bg-indigo-500 flex items-center justify-center text-white text-[10px] font-bold">
                                                                     {initial}
                                                                 </div>
                                                                 <span class="text-sm font-bold text-gray-900">{s_email}</span>
                                                             </div>
                                                         </td>
                                                         <td class="px-6 py-4">
                                                             <span class="text-xs text-gray-400 font-medium font-mono uppercase tracking-widest">{s_role}</span>
                                                         </td>
                                                         <td class="px-6 py-4">
                                                             <span class="px-2.5 py-1 rounded-lg bg-emerald-50 text-emerald-600 text-[10px] font-black uppercase tracking-widest">
                                                                 "Active"
                                                             </span>
                                                         </td>
                                                         <td class="px-4 py-3.5 text-center">
                                                             <button class="text-gray-400 hover:text-rose-500 transition-colors">"Remove"</button>
                                                         </td>
                                                     </tr>
                                                 }
                                             }).collect_view().into_any(),
                                             _ => view! { <tr><td colspan="4" class="px-6 py-8 text-center text-gray-400">"No staff found."</td></tr> }.into_any()
                                         }
                                     },
                                     None => view! { <tr><td colspan="4" class="px-6 py-8 text-center text-gray-400">"Loading staff list..."</td></tr> }.into_any()
                                 }}
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
