//! Orthogonal edge router for flowcharts.
//! Computes orthogonal polyline paths with collision avoidance, back-edge bypasses,
//! and smooth rounded corners.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ir::{FlowEdge, FlowchartGraph, NodeId, PortSide};
use super::layout::Rect;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutedEdge {
    pub edge_id: usize,
    pub from: NodeId,
    pub to: NodeId,
    pub points: Vec<[f32; 2]>,
    pub label: Option<String>,
    pub label_pos: [f32; 2],
    pub is_back_edge: bool,
}

impl RoutedEdge {
    /// Render this orthogonal path to an SVG 'd' path string with optional rounded corners.
    pub fn to_svg_path(&self, corner_radius: f32) -> String {
        if self.points.len() < 2 {
            return String::new();
        }

        if corner_radius <= 0.0 || self.points.len() == 2 {
            // Straight lines without rounding
            let mut d = format!("M {:.1} {:.1}", self.points[0][0], self.points[0][1]);
            for pt in &self.points[1..] {
                d.push_str(&format!(" L {:.1} {:.1}", pt[0], pt[1]));
            }
            return d;
        }

        // Rounded corners on 90 degree bends
        let mut d = format!("M {:.1} {:.1}", self.points[0][0], self.points[0][1]);
        let pts = &self.points;

        for i in 1..pts.len() - 1 {
            let prev = pts[i - 1];
            let curr = pts[i];
            let next = pts[i + 1];

            let d1x = curr[0] - prev[0];
            let d1y = curr[1] - prev[1];
            let len1 = (d1x * d1x + d1y * d1y).sqrt().max(0.001);

            let d2x = next[0] - curr[0];
            let d2y = next[1] - curr[1];
            let len2 = (d2x * d2x + d2y * d2y).sqrt().max(0.001);

            let r = corner_radius.min(len1 * 0.45).min(len2 * 0.45);

            // Point before corner
            let p_before = [curr[0] - (d1x / len1) * r, curr[1] - (d1y / len1) * r];
            // Point after corner
            let p_after = [curr[0] + (d2x / len2) * r, curr[1] + (d2y / len2) * r];

            d.push_str(&format!(" L {:.1} {:.1}", p_before[0], p_before[1]));
            d.push_str(&format!(
                " Q {:.1} {:.1} {:.1} {:.1}",
                curr[0], curr[1], p_after[0], p_after[1]
            ));
        }

        let last = pts[pts.len() - 1];
        d.push_str(&format!(" L {:.1} {:.1}", last[0], last[1]));

        d
    }
}

pub struct OrthogonalRouter {
    pub clearance: f32,
    pub corner_radius: f32,
}

impl Default for OrthogonalRouter {
    fn default() -> Self {
        Self {
            clearance: 24.0,
            corner_radius: 8.0,
        }
    }
}

