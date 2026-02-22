/// Theme configuration for charts
use serde::{Deserialize, Serialize};

/// Grid line styling configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridStyle {
    /// Grid line color (hex)
    pub color: String,
    /// Grid line opacity (0.0 to 1.0)
    pub opacity: f64,
    /// Grid line stroke width (pixels)
    pub width: f64,
    /// SVG stroke-dasharray pattern (e.g. "4,4" for dashed)
    pub dash: Option<String>,
    /// Show vertical grid lines (x-axis ticks)
    pub show_x: bool,
    /// Show horizontal grid lines (y-axis ticks)
    pub show_y: bool,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            color: "#e0e0e0".to_string(),
            opacity: 0.3,
            width: 0.5,
            dash: None,
            show_x: true,
            show_y: true,
        }
    }
}

/// Global theme settings for charts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartTheme {
    /// Background color of the chart area
    pub background_color: String,
    /// Default text color
    pub text_color: String,
    /// Color palette for data series
    pub palette: Vec<String>,
    /// Font family for text elements
    pub font_family: String,
    /// Default font size
    pub font_size: f64,
    /// Axis label font size
    pub axis_font_size: f64,
    /// Grid line styling
    pub grid: GridStyle,
    /// Axis line color
    pub axis_color: String,
    /// Point radius for scatter plots (pixels)
    pub point_radius: f64,
    /// Stroke width for line charts (pixels)
    pub stroke_width: f64,
    /// Point opacity for scatter plots (0.0 to 1.0)
    pub point_opacity: f64,
    /// Line opacity for line charts (0.0 to 1.0)
    pub line_opacity: f64,
    /// Fill opacity for area charts (0.0 to 1.0)
    pub area_opacity: f64,
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self {
            background_color: "#ffffff".to_string(),
            text_color: "#333333".to_string(),
            palette: vec![
                "#5470c6".to_string(),
                "#91cc75".to_string(),
                "#fac858".to_string(),
                "#ee6666".to_string(),
                "#73c0de".to_string(),
                "#3ba272".to_string(),
                "#fc8452".to_string(),
                "#9a60b4".to_string(),
                "#ea7ccc".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            axis_font_size: 10.0,
            grid: GridStyle::default(),
            axis_color: "#6E7079".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }
}

impl ChartTheme {
    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            background_color: "#100c2a".to_string(),
            text_color: "#eeeeee".to_string(),
            palette: vec![
                "#4992ff".to_string(),
                "#7cffb2".to_string(),
                "#fddd60".to_string(),
                "#ff6e76".to_string(),
                "#58d9f9".to_string(),
                "#05c091".to_string(),
                "#ff8a45".to_string(),
                "#8d48e3".to_string(),
                "#dd79ff".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            axis_font_size: 10.0,
            grid: GridStyle {
                color: "#2b2b2b".to_string(),
                ..GridStyle::default()
            },
            axis_color: "#B9B8CE".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }

