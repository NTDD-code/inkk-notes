//! Automatic layout engine for flowcharts.
//! Computes optimal 2D node placement using layered DAG hierarchy with branching offsets,
//! while respecting user pinned positions and swimlanes.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use super::ir::{FlowchartGraph, NodeId, PortSide};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn center(&self) -> [f32; 2] {
        [self.x + self.width * 0.5, self.y + self.height * 0.5]
    }

    pub fn port(&self, side: PortSide) -> [f32; 2] {
        match side {
            PortSide::Top => [self.x + self.width * 0.5, self.y],
            PortSide::Bottom => [self.x + self.width * 0.5, self.y + self.height],
            PortSide::Left => [self.x, self.y + self.height * 0.5],
            PortSide::Right => [self.x + self.width, self.y + self.height * 0.5],
        }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    pub node_bounds: HashMap<NodeId, Rect>,
    pub total_width: f32,
    pub total_height: f32,
}

pub struct LayoutEngine {
    pub layer_gap: f32,
    pub node_gap: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            layer_gap: 55.0,
            node_gap: 45.0,
            origin_x: 60.0,
            origin_y: 60.0,
        }
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute 2D node coordinates for all nodes in the flowchart.
    pub fn compute_layout(&self, graph: &FlowchartGraph) -> LayoutResult {
        if graph.nodes.is_empty() {
            return LayoutResult {
                node_bounds: HashMap::new(),
                total_width: 0.0,
                total_height: 0.0,
            };
        }

        // 1. Identify back edges using DFS to obtain a directed acyclic view
        let back_edges = self.detect_back_edges(graph);

        // 2. Assign ranks (layers) using topological longest-path on forward edges
        let ranks = self.assign_ranks(graph, &back_edges);

        // Group node IDs by layer rank
        let mut max_rank = 0usize;
        for &r in ranks.values() {
            if r > max_rank {
                max_rank = r;
            }
        }

        let mut layers: Vec<Vec<NodeId>> = vec![Vec::new(); max_rank + 1];
        for node in &graph.nodes {
            let r = *ranks.get(&node.id).unwrap_or(&0);
            layers[r].push(node.id);
        }

        // Sort nodes in each layer to minimize edge crossings (barycenter heuristic)
        self.order_nodes_in_layers(graph, &mut layers, &back_edges);

        // 3. Compute X and Y positions
        let mut bounds = HashMap::new();
        let mut current_y = self.origin_y;

        // Compute layer heights
        for layer in &layers {
            if layer.is_empty() {
                continue;
            }

            let layer_height = layer
                .iter()
                .filter_map(|id| graph.node(*id))
                .map(|n| n.height)
                .fold(0.0f32, f32::max);

            // Compute total width of layer
            let mut total_layer_w = 0.0f32;
            for (idx, &id) in layer.iter().enumerate() {
                if let Some(n) = graph.node(id) {
                    total_layer_w += n.width;
                    if idx + 1 < layer.len() {
                        total_layer_w += self.node_gap;
                    }
                }
            }

            // Center-align layer along X axis
            let base_center_x = self.origin_x + 300.0;
            let mut current_x = base_center_x - (total_layer_w * 0.5);

            for &id in layer {
                if let Some(node) = graph.node(id) {
                    let rect = if let Some(pin) = node.pinned_pos {
                        Rect::new(pin.0, pin.1, node.width, node.height)
                    } else {
                        // Align vertically in the center of the layer
                        let y = current_y + (layer_height - node.height) * 0.5;
                        Rect::new(current_x, y, node.width, node.height)
                    };

                    bounds.insert(id, rect);
                    current_x += node.width + self.node_gap;
                }
            }

            current_y += layer_height + self.layer_gap;
        }

        // Calculate bounding box across all nodes
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for r in bounds.values() {
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.x + r.width);
            max_y = max_y.max(r.y + r.height);
        }

        let total_w = if max_x >= min_x {
            max_x - min_x + self.origin_x * 2.0
        } else {
            0.0
        };
        let total_h = if max_y >= min_y {
            max_y - min_y + self.origin_y * 2.0
        } else {
            0.0
        };

        LayoutResult {
            node_bounds: bounds,
            total_width: total_w,
            total_height: total_h,
        }
    }

    /// Detect cycle back-edges to permit acyclic layering.
    fn detect_back_edges(&self, graph: &FlowchartGraph) -> HashSet<usize> {
        let mut back_edges = HashSet::new();

        // Include edges already explicitly flagged as back edges
        for edge in &graph.edges {
            if edge.is_back_edge {
                back_edges.insert(edge.id);
            }
        }

        // Also run standard DFS cycle detection
        let mut visited = HashSet::new();
        let mut on_path = HashSet::new();

        fn dfs(
            curr: NodeId,
            graph: &FlowchartGraph,
            visited: &mut HashSet<NodeId>,
            on_path: &mut HashSet<NodeId>,
            back_edges: &mut HashSet<usize>,
        ) {
            visited.insert(curr);
            on_path.insert(curr);

            for edge in graph.outgoing_edges(curr) {
                if on_path.contains(&edge.to) {
                    back_edges.insert(edge.id);
                } else if !visited.contains(&edge.to) {
                    dfs(edge.to, graph, visited, on_path, back_edges);
                }
            }

            on_path.remove(&curr);
        }

        for node in &graph.nodes {
            if !visited.contains(&node.id) {
                dfs(node.id, graph, &mut visited, &mut on_path, &mut back_edges);
            }
        }

        back_edges
    }

    /// Assign integer ranks using longest path DAG traversal from root nodes.
    fn assign_ranks(
        &self,
        graph: &FlowchartGraph,
        back_edges: &HashSet<usize>,
    ) -> HashMap<NodeId, usize> {
        let mut ranks: HashMap<NodeId, usize> = HashMap::new();
        for node in &graph.nodes {
            ranks.insert(node.id, 0);
        }

        // Topological ordering (Kahn's algorithm on forward edges)
        let mut in_degrees: HashMap<NodeId, usize> = HashMap::new();
        for node in &graph.nodes {
            in_degrees.insert(node.id, 0);
        }

        for edge in &graph.edges {
            if !back_edges.contains(&edge.id) {
                *in_degrees.entry(edge.to).or_insert(0) += 1;
            }
        }

        let mut queue = VecDeque::new();
        for node in &graph.nodes {
            if in_degrees[&node.id] == 0 {
                queue.push_back(node.id);
            }
        }

        while let Some(u) = queue.pop_front() {
            let u_rank = ranks[&u];
            for edge in graph.outgoing_edges(u) {
                if !back_edges.contains(&edge.id) {
                    let v = edge.to;
                    let v_rank = ranks.get_mut(&v).unwrap();
                    *v_rank = (*v_rank).max(u_rank + 1);

                    let deg = in_degrees.get_mut(&v).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(v);
                    }
                }
            }
        }

        ranks
    }

    /// Order nodes within each layer using barycenter heuristic.
    fn order_nodes_in_layers(
        &self,
        graph: &FlowchartGraph,
        layers: &mut [Vec<NodeId>],
        back_edges: &HashSet<usize>,
    ) {
        if layers.len() <= 1 {
            return;
        }

        let mut pos_in_layer: HashMap<NodeId, f32> = HashMap::new();
        for (i, &id) in layers[0].iter().enumerate() {
            pos_in_layer.insert(id, i as f32);
        }

        for r in 1..layers.len() {
            let mut barycenters: Vec<(NodeId, f32)> = Vec::new();

            for &v in &layers[r] {
                // Find all parents in forward edges
                let mut sum = 0.0f32;
                let mut count = 0usize;

                for edge in graph.incoming_edges(v) {
                    if !back_edges.contains(&edge.id) {
                        if let Some(&p_pos) = pos_in_layer.get(&edge.from) {
                            sum += p_pos;
                            count += 1;
                        }
                    }
                }

                let bc = if count > 0 { sum / (count as f32) } else { 0.0 };
                barycenters.push((v, bc));
            }

            barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            layers[r] = barycenters.into_iter().map(|(id, _)| id).collect();

            for (i, &id) in layers[r].iter().enumerate() {
                pos_in_layer.insert(id, i as f32);
            }
        }
    }
}
