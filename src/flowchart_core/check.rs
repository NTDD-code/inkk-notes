//! Semantic checker and compiler-grade diagnostics (F0101..F0120) for flowcharts.
//! Produces rustc-style error messages with exact spans, code lines, ASCII carets, and suggested fixes.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use super::ast::{AstStmt, BlockKind, FlowchartAst, Span};
use super::ir::{FlowchartGraph, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Help,
    Note,
    Warning,
    Error,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::Help => "help",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub span: Option<Span>,
    pub node_id: Option<NodeId>,
    pub help: Option<String>,
    pub note: Option<String>,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: DiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            span: None,
            node_id: None,
            help: None,
            note: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_node(mut self, node_id: NodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Render this diagnostic in standard rustc style with source lines and carets.
    pub fn render_rustc(&self, source: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{}[{}]: {}\n",
            self.severity.as_str(),
            self.code,
            self.message
        ));

        if let Some(span) = self.span {
            let line_no = span.line.max(1);
            let col_no = span.col.max(1);
            out.push_str(&format!("  --> {}:{}\n", line_no, col_no));

            // Extract the relevant line from source
            let lines: Vec<&str> = source.lines().collect();
            if line_no <= lines.len() {
                let line_str = lines[line_no - 1];
                let line_num_str = format!("{}", line_no);
                let gutter_pad = " ".repeat(line_num_str.len());

                out.push_str(&format!("{} |\n", gutter_pad));
                out.push_str(&format!("{} | {}\n", line_num_str, line_str));

                // Caret underline
                let start_col = (col_no - 1).min(line_str.len());
                let span_len = if span.end > span.start {
                    (span.end - span.start).max(1)
                } else {
                    1
                };
                let carets_len = span_len
                    .min(line_str.len().saturating_sub(start_col))
                    .max(1);
                let caret_line = format!("{}{}", " ".repeat(start_col), "^".repeat(carets_len));

                out.push_str(&format!("{} | {}", gutter_pad, caret_line));
                if let Some(help) = &self.help {
                    out.push_str(&format!(" help: {}", help));
                }
                out.push('\n');
            }
        }

        if let Some(note) = &self.note {
            out.push_str(&format!("  = note: {}\n", note));
        }

        out
    }
}

pub struct FlowchartChecker;

impl FlowchartChecker {
    pub fn check(graph: &FlowchartGraph, ast: Option<&FlowchartAst>) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        if graph.nodes.is_empty() {
            return diags;
        }

        // F0101: Missing Start terminal
        let start_nodes = graph.start_nodes();
        if start_nodes.is_empty() {
            diags.push(
                Diagnostic::new(
                    "F0101",
                    DiagnosticSeverity::Warning,
                    "flowchart has no 'start' terminal node",
                )
                .with_help(
                    "ISO 5807 recommends starting every algorithm flowchart with a 'start' block",
                ),
            );
        }

        // F0102: Multiple Start terminals
        if start_nodes.len() > 1 {
            for (_idx, start_node) in start_nodes.iter().enumerate().skip(1) {
                diags.push(
                    Diagnostic::new(
                        "F0102",
                        DiagnosticSeverity::Error,
                        format!("multiple 'start' terminal nodes found in flowchart (found {})", start_nodes.len()),
                    )
                    .with_span(start_node.span)
                    .with_node(start_node.id)
                    .with_help("an algorithm flow should have exactly one entry point; consider combining them or using a subroutine"),
                );
            }
        }

        // F0103: Start node has incoming edge
        for start_node in &start_nodes {
            let in_count = graph.incoming_edges(start_node.id).count();
            if in_count > 0 {
                diags.push(
                    Diagnostic::new(
                        "F0103",
                        DiagnosticSeverity::Error,
                        format!("'start' terminal cannot have incoming flows (found {} incoming)", in_count),
                    )
                    .with_span(start_node.span)
                    .with_node(start_node.id)
                    .with_help("loop backs should target a process or label, not the algorithm start block"),
                );
            }
        }

        // F0104: Start node has no outgoing edge
        for start_node in &start_nodes {
            if graph.nodes.len() > 1 && graph.outgoing_edges(start_node.id).count() == 0 {
                diags.push(
                    Diagnostic::new(
                        "F0104",
                        DiagnosticSeverity::Warning,
                        "'start' terminal is not connected to any subsequent step",
                    )
                    .with_span(start_node.span)
                    .with_node(start_node.id)
                    .with_help("connect 'start' to the first process or input node"),
                );
            }
        }