    /// Nord theme — muted Arctic tones
    pub fn nord() -> Self {
        Self {
            background_color: "#2e3440".to_string(),
            text_color: "#eceff4".to_string(),
            palette: vec![
                "#88c0d0".to_string(),
                "#81a1c1".to_string(),
                "#5e81ac".to_string(),
                "#a3be8c".to_string(),
                "#ebcb8b".to_string(),
                "#d08770".to_string(),
                "#bf616a".to_string(),
                "#b48ead".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            axis_font_size: 10.0,
            grid: GridStyle {
                color: "#3b4252".to_string(),
                ..GridStyle::default()
            },
            axis_color: "#d8dee9".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }

    /// Solarized Light theme
    pub fn solarized_light() -> Self {
        Self {
            background_color: "#fdf6e3".to_string(),
            text_color: "#657b83".to_string(),
            palette: vec![
                "#268bd2".to_string(),
                "#2aa198".to_string(),
                "#859900".to_string(),
                "#b58900".to_string(),
                "#cb4b16".to_string(),
                "#dc322f".to_string(),
                "#d33682".to_string(),
                "#6c71c4".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            axis_font_size: 10.0,
            grid: GridStyle {
                color: "#eee8d5".to_string(),
                ..GridStyle::default()
            },
            axis_color: "#93a1a1".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }

    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            background_color: "#002b36".to_string(),
            text_color: "#839496".to_string(),
            palette: vec![
                "#268bd2".to_string(),
                "#2aa198".to_string(),
                "#859900".to_string(),
                "#b58900".to_string(),
                "#cb4b16".to_string(),
                "#dc322f".to_string(),
                "#d33682".to_string(),
                "#6c71c4".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            axis_font_size: 10.0,
            grid: GridStyle {
                color: "#073642".to_string(),
                ..GridStyle::default()
            },
            axis_color: "#586e75".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }

    /// High-contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            background_color: "#000000".to_string(),
            text_color: "#ffffff".to_string(),
            palette: vec![
                "#ffff00".to_string(),
                "#00ffff".to_string(),
                "#ff00ff".to_string(),
                "#00ff00".to_string(),
                "#ff6600".to_string(),
                "#6666ff".to_string(),
                "#ff0066".to_string(),
                "#66ff66".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 14.0,
            axis_font_size: 12.0,
            grid: GridStyle {
                color: "#333333".to_string(),
                ..GridStyle::default()
            },
            axis_color: "#ffffff".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }

    /// Monokai theme — warm dark tones
    pub fn monokai() -> Self {
        Self {
            background_color: "#272822".to_string(),
            text_color: "#f8f8f2".to_string(),
            palette: vec![
                "#a6e22e".to_string(),
                "#66d9ef".to_string(),
                "#f92672".to_string(),
                "#fd971f".to_string(),
                "#e6db74".to_string(),
                "#ae81ff".to_string(),
                "#a1efe4".to_string(),
                "#f8f8f2".to_string(),
            ],
            font_family: "sans-serif".to_string(),
            font_size: 12.0,
            axis_font_size: 10.0,
            grid: GridStyle {
                color: "#3e3d32".to_string(),
                ..GridStyle::default()
            },
            axis_color: "#75715e".to_string(),
            point_radius: 5.0,
            stroke_width: 5.0,
            point_opacity: 0.5,
            line_opacity: 0.5,
            area_opacity: 0.5,
        }
    }

    /// Get all available preset theme names
    pub fn preset_names() -> &'static [&'static str] {
        &[
            "default",
            "dark",
            "nord",
            "solarized_light",
            "solarized_dark",
            "high_contrast",
            "monokai",
        ]
    }

    /// Get a preset theme by name
    pub fn from_preset(name: &str) -> Self {
        match name {
            "dark" => Self::dark(),
            "nord" => Self::nord(),
            "solarized_light" => Self::solarized_light(),
            "solarized_dark" => Self::solarized_dark(),
            "high_contrast" => Self::high_contrast(),
            "monokai" => Self::monokai(),
            _ => Self::default(),
        }
    }
}

/// Choose text color (light or dark) for best contrast against a background
///
/// Returns "#ffffff" for dark backgrounds, "#1a1a1a" for light backgrounds.
pub fn auto_text_color(background: &str) -> &'static str {
    let Some((r, g, b)) = parse_hex_color(background) else {
        return "#1a1a1a";
    };
    let lum = relative_luminance(r, g, b);
    if lum > 0.179 {
        "#1a1a1a"
    } else {
        "#ffffff"
    }
}

/// Interpolate between two hex colors in sRGB space
///
/// `t` is in range 0.0 (= c1) to 1.0 (= c2)
pub fn interpolate_color(c1: &str, c2: &str, t: f64) -> String {
    let t = t.clamp(0.0, 1.0);
    let Some((r1, g1, b1)) = parse_hex_color(c1) else {
        return c1.to_string();
    };
    let Some((r2, g2, b2)) = parse_hex_color(c2) else {
        return c1.to_string();
    };

    let r = (r1 + (r2 - r1) * t).clamp(0.0, 1.0);
    let g = (g1 + (g2 - g1) * t).clamp(0.0, 1.0);
    let b = (b1 + (b2 - b1) * t).clamp(0.0, 1.0);

    format!(
        "#{:02x}{:02x}{:02x}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

/// Generate a palette of N colors interpolated between anchor colors
///
/// `anchors` must have at least 2 colors. Colors are evenly distributed.
pub fn generate_palette(anchors: &[&str], n: usize) -> Vec<String> {
    if anchors.len() < 2 || n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![anchors[0].to_string()];
    }

    let segments = anchors.len() - 1;
    (0..n)
        .map(|i| {
            let t = i as f64 / (n - 1) as f64;
            let scaled = t * segments as f64;
            let seg = (scaled.floor() as usize).min(segments - 1);
            let local_t = scaled - seg as f64;
            interpolate_color(anchors[seg], anchors[seg + 1], local_t)
        })
        .collect()
}

/// Predefined sequential palette: Viridis (8 colors)
pub const VIRIDIS: [&str; 8] = [
    "#440154", "#46327e", "#365c8d", "#277f8e", "#1fa187", "#4ac16d", "#9fda3a", "#fde725",
];

/// Predefined sequential palette: Plasma (8 colors)
pub const PLASMA: [&str; 8] = [
    "#0d0887", "#5b02a3", "#9a179b", "#cb4678", "#eb7852", "#fbb32b", "#eff821", "#f0f921",
];

/// Predefined diverging palette: Red-Blue (8 colors)
pub const RD_BU: [&str; 8] = [
    "#b2182b", "#d6604d", "#f4a582", "#fddbc7", "#d1e5f0", "#92c5de", "#4393c3", "#2166ac",
];

/// per-instance configuration for charts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ChartConfig {
    /// Override for theme
    pub theme: Option<ChartTheme>,
    /// Chart title
    pub title: Option<String>,
    /// Grid style override
    pub grid: Option<GridStyle>,
    /// Show tooltip
    pub show_tooltip: Option<bool>,
    /// Custom width (overrides responsive)
    pub width: Option<u32>,
    /// Custom height (overrides responsive)
    pub height: Option<u32>,
    /// Margins
    pub margin: Option<Margin>,
    /// Legend visibility. None = auto (show if series > 1), Some(true) = always, Some(false) = never
    pub show_legend: Option<bool>,
    /// Legend placement. true = outside the plot area (right margin). None/false = overlay inside plot.
    pub legend_outside: Option<bool>,
}

/// Margin configuration around the chart area
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Margin {
    /// Top margin in pixels
    pub top: f64,
    /// Right margin in pixels
    pub right: f64,
    /// Bottom margin in pixels
    pub bottom: f64,
    /// Left margin in pixels
    pub left: f64,
}

impl Default for Margin {
    fn default() -> Self {
        Self {
            top: 20.0,
            right: 20.0,
            bottom: 50.0,
            left: 60.0,
        }
    }
}

impl ChartConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the chart title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set grid style
    pub fn with_grid(mut self, grid: GridStyle) -> Self {
        self.grid = Some(grid);
        self
    }

    /// Toggle grid visibility
    pub fn with_grid_visible(mut self, show: bool) -> Self {
        let mut g = self.grid.unwrap_or_default();
        g.show_x = show;
        g.show_y = show;
        self.grid = Some(g);
        self
    }

    /// Set custom dimensions
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Control legend visibility explicitly
    pub fn with_legend(mut self, show: bool) -> Self {
        self.show_legend = Some(show);
        self
    }

    /// Place the legend outside the plot area (adjacent to the right edge) instead of overlaid
    pub fn with_legend_outside(mut self, outside: bool) -> Self {
        self.legend_outside = Some(outside);
        self
    }
}

/// Parse a hex color string (#RGB, #RRGGBB, or #RRGGBBAA) into (r, g, b) as 0.0..1.0
fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((
                f64::from(r) / 255.0,
                f64::from(g) / 255.0,
                f64::from(b) / 255.0,
            ))
        }
        6 | 8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((
                f64::from(r) / 255.0,
                f64::from(g) / 255.0,
                f64::from(b) / 255.0,
            ))
        }
        _ => None,
    }
}

