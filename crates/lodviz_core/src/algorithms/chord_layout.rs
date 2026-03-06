/// Chord diagram layout algorithm
///
/// Distributes arcs around a circle proportional to row totals,
/// then generates SVG arc + quadratic Bézier ribbon paths.
use std::f64::consts::TAU;

/// A rendered arc segment for one group in the chord diagram
#[derive(Debug, Clone, PartialEq)]
pub struct ChordArc {
    /// Start angle in radians (0 = top, clockwise)
    pub start_angle: f64,
    /// End angle in radians
    pub end_angle: f64,
    /// Index of the group in the original matrix
    pub index: usize,
    /// SVG path for the arc (thick band)
    pub path: String,
    /// Display label
    pub label: String,
    /// Fill color (hex string)
    pub color: String,
}

/// A rendered ribbon between two chord groups
#[derive(Debug, Clone, PartialEq)]
pub struct ChordRibbon {
    /// SVG path (quadratic Bézier from source arc to target arc)
    pub path: String,
    /// Index of the source group
    pub source: usize,
    /// Index of the target group
    pub target: usize,
    /// Flow value (matrix[source][target])
    pub value: f64,
    /// Fill color (hex string, matches source arc)
    pub color: String,
}

/// Complete layout output for a chord diagram
#[derive(Debug, Clone, PartialEq)]
pub struct ChordLayoutResult {
    /// Outer arc bands (one per group)
    pub arcs: Vec<ChordArc>,
    /// Inner ribbons (one per non-zero matrix cell)
    pub ribbons: Vec<ChordRibbon>,
}

/// Default palette
const DEFAULT_PALETTE: &[&str] = &[
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
    "#9c755f", "#bab0ac",
];

/// Build an SVG arc path for a thick band between `inner_r` and `outer_r`.
fn arc_path(cx: f64, cy: f64, inner_r: f64, outer_r: f64, start: f64, end: f64) -> String {
    let (sx_o, sy_o) = polar(cx, cy, outer_r, start);
    let (ex_o, ey_o) = polar(cx, cy, outer_r, end);
    let (sx_i, sy_i) = polar(cx, cy, inner_r, end);
    let (ex_i, ey_i) = polar(cx, cy, inner_r, start);
    let large = if end - start > std::f64::consts::PI {
        1
    } else {
        0
    };
    format!(
        "M {sx_o:.2} {sy_o:.2} A {outer_r:.2} {outer_r:.2} 0 {large} 1 {ex_o:.2} {ey_o:.2} \
         L {sx_i:.2} {sy_i:.2} A {inner_r:.2} {inner_r:.2} 0 {large} 0 {ex_i:.2} {ey_i:.2} Z"
    )
}

/// Build an SVG ribbon path (quadratic Bézier) between two arcs.
fn ribbon_path(
    cx: f64,
    cy: f64,
    r: f64,
    src_start: f64,
    src_end: f64,
    dst_start: f64,
    dst_end: f64,
) -> String {
    let src_mid = (src_start + src_end) / 2.0;
    let dst_mid = (dst_start + dst_end) / 2.0;
    let (x0, y0) = polar(cx, cy, r, src_mid);
    let (x1, y1) = polar(cx, cy, r, dst_mid);
    format!(
        "M {x0:.2} {y0:.2} Q {cx:.2} {cy:.2} {x1:.2} {y1:.2} Q {cx:.2} {cy:.2} {x0:.2} {y0:.2} Z"
    )
}

fn polar(cx: f64, cy: f64, r: f64, angle: f64) -> (f64, f64) {
    // angle=0 → top, clockwise (standard SVG convention)
    (cx + r * angle.sin(), cy - r * angle.cos())
}

