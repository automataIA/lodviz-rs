/// Sankey diagram layout algorithm
///
/// Assigns column positions to nodes via BFS from source nodes,
/// stacks nodes vertically within each column proportional to flow,
/// and generates SVG cubic Bézier ribbon paths.
use crate::core::data::SankeyData;

/// Screen rectangle for a Sankey node
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyNodeRect {
    /// Left edge x coordinate
    pub x: f64,
    /// Top edge y coordinate
    pub y: f64,
    /// Width of the node rectangle (= `node_width` parameter)
    pub width: f64,
    /// Height of the node rectangle (proportional to total flow)
    pub height: f64,
    /// Index of the node in the original `SankeyData::nodes`
    pub index: usize,
    /// Display label (copied from `SankeyNode::label`)
    pub label: String,
    /// Fill color for this node (hex string)
    pub color: String,
}

/// A rendered ribbon between two Sankey nodes
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyRibbon {
    /// SVG path data (cubic Bézier)
    pub path: String,
    /// Index of the source node
    pub source: usize,
    /// Index of the target node
    pub target: usize,
    /// Flow value (used for tooltip)
    pub value: f64,
    /// Fill color (hex string)
    pub color: String,
}

/// Output of the Sankey layout computation
#[derive(Debug, Clone, PartialEq)]
pub struct SankeyLayoutResult {
    /// Positioned node rectangles
    pub nodes: Vec<SankeyNodeRect>,
    /// Ribbon paths for each link
    pub links: Vec<SankeyRibbon>,
}

/// Default palette used when a node has no explicit color
const DEFAULT_PALETTE: &[&str] = &[
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
    "#9c755f", "#bab0ac",
];