        // F0105: Missing End/Return terminal
        let end_nodes = graph.end_nodes();
        if end_nodes.is_empty() {
            diags.push(
                Diagnostic::new(
                    "F0105",
                    DiagnosticSeverity::Warning,
                    "flowchart has no 'end' or 'return' terminal",
                )
                .with_help("every flowchart should have at least one terminal termination point ('end' or 'return')"),
            );
        }

        // F0106: End node has outgoing edge
        for end_node in &end_nodes {
            let out_count = graph.outgoing_edges(end_node.id).count();
            if out_count > 0 {
                diags.push(
                    Diagnostic::new(
                        "F0106",
                        DiagnosticSeverity::Error,
                        format!(
                            "'end' terminal cannot have outgoing flows (found {})",
                            out_count
                        ),
                    )
                    .with_span(end_node.span)
                    .with_node(end_node.id)
                    .with_help("flows terminate at an 'end' block; remove outgoing transitions"),
                );
            }
        }

        // F0107: Unreachable nodes (from Start node if present, or from root nodes)
        let reachable = compute_reachable_nodes(graph);
        for node in &graph.nodes {
            if !reachable.contains(&node.id) && node.kind != BlockKind::Start {
                diags.push(
                    Diagnostic::new(
                        "F0107",
                        DiagnosticSeverity::Warning,
                        format!("unreachable node '{}' [{:?}]", node.label, node.kind),
                    )
                    .with_span(node.span)
                    .with_node(node.id)
                    .with_help("this block cannot be reached from any algorithm entry point; verify your flow connections"),
                );
            }
        }

        // F0108: Dead end (non-End, non-Goto node with 0 outgoing edges)
        for node in &graph.nodes {
            if node.kind != BlockKind::End
                && node.kind != BlockKind::Connector
                && graph.outgoing_edges(node.id).count() == 0
                && graph.nodes.len() > 1
                && !node.label.starts_with("⚠️")
            {
                diags.push(
                    Diagnostic::new(
                        "F0108",
                        DiagnosticSeverity::Error,
                        format!(
                            "dead-end block '{}' [{:?}] has no outgoing flow",
                            node.label, node.kind
                        ),
                    )
                    .with_span(node.span)
                    .with_node(node.id)
                    .with_help("add a transition to the next step or an 'end' terminal"),
                );
            }
        }

        // F0109 & F0110: Decision node branch validation
        for node in &graph.nodes {
            if node.kind == BlockKind::Decision {
                let outs: Vec<_> = graph.outgoing_edges(node.id).collect();
                if outs.len() != 2 {
                    diags.push(
                        Diagnostic::new(
                            "F0109",
                            DiagnosticSeverity::Error,
                            format!("decision diamond '{}' must have exactly 2 outgoing branches (found {})", node.label, outs.len()),
                        )
                        .with_span(node.span)
                        .with_node(node.id)
                        .with_help("ISO 5807 conditional decisions require both a 'True' ('Yes') and 'False' ('No') branch"),
                    );
                } else {
                    let lbl0 = outs[0].label.as_deref().unwrap_or("").trim().to_lowercase();
                    let lbl1 = outs[1].label.as_deref().unwrap_or("").trim().to_lowercase();
                    if lbl0.is_empty() && lbl1.is_empty() {
                        diags.push(
                            Diagnostic::new(
                                "F0110",
                                DiagnosticSeverity::Warning,
                                format!(
                                    "decision diamond '{}' is missing branch labels",
                                    node.label
                                ),
                            )
                            .with_span(node.span)
                            .with_node(node.id)
                            .with_help("label outgoing branches with 'Yes' and 'No' for clarity"),
                        );
                    } else if !lbl0.is_empty() && lbl0 == lbl1 {
                        diags.push(
                            Diagnostic::new(
                                "F0110",
                                DiagnosticSeverity::Error,
                                format!("decision branches have identical label '{}'", lbl0),
                            )
                            .with_span(node.span)
                            .with_node(node.id)
                            .with_help(
                                "branch labels must be distinguishable (e.g., 'Yes' and 'No')",
                            ),
                        );
                    }
                }
            }
        }