/// Compute a Chord diagram layout.
///
/// * `matrix` — square adjacency matrix (`matrix[i][j]` = flow from i to j)
/// * `labels` — label for each group
/// * `colors` — optional per-group colors; falls back to `DEFAULT_PALETTE`
/// * `gap_degrees` — gap in degrees between adjacent arcs
/// * `radius` — outer radius of the arc bands (in SVG units)
/// * `inner_radius` — inner radius of the arc bands
pub fn layout_chord(
    matrix: &[Vec<f64>],
    labels: &[String],
    colors: Option<&[String]>,
    gap_degrees: f64,
    radius: f64,
    inner_radius: f64,
) -> ChordLayoutResult {
    let n = matrix.len();
    if n == 0 {
        return ChordLayoutResult {
            arcs: vec![],
            ribbons: vec![],
        };
    }

    let cx = radius;
    let cy = radius;
    let gap_rad = gap_degrees.to_radians();
    let total_gap = gap_rad * n as f64;
    let available = TAU - total_gap;

    // Row totals drive arc lengths
    let row_totals: Vec<f64> = matrix.iter().map(|row| row.iter().sum::<f64>()).collect();
    let grand_total: f64 = row_totals.iter().sum();
    if grand_total <= 0.0 {
        return ChordLayoutResult {
            arcs: vec![],
            ribbons: vec![],
        };
    }

    // Assign start/end angles for each group arc
    let mut arc_starts = vec![0.0_f64; n];
    let mut arc_ends = vec![0.0_f64; n];
    let mut cursor = 0.0_f64;
    for i in 0..n {
        let span = row_totals[i] / grand_total * available;
        arc_starts[i] = cursor;
        arc_ends[i] = cursor + span;
        cursor += span + gap_rad;
    }

    // Build arcs
    let arcs: Vec<ChordArc> = (0..n)
        .map(|i| {
            let color = colors
                .and_then(|c| c.get(i))
                .map(|s| s.as_str())
                .unwrap_or(DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()])
                .to_string();
            let path = arc_path(cx, cy, inner_radius, radius, arc_starts[i], arc_ends[i]);
            ChordArc {
                start_angle: arc_starts[i],
                end_angle: arc_ends[i],
                index: i,
                path,
                label: labels.get(i).cloned().unwrap_or_default(),
                color,
            }
        })
        .collect();

    // Build ribbons — track offset within each arc for stacking
    let mut src_offset = vec![0.0_f64; n];
    let mut dst_offset = vec![0.0_f64; n];
    let mut ribbons: Vec<ChordRibbon> = Vec::new();

    for i in 0..n {
        for j in 0..n {
            let v = matrix[i].get(j).copied().unwrap_or(0.0);
            if v <= 0.0 || j >= matrix[j].len() {
                continue;
            }

            let src_span = arc_ends[i] - arc_starts[i];
            let dst_span = arc_ends[j] - arc_starts[j];

            let src_frac = v / grand_total * available / src_span.max(1e-9) * src_span;
            let dst_frac = v / grand_total * available / dst_span.max(1e-9) * dst_span;

            let ss = arc_starts[i] + src_offset[i];
            let se = ss + src_frac.min(src_span - src_offset[i]);
            let ds = arc_starts[j] + dst_offset[j];
            let de = ds + dst_frac.min(dst_span - dst_offset[j]);

            src_offset[i] += src_frac.min(src_span - src_offset[i]);
            dst_offset[j] += dst_frac.min(dst_span - dst_offset[j]);

            let color = arcs[i].color.clone();
            let path = ribbon_path(cx, cy, inner_radius, ss, se, ds, de);
            ribbons.push(ChordRibbon {
                path,
                source: i,
                target: j,
                value: v,
                color,
            });
        }
    }

    ChordLayoutResult { arcs, ribbons }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    fn make_matrix() -> Vec<Vec<f64>> {
        vec![
            vec![0.0, 10.0, 5.0],
            vec![8.0, 0.0, 12.0],
            vec![3.0, 7.0, 0.0],
        ]
    }

    #[test]
    fn test_arc_count() {
        let m = make_matrix();
        let labels: Vec<String> = ["A", "B", "C"].iter().map(|s| s.to_string()).collect();
        let result = layout_chord(&m, &labels, None, 2.0, 100.0, 80.0);
        assert_eq!(result.arcs.len(), 3);
    }

    #[test]
    fn test_arc_sum() {
        let m = make_matrix();
        let labels: Vec<String> = ["A", "B", "C"].iter().map(|s| s.to_string()).collect();
        let gap = 2.0_f64;
        let result = layout_chord(&m, &labels, None, gap, 100.0, 80.0);
        let total_arc: f64 = result
            .arcs
            .iter()
            .map(|a| a.end_angle - a.start_angle)
            .sum();
        let total_gap = gap.to_radians() * 3.0;
        let expected = TAU - total_gap;
        assert!(
            (total_arc - expected).abs() < 1e-6,
            "arc sum {total_arc:.6} ≠ expected {expected:.6}"
        );
    }

    #[test]
    fn test_empty_chord() {
        let result = layout_chord(&[], &[], None, 2.0, 100.0, 80.0);
        assert!(result.arcs.is_empty());
        assert!(result.ribbons.is_empty());
    }
}
