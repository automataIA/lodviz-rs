/// Unified column filter menu.
use std::collections::{HashMap, HashSet};

use leptos::prelude::*;
use lodviz_core::core::table_data::{ColumnType, CompareOp, FilterOp, TableData};

/// Filter menu for selecting a column and applying a filter.
#[component]
pub fn UnifiedFilterMenu(
    data: Signal<TableData>,
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    /// Called when the user clicks "Chiudi".
    on_close: impl Fn() + Send + Sync + Clone + 'static,
) -> impl IntoView {
    // 1. Select which column to filter
    // Default to the first filterable column if any.
    let selected_col_idx = RwSignal::new(
        data.get_untracked()
            .columns
            .iter()
            .position(|c| c.filterable)
            .unwrap_or(0),
    );

    let col_type = Signal::derive(move || {
        data.get()
            .columns
            .get(selected_col_idx.get())
            .map(|c| c.col_type.clone())
            .unwrap_or(ColumnType::Text)
    });

    // ── Temporary form states ───────────────────────────────────────────────
    // We only create the FilterOp when the user clicks "Applica" or interacts
    // heavily (for instant applying, like the category list).

    let text_input = RwSignal::new(String::new());
    let min_input = RwSignal::new(String::new());
    let max_input = RwSignal::new(String::new());

    // When the selected column changes, reset the inputs
    Effect::new(move |_| {
        let _ = selected_col_idx.get();
        text_input.set(String::new());
        min_input.set(String::new());
        max_input.set(String::new());
    });

    let on_close_clear = on_close.clone();

    view! {
        <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl p-3 min-w-64 max-w-sm">
            <div class="space-y-4">
                // ── Column Selector ─────────────────────────────────────────────
                <div class="space-y-1.5">
                    <label class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider block">
                        "Column to filter"
                    </label>
                    <select
                        class="w-full px-2.5 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        on:change=move |ev| {
                            if let Ok(idx) = event_target_value(&ev).parse::<usize>() {
                                selected_col_idx.set(idx);
                            }
                        }
                    >
                        {move || {
                            data.get()
                                .columns
                                .iter()
                                .enumerate()
                                .filter(|(_, c)| c.filterable)
                                .map(|(i, c)| {
                                    view! {
                                        <option
                                            value=i
                                            selected=move || selected_col_idx.get() == i
                                        >
                                            {c.label.clone()}
                                        </option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                </div>

                // ── Filter Inputs based on type ─────────────────────────────────
                {move || {
                    let c_idx = selected_col_idx.get();
                    match col_type.get() {
                        ColumnType::Text => {
                            view! {
                                <div class="space-y-2">
                                    <p class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                        "Search text"
                                    </p>
                                    <div class="flex gap-2">
                                        <input
                                            type="text"
                                            placeholder="Text to search..."
                                            class="flex-1 px-2.5 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            prop:value=text_input
                                            on:input=move |ev| text_input.set(event_target_value(&ev))
                                            on:keydown={
                                                let oc = on_close.clone();
                                                move |ev| {
                                                    if ev.key() == "Enter" {
                                                        let val = text_input.get();
                                                        if !val.is_empty() {
                                                            filter_state
                                                                .update(|m| {
                                                                    m.insert(c_idx, FilterOp::TextContains(val));
                                                                });
                                                            oc();
                                                        }
                                                    }
                                                }
                                            }
                                        />
                                        <button
                                            class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                                            disabled=move || text_input.get().is_empty()
                                            on:click={
                                                let oc = on_close.clone();
                                                move |_| {
                                                    filter_state
                                                        .update(|m| {
                                                            m.insert(c_idx, FilterOp::TextContains(text_input.get()));
                                                        });
                                                    oc();
                                                }
                                            }
                                        >
                                            "Apply"
                                        </button>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                        ColumnType::Number => {
                            view! {
                                <div class="space-y-2">
                                    <p class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                        "Numeric range"
                                    </p>
                                    <div class="flex items-center gap-2">
                                        <input
                                            type="number"
                                            placeholder="Min"
                                            class="w-full px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            prop:value=min_input
                                            on:input=move |ev| min_input.set(event_target_value(&ev))
                                        />
                                        <span class="shrink-0 text-gray-400 text-sm">"–"</span>
                                        <input
                                            type="number"
                                            placeholder="Max"
                                            class="w-full px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            prop:value=max_input
                                            on:input=move |ev| max_input.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <button
                                        class="w-full mt-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                                        disabled=move || {
                                            min_input.get().is_empty() && max_input.get().is_empty()
                                        }
                                        on:click={
                                            let oc = on_close.clone();
                                            move |_| {
                                                let min = min_input.get().parse::<f64>().ok();
                                                let max = max_input.get().parse::<f64>().ok();
                                                if let (Some(min_val), Some(max_val)) = (min, max) {
                                                    if (min_val - max_val).abs() < f64::EPSILON {
                                                        filter_state
                                                            .update(|m| {
                                                                m.insert(
                                                                    c_idx,
                                                                    FilterOp::NumberCompare {
                                                                        operator: CompareOp::Equal,
                                                                        value: min_val,
                                                                    },
                                                                );
                                                            });
                                                    } else {
                                                        filter_state
                                                            .update(|m| {
                                                                m.insert(
                                                                    c_idx,
                                                                    FilterOp::NumberCompare {
                                                                        operator: CompareOp::GreaterEq,
                                                                        value: min_val,
                                                                    },
                                                                );
                                                            });
                                                    }
                                                    oc();
                                                } else if let Some(min_val) = min {
                                                    filter_state
                                                        .update(|m| {
                                                            m.insert(
                                                                c_idx,
                                                                FilterOp::NumberCompare {
                                                                    operator: CompareOp::GreaterEq,
                                                                    value: min_val,
                                                                },
                                                            );
                                                        });
                                                    oc();
                                                } else if let Some(max_val) = max {
                                                    filter_state
                                                        .update(|m| {
                                                            m.insert(
                                                                c_idx,
                                                                FilterOp::NumberCompare {
                                                                    operator: CompareOp::LessEq,
                                                                    value: max_val,
                                                                },
                                                            );
                                                        });
                                                    oc();
                                                }
                                            }
                                        }
                                    >
                                        "Apply range"
                                    </button>
                                </div>
                            }
                                .into_any()
                        }
                        ColumnType::Boolean => {
                            view! {
                                <div class="space-y-1.5">
                                    <p class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                        "Boolean value"
                                    </p>
                                    <div class="flex gap-2">
                                        <button
                                            class="flex-1 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-sm font-medium rounded-md border border-gray-300 dark:border-gray-600"
                                            on:click={
                                                let oc = on_close.clone();
                                                move |_| {
                                                    filter_state
                                                        .update(|m| {
                                                            m.insert(
                                                                c_idx,
                                                                FilterOp::CategoryIn(vec!["Yes".to_owned()]),
                                                            );
                                                        });
                                                    oc();
                                                }
                                            }
                                        >
                                            "Only 'Yes'"
                                        </button>
                                        <button
                                            class="flex-1 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-sm font-medium rounded-md border border-gray-300 dark:border-gray-600"
                                            on:click={
                                                let oc = on_close.clone();
                                                move |_| {
                                                    filter_state
                                                        .update(|m| {
                                                            m.insert(
                                                                c_idx,
                                                                FilterOp::CategoryIn(vec!["No".to_owned()]),
                                                            );
                                                        });
                                                    oc();
                                                }
                                            }
                                        >
                                            "Only 'No'"
                                        </button>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                        ColumnType::Category(options) => {
                            let opts: Vec<String> = if options.is_empty() {
                                data.get().distinct_values(c_idx)
                            } else {
                                options
                            };
                            let checked_cats: RwSignal<HashSet<String>> = RwSignal::new({
                                match filter_state.get_untracked().get(&c_idx) {
                                    Some(FilterOp::CategoryIn(items)) => {
                                        items.iter().cloned().collect()
                                    }
                                    _ => HashSet::new(),
                                }
                            });

                            // For category, we can read the existing state to pre-check boxes

                            view! {
                                <div class="space-y-1.5">
                                    <div class="flex items-center justify-between">
                                        <p class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                            "Selectable values"
                                        </p>
                                        <button
                                            class="text-xs text-blue-600 hover:text-blue-800"
                                            on:click=move |_| checked_cats.set(HashSet::new())
                                        >
                                            "Deselect all"
                                        </button>
                                    </div>
                                    <div class="max-h-48 overflow-y-auto space-y-0.5 border border-gray-200 dark:border-gray-700 rounded p-1">
                                        {opts
                                            .into_iter()
                                            .map(|opt| {
                                                let opt_check = opt.clone();
                                                let opt_label = opt.clone();
                                                let is_checked = Signal::derive(move || {
                                                    checked_cats.get().contains(&opt_check)
                                                });
                                                view! {
                                                    <label class="flex items-center gap-2 px-1 py-0.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 rounded cursor-pointer">
                                                        <input
                                                            type="checkbox"
                                                            class="rounded text-blue-600"
                                                            prop:checked=is_checked
                                                            on:change=move |_| {
                                                                checked_cats
                                                                    .update(|set| {
                                                                        if set.contains(&opt_label) {
                                                                            set.remove(&opt_label);
                                                                        } else {
                                                                            set.insert(opt_label.clone());
                                                                        }
                                                                    });
                                                            }
                                                        />
                                                        {opt.clone()}
                                                    </label>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                    <button
                                        class="w-full mt-2 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-md disabled:opacity-50"
                                        disabled=move || checked_cats.get().is_empty()
                                        on:click={
                                            let oc = on_close.clone();
                                            move |_| {
                                                let items: Vec<String> = checked_cats
                                                    .get()
                                                    .into_iter()
                                                    .collect();
                                                if !items.is_empty() {
                                                    filter_state
                                                        .update(|m| {
                                                            m.insert(c_idx, FilterOp::CategoryIn(items));
                                                        });
                                                    oc();
                                                }
                                            }
                                        }
                                    >
                                        "Apply selection"
                                    </button>
                                </div>
                            }
                                .into_any()
                        }
                    }
                }}
            </div>

            // ── Footer ──────────────────────────────────────────────────────
            <div class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
                <button
                    class="px-3 py-1.5 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
                    on:click=move |_| on_close_clear()
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}
