/// Toolbar: column visibility toggle + filter menu.
use leptos::prelude::*;
use lodviz_core::core::table_data::{FilterOp, TableData};
use std::collections::HashMap;

use super::column_filter::UnifiedFilterMenu;

#[component]
pub fn TableToolbar(
    data: Signal<TableData>,
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    col_visibility: RwSignal<Vec<bool>>,
    selected_count: Signal<usize>,
    show_global_filters: RwSignal<bool>,
) -> impl IntoView {
    let show_col_menu = RwSignal::new(false);

    view! {
        <div class="flex items-center gap-3 px-4 py-2.5 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 flex-wrap">

            // ── Selection badge ──────────────────────────────────────────────
            <Show when=move || selected_count.get() != 0>
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300">
                    {move || format!("{} selected", selected_count.get())}
                </span>
            </Show>

            // ── Toolbar Actions ─────────────────────────────────────────────
            <div class="flex items-center gap-2 ml-auto">

                // ── Global Filter Toggle ───────────────────────────────────────
                <div class="relative">
                    <button
                        class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium border rounded-md transition-colors"
                        class:border-blue-500=move || show_global_filters.get()
                        class:bg-blue-50=move || show_global_filters.get()
                        class:dark:bg-blue-900_slash_30=move || show_global_filters.get()
                        class:text-blue-700=move || show_global_filters.get()
                        class:border-gray-300=move || !show_global_filters.get()
                        class:dark:border-gray-600=move || !show_global_filters.get()
                        class:bg-white=move || !show_global_filters.get()
                        class:dark:bg-gray-800=move || !show_global_filters.get()
                        class:text-gray-700=move || !show_global_filters.get()
                        class:dark:text-gray-300=move || !show_global_filters.get()
                        class:hover:bg-gray-50=move || !show_global_filters.get()
                        class:dark:hover:bg-gray-700=move || !show_global_filters.get()
                        on:click=move |_| {
                            show_col_menu.set(false);
                            show_global_filters.update(|v| *v = !*v)
                        }
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2a1 1 0 01-.293.707L13 13.414V19a1 1 0 01-.553.894l-4 2A1 1 0 017 21v-7.586L3.293 6.707A1 1 0 013 6V4z"
                            />
                        </svg>
                        "Filters"
                        <Show when=move || !filter_state.get().is_empty()>
                            <span class="inline-flex items-center justify-center w-4 h-4 rounded-full text-[10px] font-bold bg-blue-600 text-white ml-1">
                                {move || filter_state.get().len()}
                            </span>
                        </Show>
                    </button>

                    // ── Global Filter Popover ───────────────────────────────────────
                    <Show when=move || show_global_filters.get()>
                        <div
                            class="fixed inset-0 z-40"
                            on:click=move |_| show_global_filters.set(false)
                        />
                        <div class="absolute right-0 top-full mt-1 z-50">
                            <UnifiedFilterMenu
                                data=data
                                filter_state=filter_state
                                on_close=move || show_global_filters.set(false)
                            />
                        </div>
                    </Show>
                // End of Filters relative wrapper
                </div>

                // ── Column visibility menu ───────────────────────────────────────
                <div class="relative">
                    <button
                        class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                        on:click=move |_| show_col_menu.update(|v| *v = !*v)
                    >
                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7"
                            />
                        </svg>
                        "Columns"
                    </button>

                    <Show when=move || show_col_menu.get()>
                        // Backdrop
                        <div
                            class="fixed inset-0 z-40"
                            on:click=move |_| show_col_menu.set(false)
                        />
                        // Dropdown
                        <div class="absolute right-0 top-full mt-1 z-50 min-w-40 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden">
                            <div class="px-3 py-2 border-b border-gray-100 dark:border-gray-700 flex items-center justify-between gap-4">
                                <span class="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                                    "Visible Columns"
                                </span>
                                <button
                                    class="text-xs text-blue-600 hover:text-blue-800"
                                    on:click=move |_| {
                                        col_visibility
                                            .update(|v| v.iter_mut().for_each(|x| *x = true))
                                    }
                                >
                                    "All"
                                </button>
                            </div>
                            <div class="py-1 max-h-64 overflow-y-auto w-max">
                                {move || {
                                    data.get()
                                        .columns
                                        .iter()
                                        .enumerate()
                                        .map(|(ci, col)| {
                                            let label = col.label.clone();
                                            let is_visible = Signal::derive(move || {
                                                col_visibility.get().get(ci).copied().unwrap_or(true)
                                            });
                                            view! {
                                                <label class="flex items-center gap-2.5 px-4 py-1.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer whitespace-nowrap">
                                                    <input
                                                        type="checkbox"
                                                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                        prop:checked=is_visible
                                                        on:change=move |_| {
                                                            col_visibility
                                                                .update(|v| {
                                                                    if let Some(vis) = v.get_mut(ci) {
                                                                        *vis = !*vis;
                                                                    }
                                                                });
                                                        }
                                                    />
                                                    {label}
                                                </label>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>
                        </div>
                    </Show>
                // End of Columns relative wrapper
                </div>

            // End of flex container
            </div>
        </div>
    }
}
