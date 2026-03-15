use leptos::prelude::*;

fn icon_download() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
    }
}

struct Invoice {
    num: &'static str,
    customer: &'static str,
    date: &'static str,
    due: &'static str,
    amount: &'static str,
    method: &'static str,
    status: &'static str,
}

#[component]
pub fn AdminPaymentInvoicesPage() -> impl IntoView {
    let invoices = vec![
        Invoice {
            num: "INV-2026-041",
            customer: "Nguyễn Văn An",
            date: "12/03/2026",
            due: "19/03/2026",
            amount: "₫378k",
            method: "VNPAY",
            status: "Paid",
        },
        Invoice {
            num: "INV-2026-040",
            customer: "Trần Thị Bình",
            date: "12/03/2026",
            due: "19/03/2026",
            amount: "₫249k",
            method: "MoMo",
            status: "Paid",
        },
        Invoice {
            num: "INV-2026-039",
            customer: "Lê Hoàng Cường",
            date: "11/03/2026",
            due: "18/03/2026",
            amount: "₫589k",
            method: "COD",
            status: "Pending",
        },
        Invoice {
            num: "INV-2026-038",
            customer: "Phạm Thu Dung",
            date: "11/03/2026",
            due: "18/03/2026",
            amount: "₫657k",
            method: "ZaloPay",
            status: "Paid",
        },
        Invoice {
            num: "INV-2026-037",
            customer: "Hoàng Minh Đức",
            date: "10/03/2026",
            due: "17/03/2026",
            amount: "₫438k",
            method: "VNPAY",
            status: "Overdue",
        },
        Invoice {
            num: "INV-2026-036",
            customer: "Vũ Thị Hoa",
            date: "10/03/2026",
            due: "17/03/2026",
            amount: "₫219k",
            method: "MoMo",
            status: "Paid",
        },
    ];

    // Donut chart slices for payment methods
    let methods = [
        ("VNPAY", 42, "#6366F1"),
        ("MoMo", 31, "#F59E0B"),
        ("ZaloPay", 15, "#10B981"),
        ("COD", 12, "#F43F5E"),
    ];

    view! {
        <div class="p-6 space-y-6">
            // Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-xl font-black text-gray-900">"Payment & Invoices"</h1>
                    <p class="text-xs text-gray-400 mt-0.5">"Track revenue, invoices and payment methods"</p>
                </div>
                <button class="flex items-center gap-1.5 border border-gray-200 text-gray-700 text-xs font-bold px-4 py-2.5 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer shadow-sm">
                    {icon_download()} "Export"
                </button>
            </div>

            // KPI Cards
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                {[
                    ("Total Revenue", "₫128.4M", "#6366F1", "bg-indigo-50 border-indigo-100"),
                    ("Outstanding",   "₫12.8M",  "#F59E0B", "bg-amber-50 border-amber-100"),
                    ("Paid This Month","₫42.6M", "#10B981", "bg-emerald-50 border-emerald-100"),
                ].iter().map(|&(label, val, color, bg)| view! {
                    <div class=format!("rounded-2xl p-5 border shadow-sm {}", bg)>
                        <p class="text-xs text-gray-500 font-medium">{label}</p>
                        <p class="text-2xl font-black mt-1.5" style={format!("color:{}", color)}>{val}</p>
                    </div>
                }).collect_view()}
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
                // Invoice table (2/3)
                <div class="lg:col-span-2 bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                    <div class="flex items-center justify-between px-5 py-4 border-b border-gray-100">
                        <h2 class="text-sm font-bold text-gray-900">"Recent Invoices"</h2>
                        <button class="text-xs text-indigo-500 font-semibold hover:text-indigo-700 cursor-pointer transition-colors">"View All"</button>
                    </div>
                    <div class="overflow-x-auto">
                        <table class="w-full text-xs">
                            <thead>
                                <tr class="border-b border-gray-100 text-gray-400 uppercase tracking-wider text-[10px]">
                                    <th class="px-5 py-3 text-left font-medium">"Invoice"</th>
                                    <th class="px-4 py-3 text-left font-medium hidden sm:table-cell">"Customer"</th>
                                    <th class="px-4 py-3 text-center font-medium hidden md:table-cell">"Method"</th>
                                    <th class="px-4 py-3 text-right font-medium">"Amount"</th>
                                    <th class="px-4 py-3 text-center font-medium">"Status"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-50">
                                {invoices.into_iter().map(|inv| {
                                    let badge_bg = match inv.status {
                                        "Paid"    => "bg-emerald-50 text-emerald-600 border-emerald-100",
                                        "Pending" => "bg-amber-50 text-amber-600 border-amber-100",
                                        _         => "bg-rose-50 text-rose-600 border-rose-100",
                                    };
                                    let method_color = match inv.method {
                                        "VNPAY"   => "#6366F1",
                                        "MoMo"    => "#F59E0B",
                                        "ZaloPay" => "#10B981",
                                        _         => "#94A3B8",
                                    };
                                    view! {
                                        <tr class="hover:bg-gray-50 transition-colors duration-100">
                                            <td class="px-5 py-3.5 font-mono text-indigo-600 font-semibold whitespace-nowrap">{inv.num}</td>
                                            <td class="px-4 py-3.5 text-gray-800 hidden sm:table-cell whitespace-nowrap">{inv.customer}</td>
                                            <td class="px-4 py-3.5 text-center hidden md:table-cell">
                                                <span class="font-semibold text-xs px-2 py-0.5 rounded-md" style={format!("color:{};background:{}20", method_color, method_color)}>
                                                    {inv.method}
                                                </span>
                                            </td>
                                            <td class="px-4 py-3.5 text-right text-gray-900 font-bold whitespace-nowrap">{inv.amount}</td>
                                            <td class="px-4 py-3.5 text-center">
                                                <span class=format!("text-[10px] font-semibold px-2.5 py-1 rounded-full border {}", badge_bg)>
                                                    {inv.status}
                                                </span>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                </div>

                // Payment method breakdown (1/3)
                <div class="bg-white rounded-2xl p-5 shadow-sm border border-gray-100">
                    <h2 class="text-sm font-bold text-gray-900 mb-4">"Payment Methods"</h2>

                    // Visual bar stack
                    <div class="h-4 rounded-full overflow-hidden flex mb-4">
                        {methods.iter().map(|&(_, pct, color)| view! {
                            <div class="h-full transition-all duration-500" style={format!("width:{}%; background:{}", pct, color)}></div>
                        }).collect_view()}
                    </div>

                    <div class="space-y-3">
                        {methods.iter().map(|&(name, pct, color)| view! {
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-2">
                                    <span class="w-3 h-3 rounded-full flex-shrink-0" style={format!("background:{}", color)}></span>
                                    <span class="text-xs text-gray-700 font-medium">{name}</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <div class="h-1.5 w-20 bg-gray-100 rounded-full overflow-hidden">
                                        <div class="h-full rounded-full" style={format!("width:{}%; background:{}", pct, color)}></div>
                                    </div>
                                    <span class="text-xs font-bold text-gray-900 w-8 text-right">{pct}"%"</span>
                                </div>
                            </div>
                        }).collect_view()}
                    </div>

                    // Total transactions
                    <div class="mt-4 pt-4 border-t border-gray-100 text-center">
                        <p class="text-xs text-gray-400">"Total transactions"</p>
                        <p class="text-2xl font-black text-gray-900 mt-0.5">"3,247"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
