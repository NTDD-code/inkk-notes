//! Flowchart Exporter: Headless SVG rendering, Mermaid syntax export, and Pseudocode generator.

use super::ast::BlockKind;
use super::ir::FlowchartGraph;
use super::layout::{LayoutEngine, LayoutResult, Rect};
use super::route::OrthogonalRouter;

pub struct SvgExportOptions {
    pub show_grid: bool,
    pub dark_mode: bool,
    pub corner_radius: f32,
    pub padding: f32,
}

impl Default for SvgExportOptions {
    fn default() -> Self {
        Self {
            show_grid: false,
            dark_mode: false,
            corner_radius: 8.0,
            padding: 40.0,
        }
    }
}

pub struct FlowchartExporter;

impl FlowchartExporter {
    /// Render flowchart to standalone scalable SVG.
    pub fn to_svg(
        graph: &FlowchartGraph,
        layout: Option<&LayoutResult>,
        options: Option<SvgExportOptions>,
    ) -> String {
        let opts = options.unwrap_or_default();

        // Compute layout if not provided
        let default_layout;
        let layout_ref = if let Some(l) = layout {
            l
        } else {
            let engine = LayoutEngine::new();
            default_layout = engine.compute_layout(graph);
            &default_layout
        };

        let router = OrthogonalRouter {
            corner_radius: opts.corner_radius,
            ..Default::default()
        };
        let routed_edges = router.route_all(graph, &layout_ref.node_bounds);

        let pad = opts.padding;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for r in layout_ref.node_bounds.values() {
            min_x = min_x.min(r.x);
            min_y = min_y.min(r.y);
            max_x = max_x.max(r.x + r.width);
            max_y = max_y.max(r.y + r.height);
        }

        for edge in &routed_edges {
            for pt in &edge.points {
                min_x = min_x.min(pt[0]);
                min_y = min_y.min(pt[1]);
                max_x = max_x.max(pt[0]);
                max_y = max_y.max(pt[1]);
            }
        }

        if min_x > max_x {
            min_x = 0.0;
            min_y = 0.0;
            max_x = 400.0;
            max_y = 300.0;
        }

        let vb_x = min_x - pad;
        let vb_y = min_y - pad;
        let vb_w = (max_x - min_x) + pad * 2.0;
        let vb_h = (max_y - min_y) + pad * 2.0;

        let bg_color = if opts.dark_mode { "#1E1D1A" } else { "#FFFFFF" };
        let text_color = if opts.dark_mode { "#ECEAE4" } else { "#1F1E1B" };
        let stroke_color = if opts.dark_mode { "#E0DDD5" } else { "#2D2A26" };
        let node_bg = if opts.dark_mode { "#2B2925" } else { "#FDFCFA" };
        let decision_bg = if opts.dark_mode { "#383226" } else { "#FEF9EE" };
        let terminal_bg = if opts.dark_mode { "#33302B" } else { "#F4F1EA" };

        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.1} {:.1} {:.1} {:.1}" width="{:.1}" height="{:.1}" style="background-color: {}; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">"#,
            vb_x, vb_y, vb_w, vb_h, vb_w, vb_h, bg_color
        ));
        svg.push('\n');

        // SVG Markers (Arrowhead)
        svg.push_str(&format!(
            r#"  <defs>
    <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
      <path d="M 0 1.5 L 8 5 L 0 8.5 z" fill="{}" />
    </marker>
  </defs>
"#,
            stroke_color
        ));

        // Optional Title
        if let Some(title) = &graph.title {
            svg.push_str(&format!(
                r#"  <text x="{:.1}" y="{:.1}" font-size="18" font-weight="bold" fill="{}" text-anchor="middle">{}</text>"#,
                vb_x + vb_w * 0.5,
                vb_y + 26.0,
                text_color,
                escape_xml(title)
            ));
            svg.push('\n');
        }

        // Render Edges
        svg.push_str("  <g id=\"edges\">\n");
        for edge in &routed_edges {
            let path_d = edge.to_svg_path(opts.corner_radius);
            svg.push_str(&format!(
                r#"    <path d="{}" fill="none" stroke="{}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" marker-end="url(#arrow)" />"#,
                path_d, stroke_color
            ));
            svg.push('\n');

            // Edge label
            if let Some(lbl) = &edge.label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    let lx = edge.label_pos[0];
                    let ly = edge.label_pos[1];
                    let escaped_lbl = escape_xml(trimmed);
                    let badge_w = (escaped_lbl.len() as f32) * 7.5 + 14.0;
                    svg.push_str(&format!(
                        r#"    <g transform="translate({:.1}, {:.1})">
      <rect x="-{:.1}" y="-9" width="{:.1}" height="18" rx="4" fill="{}" stroke="{}" stroke-width="0.8" opacity="0.95" />
      <text x="0" y="4" font-family="Arial, Helvetica, sans-serif" font-size="11" fill="{}" text-anchor="middle" font-weight="500">{}</text>
    </g>
"#,
                        lx, ly, badge_w * 0.5, badge_w, bg_color, stroke_color, text_color, escaped_lbl
                    ));
                }
            }
        }
        svg.push_str("  </g>\n");

        // Render Nodes
        svg.push_str("  <g id=\"nodes\">\n");
        for node in &graph.nodes {
            if let Some(bounds) = layout_ref.node_bounds.get(&node.id) {
                let shape_svg = render_node_shape(
                    node.kind,
                    bounds,
                    &node.label,
                    node_bg,
                    decision_bg,
                    terminal_bg,
                    stroke_color,
                    text_color,
                );
                svg.push_str(&shape_svg);
            }
        }
        svg.push_str("  </g>\n");

        svg.push_str("</svg>\n");
        svg
    }

    /// Export graph to standard Mermaid flowchart syntax (`graph TD`).
    pub fn to_mermaid(graph: &FlowchartGraph) -> String {
        let mut out = String::new();
        out.push_str("graph TD\n");

        // Nodes definitions
        for node in &graph.nodes {
            let safe_id = format!("N{}", node.id.0);
            let lbl = node.label.replace('"', "&quot;");

            match node.kind {
                BlockKind::Start | BlockKind::End => {
                    out.push_str(&format!("    {safe_id}([\"{lbl}\"])\n"));
                }
                BlockKind::Decision => {
                    out.push_str(&format!("    {safe_id}{{\"{lbl}\"}}\n"));
                }
                BlockKind::Input | BlockKind::Output => {
                    out.push_str(&format!("    {safe_id}[/\"{lbl}\"/]\n"));
                }
                BlockKind::Subroutine => {
                    out.push_str(&format!("    {safe_id}[[\"{lbl}\"]]\n"));
                }
                BlockKind::Database => {
                    out.push_str(&format!("    {safe_id}[(\"{lbl}\")]\n"));
                }
                _ => {
                    out.push_str(&format!("    {safe_id}[\"{lbl}\"]\n"));
                }
            }
        }

        // Edges definitions
        for edge in &graph.edges {
            let from_id = format!("N{}", edge.from.0);
            let to_id = format!("N{}", edge.to.0);
            if let Some(lbl) = &edge.label {
                out.push_str(&format!("    {from_id} -->|{lbl}| {to_id}\n"));
            } else {
                out.push_str(&format!("    {from_id} --> {to_id}\n"));
            }
        }

        out
    }
}