/// Convert sRGB channel to linear value for luminance calculation
fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Calculate relative luminance per WCAG 2.1 definition
fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    0.2126 * srgb_to_linear(r) + 0.7152 * srgb_to_linear(g) + 0.0722 * srgb_to_linear(b)
}

/// Calculate WCAG contrast ratio between two hex colors.
/// Returns a value between 1.0 and 21.0.
/// Returns 1.0 if either color cannot be parsed.
pub fn contrast_ratio(fg: &str, bg: &str) -> f64 {
    let Some((r1, g1, b1)) = parse_hex_color(fg) else {
        return 1.0;
    };
    let Some((r2, g2, b2)) = parse_hex_color(bg) else {
        return 1.0;
    };

    let l1 = relative_luminance(r1, g1, b1);
    let l2 = relative_luminance(r2, g2, b2);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Check if contrast meets WCAG AA for non-text elements (>= 3:1)
pub fn meets_wcag_aa_graphics(fg: &str, bg: &str) -> bool {
    contrast_ratio(fg, bg) >= 3.0
}

/// Check if contrast meets WCAG AA for normal text (>= 4.5:1)
pub fn meets_wcag_aa_text(fg: &str, bg: &str) -> bool {
    contrast_ratio(fg, bg) >= 4.5
}

/// Color scheme variants for charts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorScheme {
    #[default]
    /// Ocean-inspired blues and teals (professional, calm)
    Ocean,
    /// Warm sunset colors (energetic, attention-grabbing)
    Sunset,
    /// Dark matter theme (high contrast, modern)
    DarkMatter,
    /// Viridis colormap (perceptually uniform, colorblind-friendly)
    Viridis,
    /// Categorical colors (distinct, for categories)
    Categorical,
    /// Okabe-Ito — 8 colorblind-safe categorical colors (Nature Methods 2011)
    OkabeIto,
}