        // F0111 & F0112: Switch multi-way branch validation
        for node in &graph.nodes {
            if node.kind == BlockKind::Switch {
                let outs: Vec<_> = graph.outgoing_edges(node.id).collect();
                if outs.len() < 2 {
                    diags.push(
                        Diagnostic::new(
                            "F0112",
                            DiagnosticSeverity::Error,
                            format!(
                                "switch block '{}' must have at least 2 case branches",
                                node.label
                            ),
                        )
                        .with_span(node.span)
                        .with_node(node.id)
                        .with_help(
                            "define at least two case branches or use a simple 'if' decision",
                        ),
                    );
                }
                let has_default = outs.iter().any(|e| {
                    if let Some(l) = &e.label {
                        let l_low = l.to_lowercase();
                        l_low.contains("default")
                            || l_low.contains("otherwise")
                            || l_low.contains("else")
                    } else {
                        false
                    }
                });
                if !has_default {
                    diags.push(
                        Diagnostic::new(
                            "F0111",
                            DiagnosticSeverity::Warning,
                            format!(
                                "switch block '{}' has no 'default' fallback branch",
                                node.label
                            ),
                        )
                        .with_span(node.span)
                        .with_node(node.id)
                        .with_help("add a 'default' branch to handle unhandled cases"),
                    );
                }
            }
        }

        // F0113 & F0114: Check AST for Goto labels & duplicates
        if let Some(ast) = ast {
            let mut declared_labels = HashMap::new();
            check_ast_labels(ast, &mut declared_labels, &mut diags);
        }

        // F0116: Infinite cycle detection (cycles with zero exit branches)
        check_cycle_exits(graph, &mut diags);

        // F0117: Process / IO node degree violation (>1 outgoing edge)
        for node in &graph.nodes {
            match node.kind {
                BlockKind::Process
                | BlockKind::Input
                | BlockKind::Output
                | BlockKind::Subroutine => {
                    let out_count = graph.outgoing_edges(node.id).count();
                    if out_count > 1 {
                        diags.push(
                            Diagnostic::new(
                                "F0117",
                                DiagnosticSeverity::Error,
                                format!("{:?} block '{}' cannot have multiple outgoing flows without a decision or fork (found {})", node.kind, node.label, out_count),
                            )
                            .with_span(node.span)
                            .with_node(node.id)
                            .with_help("use an 'if' decision or a 'parallel fork' to split control flow"),
                        );
                    }
                }
                _ => {}
            }
        }

        // F0118: Empty subroutine or empty block check
        if let Some(ast) = ast {
            check_empty_blocks(ast, &mut diags);
        }

        // F0119: Disconnected components
        check_connected_components(graph, &mut diags);

        // F0120: Syntax recovery placeholder remaining
        for node in &graph.nodes {
            if node.label.starts_with("⚠️") {
                diags.push(
                    Diagnostic::new(
                        "F0120",
                        DiagnosticSeverity::Error,
                        format!(
                            "syntax error recovery placeholder remaining: {}",
                            node.label
                        ),
                    )
                    .with_span(node.span)
                    .with_node(node.id)
                    .with_help("fix the syntax error in DSL to restore full node semantics"),
                );
            }
        }

        diags
    }
}

/// Compute all nodes reachable from the Start node (or all in-degree 0 nodes if no Start exists).
fn compute_reachable_nodes(graph: &FlowchartGraph) -> HashSet<NodeId> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    let start_nodes = graph.start_nodes();
    if !start_nodes.is_empty() {
        for s in start_nodes {
            queue.push_back(s.id);
            visited.insert(s.id);
        }
    } else {
        // Fallback: start from all nodes with 0 in-degree
        for node in &graph.nodes {
            if graph.incoming_edges(node.id).count() == 0 {
                queue.push_back(node.id);
                visited.insert(node.id);
            }
        }
    }

    while let Some(curr) = queue.pop_front() {
        for edge in graph.outgoing_edges(curr) {
            if visited.insert(edge.to) {
                queue.push_back(edge.to);
            }
        }
    }

    visited
}

/// Check AST statements for duplicate labels and missing goto targets.
fn check_ast_labels(
    ast: &FlowchartAst,
    declared: &mut HashMap<String, Span>,
    diags: &mut Vec<Diagnostic>,
) {
    let mut gotos = Vec::new();

    collect_labels_and_gotos(&ast.statements, declared, &mut gotos, diags);

    for (target_lbl, span) in gotos {
        if !declared.contains_key(&target_lbl) {
            diags.push(
                Diagnostic::new(
                    "F0113",
                    DiagnosticSeverity::Error,
                    format!("unresolved goto target 'label {}'", target_lbl),
                )
                .with_span(span)
                .with_help(format!(
                    "define 'label {}:' somewhere in the algorithm",
                    target_lbl
                )),
            );
        }
    }
}

