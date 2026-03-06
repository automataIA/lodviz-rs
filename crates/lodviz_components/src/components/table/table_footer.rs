/// Pagination footer: row count, page size selector, page navigation.
use leptos::prelude::*;

#[component]
pub fn TableFooter(
    total_rows: Signal<usize>,
    page: RwSignal<usize>,
    page_count: Memo<usize>,
    page_size: Signal<usize>,
) -> impl IntoView {
    // Pre-compute disabled states to avoid `>` ambiguity inside view! macro
    let at_first = Signal::derive(move || page.get() == 0);
    let at_last = Signal::derive(move || {
        let pc = page_count.get();
        pc == 0 || page.get() + 1 == pc
    });

    let range_info = Signal::derive(move || {
        let total = total_rows.get();
        if total == 0 {
            return "No results".to_owned();
        }
        let size = page_size.get();
        let p = page.get();
        let start = p * size + 1;
        let end = ((p + 1) * size).min(total);
        format!("{start}-{end} of {total}")
    });

    view! {
        <div class="flex items-center justify-between px-4 py-2.5 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 flex-wrap gap-2">

            // ── Row range info ────────────────────────────────────────────
            <span class="text-xs text-gray-500 dark:text-gray-400">{range_info}</span>

            <div class="flex items-center gap-4 ml-auto">
                // ── Page size selector (removed, dynamically adapted) ──────────────────
                // ── Page navigation ───────────────────────────────────────
                <div class="flex items-center gap-1">
                    <button
                        class="px-1.5 py-1 rounded text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed"
                        disabled=at_first
                        on:click=move |_| page.set(0)
                        title="First page"
                    >
                        "«"
                    </button>
                    <button
                        class="px-1.5 py-1 rounded text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed"
                        disabled=at_first
                        on:click=move |_| page.update(|p| *p = p.saturating_sub(1))
                        title="Previous page"
                    >
                        "‹"
                    </button>

                    <span class="px-2 py-1 text-xs text-gray-700 dark:text-gray-300 tabular-nums min-w-16 text-center">
                        {move || {
                            let pc = page_count.get();
                            if pc == 0 {
                                "–".to_owned()
                            } else {
                                format!("{} / {pc}", page.get() + 1)
                            }
                        }}
                    </span>

                    <button
                        class="px-1.5 py-1 rounded text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed"
                        disabled=at_last
                        on:click=move |_| {
                            let pc = page_count.get_untracked();
                            page.update(|p| {
                                if *p + 1 < pc {
                                    *p += 1;
                                }
                            });
                        }
                        title="Next page"
                    >
                        "›"
                    </button>
                    <button
                        class="px-1.5 py-1 rounded text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed"
                        disabled=at_last
                        on:click=move |_| page.set(page_count.get_untracked().saturating_sub(1))
                        title="Last page"
                    >
                        "»"
                    </button>
                </div>
            </div>
        </div>
    }
}