/// Compute a full Sankey layout.
///
/// * `data` — nodes and directed links
/// * `width` — total SVG width available
/// * `height` — total SVG height available
/// * `node_width` — pixel width of each node rectangle
/// * `node_gap` — minimum gap between node rectangles in the same column
pub fn layout_sankey(
    data: &SankeyData,
    width: f64,
    height: f64,
    node_width: f64,
    node_gap: f64,
) -> SankeyLayoutResult {
    let n = data.nodes.len();
    if n == 0 {
        return SankeyLayoutResult {
            nodes: vec![],
            links: vec![],
        };
    }

    // -----------------------------------------------------------------------
    // 1. Assign column (depth) to each node via BFS from source nodes
    // -----------------------------------------------------------------------
    let mut depth = vec![usize::MAX; n];
    let has_incoming: Vec<bool> = {
        let mut v = vec![false; n];
        for link in &data.links {
            if link.target < n {
                v[link.target] = true;
            }
        }
        v
    };

    // Initialize sources (nodes with no incoming links)
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for i in 0..n {
        if !has_incoming[i] {
            depth[i] = 0;
            queue.push_back(i);
        }
    }

    // BFS to propagate depths
    while let Some(node) = queue.pop_front() {
        let d = depth[node];
        for link in &data.links {
            if link.source == node && link.target < n && depth[link.target] == usize::MAX {
                depth[link.target] = d + 1;
                queue.push_back(link.target);
            }
        }
    }

    // Clamp any unreachable nodes
    for d in &mut depth {
        if *d == usize::MAX {
            *d = 0;
        }
    }

    let max_depth = depth.iter().copied().max().unwrap_or(0);
    let num_cols = max_depth + 1;

    // -----------------------------------------------------------------------
    // 2. Compute node total flow for heights
    // -----------------------------------------------------------------------
    let mut flow_in = vec![0.0_f64; n];
    let mut flow_out = vec![0.0_f64; n];
    for link in &data.links {
        if link.source < n {
            flow_out[link.source] += link.value;
        }
        if link.target < n {
            flow_in[link.target] += link.value;
        }
    }
    let node_flow: Vec<f64> = (0..n)
        .map(|i| flow_in[i].max(flow_out[i]).max(1.0))
        .collect();

    // Per-column: total flow to scale heights
    let mut col_nodes: Vec<Vec<usize>> = vec![vec![]; num_cols];
    for i in 0..n {
        col_nodes[depth[i]].push(i);
    }

    // Compute column x positions
    let col_x: Vec<f64> = if num_cols <= 1 {
        vec![0.0]
    } else {
        let step = (width - node_width) / (num_cols - 1) as f64;
        (0..num_cols).map(|c| c as f64 * step).collect()
    };

    // -----------------------------------------------------------------------
    // 3. Stack nodes vertically within each column
    // -----------------------------------------------------------------------
    let usable_height = height - node_gap * (n as f64); // rough guard

    let mut node_rects: Vec<SankeyNodeRect> = vec![
        SankeyNodeRect {
            x: 0.0,
            y: 0.0,
            width: node_width,
            height: 0.0,
            index: 0,
            label: String::new(),
            color: String::new(),
        };
        n
    ];

    for (col, nodes_in_col) in col_nodes.iter().enumerate() {
        if nodes_in_col.is_empty() {
            continue;
        }
        let total_flow: f64 = nodes_in_col.iter().map(|&i| node_flow[i]).sum();
        let total_gaps = node_gap * (nodes_in_col.len() as f64 + 1.0);
        let available = (usable_height - total_gaps).max(10.0 * nodes_in_col.len() as f64);

        let mut y_cursor = node_gap;
        for &node_idx in nodes_in_col {
            let h = (node_flow[node_idx] / total_flow * available).max(4.0);
            let color = data.nodes[node_idx]
                .color
                .clone()
                .unwrap_or_else(|| DEFAULT_PALETTE[node_idx % DEFAULT_PALETTE.len()].to_string());
            node_rects[node_idx] = SankeyNodeRect {
                x: col_x[col],
                y: y_cursor,
                width: node_width,
                height: h,
                index: node_idx,
                label: data.nodes[node_idx].label.clone(),
                color,
            };
            y_cursor += h + node_gap;
        }
    }

    // -----------------------------------------------------------------------
    // 4. Generate ribbon paths (cubic Bézier)
    // -----------------------------------------------------------------------
    // Track used offsets per node edge (for stacking ribbons)
    let mut src_y_used = vec![0.0_f64; n];
    let mut dst_y_used = vec![0.0_f64; n];

    // Sort links by source then target for deterministic stacking
    let mut link_order: Vec<usize> = (0..data.links.len()).collect();
    link_order.sort_by_key(|&i| (data.links[i].source, data.links[i].target));

    let mut ribbons: Vec<SankeyRibbon> = Vec::with_capacity(data.links.len());

    for &li in &link_order {
        let link = &data.links[li];
        if link.source >= n || link.target >= n {
            continue;
        }
        let src = &node_rects[link.source];
        let dst = &node_rects[link.target];

        let src_flow = flow_out[link.source].max(1.0);
        let dst_flow = flow_in[link.target].max(1.0);

        let ribbon_h_src = (link.value / src_flow * src.height).max(1.0);
        let ribbon_h_dst = (link.value / dst_flow * dst.height).max(1.0);

        let y0_top = src.y + src_y_used[link.source];
        let y0_bot = y0_top + ribbon_h_src;
        let y1_top = dst.y + dst_y_used[link.target];
        let y1_bot = y1_top + ribbon_h_dst;

        src_y_used[link.source] += ribbon_h_src;
        dst_y_used[link.target] += ribbon_h_dst;

        let x0 = src.x + src.width;
        let x1 = dst.x;
        let cx = (x0 + x1) / 2.0;

        // Cubic Bézier ribbon path
        let path = format!(
            "M {x0:.2} {y0_top:.2} C {cx:.2} {y0_top:.2}, {cx:.2} {y1_top:.2}, {x1:.2} {y1_top:.2} \
             L {x1:.2} {y1_bot:.2} C {cx:.2} {y1_bot:.2}, {cx:.2} {y0_bot:.2}, {x0:.2} {y0_bot:.2} Z"
        );

        let color = link
            .color
            .clone()
            .unwrap_or_else(|| node_rects[link.source].color.clone());

        ribbons.push(SankeyRibbon {
            path,
            source: link.source,
            target: link.target,
            value: link.value,
            color,
        });
    }

    SankeyLayoutResult {
        nodes: node_rects,
        links: ribbons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::data::{SankeyLink, SankeyNode};

    fn make_simple() -> SankeyData {
        SankeyData {
            nodes: vec![
                SankeyNode {
                    label: "A".into(),
                    color: None,
                },
                SankeyNode {
                    label: "B".into(),
                    color: None,
                },
                SankeyNode {
                    label: "C".into(),
                    color: None,
                },
            ],
            links: vec![
                SankeyLink {
                    source: 0,
                    target: 1,
                    value: 10.0,
                    color: None,
                },
                SankeyLink {
                    source: 0,
                    target: 2,
                    value: 20.0,
                    color: None,
                },
            ],
        }
    }

    #[test]
    fn test_layout_produces_correct_node_count() {
        let data = make_simple();
        let result = layout_sankey(&data, 400.0, 300.0, 20.0, 8.0);
        assert_eq!(result.nodes.len(), 3);
        assert_eq!(result.links.len(), 2);
    }

    #[test]
    fn test_conservation() {
        // For node B: total in flow should equal total link values targeting B
        let data = make_simple();
        let result = layout_sankey(&data, 400.0, 300.0, 20.0, 8.0);
        // Node 1 (B) has a single incoming link with value 10.0
        let link_to_b: f64 = data
            .links
            .iter()
            .filter(|l| l.target == 1)
            .map(|l| l.value)
            .sum();
        let ribbon_to_b: f64 = result
            .links
            .iter()
            .filter(|r| r.target == 1)
            .map(|r| r.value)
            .sum();
        assert_eq!(
            link_to_b, ribbon_to_b,
            "ribbon values should match link values"
        );
    }

    #[test]
    fn test_empty_sankey() {
        let data = SankeyData::default();
        let result = layout_sankey(&data, 400.0, 300.0, 20.0, 8.0);
        assert!(result.nodes.is_empty());
        assert!(result.links.is_empty());
    }
}
