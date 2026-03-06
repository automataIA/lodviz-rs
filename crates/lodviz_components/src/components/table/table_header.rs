use std::collections::HashMap;

use leptos::prelude::*;
use lodviz_core::core::table_data::{FilterOp, SortDir, SortKey, TableData};

use super::column_filter_popover::ColumnFilterPopover;

#[component]
pub fn TableHeader(
    data: Signal<TableData>,
    sort_state: RwSignal<Vec<SortKey>>,
    col_visibility: RwSignal<Vec<bool>>,
    /// Global filter state (multi-column filters)
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    /// Whether all rows are selected (drives select-all checkbox state).
    all_selected: Signal<bool>,
    /// Whether some but not all rows are selected (indeterminate state).
    some_selected: Signal<bool>,
    on_select_all: impl Fn(bool) + 'static,
) -> impl IntoView {
    // Which column's filter popover is currently open (if any)
    let active_filter_popover = RwSignal::new(None::<usize>);
    view! {
        <thead class="bg-gray-50 dark:bg-gray-800 sticky top-0 z-10">
            <tr>
                // ── Select-all checkbox ──────────────────────────────────
                <th class="w-10 px-3 py-3 border-b border-gray-200 dark:border-gray-700">
                    <input
                        type="checkbox"
                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                        prop:checked=all_selected
                        prop:indeterminate=some_selected
                        on:change=move |ev| {
                            let checked = event_target_checked(&ev);
                            on_select_all(checked);
                        }
                    />
                </th>

                // ── Column headers ───────────────────────────────────────
                {move || {
                    data.get()
                        .columns
                        .iter()
                        .enumerate()
                        .filter_map(|(ci, col)| {
                            let is_visible = col_visibility.get().get(ci).copied().unwrap_or(true);
                            if !is_visible {
                                return None;
                            }
                            let label = col.label.clone();
                            let label_filter = label.clone();
                            let sortable = col.sortable;
                            let filterable = col.filterable;
                            let col_width = col.width;
                            let sort_badge = Signal::derive(move || {
                                sort_state
                                    .get()
                                    .iter()
                                    .enumerate()
                                    .find_map(|(order, k)| {
                                        if k.col_index == ci {
                                            Some((order + 1, k.direction))
                                        } else {
                                            None
                                        }
                                    })
                            });
                            let style = col_width
                                .map(|w| format!("width: {w}px; min-width: {w}px;"))
                                .unwrap_or_default();
                            Some(
                                // Clone for filter aria-label

                                // Sort state for this column

                                // Filter active indicator

                                // Filter popover open for this column

                                view! {
                                    <th
                                        class="relative px-3 py-3 text-left text-xs font-semibold text-gray-600 dark:text-gray-300 uppercase tracking-wider border-b border-gray-200 dark:border-gray-700 whitespace-nowrap"
                                        style=style
                                    >
                                        <div class="flex items-center gap-1.5">
                                            // Column label + sort button
                                            <button
                                                class="flex items-center gap-1 group"
                                                class:cursor-default=move || !sortable
                                                disabled=move || !sortable
                                                on:click=move |ev| {
                                                    if !sortable {
                                                        return;
                                                    }
                                                    let shift = ev.shift_key();
                                                    sort_state
                                                        .update(|keys| {
                                                            if let Some(pos) = keys
                                                                .iter()
                                                                .position(|k| k.col_index == ci)
                                                            {
                                                                let dir = keys[pos].direction.toggle();
                                                                if keys[pos].direction == SortDir::Desc {
                                                                    keys.remove(pos);
                                                                } else {
                                                                    keys[pos].direction = dir;
                                                                }
                                                            } else if shift {
                                                                keys.push(SortKey {
                                                                    col_index: ci,
                                                                    direction: SortDir::Asc,
                                                                });
                                                            } else {
                                                                keys.clear();
                                                                keys.push(SortKey {
                                                                    col_index: ci,
                                                                    direction: SortDir::Asc,
                                                                });
                                                            }
                                                        });
                                                }
                                            >
                                                <span>{label}</span>

                                                // Sort icon (always render, vary opacity)
                                                {move || {
                                                    match sort_badge.get() {
                                                        Some((_, SortDir::Asc)) => {
                                                            view! {
                                                                <svg
                                                                    class="w-3 h-3 text-blue-600"
                                                                    fill="none"
                                                                    viewBox="0 0 24 24"
                                                                    stroke="currentColor"
                                                                >
                                                                    <path
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        stroke-width="2.5"
                                                                        d="M5 15l7-7 7 7"
                                                                    />
                                                                </svg>
                                                            }
                                                                .into_any()
                                                        }
                                                        Some((_, SortDir::Desc)) => {
                                                            view! {
                                                                <svg
                                                                    class="w-3 h-3 text-blue-600"
                                                                    fill="none"
                                                                    viewBox="0 0 24 24"
                                                                    stroke="currentColor"
                                                                >
                                                                    <path
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        stroke-width="2.5"
                                                                        d="M19 9l-7 7-7-7"
                                                                    />
                                                                </svg>
                                                            }
                                                                .into_any()
                                                        }
                                                        None if sortable => {
                                                            view! {
                                                                <svg
                                                                    class="w-3 h-3 text-gray-300 dark:text-gray-600 group-hover:text-gray-400"
                                                                    fill="none"
                                                                    viewBox="0 0 24 24"
                                                                    stroke="currentColor"
                                                                >
                                                                    <path
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        stroke-width="2"
                                                                        d="M7 16V4m0 0L3 8m4-4l4 4M17 8v12m0 0l4-4m-4 4l-4-4"
                                                                    />
                                                                </svg>
                                                            }
                                                                .into_any()
                                                        }
                                                        _ => view! { <span /> }.into_any(),
                                                    }
                                                }}

                                                // Multi-sort order badge
                                                {move || {
                                                    sort_badge
                                                        .get()
                                                        .and_then(|(order, _)| {
                                                            if order <= 1 {
                                                                return None;
                                                            }
                                                            Some(
                                                                // only show for secondary+
                                                                view! {
                                                                    <span class="inline-flex items-center justify-center w-4 h-4 rounded-full text-[10px] font-bold bg-blue-600 text-white">
                                                                        {order}
                                                                    </span>
                                                                },
                                                            )
                                                        })
                                                }}
                                            </button>

                                            // ── Filter icon button ──────────────────────
                                            {move || {
                                                if !filterable {
                                                    return view! { <span /> }.into_any();
                                                }
                                                let has_filter = Signal::derive(move || {
                                                    filter_state.get().contains_key(&ci)
                                                });
                                                let is_popover_open = Signal::derive(move || {
                                                    active_filter_popover.get() == Some(ci)
                                                });
                                                view! {
                                                    <div class="relative">
                                                        <button
                                                            class="p-0.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                                                            class:text-blue-600=has_filter
                                                            class:text-gray-400=move || !has_filter.get()
                                                            aria-label=format!("Filtra colonna {}", label_filter)
                                                            aria-pressed=is_popover_open
                                                            on:click=move |ev| {
                                                                ev.stop_propagation();
                                                                active_filter_popover
                                                                    .update(|cur| {
                                                                        if *cur == Some(ci) {
                                                                            *cur = None;
                                                                        } else {
                                                                            *cur = Some(ci);
                                                                        }
                                                                    });
                                                            }
                                                        >
                                                            // Filter funnel icon
                                                            <svg
                                                                class="w-3.5 h-3.5"
                                                                fill="none"
                                                                viewBox="0 0 24 24"
                                                                stroke="currentColor"
                                                            >
                                                                <path
                                                                    stroke-linecap="round"
                                                                    stroke-linejoin="round"
                                                                    stroke-width="2"
                                                                    d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                                                                />
                                                            </svg>

                                                            // Badge dot when filter active
                                                            {move || {
                                                                if has_filter.get() {
                                                                    view! {
                                                                        <span class="absolute -top-0.5 -right-0.5 w-1.5 h-1.5 bg-blue-600 rounded-full" />
                                                                    }
                                                                        .into_any()
                                                                } else {
                                                                    view! { <span /> }.into_any()
                                                                }
                                                            }}
                                                        </button>
                                                    </div>
                                                }
                                                    .into_any()
                                            }}
                                        </div>

                                        // ── Filter Popover (rendered below header) ─────
                                        {move || {
                                            if active_filter_popover.get() == Some(ci) {
                                                let total_cols = data.get().columns.len();
                                                let is_right_side = ci >= total_cols / 2;
                                                let position_class = if is_right_side {
                                                    "absolute right-0 mt-1 z-50"
                                                } else {
                                                    "absolute left-0 mt-1 z-50"
                                                };
                                                // Smart positioning: right-align if in second half of columns
                                                view! {
                                                    <div>
                                                        // Backdrop (click to close)
                                                        <div
                                                            class="fixed inset-0 z-40"
                                                            on:click=move |_| active_filter_popover.set(None)
                                                        />

                                                        // Popover with smart positioning
                                                        <div class=position_class>
                                                            <ColumnFilterPopover
                                                                col_index=ci
                                                                filter_state=filter_state
                                                                data=data
                                                                on_close=move || active_filter_popover.set(None)
                                                            />
                                                        </div>
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                view! { <span /> }.into_any()
                                            }
                                        }}
                                    </th>
                                },
                            )
                        })
                        .collect_view()
                }}
            </tr>
        </thead>
    }
}
