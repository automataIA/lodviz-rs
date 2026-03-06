//! Marching squares algorithm for contour line extraction
//!
//! Generates iso-level polylines from a 2-D scalar grid.

/// A single iso-level contour: one level value and zero or more polylines
#[derive(Debug, Clone, PartialEq)]
pub struct ContourLine {
    /// The iso-value for this contour
    pub level: f64,
    /// Each inner `Vec<(f64, f64)>` is one connected polyline segment in screen coordinates
    pub paths: Vec<Vec<(f64, f64)>>,
}

/// Run marching squares on `grid` for each level in `levels`.
///
/// * `grid` — row-major 2-D data (`grid[row][col]`)
/// * `levels` — sorted iso-values to extract
/// * `x_range` — (min, max) screen-space x range
/// * `y_range` — (min, max) screen-space y range
///
/// Returns one `ContourLine` per level.
pub fn marching_squares(
    grid: &[Vec<f64>],
    levels: &[f64],
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Vec<ContourLine> {
    let nrows = grid.len();
    if nrows == 0 {
        return levels
            .iter()
            .map(|&l| ContourLine {
                level: l,
                paths: vec![],
            })
            .collect();
    }
    let ncols = grid[0].len();
    if ncols == 0 {
        return levels
            .iter()
            .map(|&l| ContourLine {
                level: l,
                paths: vec![],
            })
            .collect();
    }

    let (x0, x1) = x_range;
    let (y0, y1) = y_range;
    let cell_w = (x1 - x0) / (ncols - 1).max(1) as f64;
    let cell_h = (y1 - y0) / (nrows - 1).max(1) as f64;

    let to_screen = |col: f64, row: f64| -> (f64, f64) { (x0 + col * cell_w, y0 + row * cell_h) };

    levels
        .iter()
        .map(|&level| {
            let mut paths: Vec<Vec<(f64, f64)>> = Vec::new();

            for row in 0..(nrows - 1) {
                for col in 0..(ncols - 1) {
                    let v00 = grid[row][col];
                    let v10 = grid[row][col + 1];
                    let v01 = grid[row + 1][col];
                    let v11 = grid[row + 1][col + 1];

                    // Case index (bit 3..0 = TL TR BL BR, above=1, below=0)
                    let above = |v: f64| if v >= level { 1u8 } else { 0u8 };
                    let case =
                        (above(v00) << 3) | (above(v10) << 2) | (above(v01) << 1) | above(v11);

                    if case == 0 || case == 15 {
                        continue; // all same side — no crossing
                    }

                    // Linear interpolation along an edge
                    let lerp = |va: f64, vb: f64| -> f64 {
                        if (vb - va).abs() < 1e-12 {
                            0.5
                        } else {
                            (level - va) / (vb - va)
                        }
                    };

                    // Edge midpoints in cell-local [0..1] coordinates
                    // Top edge (row, col) → (row, col+1)
                    let top = (col as f64 + lerp(v00, v10), row as f64);
                    // Bottom edge (row+1, col) → (row+1, col+1)
                    let bot = (col as f64 + lerp(v01, v11), row as f64 + 1.0);
                    // Left edge (row, col) → (row+1, col)
                    let left = (col as f64, row as f64 + lerp(v00, v01));
                    // Right edge (row, col+1) → (row+1, col+1)
                    let right = (col as f64 + 1.0, row as f64 + lerp(v10, v11));

                    let to_scr = |(cx, cr): (f64, f64)| to_screen(cx, cr);

                    type Segment = ((f64, f64), (f64, f64));
                    // Lookup table: pairs of edge points to connect
                    let segments: &[Segment] = match case {
                        1 => &[(bot, right)],
                        2 => &[(left, bot)],
                        3 => &[(left, right)],
                        4 => &[(top, right)],
                        5 => &[(top, bot)], // saddle — pick one
                        6 => &[(top, left), (bot, right)],
                        7 => &[(top, left)],
                        8 => &[(top, left)],
                        9 => &[(top, right), (left, bot)],
                        10 => &[(top, bot)],
                        11 => &[(top, right)],
                        12 => &[(left, right)],
                        13 => &[(left, bot)],
                        14 => &[(bot, right)],
                        _ => &[],
                    };

                    for &(a, b) in segments {
                        paths.push(vec![to_scr(a), to_scr(b)]);
                    }
                }
            }

            // Simple chain-stitching to connect adjacent segments
            let paths = stitch_segments(paths);

            ContourLine { level, paths }
        })
        .collect()
}

/// Try to chain short line segments end-to-end into longer polylines.
/// This is a greedy O(n²) approach — fine for typical grid sizes.
fn stitch_segments(mut segments: Vec<Vec<(f64, f64)>>) -> Vec<Vec<(f64, f64)>> {
    let eps = 1e-6_f64;
    let mut result: Vec<Vec<(f64, f64)>> = Vec::new();

    while !segments.is_empty() {
        let mut chain = segments.remove(0);
        let mut changed = true;
        while changed {
            changed = false;
            let mut i = 0;
            while i < segments.len() {
                let seg = &segments[i];
                let chain_end = *chain.last().unwrap();
                let seg_start = seg[0];
                let seg_end = *seg.last().unwrap();
                let chain_start = chain[0];

                let close = |a: (f64, f64), b: (f64, f64)| -> bool {
                    (a.0 - b.0).abs() < eps && (a.1 - b.1).abs() < eps
                };

                if close(chain_end, seg_start) {
                    let s = segments.remove(i);
                    chain.extend_from_slice(&s[1..]);
                    changed = true;
                } else if close(chain_end, seg_end) {
                    let mut s = segments.remove(i);
                    s.reverse();
                    chain.extend_from_slice(&s[1..]);
                    changed = true;
                } else if close(chain_start, seg_end) {
                    let s = segments.remove(i);
                    let mut new_chain = s;
                    new_chain.extend_from_slice(&chain[1..]);
                    chain = new_chain;
                    changed = true;
                } else if close(chain_start, seg_start) {
                    let mut s = segments.remove(i);
                    s.reverse();
                    let mut new_chain = s;
                    new_chain.extend_from_slice(&chain[1..]);
                    chain = new_chain;
                    changed = true;
                } else {
                    i += 1;
                }
            }
        }
        result.push(chain);
    }
    result
}

/// Convert a polyline of screen coordinates to an SVG path `d` attribute string.
pub fn contour_to_svg_path(polyline: &[(f64, f64)]) -> String {
    if polyline.is_empty() {
        return String::new();
    }
    let mut parts = Vec::with_capacity(polyline.len());
    for (i, &(x, y)) in polyline.iter().enumerate() {
        if i == 0 {
            parts.push(format!("M {x:.2} {y:.2}"));
        } else {
            parts.push(format!("L {x:.2} {y:.2}"));
        }
    }
    parts.join(" ")
}

/// Close an open contour polyline by tracing the bounding-box boundary.
///
/// When a contour arc exits the grid, SVG auto-close draws a diagonal line
/// that creates visual artifacts.  This function inserts the necessary boundary
/// corner points so the filled region is properly closed with straight edges
/// along the domain boundary.
///
/// The direction (CW or CCW) is chosen as the shorter arc around the perimeter,
/// which always encloses the smaller (correct) iso-level region.
///
/// Returns the path unchanged when:
/// - the path is already closed (start ≈ end),
/// - either endpoint is not on the boundary.
pub fn close_open_path_at_boundary(
    path: &[(f64, f64)],
    x_range: (f64, f64),
    y_range: (f64, f64),
) -> Vec<(f64, f64)> {
    if path.len() < 2 {
        return path.to_vec();
    }
    let (x0, x1) = x_range;
    let (y0, y1) = y_range;
    let eps = 0.5_f64;

    let start = path[0];
    let end = *path.last().unwrap();

    if (start.0 - end.0).abs() < eps && (start.1 - end.1).abs() < eps {
        return path.to_vec();
    }

    // Boundary parameterisation (CW from top-left = 0.0):
    // [0,1) top left→right  [1,2) right top→bottom  [2,3) bottom right→left  [3,4) left bottom→top
    let w = (x1 - x0).max(1e-12);
    let h = (y1 - y0).max(1e-12);

    let on_boundary = |p: (f64, f64)| -> Option<f64> {
        if (p.1 - y0).abs() <= eps && p.0 >= x0 - eps && p.0 <= x1 + eps {
            Some(((p.0 - x0) / w).clamp(0.0, 1.0 - 1e-9))
        } else if (p.0 - x1).abs() <= eps && p.1 >= y0 - eps && p.1 <= y1 + eps {
            Some(1.0 + ((p.1 - y0) / h).clamp(0.0, 1.0 - 1e-9))
        } else if (p.1 - y1).abs() <= eps && p.0 >= x0 - eps && p.0 <= x1 + eps {
            Some(2.0 + ((x1 - p.0) / w).clamp(0.0, 1.0 - 1e-9))
        } else if (p.0 - x0).abs() <= eps && p.1 >= y0 - eps && p.1 <= y1 + eps {
            Some(3.0 + ((y1 - p.1) / h).clamp(0.0, 1.0 - 1e-9))
        } else {
            None
        }
    };

    let t_end = match on_boundary(end) {
        Some(t) => t,
        None => return path.to_vec(),
    };
    let t_start = match on_boundary(start) {
        Some(t) => t,
        None => return path.to_vec(),
    };

    let boundary_pt = |t: f64| -> (f64, f64) {
        let t = t.rem_euclid(4.0);
        if t < 1.0 {
            (x0 + t * w, y0)
        } else if t < 2.0 {
            (x1, y0 + (t - 1.0) * h)
        } else if t < 3.0 {
            (x1 - (t - 2.0) * w, y1)
        } else {
            (x0, y1 - (t - 3.0) * h)
        }
    };

    // Choose the shorter arc: CW (increasing t) or CCW (decreasing t)
    let arc_cw = (t_start - t_end).rem_euclid(4.0);
    let arc_ccw = (t_end - t_start).rem_euclid(4.0);
    let go_cw = arc_cw <= arc_ccw;

    let corner_ts: [f64; 4] = [0.0, 1.0, 2.0, 3.0];
    let mut closing: Vec<(f64, f64)> = Vec::new();

    if go_cw {
        let t_target = t_end + arc_cw;
        for &c in &corner_ts {
            for &ca in &[c, c + 4.0] {
                if ca > t_end + 1e-9 && ca < t_target - 1e-9 {
                    closing.push(boundary_pt(ca));
                    break;
                }
            }
        }
    } else {
        let t_target = t_end - arc_ccw;
        for &c in corner_ts.iter().rev() {
            for &ca in &[c, c - 4.0] {
                if ca < t_end - 1e-9 && ca > t_target + 1e-9 {
                    closing.push(boundary_pt(ca));
                    break;
                }
            }
        }
    }

    let mut result = path.to_vec();
    result.extend(closing);
    result.push(start);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_field_no_contours() {
        // A uniform field has no crossings — contours should be empty
        let grid: Vec<Vec<f64>> = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let result = marching_squares(&grid, &[0.5], (0.0, 100.0), (0.0, 100.0));
        assert_eq!(result.len(), 1);
        assert!(
            result[0].paths.is_empty(),
            "flat field above level should produce no segments"
        );
    }

    #[test]
    fn test_simple_gradient() {
        // A simple left-right gradient from 0 to 1 should produce a vertical crossing
        let grid: Vec<Vec<f64>> = vec![vec![0.0, 1.0], vec![0.0, 1.0]];
        let result = marching_squares(&grid, &[0.5], (0.0, 10.0), (0.0, 10.0));
        assert_eq!(result.len(), 1);
        // Should have at least one segment
        assert!(
            !result[0].paths.is_empty(),
            "gradient should produce at least one contour segment"
        );
    }

    #[test]
    fn test_contour_to_svg_path_empty() {
        assert_eq!(contour_to_svg_path(&[]), "");
    }

    #[test]
    fn test_contour_to_svg_path_two_points() {
        let path = contour_to_svg_path(&[(0.0, 0.0), (10.0, 5.0)]);
        assert!(path.starts_with("M "));
        assert!(path.contains('L'));
    }

    #[test]
    fn test_close_already_closed() {
        let path = vec![(5.0, 0.0), (8.0, 3.0), (5.0, 0.0)];
        let result = close_open_path_at_boundary(&path, (0.0, 10.0), (0.0, 10.0));
        assert_eq!(result, path, "closed path should be returned unchanged");
    }

    #[test]
    fn test_close_same_top_boundary() {
        // Arc from top-left to top-right: close should add no corners
        let path = vec![(2.0, 0.0), (5.0, 3.0), (8.0, 0.0)];
        let result = close_open_path_at_boundary(&path, (0.0, 10.0), (0.0, 10.0));
        // Should close back to start without extra corners
        assert_eq!(*result.last().unwrap(), path[0]);
        assert_eq!(result.len(), path.len() + 1);
    }

    #[test]
    fn test_close_top_to_right_boundary() {
        // Arc from top boundary to right boundary — must add top-right corner
        let path = vec![(4.0, 0.0), (8.0, 2.0), (10.0, 4.0)];
        let result = close_open_path_at_boundary(&path, (0.0, 10.0), (0.0, 10.0));
        // top-right corner (10, 0) must appear before the closing start point
        assert!(
            result.contains(&(10.0, 0.0)),
            "top-right corner must be inserted"
        );
        assert_eq!(*result.last().unwrap(), path[0]);
    }

    #[test]
    fn test_close_not_on_boundary_unchanged() {
        // Interior endpoint — cannot close, returned unchanged
        let path = vec![(1.0, 0.0), (5.0, 5.0), (9.0, 3.0)];
        let result = close_open_path_at_boundary(&path, (0.0, 10.0), (0.0, 10.0));
        assert_eq!(result, path);
    }
}
