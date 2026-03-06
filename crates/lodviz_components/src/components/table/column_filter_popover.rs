/// Column-specific filter popover component.
use std::collections::{HashMap, HashSet};

use leptos::prelude::*;
use lodviz_core::core::table_data::{
    Alignment, ColumnDef, ColumnType, CompareOp, FilterOp, TableData,
};

/// Filter popover for a single column, embedded in table header.
#[component]
pub fn ColumnFilterPopover(
    /// Index of the column being filtered
    col_index: usize,
    /// Global filter state (shared across all columns)
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    /// Table data (needed for distinct_values and column definition)
    data: Signal<TableData>,
    /// Called when the user closes the popover
    on_close: impl Fn() + Send + Sync + Clone + 'static,
) -> impl IntoView {
    // Derive column definition from data
    let col_def = Signal::derive(move || {
        data.get()
            .columns
            .get(col_index)
            .cloned()
            .unwrap_or_else(|| ColumnDef {
                key: String::new(),
                label: String::new(),
                col_type: ColumnType::Text,
                sortable: false,
                filterable: false,
                width: None,
                alignment: Alignment::Left,
                conditional: None,
            })
    });
    // ── Temporary form states ───────────────────────────────────────────────
    let text_input = RwSignal::new(String::new());
    let number_operator = RwSignal::new(CompareOp::Equal); // Default operator
    let number_value = RwSignal::new(String::new());

    // Pre-populate form inputs from existing filter state
    Effect::new(move |_| {
        if let Some(filter_op) = filter_state.get().get(&col_index) {
            match filter_op {
                FilterOp::TextContains(val) => {
                    text_input.set(val.clone());
                }
                FilterOp::NumberCompare { operator, value } => {
                    number_operator.set(*operator);
                    number_value.set(value.to_string());
                }
                _ => {}
            }
        }
    });

    let col_label = Signal::derive(move || col_def.get().label.clone());
    let col_type = Signal::derive(move || col_def.get().col_type.clone());

    let on_close_clear = on_close.clone();

    view! {
        <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl shadow-2xl p-4 min-w-80 max-w-md">
            // ── Header ──────────────────────────────────────────────────────
            <div class="flex items-center justify-between mb-4 pb-3 border-b border-gray-200 dark:border-gray-700">
                <h3 class="text-base font-semibold text-gray-800 dark:text-gray-200">
                    {move || format!("Filtra: {}", col_label.get())}
                </h3>
                <button
                    class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                    on:click=move |_| on_close_clear()
                    aria-label="Chiudi filtro"
                >
                    <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>

            // ── Filter Content (type-specific) ─────────────────────────────
            {move || {
                match col_type.get() {
                    ColumnType::Text => {
                        render_text_filter(col_index, text_input, filter_state, on_close.clone())
                            .into_any()
                    }
                    ColumnType::Number => {
                        render_number_filter(
                                col_index,
                                number_operator,
                                number_value,
                                filter_state,
                                on_close.clone(),
                            )
                            .into_any()
                    }
                    ColumnType::Boolean => {
                        render_boolean_filter(col_index, filter_state, on_close.clone()).into_any()
                    }
                    ColumnType::Category(options) => {
                        render_category_filter(
                                col_index,
                                options,
                                data,
                                filter_state,
                                on_close.clone(),
                            )
                            .into_any()
                    }
                }
            }}
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Filter Renderers (type-specific)
// ═══════════════════════════════════════════════════════════════════════════

/// Text filter: search input + Clear/Apply buttons + Enter support
fn render_text_filter(
    col_index: usize,
    text_input: RwSignal<String>,
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    on_close: impl Fn() + Send + Sync + Clone + 'static,
) -> impl IntoView {
    view! {
        <div class="space-y-3">
            // Search input
            <div>
                <input
                    type="text"
                    placeholder="Cerca testo..."
                    class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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
                                            m.insert(col_index, FilterOp::TextContains(val));
                                        });
                                    oc();
                                }
                            }
                        }
                    }
                />
            </div>

            // Action buttons
            <div class="flex gap-2 pt-1">
                <button
                    class="flex-1 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            text_input.set(String::new());
                            filter_state
                                .update(|m| {
                                    m.remove(&col_index);
                                });
                            oc();
                        }
                    }
                >
                    "Cancella"
                </button>
                <button
                    class="flex-1 px-3 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled=move || text_input.get().is_empty()
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            filter_state
                                .update(|m| {
                                    m.insert(col_index, FilterOp::TextContains(text_input.get()));
                                });
                            oc();
                        }
                    }
                >
                    "Applica"
                </button>
            </div>
        </div>
    }
}

