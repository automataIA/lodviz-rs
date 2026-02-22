/// Cross-chart linking via shared context
///
/// Provides a `DashboardContext` that synchronizes hover and selection
/// state between multiple charts in the same dashboard.
use leptos::prelude::*;
use lodviz_core::core::selection::Selection;

/// Shared dashboard interaction state
///
/// Charts publish their hover/selection state here, and
/// other charts consume it to highlight corresponding data.
#[derive(Debug, Clone)]
pub struct DashboardContext {
    /// Currently hovered x-domain value (shared across charts)
    pub hover_x: RwSignal<Option<f64>>,
    /// Current brush/click selection (shared across charts)
    pub selection: RwSignal<Option<Selection>>,
}

impl DashboardContext {
    /// Create a new empty dashboard context
    pub fn new() -> Self {
        Self {
            hover_x: RwSignal::new(None),
            selection: RwSignal::new(None),
        }
    }
}

impl Default for DashboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider component for cross-chart linking
///
/// Wrap multiple charts in `<LinkedDashboard>` to enable
/// synchronized hover and selection between them.
///
/// Charts can access the shared state via:
/// ```rust,ignore
/// let ctx = use_context::<DashboardContext>();
/// ```
#[component]
pub fn LinkedDashboard(children: Children) -> impl IntoView {
    let ctx = DashboardContext::new();
    provide_context(ctx);
    children()
}