impl OrthogonalRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Route all edges in the graph using the provided node bounding boxes.
    pub fn route_all(
        &self,
        graph: &FlowchartGraph,
        bounds: &HashMap<NodeId, Rect>,
    ) -> Vec<RoutedEdge> {
        let mut routed = Vec::new();

        // Calculate flowchart global bounds for outside margin routing
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        for r in bounds.values() {
            min_x = min_x.min(r.x);
            max_x = max_x.max(r.x + r.width);
        }
        if min_x > max_x {
            min_x = 0.0;
            max_x = 400.0;
        }

        let left_bypass_x = min_x - 36.0;
        let right_bypass_x = max_x + 36.0;

        for edge in &graph.edges {
            if let (Some(from_box), Some(to_box)) = (bounds.get(&edge.from), bounds.get(&edge.to)) {
                let r_edge = self.route_edge(edge, from_box, to_box, left_bypass_x, right_bypass_x);
                routed.push(r_edge);
            }
        }

        routed
    }

    fn route_edge(
        &self,
        edge: &FlowEdge,
        from_box: &Rect,
        to_box: &Rect,
        left_bypass_x: f32,
        right_bypass_x: f32,
    ) -> RoutedEdge {
        let start = from_box.port(edge.from_side);
        let end = to_box.port(edge.to_side);

        let mut points = Vec::new();
        points.push(start);

        let is_upward = end[1] < start[1] - 5.0;

        if edge.is_back_edge || is_upward {
            // Route as a backward bypass along outside margin
            let bypass_x = if edge.from_side == PortSide::Right || edge.to_side == PortSide::Right {
                right_bypass_x
            } else {
                left_bypass_x
            };

            // Exit from source node
            match edge.from_side {
                PortSide::Bottom => {
                    let y_step = start[1] + self.clearance;
                    points.push([start[0], y_step]);
                    points.push([bypass_x, y_step]);
                }
                PortSide::Left | PortSide::Right => {
                    points.push([bypass_x, start[1]]);
                }
                PortSide::Top => {
                    let y_step = start[1] - self.clearance;
                    points.push([start[0], y_step]);
                    points.push([bypass_x, y_step]);
                }
            }

            // Route vertically along margin to target's level
            match edge.to_side {
                PortSide::Left | PortSide::Right => {
                    points.push([bypass_x, end[1]]);
                }
                PortSide::Top => {
                    let y_step = end[1] - self.clearance;
                    points.push([bypass_x, y_step]);
                    points.push([end[0], y_step]);
                }
                PortSide::Bottom => {
                    let y_step = end[1] + self.clearance;
                    points.push([bypass_x, y_step]);
                    points.push([end[0], y_step]);
                }
            }

            points.push(end);
        } else {
            // Forward edge (downward or horizontal)
            match (edge.from_side, edge.to_side) {
                (PortSide::Bottom, PortSide::Top) => {
                    if (start[0] - end[0]).abs() < 2.0 {
                        // Straight vertical drop
                        points.push(end);
                    } else {
                        // Z / S step at midpoint Y
                        let mid_y = (start[1] + end[1]) * 0.5;
                        points.push([start[0], mid_y]);
                        points.push([end[0], mid_y]);
                        points.push(end);
                    }
                }
                (PortSide::Right, PortSide::Top) => {
                    points.push([end[0], start[1]]);
                    points.push(end);
                }
                (PortSide::Left, PortSide::Top) => {
                    points.push([end[0], start[1]]);
                    points.push(end);
                }
                (PortSide::Right, PortSide::Left) => {
                    let mid_x = (start[0] + end[0]) * 0.5;
                    points.push([mid_x, start[1]]);
                    points.push([mid_x, end[1]]);
                    points.push(end);
                }
                (PortSide::Bottom, PortSide::Left) => {
                    let y_step = end[1];
                    points.push([start[0], y_step]);
                    points.push(end);
                }
                (PortSide::Bottom, PortSide::Right) => {
                    let y_step = end[1];
                    points.push([start[0], y_step]);
                    points.push(end);
                }
                _ => {
                    // Generic 2-segment orthogonal fallback
                    points.push([start[0], end[1]]);
                    points.push(end);
                }
            }
        }

        // Deduplicate consecutive identical points
        let mut deduped_points: Vec<[f32; 2]> = Vec::new();
        for pt in points {
            if let Some(last) = deduped_points.last() {
                if (last[0] - pt[0]).abs() < 0.5 && (last[1] - pt[1]).abs() < 0.5 {
                    continue;
                }
            }
            deduped_points.push(pt);
        }

        // Calculate label position near the start, cleanly offset so it never overlaps the source node
        let label_pos = match edge.from_side {
            PortSide::Bottom => [start[0] + 18.0, start[1] + 16.0],
            PortSide::Right => [start[0] + 22.0, start[1] - 12.0],
            PortSide::Left => [start[0] - 22.0, start[1] - 12.0],
            PortSide::Top => [start[0] + 18.0, start[1] - 16.0],
        };

        RoutedEdge {
            edge_id: edge.id,
            from: edge.from,
            to: edge.to,
            points: deduped_points,
            label: edge.label.clone(),
            label_pos,
            is_back_edge: edge.is_back_edge || is_upward,
        }
    }
}