/// Number filter: Operator dropdown + value input + Clear/Apply buttons
fn render_number_filter(
    col_index: usize,
    operator: RwSignal<CompareOp>,
    value: RwSignal<String>,
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    on_close: impl Fn() + Send + Sync + Clone + 'static,
) -> impl IntoView {
    const OPERATORS: &[CompareOp] = &[
        CompareOp::Equal,
        CompareOp::NotEqual,
        CompareOp::Greater,
        CompareOp::GreaterEq,
        CompareOp::Less,
        CompareOp::LessEq,
    ];

    view! {
        <div class="space-y-3">
            // Operator dropdown
            <div>
                <select
                    class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    on:change=move |ev| {
                        let idx = event_target_value(&ev).parse::<usize>().unwrap_or(0);
                        operator.set(OPERATORS[idx]);
                    }
                >
                    {OPERATORS
                        .iter()
                        .enumerate()
                        .map(|(idx, op)| {
                            let is_selected = Signal::derive(move || operator.get() == *op);
                            view! {
                                <option value=idx selected=is_selected>
                                    {format!("{} ({})", op.label(), op.symbol())}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </div>

            // Value input
            <div>
                <input
                    type="number"
                    placeholder="Inserisci valore..."
                    class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    prop:value=value
                    on:input=move |ev| value.set(event_target_value(&ev))
                />
            </div>

            // Action buttons
            <div class="flex gap-2 pt-1">
                <button
                    class="flex-1 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            value.set(String::new());
                            filter_state
                                .update(|m| {
                                    m.remove(&col_index);
                                });
                            oc();
                        }
                    }
                >
                    "Cancella"
                </button>
                <button
                    class="flex-1 px-3 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled=move || value.get().is_empty()
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            if let Ok(num_value) = value.get().parse::<f64>() {
                                filter_state
                                    .update(|m| {
                                        m.insert(
                                            col_index,
                                            FilterOp::NumberCompare {
                                                operator: operator.get(),
                                                value: num_value,
                                            },
                                        );
                                    });
                                oc();
                            }
                        }
                    }
                >
                    "Applica"
                </button>
            </div>
        </div>
    }
}

/// Boolean filter: instant feedback radio-style buttons
fn render_boolean_filter(
    col_index: usize,
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    on_close: impl Fn() + Send + Sync + Clone + 'static,
) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <div class="flex flex-col gap-2">
                <button
                    class="w-full px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border-2 border-gray-300 dark:border-gray-600 rounded-lg hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-gray-600 transition-all"
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            filter_state
                                .update(|m| {
                                    m.insert(
                                        col_index,
                                        FilterOp::CategoryIn(vec!["Yes".to_owned()]),
                                    );
                                });
                            oc();
                        }
                    }
                >
                    <div class="flex items-center justify-center gap-2">
                        <svg class="w-4 h-4 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                            <path
                                fill-rule="evenodd"
                                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                                clip-rule="evenodd"
                            />
                        </svg>
                        <span>"Solo 'Yes'"</span>
                    </div>
                </button>
                <button
                    class="w-full px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border-2 border-gray-300 dark:border-gray-600 rounded-lg hover:border-red-500 hover:bg-red-50 dark:hover:bg-gray-600 transition-all"
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            filter_state
                                .update(|m| {
                                    m.insert(
                                        col_index,
                                        FilterOp::CategoryIn(vec!["No".to_owned()]),
                                    );
                                });
                            oc();
                        }
                    }
                >
                    <div class="flex items-center justify-center gap-2">
                        <svg class="w-4 h-4 text-red-600" fill="currentColor" viewBox="0 0 20 20">
                            <path
                                fill-rule="evenodd"
                                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                                clip-rule="evenodd"
                            />
                        </svg>
                        <span>"Solo 'No'"</span>
                    </div>
                </button>
            </div>
        </div>
    }
}

/// Category filter: search + checkbox list + Deselect/Apply buttons
fn render_category_filter(
    col_index: usize,
    options: Vec<String>,
    data: Signal<TableData>,
    filter_state: RwSignal<HashMap<usize, FilterOp>>,
    on_close: impl Fn() + Send + Sync + Clone + 'static,
) -> impl IntoView {
    // Get distinct values from data if options not provided
    let opts: Vec<String> = if options.is_empty() {
        data.get_untracked().distinct_values(col_index)
    } else {
        options
    };

    // Pre-populate from existing filter state
    let checked_cats: RwSignal<HashSet<String>> = RwSignal::new({
        match filter_state.get_untracked().get(&col_index) {
            Some(FilterOp::CategoryIn(items)) => items.iter().cloned().collect(),
            _ => HashSet::new(),
        }
    });

    // Search filter
    let search_query = RwSignal::new(String::new());

    view! {
        <div class="space-y-3">
            // Search input (always visible for consistency)
            <div>
                <input
                    type="text"
                    placeholder="Cerca..."
                    class="w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    prop:value=search_query
                    on:input=move |ev| search_query.set(event_target_value(&ev))
                />
            </div>

            // Checkbox list
            <div class="max-h-56 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded-lg">
                {move || {
                    let query = search_query.get().to_lowercase();
                    opts.iter()
                        .filter(|opt| {
                            if query.is_empty() {
                                true
                            } else {
                                opt.to_lowercase().contains(&query)
                            }
                        })
                        .map(|opt| {
                            let opt_check = opt.clone();
                            let opt_label = opt.clone();
                            let is_checked = Signal::derive(move || {
                                checked_cats.get().contains(&opt_check)
                            });
                            view! {
                                <label class="flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer border-b border-gray-100 dark:border-gray-700 last:border-b-0">
                                    <input
                                        type="checkbox"
                                        class="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-2 focus:ring-blue-500"
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
                                    <span class="flex-1">{opt.clone()}</span>
                                </label>
                            }
                        })
                        .collect_view()
                }}
            </div>

            // Action buttons
            <div class="flex items-center justify-between pt-1">
                <button
                    class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 font-medium"
                    on:click=move |_| checked_cats.set(HashSet::new())
                >
                    "Deseleziona tutti"
                </button>
                <button
                    class="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                    disabled=move || checked_cats.get().is_empty()
                    on:click={
                        let oc = on_close.clone();
                        move |_| {
                            let items: Vec<String> = checked_cats.get().into_iter().collect();
                            if !items.is_empty() {
                                filter_state
                                    .update(|m| {
                                        m.insert(col_index, FilterOp::CategoryIn(items));
                                    });
                                oc();
                            }
                        }
                    }
                >
                    "Applica"
                </button>
            </div>
        </div>
    }
}
