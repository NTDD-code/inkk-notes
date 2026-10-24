use crate::flowchart_core::ast::{AstStmt, BlockKind, FlowchartAst};

/// Formats a Flowchart AST back into standard, canonical Flowchart DSL code.
pub fn format_dsl(ast: &FlowchartAst) -> String {
    let mut out = String::new();

    if let Some(ref title) = ast.title {
        out.push_str(&format!("algorithm \"{}\"\n\n", title));
    }

    format_stmts(&ast.statements, 0, &mut out);

    out
}

fn format_stmts(stmts: &[AstStmt], indent_level: usize, out: &mut String) {
    let indent = "    ".repeat(indent_level);

    for stmt in stmts {
        match stmt {
            AstStmt::AlgorithmTitle(_) => {}
            AstStmt::Start(_) => {
                out.push_str(&format!("{}start\n", indent));
            }
            AstStmt::End(_) => {
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::Return(spanned) => match &spanned.value {
                Some(v) => out.push_str(&format!("{}return {}\n", indent, v)),
                None => out.push_str(&format!("{}return\n", indent)),
            },
            AstStmt::Process(s) => {
                out.push_str(&format!("{}set {}\n", indent, s.value));
            }
            AstStmt::Input(s) => {
                out.push_str(&format!("{}read {}\n", indent, s.value));
            }
            AstStmt::Output(s) => {
                out.push_str(&format!("{}print \"{}\"\n", indent, s.value));
            }
            AstStmt::Decision {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                out.push_str(&format!("{}if {} then\n", indent, condition.value));
                format_stmts(then_branch, indent_level + 1, out);
                if let Some(else_stmts) = else_branch {
                    out.push_str(&format!("{}else\n", indent));
                    format_stmts(else_stmts, indent_level + 1, out);
                }
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::Switch {
                target,
                cases,
                default_case,
                ..
            } => {
                out.push_str(&format!("{}switch {}\n", indent, target.value));
                for case in cases {
                    out.push_str(&format!("{}    case {}\n", indent, case.value.value));
                    format_stmts(&case.body, indent_level + 2, out);
                }
                if let Some(def_stmts) = default_case {
                    out.push_str(&format!("{}    default\n", indent));
                    format_stmts(def_stmts, indent_level + 2, out);
                }
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::LoopFor {
                var,
                from,
                to,
                body,
                ..
            } => {
                out.push_str(&format!(
                    "{}for {} from {} to {}\n",
                    indent, var.value, from.value, to.value
                ));
                format_stmts(body, indent_level + 1, out);
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::LoopWhile {
                condition, body, ..
            } => {
                out.push_str(&format!("{}while {}\n", indent, condition.value));
                format_stmts(body, indent_level + 1, out);
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::LoopRepeat {
                body, until_cond, ..
            } => {
                out.push_str(&format!("{}repeat\n", indent));
                format_stmts(body, indent_level + 1, out);
                out.push_str(&format!("{}until {}\n", indent, until_cond.value));
            }
            AstStmt::SubroutineCall { name, .. } => {
                out.push_str(&format!("{}call {}\n", indent, name.value));
            }
            AstStmt::Goto(s) => {
                out.push_str(&format!("{}goto {}\n", indent, s.value));
            }
            AstStmt::Label(s) => {
                out.push_str(&format!("{}label {}\n", indent, s.value));
            }
            AstStmt::OffpageLink(s) => {
                out.push_str(&format!("{}link \"{}\"\n", indent, s.value));
            }
            AstStmt::Document(s) => {
                out.push_str(&format!("{}document \"{}\"\n", indent, s.value));
            }
            AstStmt::Database(s) => {
                out.push_str(&format!("{}database \"{}\"\n", indent, s.value));
            }
            AstStmt::ManualInput(s) => {
                out.push_str(&format!("{}manual \"{}\"\n", indent, s.value));
            }
            AstStmt::Delay(s) => {
                out.push_str(&format!("{}wait \"{}\"\n", indent, s.value));
            }
            AstStmt::Display(s) => {
                out.push_str(&format!("{}display \"{}\"\n", indent, s.value));
            }
            AstStmt::Parallel { branches, .. } => {
                out.push_str(&format!("{}parallel\n", indent));
                for (i, branch) in branches.iter().enumerate() {
                    if i > 0 {
                        out.push_str(&format!("{}and\n", indent));
                    }
                    format_stmts(branch, indent_level + 1, out);
                }
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::Swimlane { name, body, .. } => {
                out.push_str(&format!("{}lane \"{}\"\n", indent, name.value));
                format_stmts(body, indent_level + 1, out);
                out.push_str(&format!("{}end\n", indent));
            }
            AstStmt::Note(s) => {
                out.push_str(&format!("{}# {}\n", indent, s.value));
            }
            AstStmt::RawNodeDef {
                id, kind, label, ..
            } => {
                let kind_str = match kind {
                    BlockKind::Start => "start",
                    BlockKind::End => "end",
                    BlockKind::Decision => "decision",
                    BlockKind::Input => "input",
                    BlockKind::Output => "output",
                    _ => "process",
                };
                out.push_str(&format!(
                    "{}node {}: {} \"{}\"\n",
                    indent, id.value, kind_str, label.value
                ));
            }
            AstStmt::RawEdge {
                from, to, label, ..
            } => {
                if let Some(lbl) = label {
                    out.push_str(&format!(
                        "{}{} -[{}]-> {}\n",
                        indent, from.value, lbl.value, to.value
                    ));
                } else {
                    out.push_str(&format!("{}{} -> {}\n", indent, from.value, to.value));
                }
            }
            AstStmt::ErrorRecovery { raw_text, .. } => {
                out.push_str(&format!("{}{}\n", indent, raw_text));
            }
        }
    }
}
