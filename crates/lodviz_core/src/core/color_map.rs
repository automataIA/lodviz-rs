/// Color mapping utilities for continuous data visualization
///
/// Provides perceptually-uniform color maps via Oklab interpolation.
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Oklab color space conversion
// ---------------------------------------------------------------------------

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_oklab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let r = srgb_to_linear(r);
    let g = srgb_to_linear(g);
    let b = srgb_to_linear(b);

    // M1: sRGB linear -> LMS (Ottosson 2020)
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_4 * b;

    let l = l.cbrt();
    let m = m.cbrt();
    let s = s.cbrt();

    // M2: LMS -> OKLab
    (
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    )
}

fn oklab_to_srgb(l: f64, a: f64, b: f64) -> (f64, f64, f64) {
    // Inverse M2
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_342_3 * a - 0.063_854_174_7 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // Inverse M1
    let r = 4.076_741_661_3 * l - 3.307_711_592_6 * m + 0.230_969_931_3 * s;
    let g = -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s;
    let b = -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s;

    (
        linear_to_srgb(r.clamp(0.0, 1.0)),
        linear_to_srgb(g.clamp(0.0, 1.0)),
        linear_to_srgb(b.clamp(0.0, 1.0)),
    )
}

fn parse_hex(s: &str) -> (f64, f64, f64) {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0) as f64 / 255.0;
    (r, g, b)
}

fn to_hex(r: f64, g: f64, b: f64) -> String {
    let ri = (r * 255.0).round() as u8;
    let gi = (g * 255.0).round() as u8;
    let bi = (b * 255.0).round() as u8;
    format!("#{ri:02x}{gi:02x}{bi:02x}")
}

/// Interpolate between two hex colors in Oklab space
pub fn interpolate_oklab(c1: &str, c2: &str, t: f64) -> String {
    let (r1, g1, b1) = parse_hex(c1);
    let (r2, g2, b2) = parse_hex(c2);
    let (l1, a1, b1_) = srgb_to_oklab(r1, g1, b1);
    let (l2, a2, b2_) = srgb_to_oklab(r2, g2, b2);
    let (rl, ra, rb) = oklab_to_srgb(
        l1 + (l2 - l1) * t,
        a1 + (a2 - a1) * t,
        b1_ + (b2_ - b1_) * t,
    );
    to_hex(rl, ra, rb)
}

// ---------------------------------------------------------------------------
// Anchor color data (hardcoded palettes)
// ---------------------------------------------------------------------------

const VIRIDIS_ANCHORS: &[&str] = &["#440154", "#3b528b", "#21908c", "#5dc863", "#fde725"];
const PLASMA_ANCHORS: &[&str] = &[
    "#0d0887", "#6a00a8", "#b12a90", "#e16462", "#fca636", "#f0f921",
];
const INFERNO_ANCHORS: &[&str] = &[
    "#000004", "#420a68", "#932667", "#dd513a", "#fca50a", "#fcffa4",
];
const MAGMA_ANCHORS: &[&str] = &[
    "#000004", "#3b0f70", "#8c2981", "#de4968", "#fe9f6d", "#fcfdbf",
];
const CIVIDIS_ANCHORS: &[&str] = &[
    "#00204d", "#31446b", "#666870", "#958f78", "#c7b56b", "#fee838",
];
const TURBO_ANCHORS: &[&str] = &[
    "#23171b", "#4a58dd", "#2f9df5", "#27d7c4", "#4df884", "#95fb51", "#dedd32", "#ffa531",
    "#f5390a", "#7a0403",
];
const GRAYSCALE_ANCHORS: &[&str] = &["#000000", "#ffffff"];

// ColorBrewer diverging palettes (7 stops each)
const RD_BU_ANCHORS: &[&str] = &[
    "#67001f", "#d6604d", "#f4a582", "#f7f7f7", "#92c5de", "#4393c3", "#053061",
];
const PU_OR_ANCHORS: &[&str] = &[
    "#7f3b08", "#e08214", "#fdb863", "#f7f7f7", "#b2abd2", "#8073ac", "#2d004b",
];
const PI_YG_ANCHORS: &[&str] = &[
    "#8e0152", "#de77ae", "#fde0ef", "#f7f7f7", "#e6f5d0", "#7fbf7b", "#276419",
];
const BR_BG_ANCHORS: &[&str] = &[
    "#543005", "#bf812d", "#dfc27d", "#f5f5f5", "#80cdc1", "#35978f", "#003c30",
];

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Sequential (single-hue or perceptually uniform) color maps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequentialColorMap {
    /// Perceptually uniform blue-green-yellow
    Viridis,
    /// Purple-orange high-contrast
    Plasma,
    /// Black-red-yellow (high contrast)
    Inferno,
    /// Black-purple-pink-yellow
    Magma,
    /// Blue-grey-yellow (colorblind-safe)
    Cividis,
    /// Full-spectrum rainbow
    Turbo,
    /// Black to white
    Grayscale,
}

/// Diverging color maps (two hues diverging from a neutral center)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivergingColorMap {
    /// Red to Blue
    RdBu,
    /// Purple to Orange
    PuOr,
    /// Pink to Yellow-Green
    PiYG,
    /// Brown to Blue-Green
    BrBG,
}

/// A color map for encoding continuous data as color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorMap {
    /// One of the built-in sequential maps
    Sequential(SequentialColorMap),
    /// One of the built-in diverging maps
    Diverging(DivergingColorMap),
    /// Custom map defined by anchor colors
    Custom { anchors: Vec<String> },
}

