use leptos::prelude::*;
use crate::api::client::memberships::*;

fn icon_crown() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 20h20M4 20V10l4 4 4-8 4 8 4-4v10"></path>
        </svg>
    }
}


#[component]
pub fn AdminMembershipsPage() -> impl IntoView {
    let summary_resource = LocalResource::new(|| async move { get_membership_summary().await });
    let members_resource = LocalResource::new(|| async move { list_memberships().await });

    let tiers = [
        (
            "Bronze",
            "320",
            "₫0 – ₫2M",
            "#CD7F32",
            "bg-orange-50  border-orange-100",
        ),
        (
            "Silver",
            "184",
            "₫2M – ₫5M",
            "#9CA3AF",
            "bg-gray-50    border-gray-200",
        ),
        (
            "Gold",
            "97",
            "₫5M – ₫15M",
            "#F59E0B",
            "bg-amber-50   border-amber-100",
        ),
        (
            "Platinum",
            "28",
            "₫15M+",
            "#6366F1",
            "bg-indigo-50  border-indigo-100",
        ),
    ];

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Memberships"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Loyalty tiers, member benefits and tracking"</p>
                </div>
                <div class="flex items-center gap-1.5 px-3 py-1.5 bg-[#FCE300]/20 border border-[#FCE300] rounded-xl">
                    <span class="text-amber-600">{icon_crown()}</span>
                    <span class="text-xs font-bold text-amber-700">
                        {move || match summary_resource.get() {
                            Some(sw) => {
                                let res = &*sw;
                                match res {
                                    Ok(s) => format!("{} Total Members", s.total_members),
                                    _ => "... Total Members".to_string()
                                }
                            },
                            _ => "... Total Members".to_string()
                        }}
                    </span>
                </div>
            </div>

            // Tier cards
            <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                {tiers.iter().map(|&(tier_name, _count, range, color, bg)| {
                    let tier_name_str = tier_name.to_string();
                    view! {
                        <div class=format!("rounded-2xl p-5 border shadow-sm cursor-pointer hover:shadow-md transition-shadow {}", bg)>
                            <div class="flex items-center gap-2 mb-3">
                                <div class="w-8 h-8 rounded-xl flex items-center justify-center" style={format!("background:{}20", color)}>
                                    <span class="text-lg font-black" style={format!("color:{}", color)}>
                                        {match tier_name {
                                            "Bronze"   => "B",
                                            "Silver"   => "S",
                                            "Gold"     => "G",
                                            _          => "P",
                                        }}
                                    </span>
                                </div>
                                <span class="text-sm font-bold text-gray-900">{tier_name}</span>
                            </div>
                            {move || {
                                let tier_count = summary_resource.get()
                                    .and_then(|sw| {
                                        let res = &*sw;
                                        match res {
                                            Ok(s) => s.tier_counts.get(&tier_name_str).cloned(),
                                            _ => None
                                        }
                                    })
                                    .unwrap_or(0);
                                view! { <p class="text-3xl font-black mt-1" style={format!("color:{}", color)}>{tier_count}</p> }
                            }}
                            <p class="text-[11px] text-gray-400 mt-1">"members"</p>
                            <div class="mt-2 pt-2 border-t border-gray-200">
                                <p class="text-[10px] text-gray-400">"Spend range"</p>
                                <p class="text-xs font-semibold text-gray-700">{range}</p>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            // Benefit comparison table
            <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                <h2 class="text-sm font-bold text-gray-900 mb-4">"Tier Benefits"</h2>
                <div class="overflow-x-auto">
                    <table class="w-full text-xs text-center">
                        <thead>
                            <tr class="text-gray-400 uppercase text-[10px] tracking-wider">
                                <th class="py-2 text-left px-3 font-medium">"Benefit"</th>
                                {["Bronze", "Silver", "Gold", "Platinum"].iter().map(|&t| {
                                    let color = match t {
                                        "Bronze"   => "#CD7F32",
                                        "Silver"   => "#9CA3AF",
                                        "Gold"     => "#F59E0B",
                                        _          => "#6366F1",
                                    };
                                    view! {
                                        <th class="py-2 px-3 font-bold" style={format!("color:{}", color)}>{t}</th>
                                    }
                                }).collect_view()}
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-50">
                            {[
                                ("Free Shipping",    "–", "–", "✓", "✓"),
                                ("Early Access",     "–", "✓", "✓", "✓"),
                                ("Birthday Gift",    "–", "–", "–", "✓"),
                                ("Priority Support", "–", "–", "✓", "✓"),
                                ("Cashback",        "1%", "2%","4%","6%"),
                            ].iter().map(|&(label, b, s, g, p)| {
                                let col_b = if b == "–" { "text-gray-300" } else { "text-emerald-500 font-bold" };
                                let col_s = if s == "–" { "text-gray-300" } else { "text-amber-500 font-bold" };
                                let col_g = if g == "–" { "text-gray-300" } else { "text-amber-500 font-bold" };
                                let col_p = if p == "–" { "text-gray-300" } else { "text-indigo-500 font-bold" };
                                view! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="py-2.5 px-3 text-left text-gray-700 font-medium">{label}</td>
                                        <td class={format!("py-2.5 px-3 {}", col_b)}>{b}</td>
                                        <td class={format!("py-2.5 px-3 {}", col_s)}>{s}</td>
                                        <td class={format!("py-2.5 px-3 {}", col_g)}>{g}</td>
                                        <td class={format!("py-2.5 px-3 {}", col_p)}>{p}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </div>

            // Recent members table
            <div class="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                <div class="px-5 py-4 border-b border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900">"Recent Members"</h2>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-xs">
                        <thead>
                            <tr class="border-b border-gray-100 text-gray-400 uppercase tracking-wider text-[10px]">
                                <th class="px-5 py-3 text-left font-medium">"Member"</th>
                                <th class="px-4 py-3 text-left font-medium hidden sm:table-cell">"Email"</th>
                                <th class="px-4 py-3 text-center font-medium">"Tier"</th>
                                <th class="px-4 py-3 text-center font-medium hidden md:table-cell">"Joined"</th>
                                <th class="px-4 py-3 text-right font-medium">"Total Spent"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-50">
                            <Suspense fallback=move || view! { <tr><td colspan="5" class="text-center py-4">"Loading members..."</td></tr> }>
                                {move || match members_resource.get() {
                                    Some(sw) => {
                                        let res = &*sw;
                                        match res {
                                            Ok(items) => items.iter().map(|m| {
                                                let email = m.email.clone();
                                                let (tier_color, tier_bg) = match m.tier.as_str() {
                                                    "Platinum" => ("#6366F1", "bg-indigo-50 border-indigo-100 text-indigo-600"),
                                                    "Gold"     => ("#F59E0B", "bg-amber-50 border-amber-100 text-amber-600"),
                                                    "Silver"   => ("#9CA3AF", "bg-gray-100 border-gray-200 text-gray-600"),
                                                    _          => ("#CD7F32", "bg-orange-50 border-orange-100 text-orange-600"),
                                                };
                                                let initial = email.chars().next().unwrap_or('?').to_string();
                                                let tier = m.tier.clone();
                                                let joined = m.joined_at.format("%d/%m/%Y").to_string();
                                                let spent = format!("₫{}k", m.total_spent / rust_decimal::Decimal::from(1000u32));

                                                view! {
                                                    <tr class="hover:bg-gray-50 transition-colors">
                                                        <td class="px-5 py-3.5">
                                                            <div class="flex items-center gap-3">
                                                                <div class="w-8 h-8 rounded-full flex items-center justify-center text-white text-xs font-bold flex-shrink-0"
                                                                    style={format!("background:{}", tier_color)}>
                                                                    {initial}
                                                                </div>
                                                                <span class="font-semibold text-gray-900">{email}</span>
                                                            </div>
                                                        </td>
                                                        <td class="px-4 py-3.5 text-gray-400 hidden sm:table-cell text-xs">{m.email.clone()}</td>
                                                        <td class="px-4 py-3.5 text-center">
                                                            <span class=format!("text-[10px] font-bold px-2.5 py-1 rounded-full border {}", tier_bg)>
                                                                {tier}
                                                            </span>
                                                        </td>
                                                        <td class="px-4 py-3.5 text-center text-gray-400 hidden md:table-cell">{joined}</td>
                                                        <td class="px-4 py-3.5 text-right text-gray-900 font-bold">{spent}</td>
                                                    </tr>
                                                }
                                            }).collect_view().into_any(),
                                            _ => view! { <tr><td colspan="5" class="text-center py-4">"No members found"</td></tr> }.into_any()
                                        }
                                    },
                                    None => view! { <tr><td colspan="5" class="text-center py-4">"No data"</td></tr> }.into_any()
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