fn collect_labels_and_gotos(
    stmts: &[AstStmt],
    declared: &mut HashMap<String, Span>,
    gotos: &mut Vec<(String, Span)>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            AstStmt::Label(name) => {
                if let Some(prev_span) = declared.get(&name.value) {
                    diags.push(
                        Diagnostic::new(
                            "F0114",
                            DiagnosticSeverity::Error,
                            format!("duplicate label definition 'label {}'", name.value),
                        )
                        .with_span(name.span)
                        .with_note(format!("previously defined at line {}", prev_span.line))
                        .with_help("label identifiers must be unique across the flowchart"),
                    );
                } else {
                    declared.insert(name.value.clone(), name.span);
                }
            }
            AstStmt::Goto(target) => {
                gotos.push((target.value.clone(), target.span));
            }
            AstStmt::Decision {
                then_branch,
                else_branch,
                ..
            } => {
                collect_labels_and_gotos(then_branch, declared, gotos, diags);
                if let Some(eb) = else_branch {
                    collect_labels_and_gotos(eb, declared, gotos, diags);
                }
            }
            AstStmt::LoopFor { body, .. }
            | AstStmt::LoopWhile { body, .. }
            | AstStmt::LoopRepeat { body, .. } => {
                collect_labels_and_gotos(body, declared, gotos, diags);
            }
            AstStmt::Switch {
                cases,
                default_case,
                ..
            } => {
                for c in cases {
                    collect_labels_and_gotos(&c.body, declared, gotos, diags);
                }
                if let Some(dc) = default_case {
                    collect_labels_and_gotos(dc, declared, gotos, diags);
                }
            }
            AstStmt::Parallel { branches, .. } => {
                for b in branches {
                    collect_labels_and_gotos(b, declared, gotos, diags);
                }
            }
            AstStmt::Swimlane { body, .. } => {
                collect_labels_and_gotos(body, declared, gotos, diags);
            }
            _ => {}
        }
    }
}

/// Check for empty blocks (empty subroutine, empty loop body, empty branch).
fn check_empty_blocks(ast: &FlowchartAst, diags: &mut Vec<Diagnostic>) {
    fn scan(stmts: &[AstStmt], diags: &mut Vec<Diagnostic>) {
        for stmt in stmts {
            match stmt {
                AstStmt::Decision {
                    condition,
                    then_branch,
                    else_branch,
                    span,
                } => {
                    if then_branch.is_empty() {
                        diags.push(
                            Diagnostic::new(
                                "F0118",
                                DiagnosticSeverity::Warning,
                                format!(
                                    "'then' branch for condition '{}' is empty",
                                    condition.value
                                ),
                            )
                            .with_span(*span)
                            .with_help("add executable statements to the 'then' block"),
                        );
                    }
                    if let Some(eb) = else_branch {
                        if eb.is_empty() {
                            diags.push(
                                Diagnostic::new(
                                    "F0118",
                                    DiagnosticSeverity::Warning,
                                    format!(
                                        "'else' branch for condition '{}' is empty",
                                        condition.value
                                    ),
                                )
                                .with_span(*span)
                                .with_help("remove empty 'else' block or add statements"),
                            );
                        }
                    }
                    scan(then_branch, diags);
                    if let Some(eb) = else_branch {
                        scan(eb, diags);
                    }
                }
                AstStmt::LoopWhile {
                    condition,
                    body,
                    span,
                } => {
                    if body.is_empty() {
                        diags.push(
                            Diagnostic::new(
                                "F0118",
                                DiagnosticSeverity::Warning,
                                format!(
                                    "while loop body for condition '{}' is empty",
                                    condition.value
                                ),
                            )
                            .with_span(*span)
                            .with_help(
                                "loop body contains no statements; will execute as a busy loop",
                            ),
                        );
                    }
                    scan(body, diags);
                }
                AstStmt::LoopFor {
                    var, body, span, ..
                } => {
                    if body.is_empty() {
                        diags.push(
                            Diagnostic::new(
                                "F0118",
                                DiagnosticSeverity::Warning,
                                format!("for loop body for variable '{}' is empty", var.value),
                            )
                            .with_span(*span),
                        );
                    }
                    scan(body, diags);
                }
                _ => {}
            }
        }
    }
    scan(&ast.statements, diags);
}