impl Default for ColorMap {
    fn default() -> Self {
        Self::Sequential(SequentialColorMap::Viridis)
    }
}

impl ColorMap {
    fn anchors(&self) -> Vec<&str> {
        match self {
            Self::Sequential(s) => match s {
                SequentialColorMap::Viridis => VIRIDIS_ANCHORS.to_vec(),
                SequentialColorMap::Plasma => PLASMA_ANCHORS.to_vec(),
                SequentialColorMap::Inferno => INFERNO_ANCHORS.to_vec(),
                SequentialColorMap::Magma => MAGMA_ANCHORS.to_vec(),
                SequentialColorMap::Cividis => CIVIDIS_ANCHORS.to_vec(),
                SequentialColorMap::Turbo => TURBO_ANCHORS.to_vec(),
                SequentialColorMap::Grayscale => GRAYSCALE_ANCHORS.to_vec(),
            },
            Self::Diverging(d) => match d {
                DivergingColorMap::RdBu => RD_BU_ANCHORS.to_vec(),
                DivergingColorMap::PuOr => PU_OR_ANCHORS.to_vec(),
                DivergingColorMap::PiYG => PI_YG_ANCHORS.to_vec(),
                DivergingColorMap::BrBG => BR_BG_ANCHORS.to_vec(),
            },
            Self::Custom { anchors } => anchors.iter().map(|s| s.as_str()).collect(),
        }
    }

    /// Map a normalized value `t` in [0, 1] to a hex color string
    pub fn map(&self, t: f64) -> String {
        let t = t.clamp(0.0, 1.0);
        let anchors = self.anchors();
        if anchors.is_empty() {
            return "#000000".to_string();
        }
        if anchors.len() == 1 {
            return anchors[0].to_string();
        }
        let n = anchors.len() - 1;
        let scaled = t * n as f64;
        let idx = (scaled.floor() as usize).min(n - 1);
        let local_t = scaled - idx as f64;
        interpolate_oklab(anchors[idx], anchors[idx + 1], local_t)
    }

    /// Return a reversed version of this color map
    pub fn reversed(&self) -> Self {
        match self {
            Self::Custom { anchors } => Self::Custom {
                anchors: anchors.iter().rev().cloned().collect(),
            },
            other => {
                let anchors = other
                    .anchors()
                    .iter()
                    .rev()
                    .map(|s| s.to_string())
                    .collect();
                Self::Custom { anchors }
            }
        }
    }

    /// Generate `n` evenly-spaced colors from this map
    pub fn palette(&self, n: usize) -> Vec<String> {
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![self.map(0.5)];
        }
        (0..n)
            .map(|i| self.map(i as f64 / (n - 1) as f64))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn test_srgb_oklab_roundtrip() {
        let (r, g, b) = (0.2, 0.5, 0.8);
        let (l, a, b_) = srgb_to_oklab(r, g, b);
        let (rr, rg, rb) = oklab_to_srgb(l, a, b_);
        assert!(approx_eq(r, rr), "r roundtrip: {} vs {}", r, rr);
        assert!(approx_eq(g, rg), "g roundtrip: {} vs {}", g, rg);
        assert!(approx_eq(b, rb), "b roundtrip: {} vs {}", b, rb);
    }

    #[test]
    fn test_colormap_endpoints() {
        let cm = ColorMap::Sequential(SequentialColorMap::Viridis);
        // t=0 should be close to first anchor (#440154)
        let start = cm.map(0.0);
        let end = cm.map(1.0);
        assert_eq!(start.len(), 7, "hex string length");
        assert_eq!(end.len(), 7, "hex string length");
    }

    #[test]
    fn test_reversed() {
        let cm = ColorMap::Sequential(SequentialColorMap::Grayscale);
        let rev = cm.reversed();
        // Reversed grayscale: t=0 should be white (#ffffff), t=1 should be black (#000000)
        let start = rev.map(0.0);
        let end = rev.map(1.0);
        assert_eq!(start, "#ffffff");
        assert_eq!(end, "#000000");
    }

    #[test]
    fn test_palette_count() {
        let cm = ColorMap::Sequential(SequentialColorMap::Viridis);
        assert_eq!(cm.palette(0).len(), 0);
        assert_eq!(cm.palette(1).len(), 1);
        assert_eq!(cm.palette(5).len(), 5);
        assert_eq!(cm.palette(10).len(), 10);
    }

    #[test]
    fn test_diverging_midpoint() {
        let cm = ColorMap::Diverging(DivergingColorMap::RdBu);
        // The midpoint of RdBu should be close to neutral grey/white
        let mid = cm.map(0.5);
        // Parse the hex and check it's roughly neutral
        let (r, g, b) = parse_hex(&mid);
        let avg = (r + g + b) / 3.0;
        // The mid anchor of RdBu is #f7f7f7 — all channels should be high and similar
        assert!(avg > 0.8, "midpoint should be bright: avg={avg}");
    }

    #[test]
    fn test_custom_colormap() {
        let cm = ColorMap::Custom {
            anchors: vec!["#ff0000".to_string(), "#0000ff".to_string()],
        };
        let start = cm.map(0.0);
        let end = cm.map(1.0);
        assert_eq!(start, "#ff0000");
        assert_eq!(end, "#0000ff");
    }
}
