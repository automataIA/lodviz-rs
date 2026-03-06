/// Interactive data table component — BI-style (Power BI / Tableau).
///
/// Features: multi-column sort, per-column filters (text/range/category),
/// global search, pagination, row selection, conditional formatting,
/// column visibility toggle, and empty state.
use std::collections::{HashMap, HashSet};

use leptos::prelude::*;
use lodviz_core::core::table_data::{
    color_scale_bg, data_bar_pct, format_cell_value, ConditionalRule, TableData,
};

use crate::hooks::use_container_size::use_container_size;

use super::table_theme::{table_theme_css_vars, use_table_theme};
use super::{table_footer::TableFooter, table_header::TableHeader, table_toolbar::TableToolbar};

/// An interactive data table with sorting, filtering, pagination and conditional formatting.
#[component]
pub fn DataTable(
    /// The table data (column definitions + rows). Can be a reactive signal.
    data: Signal<TableData>,
    /// Optional card title displayed above the toolbar.
    #[prop(optional, into)]
    title: Option<String>,
    /// Maximum height of the scrollable body in pixels (no limit by default).
    #[prop(optional)]
    height: Option<u32>,
    /// Initial number of rows per page.
    #[prop(default = 25)]
    default_page_size: usize,
) -> impl IntoView {
    // ── Interactive state ────────────────────────────────────────────────────
    let sort_state: RwSignal<Vec<lodviz_core::core::table_data::SortKey>> =
        RwSignal::new(Vec::new());
    let filter_state: RwSignal<HashMap<usize, lodviz_core::core::table_data::FilterOp>> =
        RwSignal::new(HashMap::new());
    let page = RwSignal::new(0usize);
    let selected_rows: RwSignal<HashSet<usize>> = RwSignal::new(HashSet::new());
    let col_visibility: RwSignal<Vec<bool>> = RwSignal::new(Vec::new());
    let show_global_filters = RwSignal::new(false);

    // ── Dimension tracking ───────────────────────────────────────────────────
    let (_, container_height, container_ref) = use_container_size();

    // Dynamically calculate page size based on available container height
    let page_size = Signal::derive(move || {
        let h = container_height.get();
        // If the container has no height (e.g. initially or not rendered), use default
        if h <= 0.0 {
            return default_page_size;
        }

        // Approximate heights:
        // Table header is ~40px
        // Each row is ~37px (py-2 = 16px, text-sm = 20px, border = 1px)
        let header_height = 40.0;
        let row_height = 37.0;

        let available = h - header_height;
        if available < row_height {
            1 // show at least 1 row if possible
        } else {
            (available / row_height).floor() as usize
        }
    });

    // Initialise column visibility whenever the data schema changes
    Effect::new(move |_| {
        let ncols = data.get().columns.len();
        col_visibility.set(vec![true; ncols]);
    });

    // ── Derived: filtered row indices ────────────────────────────────────────
    let filtered_indices = Memo::new(move |_| {
        let table = data.get();
        let filters = filter_state.get();

        (0..table.rows.len())
            .filter(|&ri| {
                let row = &table.rows[ri];

                // Per-column filters
                for (&ci, op) in &filters {
                    let val = row
                        .get(ci)
                        .unwrap_or(&lodviz_core::core::field_value::FieldValue::Null);
                    if !op.matches(val) {
                        return false;
                    }
                }

                true
            })
            .collect::<Vec<_>>()
    });

    // ── Derived: sorted row indices ──────────────────────────────────────────
    let sorted_indices = Memo::new(move |_| {
        let table = data.get();
        let keys = sort_state.get();
        let mut indices = filtered_indices.get();

        if !keys.is_empty() {
            use lodviz_core::core::table_data::compare_field_values;
            indices.sort_by(|&a, &b| {
                let ra = &table.rows[a];
                let rb = &table.rows[b];
                for k in &keys {
                    let va = ra.get(k.col_index);
                    let vb = rb.get(k.col_index);
                    let cmp = compare_field_values(va, vb);
                    let cmp = if k.direction == lodviz_core::core::table_data::SortDir::Desc {
                        cmp.reverse()
                    } else {
                        cmp
                    };
                    if cmp != std::cmp::Ordering::Equal {
                        return cmp;
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        indices
    });

    // ── Pagination ───────────────────────────────────────────────────────────
    let total_filtered = Signal::derive(move || sorted_indices.get().len());

    let page_count = Memo::new(move |_| {
        let total = total_filtered.get();
        let size = page_size.get();
        if size == 0 || total == 0 {
            0
        } else {
            total.div_ceil(size)
        }
    });

    let page_rows = Memo::new(move |_| {
        let indices = sorted_indices.get();
        let p = page.get().min(page_count.get().saturating_sub(1));
        let size = page_size.get();
        let start = p * size;
        let end = (start + size).min(indices.len());
        if start < indices.len() {
            indices[start..end].to_vec()
        } else {
            vec![]
        }
    });

    // Reset page when filters change
    Effect::new(move |_| {
        let _ = filter_state.get();
        page.set(0);
    });

    // ── Selection helpers ────────────────────────────────────────────────────
    let selected_count = Signal::derive(move || selected_rows.get().len());
    let total_rows_count = Signal::derive(move || data.get().rows.len());

    let all_selected = Signal::derive(move || {
        let total = total_rows_count.get();
        total != 0 && selected_rows.get().len() == total
    });
    let some_selected = Signal::derive(move || {
        let sel = selected_rows.get().len();
        let total = total_rows_count.get();
        sel != 0 && sel != total
    });

    // Close filter popover when clicking outside the table area
    let has_any_filter = Signal::derive(move || !filter_state.get().is_empty());

    // ── Theme support ────────────────────────────────────────────────────────
    let theme = use_table_theme();
    let theme_vars = Signal::derive(move || table_theme_css_vars(&theme.get()));

    // ── Table scroll height style ─────────────────────────────────────────────
    let scroll_style = height
        .map(|h| format!("max-height: {h}px; overflow-y: auto;"))
        .unwrap_or_else(|| "overflow-y: auto;".to_owned());

    view! {
        <div class="flex flex-col h-full w-full overflow-hidden" style=move || theme_vars.get()>

            // ── Optional title ───────────────────────────────────────────────
            {title
                .map(|t| {
                    view! {
                        <div class="px-3 pt-2 pb-1">
                            <h3 class="text-sm font-semibold opacity-80">{t}</h3>
                        </div>
                    }
                })}
            // ── Toolbar ──────────────────────────────────────────────────────
            <TableToolbar
                data=data
                filter_state=filter_state
                col_visibility=col_visibility
                selected_count=selected_count
                show_global_filters=show_global_filters
            />
            // ── Active Filter Chips ──────────────────────────────────────────
            <Show when=move || !filter_state.get().is_empty()>
                <div class="px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800/80 flex flex-wrap gap-2 items-center">
                    <span class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mr-1">
                        "Active filters:"
                    </span>
                    {move || {
                        filter_state
                            .get()
                            .into_iter()
                            .map(|(ci, op)| {
                                let label = data
                                    .get()
                                    .columns
                                    .get(ci)
                                    .map(|c| c.label.clone())
                                    .unwrap_or_else(|| format!("Col {}", ci));
                                let op_desc = match &op {
                                    lodviz_core::core::table_data::FilterOp::TextContains(s) => {
                                        format!("Contains \"{}\"", s)
                                    }
                                    lodviz_core::core::table_data::FilterOp::NumberCompare {
                                        operator,
                                        value,
                                    } => format!("{} {}", operator.symbol(), value),
                                    lodviz_core::core::table_data::FilterOp::CategoryIn(opts) => {
                                        if opts.len() == 1 {
                                            format!("Essere \"{}\"", opts[0])
                                        } else {
                                            format!("In {} elementi", opts.len())
                                        }
                                    }
                                    lodviz_core::core::table_data::FilterOp::IsEmpty => {
                                        "È vuoto".to_string()
                                    }
                                };
                                view! {
                                    <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-blue-50 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 text-xs font-medium border border-blue-200 dark:border-blue-800">
                                        <span class="font-bold">{label}</span>
                                        <span class="opacity-75">{op_desc}</span>
                                        <button
                                            class="ml-1 text-blue-500 hover:text-blue-800 dark:hover:text-blue-200 hover:bg-blue-100 dark:hover:bg-blue-800 rounded-full p-0.5 transition-colors"
                                            on:click=move |_| {
                                                filter_state
                                                    .update(|m| {
                                                        m.remove(&ci);
                                                    });
                                            }
                                        >
                                            <svg
                                                class="w-3 h-3"
                                                fill="none"
                                                viewBox="0 0 24 24"
                                                stroke="currentColor"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M6 18L18 6M6 6l12 12"
                                                />
                                            </svg>
                                        </button>
                                    </span>
                                }
                            })
                            .collect_view()
                    }}
                    <button
                        class="ml-auto text-xs font-medium text-red-500 hover:text-red-700 dark:text-red-400 hover:underline"
                        on:click=move |_| filter_state.set(HashMap::new())
                    >
                        "Remove all"
                    </button>
                </div>
            </Show>
            // ── Table ────────────────────────────────────────────────────────
            <div node_ref=container_ref class="flex-1 min-h-0 relative" style=scroll_style>
                <table class="w-full border-collapse text-sm">
                    <TableHeader
                        data=data
                        sort_state=sort_state
                        col_visibility=col_visibility
                        filter_state=filter_state
                        all_selected=all_selected
                        some_selected=some_selected
                        on_select_all=move |checked| {
                            if checked {
                                let all: HashSet<usize> = (0..total_rows_count.get()).collect();
                                selected_rows.set(all);
                            } else {
                                selected_rows.set(HashSet::new());
                            }
                        }
                    />

                    <tbody>
                        // ── Data rows ────────────────────────────────────────
                        <For
                            each=move || page_rows.get()
                            key=|row_idx| *row_idx
                            children=move |row_idx| {
                                let is_selected = Signal::derive(move || {
                                    selected_rows.get().contains(&row_idx)
                                });
                                view! {
                                    <tr class=move || {
                                        let base = "border-b border-base-content/10 hover:bg-base-content/5 transition-colors";
                                        if is_selected.get() {
                                            format!("{base} bg-base-content/10")
                                        } else {
                                            base.to_owned()
                                        }
                                    }>
                                        // Checkbox
                                        <td class="w-10 px-3 py-2">
                                            <input
                                                type="checkbox"
                                                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                                                prop:checked=is_selected
                                                on:change=move |_| {
                                                    selected_rows
                                                        .update(|set| {
                                                            if set.contains(&row_idx) {
                                                                set.remove(&row_idx);
                                                            } else {
                                                                set.insert(row_idx);
                                                            }
                                                        });
                                                }
                                            />
                                        </td>

                                        // Data cells
                                        {move || {
                                            let table = data.get();
                                            let vis = col_visibility.get();
                                            let row = table
                                                .rows
                                                .get(row_idx)
                                                .cloned()
                                                .unwrap_or_default();
                                            let all_rows = table.rows.clone();
                                            table
                                                .columns
                                                .iter()
                                                .enumerate()
                                                .filter(|(ci, _)| vis.get(*ci).copied().unwrap_or(true))
                                                .map(|(ci, col_def)| {
                                                    let val = row
                                                        .get(ci)
                                                        .cloned()
                                                        .unwrap_or(
                                                            lodviz_core::core::field_value::FieldValue::Null,
                                                        );
                                                    let text = format_cell_value(&val);
                                                    let bg_color = match &col_def.conditional {
                                                        Some(ConditionalRule::ColorScale { low, mid, high }) => {
                                                            color_scale_bg(
                                                                    &val,
                                                                    &all_rows,
                                                                    ci,
                                                                    low,
                                                                    mid.as_deref(),
                                                                    high,
                                                                )
                                                                .map(|c| format!("background-color: {c};"))
                                                                .unwrap_or_default()
                                                        }
                                                        _ => String::new(),
                                                    };
                                                    let cell_style = format!("text-align: left; {bg_color}");
                                                    let data_bar = match &col_def.conditional {
                                                        Some(ConditionalRule::DataBar { color }) => {
                                                            let pct = data_bar_pct(&val, &all_rows, ci);
                                                            let bar_w = (pct * 56.0) as u32;
                                                            let bar_color = color.clone();
                                                            Some(

                                                                // Cell background color (ColorScale)

                                                                // Data bar (inline SVG)
                                                                view! {
                                                                    <svg
                                                                        class="inline-block ml-1.5 align-middle shrink-0"
                                                                        width="56"
                                                                        height="8"
                                                                        style="vertical-align: middle;"
                                                                    >
                                                                        // Track
                                                                        <rect
                                                                            x="0"
                                                                            y="0"
                                                                            width="56"
                                                                            height="8"
                                                                            fill="#e5e7eb"
                                                                            rx="2"
                                                                        />
                                                                        // Fill
                                                                        <rect
                                                                            x="0"
                                                                            y="0"
                                                                            width=bar_w
                                                                            height="8"
                                                                            fill=bar_color
                                                                            rx="2"
                                                                        />
                                                                    </svg>
                                                                },
                                                            )
                                                        }
                                                        _ => None,
                                                    };

                                                    view! {
                                                        <td
                                                            class="px-3 py-2 whitespace-nowrap text-base-content/90"
                                                            style=cell_style
                                                        >
                                                            <span class="tabular-nums">{text}</span>
                                                            {data_bar}
                                                        </td>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </tr>
                                }
                            }
                        />

                        // ── Empty state ──────────────────────────────────────
                        <Show when=move || page_rows.get().is_empty()>
                            <tr>
                                <td
                                    class="px-6 py-12 text-center text-sm text-gray-500 dark:text-gray-400"
                                    colspan=move || {
                                        (data.get().columns.iter().filter(|_| true).count() + 1)
                                            .to_string()
                                    }
                                >
                                    <div class="flex flex-col items-center gap-3">
                                        <svg
                                            class="w-10 h-10 text-gray-300 dark:text-gray-600"
                                            fill="none"
                                            viewBox="0 0 24 24"
                                            stroke="currentColor"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="1.5"
                                                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                                            />
                                        </svg>
                                        <p class="font-medium text-gray-600 dark:text-gray-400">
                                            "No results found"
                                        </p>
                                        <Show when=has_any_filter>
                                            <button
                                                class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
                                                on:click=move |_| {
                                                    filter_state.set(HashMap::new());
                                                }
                                            >
                                                "Clear all filters"
                                            </button>
                                        </Show>
                                    </div>
                                </td>
                            </tr>
                        </Show>
                    </tbody>
                </table>
            </div>
            // ── Pagination footer ────────────────────────────────────────────
            <TableFooter
                total_rows=total_filtered
                page=page
                page_count=page_count
                page_size=page_size
            />
        </div>
    }
}