/// Check for disconnected components in the flowchart (weakly connected components).
fn check_connected_components(graph: &FlowchartGraph, diags: &mut Vec<Diagnostic>) {
    if graph.nodes.len() <= 1 {
        return;
    }

    let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in &graph.nodes {
        adj.insert(node.id, Vec::new());
    }
    for edge in &graph.edges {
        adj.entry(edge.from).or_default().push(edge.to);
        adj.entry(edge.to).or_default().push(edge.from);
    }

    let mut visited = HashSet::new();
    let mut components = 0;

    for node in &graph.nodes {
        if !visited.contains(&node.id) {
            components += 1;
            let mut q = VecDeque::new();
            q.push_back(node.id);
            visited.insert(node.id);
            while let Some(curr) = q.pop_front() {
                if let Some(neighbors) = adj.get(&curr) {
                    for &nbr in neighbors {
                        if visited.insert(nbr) {
                            q.push_back(nbr);
                        }
                    }
                }
            }
        }
    }

    if components > 1 {
        diags.push(
            Diagnostic::new(
                "F0119",
                DiagnosticSeverity::Warning,
                format!(
                    "flowchart contains {} disconnected subgraphs / isolated components",
                    components
                ),
            )
            .with_help("connect all elements together into a unified execution flow"),
        );
    }
}

/// Detect cycles with no exit paths (F0116: infinite cycle with no way out).
fn check_cycle_exits(graph: &FlowchartGraph, diags: &mut Vec<Diagnostic>) {
    // Tarjan's SCC (Strongly Connected Components) to find directed cycles
    let mut index = 0usize;
    let mut indices: HashMap<NodeId, usize> = HashMap::new();
    let mut lowlinks: HashMap<NodeId, usize> = HashMap::new();
    let mut on_stack: HashSet<NodeId> = HashSet::new();
    let mut stack: Vec<NodeId> = Vec::new();
    let mut sccs: Vec<Vec<NodeId>> = Vec::new();

    fn strongconnect(
        v: NodeId,
        graph: &FlowchartGraph,
        index: &mut usize,
        indices: &mut HashMap<NodeId, usize>,
        lowlinks: &mut HashMap<NodeId, usize>,
        on_stack: &mut HashSet<NodeId>,
        stack: &mut Vec<NodeId>,
        sccs: &mut Vec<Vec<NodeId>>,
    ) {
        indices.insert(v, *index);
        lowlinks.insert(v, *index);
        *index += 1;
        stack.push(v);
        on_stack.insert(v);

        for edge in graph.outgoing_edges(v) {
            let w = edge.to;
            if !indices.contains_key(&w) {
                strongconnect(w, graph, index, indices, lowlinks, on_stack, stack, sccs);
                let w_low = lowlinks[&w];
                let v_low = lowlinks.get_mut(&v).unwrap();
                *v_low = (*v_low).min(w_low);
            } else if on_stack.contains(&w) {
                let w_idx = indices[&w];
                let v_low = lowlinks.get_mut(&v).unwrap();
                *v_low = (*v_low).min(w_idx);
            }
        }

        if lowlinks[&v] == indices[&v] {
            let mut scc = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack.remove(&w);
                scc.push(w);
                if w == v {
                    break;
                }
            }
            if scc.len() > 1 || graph.outgoing_edges(v).any(|e| e.to == v) {
                sccs.push(scc);
            }
        }
    }

    for node in &graph.nodes {
        if !indices.contains_key(&node.id) {
            strongconnect(
                node.id,
                graph,
                &mut index,
                &mut indices,
                &mut lowlinks,
                &mut on_stack,
                &mut stack,
                &mut sccs,
            );
        }
    }

    for scc in sccs {
        let scc_set: HashSet<NodeId> = scc.iter().copied().collect();
        // Check if there is at least one outgoing edge leaving this SCC
        let mut has_exit = false;
        for &node_id in &scc_set {
            for edge in graph.outgoing_edges(node_id) {
                if !scc_set.contains(&edge.to) {
                    has_exit = true;
                    break;
                }
            }
            if has_exit {
                break;
            }
        }

        if !has_exit {
            let rep_node = scc.first().and_then(|id| graph.node(*id));
            let span = rep_node.map(|n| n.span);
            diags.push(
                Diagnostic::new(
                    "F0116",
                    DiagnosticSeverity::Warning,
                    "cycle detected with no exit flow (infinite loop)",
                )
                .with_span(span.unwrap_or_default())
                .with_help(
                    "provide a conditional break or exit branch to avoid infinite loop execution",
                ),
            );
        }
    }
}
