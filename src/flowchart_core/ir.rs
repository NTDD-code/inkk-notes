use crate::flowchart_core::ast::{AstStmt, BlockKind, FlowchartAst, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub side: PortSide,
    pub port_type: PortType,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: NodeId,
    pub custom_id: Option<String>,
    pub kind: BlockKind,
    pub label: String,
    pub span: Span,
    pub pinned_pos: Option<(f32, f32)>,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowEdge {
    pub id: usize,
    pub from: NodeId,
    pub from_side: PortSide,
    pub to: NodeId,
    pub to_side: PortSide,
    pub label: Option<String>,
    pub is_back_edge: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowchartGraph {
    pub title: Option<String>,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub labels: HashMap<String, NodeId>,
    next_node_id: usize,
    next_edge_id: usize,
}

impl Default for FlowchartGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowchartGraph {
    pub fn new() -> Self {
        Self {
            title: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            labels: HashMap::new(),
            next_node_id: 1,
            next_edge_id: 1,
        }
    }

    pub fn alloc_node(&mut self, kind: BlockKind, label: String, span: Span) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        let (w, h) = match kind {
            BlockKind::Start | BlockKind::End => (110.0, 42.0),
            BlockKind::Decision => (130.0, 70.0),
            BlockKind::Input | BlockKind::Output => (130.0, 50.0),
            BlockKind::LoopFor | BlockKind::LoopWhile => (135.0, 55.0),
            _ => (130.0, 50.0),
        };

        self.nodes.push(FlowNode {
            id,
            custom_id: None,
            kind,
            label,
            span,
            pinned_pos: None,
            width: w,
            height: h,
        });

        id
    }

    pub fn add_edge(
        &mut self,
        from: NodeId,
        from_side: PortSide,
        to: NodeId,
        to_side: PortSide,
        label: Option<String>,
    ) -> usize {
        let id = self.next_edge_id;
        self.next_edge_id += 1;
        self.edges.push(FlowEdge {
            id,
            from,
            from_side,
            to,
            to_side,
            label,
            is_back_edge: false,
        });
        id
    }

    pub fn node(&self, id: NodeId) -> Option<&FlowNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut FlowNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn incoming_edges(&self, node: NodeId) -> impl Iterator<Item = &FlowEdge> {
        self.edges.iter().filter(move |e| e.to == node)
    }

    pub fn outgoing_edges(&self, node: NodeId) -> impl Iterator<Item = &FlowEdge> {
        self.edges.iter().filter(move |e| e.from == node)
    }

    pub fn start_nodes(&self) -> Vec<&FlowNode> {
        self.nodes
            .iter()
            .filter(|n| n.kind == BlockKind::Start)
            .collect()
    }

    pub fn end_nodes(&self) -> Vec<&FlowNode> {
        self.nodes
            .iter()
            .filter(|n| n.kind == BlockKind::End)
            .collect()
    }

    pub fn add_visual_node(&mut self, kind: BlockKind, label: String, pos: (f32, f32)) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        let (w, h) = match kind {
            BlockKind::Start | BlockKind::End => (110.0, 42.0),
            BlockKind::Decision => (130.0, 70.0),
            BlockKind::Input | BlockKind::Output => (130.0, 50.0),
            BlockKind::LoopFor | BlockKind::LoopWhile | BlockKind::LoopRepeat => (135.0, 55.0),
            BlockKind::Connector => (48.0, 48.0),
            BlockKind::Document => (120.0, 55.0),
            BlockKind::Database => (100.0, 60.0),
            _ => (130.0, 50.0),
        };

        self.nodes.push(FlowNode {
            id,
            custom_id: None,
            kind,
            label,
            span: Span::default(),
            pinned_pos: Some(pos),
            width: w,
            height: h,
        });

        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    pub fn remove_edge(&mut self, edge_id: usize) {
        self.edges.retain(|e| e.id != edge_id);
    }

    pub fn compute_bounds(&self) -> (f32, f32, f32, f32) {
        if self.nodes.is_empty() {
            return (0.0, 0.0, 400.0, 300.0);
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in &self.nodes {
            let (nx, ny) = node.pinned_pos.unwrap_or((60.0, 60.0));
            min_x = min_x.min(nx);
            min_y = min_y.min(ny);
            max_x = max_x.max(nx + node.width);
            max_y = max_y.max(ny + node.height);
        }

        (min_x, min_y, max_x, max_y)
    }

    pub fn shift_all_nodes(&mut self, dx: f32, dy: f32) {
        for node in &mut self.nodes {
            if let Some(ref mut pos) = node.pinned_pos {
                pos.0 += dx;
                pos.1 += dy;
            }
        }
    }

    pub fn from_ast(ast: &FlowchartAst) -> Self {
        let builder = GraphBuilder::new();
        builder.build(ast)
    }
}

struct GraphBuilder {
    graph: FlowchartGraph,
    custom_id_map: HashMap<String, NodeId>,
    pending_gotos: Vec<(NodeId, PortSide, String)>,
}

impl GraphBuilder {
    fn new() -> Self {
        Self {
            graph: FlowchartGraph::new(),
            custom_id_map: HashMap::new(),
            pending_gotos: Vec::new(),
        }
    }

    fn build(mut self, ast: &FlowchartAst) -> FlowchartGraph {
        self.graph.title = ast.title.clone();

        let mut entry_nodes = Vec::new();
        let mut last_exits = Vec::new();

        for stmt in &ast.statements {
            let (entry, exits) = self.build_stmt(stmt);
            if let Some(entry_id) = entry {
                if entry_nodes.is_empty() {
                    entry_nodes.push(entry_id);
                }
                for (prev_node, prev_side, edge_lbl) in last_exits {
                    self.graph
                        .add_edge(prev_node, prev_side, entry_id, PortSide::Top, edge_lbl);
                }
                last_exits = exits;
            }
        }

        // Resolve gotos to declared labels
        let gotos = std::mem::take(&mut self.pending_gotos);
        for (from_node, from_side, label_target) in gotos {
            if let Some(&target_node) = self.graph.labels.get(&label_target) {
                let edge_id =
                    self.graph
                        .add_edge(from_node, from_side, target_node, PortSide::Left, None);
                if let Some(edge) = self.graph.edges.iter_mut().find(|e| e.id == edge_id) {
                    edge.is_back_edge = true;
                }
            }
        }

        self.graph
    }

    /// Returns (Entry Node, List of Exit Endpoints `(NodeId, PortSide, Option<label>)`)
    fn build_stmt(
        &mut self,
        stmt: &AstStmt,
    ) -> (Option<NodeId>, Vec<(NodeId, PortSide, Option<String>)>) {
        match stmt {
            AstStmt::AlgorithmTitle(_) => (None, Vec::new()),
            AstStmt::Start(span) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Start, "Start".to_string(), *span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::End(span) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::End, "End".to_string(), *span);
                (Some(id), Vec::new()) // End node has no outgoing exits
            }
            AstStmt::Return(spanned) => {
                let label = match &spanned.value {
                    Some(v) => format!("Return {}", v),
                    None => "Return".to_string(),
                };
                let id = self.graph.alloc_node(BlockKind::End, label, spanned.span);
                (Some(id), Vec::new())
            }
            AstStmt::Process(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Process, s.value.clone(), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Input(s) => {
                let id =
                    self.graph
                        .alloc_node(BlockKind::Input, format!("Read {}", s.value), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Output(s) => {
                let id =
                    self.graph
                        .alloc_node(BlockKind::Output, format!("Print {}", s.value), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Document(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Document, s.value.clone(), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Database(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Database, s.value.clone(), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::ManualInput(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::ManualInput, s.value.clone(), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Delay(s) => {
                let id =
                    self.graph
                        .alloc_node(BlockKind::Delay, format!("Wait {}", s.value), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Display(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Display, s.value.clone(), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::SubroutineCall { name, span, .. } => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Subroutine, name.value.clone(), *span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Label(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Connector, s.value.clone(), s.span);
                self.graph.labels.insert(s.value.clone(), id);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Goto(s) => {
                let id = self.graph.alloc_node(
                    BlockKind::Connector,
                    format!("Goto {}", s.value),
                    s.span,
                );
                self.pending_gotos
                    .push((id, PortSide::Bottom, s.value.clone()));
                (Some(id), Vec::new()) // Followed by jump
            }
            AstStmt::OffpageLink(s) => {
                let id = self
                    .graph
                    .alloc_node(BlockKind::Offpage, s.value.clone(), s.span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
            AstStmt::Note(_) => (None, Vec::new()),
            AstStmt::Decision {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                let dec_id = self.graph.alloc_node(
                    BlockKind::Decision,
                    format!("{}?", condition.value),
                    *span,
                );
                let mut exits = Vec::new();

                // Then branch (Yes, goes down or right)
                let (then_entry, then_exits) = self.build_block(then_branch);
                if let Some(entry) = then_entry {
                    self.graph.add_edge(
                        dec_id,
                        PortSide::Bottom,
                        entry,
                        PortSide::Top,
                        Some("Yes".to_string()),
                    );
                    exits.extend(then_exits);
                } else {
                    exits.push((dec_id, PortSide::Bottom, Some("Yes".to_string())));
                }

                // Else branch (No, goes right)
                if let Some(else_stmts) = else_branch {
                    let (else_entry, else_exits) = self.build_block(else_stmts);
                    if let Some(entry) = else_entry {
                        self.graph.add_edge(
                            dec_id,
                            PortSide::Right,
                            entry,
                            PortSide::Top,
                            Some("No".to_string()),
                        );
                        exits.extend(else_exits);
                    } else {
                        exits.push((dec_id, PortSide::Right, Some("No".to_string())));
                    }
                } else {
                    exits.push((dec_id, PortSide::Right, Some("No".to_string())));
                }

                (Some(dec_id), exits)
            }
            AstStmt::LoopFor {
                var,
                from,
                to,
                body,
                span,
            } => {
                let label = format!("for {} = {} to {}", var.value, from.value, to.value);
                let loop_id = self.graph.alloc_node(BlockKind::LoopFor, label, *span);

                let (body_entry, body_exits) = self.build_block(body);
                if let Some(entry) = body_entry {
                    self.graph.add_edge(
                        loop_id,
                        PortSide::Bottom,
                        entry,
                        PortSide::Top,
                        Some("Loop".to_string()),
                    );
                    for (ex_node, ex_side, _) in body_exits {
                        let edge_id =
                            self.graph
                                .add_edge(ex_node, ex_side, loop_id, PortSide::Left, None);
                        if let Some(edge) = self.graph.edges.iter_mut().find(|e| e.id == edge_id) {
                            edge.is_back_edge = true;
                        }
                    }
                }

                // Loop exit branch goes right
                (
                    Some(loop_id),
                    vec![(loop_id, PortSide::Right, Some("Done".to_string()))],
                )
            }
            AstStmt::LoopWhile {
                condition,
                body,
                span,
            } => {
                let loop_id = self.graph.alloc_node(
                    BlockKind::LoopWhile,
                    format!("while {}?", condition.value),
                    *span,
                );

                let (body_entry, body_exits) = self.build_block(body);
                if let Some(entry) = body_entry {
                    self.graph.add_edge(
                        loop_id,
                        PortSide::Bottom,
                        entry,
                        PortSide::Top,
                        Some("True".to_string()),
                    );
                    for (ex_node, ex_side, _) in body_exits {
                        let edge_id =
                            self.graph
                                .add_edge(ex_node, ex_side, loop_id, PortSide::Left, None);
                        if let Some(edge) = self.graph.edges.iter_mut().find(|e| e.id == edge_id) {
                            edge.is_back_edge = true;
                        }
                    }
                }

                (
                    Some(loop_id),
                    vec![(loop_id, PortSide::Right, Some("False".to_string()))],
                )
            }
            AstStmt::LoopRepeat {
                body,
                until_cond,
                span,
            } => {
                let (body_entry, body_exits) = self.build_block(body);
                let cond_id = self.graph.alloc_node(
                    BlockKind::Decision,
                    format!("until {}?", until_cond.value),
                    *span,
                );

                if let Some(entry) = body_entry {
                    for (ex_node, ex_side, _) in body_exits {
                        self.graph
                            .add_edge(ex_node, ex_side, cond_id, PortSide::Top, None);
                    }
                    // Loop back edge when False
                    let edge_id = self.graph.add_edge(
                        cond_id,
                        PortSide::Left,
                        entry,
                        PortSide::Left,
                        Some("False".to_string()),
                    );
                    if let Some(edge) = self.graph.edges.iter_mut().find(|e| e.id == edge_id) {
                        edge.is_back_edge = true;
                    }
                    (
                        Some(entry),
                        vec![(cond_id, PortSide::Bottom, Some("True".to_string()))],
                    )
                } else {
                    (
                        Some(cond_id),
                        vec![(cond_id, PortSide::Bottom, Some("True".to_string()))],
                    )
                }
            }
            AstStmt::Switch {
                target,
                cases,
                default_case,
                span,
            } => {
                let sw_id = self.graph.alloc_node(
                    BlockKind::Switch,
                    format!("switch {}", target.value),
                    *span,
                );
                let mut exits = Vec::new();

                for case in cases {
                    let (c_entry, c_exits) = self.build_block(&case.body);
                    if let Some(entry) = c_entry {
                        self.graph.add_edge(
                            sw_id,
                            PortSide::Bottom,
                            entry,
                            PortSide::Top,
                            Some(case.value.value.clone()),
                        );
                        exits.extend(c_exits);
                    } else {
                        exits.push((sw_id, PortSide::Bottom, Some(case.value.value.clone())));
                    }
                }

                if let Some(def_body) = default_case {
                    let (d_entry, d_exits) = self.build_block(def_body);
                    if let Some(entry) = d_entry {
                        self.graph.add_edge(
                            sw_id,
                            PortSide::Right,
                            entry,
                            PortSide::Top,
                            Some("default".to_string()),
                        );
                        exits.extend(d_exits);
                    } else {
                        exits.push((sw_id, PortSide::Right, Some("default".to_string())));
                    }
                }

                (Some(sw_id), exits)
            }
            AstStmt::Parallel { branches, span } => {
                let fork_id = self
                    .graph
                    .alloc_node(BlockKind::Parallel, "Fork".to_string(), *span);
                let join_id = self
                    .graph
                    .alloc_node(BlockKind::Parallel, "Join".to_string(), *span);

                for branch in branches {
                    let (b_entry, b_exits) = self.build_block(branch);
                    if let Some(entry) = b_entry {
                        self.graph
                            .add_edge(fork_id, PortSide::Bottom, entry, PortSide::Top, None);
                        for (ex_node, ex_side, lbl) in b_exits {
                            self.graph
                                .add_edge(ex_node, ex_side, join_id, PortSide::Top, lbl);
                        }
                    } else {
                        self.graph.add_edge(
                            fork_id,
                            PortSide::Bottom,
                            join_id,
                            PortSide::Top,
                            None,
                        );
                    }
                }

                (Some(fork_id), vec![(join_id, PortSide::Bottom, None)])
            }
            AstStmt::Swimlane { name, body, span } => {
                let lane_id = self
                    .graph
                    .alloc_node(BlockKind::Swimlane, name.value.clone(), *span);
                let (b_entry, b_exits) = self.build_block(body);
                if let Some(entry) = b_entry {
                    self.graph
                        .add_edge(lane_id, PortSide::Bottom, entry, PortSide::Top, None);
                    (Some(lane_id), b_exits)
                } else {
                    (Some(lane_id), vec![(lane_id, PortSide::Bottom, None)])
                }
            }
            AstStmt::RawNodeDef {
                id,
                kind,
                label,
                span,
            } => {
                let node_id = self.graph.alloc_node(*kind, label.value.clone(), *span);
                if let Some(node) = self.graph.node_mut(node_id) {
                    node.custom_id = Some(id.value.clone());
                }
                self.custom_id_map.insert(id.value.clone(), node_id);
                (Some(node_id), Vec::new())
            }
            AstStmt::RawEdge {
                from, to, label, ..
            } => {
                let from_id = self.custom_id_map.get(&from.value).copied();
                let to_id = self.custom_id_map.get(&to.value).copied();
                if let (Some(f), Some(t)) = (from_id, to_id) {
                    self.graph.add_edge(
                        f,
                        PortSide::Bottom,
                        t,
                        PortSide::Top,
                        label.as_ref().map(|l| l.value.clone()),
                    );
                }
                (None, Vec::new())
            }
            AstStmt::ErrorRecovery { raw_text, span, .. } => {
                let id =
                    self.graph
                        .alloc_node(BlockKind::Process, format!("[!] {}", raw_text), *span);
                (Some(id), vec![(id, PortSide::Bottom, None)])
            }
        }
    }

    fn build_block(
        &mut self,
        stmts: &[AstStmt],
    ) -> (Option<NodeId>, Vec<(NodeId, PortSide, Option<String>)>) {
        let mut entry_id = None;
        let mut last_exits = Vec::new();

        for stmt in stmts {
            let (entry, exits) = self.build_stmt(stmt);
            if let Some(e) = entry {
                if entry_id.is_none() {
                    entry_id = Some(e);
                }
                for (prev_node, prev_side, edge_lbl) in last_exits {
                    self.graph
                        .add_edge(prev_node, prev_side, e, PortSide::Top, edge_lbl);
                }
                last_exits = exits;
            }
        }

        (entry_id, last_exits)
    }
}
