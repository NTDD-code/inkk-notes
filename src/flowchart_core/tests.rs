#[cfg(test)]
mod tests {
    use crate::flowchart_core::ast::BlockKind;
    use crate::flowchart_core::ir::FlowchartGraph;
    use crate::flowchart_core::parser::Parser;
    use crate::flowchart_core::print::format_dsl;

    #[test]
    fn test_01_simple_algorithm_parse() {
        let code = r#"
algorithm "Find maximum"

start
read n, A
set max = A[0]
print max
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty(), "Errors: {:?}", errors);
        assert_eq!(ast.title.as_deref(), Some("Find maximum"));
        assert_eq!(ast.statements.len(), 6);
    }

    #[test]
    fn test_02_structured_if_else() {
        let code = r#"
start
input x
if x > 0 then
    print "Positive"
else
    print "Non-positive"
end
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert_eq!(graph.nodes.len(), 6);
    }

    #[test]
    fn test_03_structured_for_loop() {
        let code = r#"
start
set sum = 0
for i from 1 to 10
    set sum = sum + i
end
print sum
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert!(graph.edges.iter().any(|e| e.is_back_edge));
    }

    #[test]
    fn test_04_while_loop() {
        let code = r#"
start
while count < 10
    set count = count + 1
end
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert!(graph.edges.iter().any(|e| e.is_back_edge));
    }

    #[test]
    fn test_05_repeat_until_loop() {
        let code = r#"
start
repeat
    read guess
until guess == secret
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert!(graph.edges.iter().any(|e| e.is_back_edge));
    }

    #[test]
    fn test_06_switch_case() {
        let code = r#"
start
switch grade
    case A
        print "Excellent"
    case B
        print "Good"
    default
        print "Pass"
end
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        // Start(1) + Switch(1) + Print(3) + End(1) = 6 nodes
        assert_eq!(graph.nodes.len(), 6);
    }

    #[test]
    fn test_07_goto_and_label() {
        let code = r#"
label retry
input pw
if pw is wrong then
    goto retry
else
    output "Welcome"
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert!(graph.labels.contains_key("retry"));
        assert!(graph.edges.iter().any(|e| e.is_back_edge));
    }

    #[test]
    fn test_08_raw_graph_mode() {
        let code = r#"
node a: process "Init"
node d: decision "x > 0?"
node b: output "Positive"
node c: output "Negative"
a -> d
d -[Yes]-> b
d -[No]-> c
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn test_09_all_fifteen_iso_blocks() {
        let code = r#"
start
set x = 1
input a
output b
document "Invoice"
database "Users"
manual "Enter PIN"
wait "5s"
display "Done"
call compute(a)
link "Page2"
return 0
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert_eq!(graph.nodes.len(), 13);
    }

    #[test]
    fn test_10_parallel_fork_join() {
        let code = r#"
start
parallel
    set task1 = 1
and
    set task2 = 2
end
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert!(graph.nodes.iter().any(|n| n.kind == BlockKind::Parallel));
    }

    #[test]
    fn test_11_swimlanes() {
        let code = r#"
lane "User"
    input card
end
lane "ATM"
    call verify()
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert_eq!(graph.nodes.len(), 4);
    }

    #[test]
    fn test_12_nested_loops_and_conditionals() {
        let code = r#"
start
for i from 1 to n
    for j from 1 to m
        if A[i][j] == target then
            print "Found"
            return
        end
    end
end
end
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert!(graph.nodes.len() >= 6);
    }

    #[test]
    fn test_13_round_trip_format_and_parse() {
        let code = r#"algorithm "Find maximum"

start
read n, A
set max = A[0]
for i from 1 to n - 1
    if A[i] > max then
        set max = A[i]
    end
end
print "max"
end
"#;
        let (ast1, _) = Parser::from_str(code).parse();
        let formatted = format_dsl(&ast1);
        let (ast2, _) = Parser::from_str(&formatted).parse();
        assert_eq!(ast1.title, ast2.title);
        assert_eq!(ast1.statements.len(), ast2.statements.len());
    }

    #[test]
    fn test_14_error_recovery_preserves_valid_parts() {
        let code = r#"
start
read a
this is a completely broken line !@#$%
print a
end
"#;
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        // Ensure Start, Read, Error node, Print, and End all got represented
        assert!(graph.nodes.len() >= 4);
        assert!(graph.nodes.iter().any(|n| n.label.contains("broken")));
    }

    #[test]
    fn test_15_case_insensitivity_and_aliases() {
        let code = r#"
START
LET a = 5
READ b
SHOW "result"
END
"#;
        let (ast, errors) = Parser::from_str(code).parse();
        assert!(errors.is_empty());
        let graph = FlowchartGraph::from_ast(&ast);
        assert_eq!(graph.nodes[0].kind, BlockKind::Start);
        assert_eq!(graph.nodes[1].kind, BlockKind::Process);
        assert_eq!(graph.nodes[2].kind, BlockKind::Input);
        assert_eq!(graph.nodes[3].kind, BlockKind::Output);
        assert_eq!(graph.nodes[4].kind, BlockKind::End);
    }

    #[test]
    fn test_16_diagnostic_missing_start_and_end() {
        use super::super::check::FlowchartChecker;
        let code = r#"
set a = 1
print a
"#;
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        let diags = FlowchartChecker::check(&graph, Some(&ast));
        assert!(diags.iter().any(|d| d.code == "F0101")); // Missing start
        assert!(diags.iter().any(|d| d.code == "F0105")); // Missing end
    }

    #[test]
    fn test_17_diagnostic_unresolved_goto_and_duplicate_label() {
        use super::super::check::FlowchartChecker;
        let code = r#"
start
label loop_start:
set x = x + 1
label loop_start:
goto nonexistent_lbl
end
"#;
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        let diags = FlowchartChecker::check(&graph, Some(&ast));
        assert!(diags.iter().any(|d| d.code == "F0114")); // Duplicate label
        assert!(diags.iter().any(|d| d.code == "F0113")); // Unresolved goto
    }

    #[test]
    fn test_18_rustc_style_render_diagnostic() {
        use super::super::check::FlowchartChecker;
        let code = "start\ngoto unknown_target\nend";
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        let diags = FlowchartChecker::check(&graph, Some(&ast));
        let goto_diag = diags
            .iter()
            .find(|d| d.code == "F0113")
            .expect("should find F0113");
        let rendered = goto_diag.render_rustc(code);
        assert!(rendered.contains("error[F0113]"));
        assert!(rendered.contains("--> 2:"));
        assert!(rendered.contains("goto unknown_target"));
        assert!(rendered.contains("^"));
        assert!(rendered.contains("help:"));
    }

    #[test]
    fn test_19_layout_hierarchical_ranks() {
        use super::super::layout::LayoutEngine;
        let code = r#"
start
read x
if x > 0 then
    print "positive"
else
    print "non-positive"
end
end
"#;
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        let engine = LayoutEngine::new();
        let layout = engine.compute_layout(&graph);

        assert_eq!(layout.node_bounds.len(), graph.nodes.len());
        // Verify Start is at the top (lowest Y) and End is near bottom (highest Y)
        let start_node = graph.start_nodes()[0];
        let end_node = graph.end_nodes()[0];
        let start_y = layout.node_bounds[&start_node.id].y;
        let end_y = layout.node_bounds[&end_node.id].y;
        assert!(start_y < end_y, "Start node should be above End node");
    }

    #[test]
    fn test_20_orthogonal_routing_segments() {
        use super::super::layout::LayoutEngine;
        use super::super::route::OrthogonalRouter;
        let code = r#"
start
read x
set y = x * 2
print y
end
"#;
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        let layout = LayoutEngine::new().compute_layout(&graph);
        let router = OrthogonalRouter::new();
        let routed = router.route_all(&graph, &layout.node_bounds);

        assert_eq!(routed.len(), graph.edges.len());
        for edge in routed {
            assert!(edge.points.len() >= 2);
            // Verify every segment is purely horizontal or vertical
            for pair in edge.points.windows(2) {
                let dx = (pair[0][0] - pair[1][0]).abs();
                let dy = (pair[0][1] - pair[1][1]).abs();
                assert!(
                    dx < 0.01 || dy < 0.01,
                    "Segment from {:?} to {:?} must be orthogonal",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    #[test]
    fn test_21_svg_and_mermaid_export() {
        use super::super::export::FlowchartExporter;
        let code = r#"algorithm "Sample Flow"
start
input a
if a > 10 then
    output "large"
else
    output "small"
end
end
"#;
        let (ast, _) = Parser::from_str(code).parse();
        let graph = FlowchartGraph::from_ast(&ast);

        let svg = FlowchartExporter::to_svg(&graph, None, None);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("marker id=\"arrow\""));
        assert!(svg.contains("Sample Flow"));
        assert!(svg.contains("</svg>"));

        let mermaid = FlowchartExporter::to_mermaid(&graph);
        assert!(mermaid.starts_with("graph TD"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_22_interactive_visual_node_placement_and_connection() {
        use crate::flowchart_core::ir::PortSide;

        let mut graph = FlowchartGraph::new();

        // 1. User clicks "Start" (Oval) in shapes menu
        let start_id = graph.add_visual_node(BlockKind::Start, "Start".to_string(), (60.0, 60.0));
        assert_eq!(graph.nodes.len(), 1);

        // 2. User clicks right anchor dot of Start -> selects Decision (Diamond)
        let decision_id = graph.add_visual_node(
            BlockKind::Decision,
            "Is Valid?".to_string(),
            (240.0, 60.0),
        );
        let e1 = graph.add_edge(start_id, PortSide::Right, decision_id, PortSide::Left, None);
        assert_eq!(e1, 1);

        // 3. User clicks bottom anchor dot of Decision -> selects Process (Rectangle)
        let process_id = graph.add_visual_node(
            BlockKind::Process,
            "Process Data".to_string(),
            (240.0, 190.0),
        );
        let e2 = graph.add_edge(decision_id, PortSide::Bottom, process_id, PortSide::Top, None);
        assert_eq!(e2, 2);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        let (min_x, min_y, max_x, max_y) = graph.compute_bounds();
        assert_eq!(min_x, 60.0);
        assert_eq!(min_y, 60.0);
        assert!(max_x >= 240.0 + 130.0);
        assert!(max_y >= 190.0 + 50.0);
    }

    #[test]
    fn test_23_visual_node_deletion_and_edge_cleanup() {
        use crate::flowchart_core::ir::PortSide;

        let mut graph = FlowchartGraph::new();
        let n1 = graph.add_visual_node(BlockKind::Start, "Start".into(), (50.0, 50.0));
        let n2 = graph.add_visual_node(BlockKind::Process, "Step".into(), (200.0, 50.0));
        let n3 = graph.add_visual_node(BlockKind::End, "End".into(), (350.0, 50.0));

        graph.add_edge(n1, PortSide::Right, n2, PortSide::Left, None);
        graph.add_edge(n2, PortSide::Right, n3, PortSide::Left, None);
        assert_eq!(graph.edges.len(), 2);

        // Deleting n2 must delete both connected edges!
        graph.remove_node(n2);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_24_graph_shift_all_nodes() {
        let mut graph = FlowchartGraph::new();
        graph.add_visual_node(BlockKind::Start, "Start".into(), (10.0, 20.0));
        graph.add_visual_node(BlockKind::End, "End".into(), (100.0, 200.0));

        graph.shift_all_nodes(50.0, 30.0);
        assert_eq!(graph.nodes[0].pinned_pos, Some((60.0, 50.0)));
        assert_eq!(graph.nodes[1].pinned_pos, Some((150.0, 230.0)));
    }
}