fn render_node_shape(
    kind: BlockKind,
    b: &Rect,
    label: &str,
    node_bg: &str,
    decision_bg: &str,
    terminal_bg: &str,
    stroke_color: &str,
    text_color: &str,
) -> String {
    let mut out = String::new();
    let cx = b.x + b.width * 0.5;
    let cy = b.y + b.height * 0.5;
    let escaped_lbl = escape_xml(label);

    match kind {
        BlockKind::Start | BlockKind::End => {
            let rx = b.height * 0.5;
            out.push_str(&format!(
                r#"    <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="{:.1}" ry="{:.1}" fill="{}" stroke="{}" stroke-width="1.8" />
"#,
                b.x, b.y, b.width, b.height, rx, rx, terminal_bg, stroke_color
            ));
        }
        BlockKind::Decision => {
            let p_top = format!("{:.1},{:.1}", cx, b.y);
            let p_right = format!("{:.1},{:.1}", b.x + b.width, cy);
            let p_bottom = format!("{:.1},{:.1}", cx, b.y + b.height);
            let p_left = format!("{:.1},{:.1}", b.x, cy);
            out.push_str(&format!(
                r#"    <polygon points="{} {} {} {}" fill="{}" stroke="{}" stroke-width="1.8" />
"#,
                p_top, p_right, p_bottom, p_left, decision_bg, stroke_color
            ));
        }
        BlockKind::Input | BlockKind::Output => {
            let skew = 14.0f32;
            let p1 = format!("{:.1},{:.1}", b.x + skew, b.y);
            let p2 = format!("{:.1},{:.1}", b.x + b.width, b.y);
            let p3 = format!("{:.1},{:.1}", b.x + b.width - skew, b.y + b.height);
            let p4 = format!("{:.1},{:.1}", b.x, b.y + b.height);
            out.push_str(&format!(
                r#"    <polygon points="{} {} {} {}" fill="{}" stroke="{}" stroke-width="1.8" />
"#,
                p1, p2, p3, p4, node_bg, stroke_color
            ));
        }
        BlockKind::Subroutine => {
            out.push_str(&format!(
                r#"    <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="4" fill="{}" stroke="{}" stroke-width="1.8" />
    <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="1.5" />
    <line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="1.5" />
"#,
                b.x, b.y, b.width, b.height, node_bg, stroke_color,
                b.x + 12.0, b.y, b.x + 12.0, b.y + b.height, stroke_color,
                b.x + b.width - 12.0, b.y, b.x + b.width - 12.0, b.y + b.height, stroke_color
            ));
        }
        BlockKind::Connector => {
            let r = b.width.min(b.height) * 0.5;
            out.push_str(&format!(
                r#"    <circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" stroke="{}" stroke-width="1.8" />
"#,
                cx, cy, r, terminal_bg, stroke_color
            ));
        }
        BlockKind::Database => {
            out.push_str(&format!(
                r#"    <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="8" fill="{}" stroke="{}" stroke-width="1.8" />
"#,
                b.x, b.y, b.width, b.height, node_bg, stroke_color
            ));
        }
        _ => {
            // Standard Process rectangle
            out.push_str(&format!(
                r#"    <rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="6" fill="{}" stroke="{}" stroke-width="1.8" />
"#,
                b.x, b.y, b.width, b.height, node_bg, stroke_color
            ));
        }
    }

    // Label Text
    out.push_str(&format!(
        r#"    <text x="{:.1}" y="{:.1}" font-family="Arial, Helvetica, sans-serif" font-size="12" fill="{}" text-anchor="middle" dominant-baseline="central" font-weight="500">{}</text>
"#,
        cx, cy, text_color, escaped_lbl
    ));

    out
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
