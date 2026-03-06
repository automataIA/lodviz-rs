//! Beeswarm / strip layout algorithms
//!
//! Computes per-point x-offsets within a categorical band so that points
//! spread out rather than overlap.

/// How to distribute points within a band
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StripLayout {
    /// Pseudo-random jitter based on value and index (deterministic, no RNG)
    Jitter,
    /// Greedy BFS beeswarm — points sorted by value and placed to avoid overlap
    Beeswarm,
    /// No offset — all points are centered on the axis
    Center,
}

/// Compute an x-offset for each point in a band.
///
/// * `values` — the data values (used for sorting/determinism)
/// * `layout` — which layout algorithm to use
/// * `point_radius` — radius of each point in screen units
/// * `band_width` — total width available for spreading (half on each side)
///
/// Returns a `Vec<f64>` of the same length as `values`, with each value
/// being the offset in screen units along the spread axis.
pub fn beeswarm_layout(
    values: &[f64],
    layout: StripLayout,
    point_radius: f64,
    band_width: f64,
) -> Vec<f64> {
    match layout {
        StripLayout::Center => vec![0.0; values.len()],
        StripLayout::Jitter => jitter_layout(values, band_width),
        StripLayout::Beeswarm => beeswarm_greedy(values, point_radius, band_width),
    }
}

fn jitter_layout(values: &[f64], band_width: f64) -> Vec<f64> {
    let max_offset = band_width * 0.4;
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            // Deterministic pseudo-random using trigonometric hash
            let raw = (i as f64 * 7.3 + v * 13.7).sin();
            raw * max_offset
        })
        .collect()
}

fn beeswarm_greedy(values: &[f64], point_radius: f64, band_width: f64) -> Vec<f64> {
    let max_offset = band_width * 0.45;
    let diameter = point_radius * 2.2; // slight gap

    let n = values.len();
    let mut offsets = vec![0.0_f64; n];

    // Sort indices by value to place in order
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        values[a]
            .partial_cmp(&values[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Placed points: (offset, value_in_band_space)
    // For simplicity we treat value as the "position along the value axis"
    // and offset as the "position along the spread axis".
    // We just need that two placed points don't overlap in 2D.
    // Since the value axis is determined externally, we only need to check
    // the offset axis — points with the same value would overlap unless offset differs.
    // We keep a list of (value, offset) and test circle-circle overlap.
    let mut placed: Vec<(f64, f64)> = Vec::with_capacity(n);

    for &idx in &order {
        let val = values[idx];
        let mut best_offset = 0.0_f64;
        let mut found = false;

        // Try offsets outward from center: 0, ±1·diameter, ±2·diameter …
        'search: for step in 0..=64 {
            for sign in &[1.0_f64, -1.0] {
                let candidate = if step == 0 {
                    0.0
                } else {
                    sign * step as f64 * diameter
                };
                if candidate.abs() > max_offset {
                    continue;
                }
                let mut ok = true;
                for &(pv, po) in &placed {
                    let dv = val - pv;
                    let do_ = candidate - po;
                    let dist2 = dv * dv + do_ * do_;
                    if dist2 < diameter * diameter {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    best_offset = candidate;
                    found = true;
                    break 'search;
                }
                if step == 0 {
                    break; // step==0 is symmetric, only try once
                }
            }
        }

        if !found {
            best_offset = best_offset.clamp(-max_offset, max_offset);
        }

        offsets[idx] = best_offset;
        placed.push((val, best_offset));
    }

    offsets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_all_zero() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let offsets = beeswarm_layout(&values, StripLayout::Center, 4.0, 40.0);
        for o in offsets {
            assert_eq!(o, 0.0);
        }
    }

    #[test]
    fn test_jitter_bounds() {
        let values: Vec<f64> = (0..50).map(|i| i as f64 * 0.5).collect();
        let band_width = 40.0;
        let offsets = beeswarm_layout(&values, StripLayout::Jitter, 4.0, band_width);
        for o in &offsets {
            assert!(
                o.abs() <= band_width / 2.0,
                "jitter offset {o} exceeds band_width/2"
            );
        }
    }

    #[test]
    fn test_beeswarm_no_overlap() {
        let values: Vec<f64> = vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0];
        let point_radius = 4.0;
        let offsets = beeswarm_layout(&values, StripLayout::Beeswarm, point_radius, 80.0);
        let diameter = point_radius * 2.2;
        // Check that no two points with the same value overlap in the offset axis
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                let dv = values[i] - values[j];
                let do_ = offsets[i] - offsets[j];
                let dist = (dv * dv + do_ * do_).sqrt();
                // Allow tiny numerical tolerance
                assert!(
                    dist >= diameter - 0.01,
                    "points {i} and {j} overlap: dist={dist:.3} < diameter={diameter}"
                );
            }
        }
    }
}
