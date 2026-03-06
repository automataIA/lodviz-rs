/// Theme support for DataTable component
///
/// Extracts the ChartTheme from context and generates CSS variables
/// for dynamic styling based on the current theme.
use leptos::prelude::*;
use lodviz_core::core::theme::ChartTheme;

/// Hook to get the current chart theme from context
///
/// Returns the theme signal, or a default theme if not in context.
pub fn use_table_theme() -> Signal<ChartTheme> {
    use_context::<Signal<ChartTheme>>().unwrap_or_else(|| Signal::stored(ChartTheme::default()))
}

/// Generate CSS variables string from theme for table styling
///
/// Returns a style attribute string like "--table-bg: #ffffff; --table-border: #e5e7eb; ..."
pub fn table_theme_css_vars(theme: &ChartTheme) -> String {
    format!(
        "--table-bg: {}; \
         --table-border: {}; \
         --table-header-bg: {}; \
         --table-header-text: {}; \
         --table-text: {}; \
         --table-hover: {}; \
         --table-selected: {}; \
         --table-primary: {}; \
         --table-primary-hover: {}; \
         --table-danger: {}; \
         --table-success: {}; \
         --table-muted: {}; \
         --table-input-bg: {}; \
         --table-input-border: {}; \
         --table-input-text: {}; \
         --table-accent: {};",
        theme.table_bg,
        theme.table_border,
        theme.table_header_bg,
        theme.table_header_text,
        theme.table_text,
        theme.table_hover,
        theme.table_selected,
        theme.table_primary,
        theme.table_primary_hover,
        theme.table_danger,
        theme.table_success,
        theme.table_muted,
        theme.table_input_bg,
        theme.table_input_border,
        theme.table_input_text,
        theme.table_accent,
    )
}