impl ColorScheme {
    /// Get the color palette as a vector of CSS color strings
    pub fn palette(&self) -> Vec<&'static str> {
        match self {
            ColorScheme::Ocean => vec![
                "#e0f3ff", // Very light blue
                "#b3e0ff", // Light blue
                "#66b3ff", // Sky blue
                "#3399ff", // Bright blue
                "#0073e6", // Medium blue
                "#005ab3", // Deep blue
                "#004080", // Dark blue
                "#003366", // Very dark blue
            ],
            ColorScheme::Sunset => vec![
                "#fff5e6", // Pale cream
                "#ffe0b3", // Light peach
                "#ffcc80", // Peach
                "#ffb84d", // Orange
                "#ff9933", // Bright orange
                "#ff6600", // Deep orange
                "#cc5200", // Dark orange
                "#b34700", // Very dark orange
            ],
            ColorScheme::DarkMatter => vec![
                "#f0f0f5", // Almost white
                "#d0d0e0", // Light grey-blue
                "#a0a0c0", // Medium grey-blue
                "#7070a0", // Dark grey-blue
                "#504080", // Purple-grey
                "#403060", // Dark purple-grey
                "#302040", // Very dark purple
                "#201030", // Almost black
            ],
            ColorScheme::Viridis => vec![
                "#fde724", // Yellow (high)
                "#b5de2b", // Yellow-green
                "#6ece58", // Green
                "#35b779", // Teal-green
                "#1f9e89", // Teal
                "#26828e", // Blue-teal
                "#31688e", // Blue
                "#443983", // Dark blue-purple
            ],
            ColorScheme::Categorical => vec![
                "#1f77b4", // Blue
                "#ff7f0e", // Orange
                "#2ca02c", // Green
                "#d62728", // Red
                "#9467bd", // Purple
                "#8c564b", // Brown
                "#e377c2", // Pink
                "#7f7f7f", // Grey
                "#bcbd22", // Olive
                "#17becf", // Cyan
            ],
            ColorScheme::OkabeIto => vec![
                "#E69F00", // Orange
                "#56B4E9", // Sky blue
                "#009E73", // Bluish green
                "#F0E442", // Yellow
                "#0072B2", // Blue
                "#D55E00", // Vermilion
                "#CC79A7", // Reddish purple
                "#000000", // Black
            ],
        }
    }

    /// Get a single color from the palette by index
    pub fn color_at(&self, index: usize) -> &'static str {
        let palette = self.palette();
        palette[index % palette.len()]
    }

    /// Get the primary color (first color in palette)
    pub fn primary(&self) -> &'static str {
        self.color_at(0)
    }

    /// Get the secondary color (middle color in palette)
    pub fn secondary(&self) -> &'static str {
        let palette = self.palette();
        self.color_at(palette.len() / 2)
    }

    /// Get the accent color (last color in palette)
    pub fn accent(&self) -> &'static str {
        let palette = self.palette();
        self.color_at(palette.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_contrast_ratio_black_white() {
        let ratio = contrast_ratio("#000000", "#ffffff");
        assert!(approx_eq(ratio, 21.0, 0.1));
    }

    #[test]
    fn test_contrast_ratio_same_color() {
        let ratio = contrast_ratio("#ff0000", "#ff0000");
        assert!(approx_eq(ratio, 1.0, 0.01));
    }

    #[test]
    fn test_contrast_ratio_invalid_color() {
        assert!(approx_eq(contrast_ratio("invalid", "#fff"), 1.0, 0.01));
    }

    #[test]
    fn test_meets_wcag_aa_graphics() {
        // Black on white: 21:1 — passes
        assert!(meets_wcag_aa_graphics("#000000", "#ffffff"));
        // Light gray on white: low contrast — fails
        assert!(!meets_wcag_aa_graphics("#cccccc", "#ffffff"));
    }

    #[test]
    fn test_meets_wcag_aa_text() {
        assert!(meets_wcag_aa_text("#000000", "#ffffff"));
        // Medium gray on white: ~4.5:1 might barely pass
        assert!(!meets_wcag_aa_text("#888888", "#ffffff"));
    }

    #[test]
    fn test_parse_shorthand_hex() {
        // #fff should parse as white
        let ratio = contrast_ratio("#000", "#fff");
        assert!(approx_eq(ratio, 21.0, 0.1));
    }

    #[test]
    fn test_contrast_ratio_with_alpha_hex() {
        // #rrggbbaa — alpha is ignored for contrast
        let ratio = contrast_ratio("#000000ff", "#ffffffff");
        assert!(approx_eq(ratio, 21.0, 0.1));
    }

    // === Theme presets ===

    #[test]
    fn test_preset_names() {
        let names = ChartTheme::preset_names();
        assert!(names.contains(&"dark"));
        assert!(names.contains(&"nord"));
        assert!(names.contains(&"high_contrast"));
    }

    #[test]
    fn test_from_preset() {
        let nord = ChartTheme::from_preset("nord");
        assert_eq!(nord.background_color, "#2e3440");
        let default = ChartTheme::from_preset("unknown");
        assert_eq!(default, ChartTheme::default());
    }

    // === auto_text_color ===

    #[test]
    fn test_auto_text_color_dark_bg() {
        assert_eq!(auto_text_color("#000000"), "#ffffff");
        assert_eq!(auto_text_color("#100c2a"), "#ffffff");
    }

    #[test]
    fn test_auto_text_color_light_bg() {
        assert_eq!(auto_text_color("#ffffff"), "#1a1a1a");
        assert_eq!(auto_text_color("#fdf6e3"), "#1a1a1a");
    }

    // === interpolate_color ===

    #[test]
    fn test_interpolate_endpoints() {
        assert_eq!(interpolate_color("#000000", "#ffffff", 0.0), "#000000");
        assert_eq!(interpolate_color("#000000", "#ffffff", 1.0), "#ffffff");
    }

    #[test]
    fn test_interpolate_midpoint() {
        let mid = interpolate_color("#000000", "#ffffff", 0.5);
        // Should be approximately #808080 (gray)
        assert_eq!(mid, "#808080");
    }

    #[test]
    fn test_interpolate_clamped() {
        assert_eq!(interpolate_color("#000000", "#ffffff", -1.0), "#000000");
        assert_eq!(interpolate_color("#000000", "#ffffff", 2.0), "#ffffff");
    }

    // === generate_palette ===

    #[test]
    fn test_generate_palette_basic() {
        let palette = generate_palette(&["#000000", "#ffffff"], 3);
        assert_eq!(palette.len(), 3);
        assert_eq!(palette[0], "#000000");
        assert_eq!(palette[1], "#808080");
        assert_eq!(palette[2], "#ffffff");
    }

    #[test]
    fn test_generate_palette_single() {
        let palette = generate_palette(&["#ff0000", "#0000ff"], 1);
        assert_eq!(palette.len(), 1);
        assert_eq!(palette[0], "#ff0000");
    }

    #[test]
    fn test_generate_palette_empty() {
        assert!(generate_palette(&["#fff"], 5).is_empty());
        assert!(generate_palette(&["#000", "#fff"], 0).is_empty());
    }
}
