//! Interactive Canvas Renderer for Flowcharts.
//! Renders ISO 5807 blocks, orthogonal routed wires, ports, and interactive handles directly into the egui painter.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use crate::flowchart_core::ast::BlockKind;
use crate::flowchart_core::ir::FlowchartGraph;
use crate::flowchart_core::layout::LayoutEngine;
use crate::flowchart_core::parser::Parser;
use crate::flowchart_core::route::OrthogonalRouter;
use crate::model::FlowchartEntity;
use crate::theme::Palette;

pub struct FlowchartRenderer;

impl FlowchartRenderer {
    /// Render an entire Flowchart entity on the whiteboard canvas.
    pub fn render(
        painter: &egui::Painter,
        entity: &FlowchartEntity,
        scale: f32,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        palette: &Palette,
        is_dark: bool,
        is_selected: bool,
    ) -> (Option<Rect>, bool) {
        let card_min = to_screen(Pos2::new(entity.x, entity.y));
        let card_size = Vec2::new(entity.width * scale, entity.height * scale);
        let card_rect = Rect::from_min_size(card_min, card_size);

        // 1. Draw Card Background Frame
        let bg_color = if is_dark {
            Color32::from_rgba_unmultiplied(31, 28, 25, 235)
        } else {
            Color32::from_rgba_unmultiplied(253, 252, 250, 245)
        };

        // Subtle drop shadow if selected
        if is_selected {
            let shadow_rect = card_rect.translate(Vec2::new(0.0, 4.0 * scale));
            painter.rect_filled(shadow_rect, 10.0, palette.accent_soft);
        }

        painter.rect_filled(card_rect, 8.0, bg_color);
        let border_stroke = if is_selected {
            Stroke::new(2.0_f32, palette.accent)
        } else {
            Stroke::new(1.0_f32, palette.border)
        };
        painter.rect_stroke(card_rect, 8.0, border_stroke);

        // 2. Flowchart Header
        let header_h = (28.0 * scale).clamp(24.0, 36.0);
        let header_rect =
            Rect::from_min_size(card_rect.min, Vec2::new(card_rect.width(), header_h));
        let header_bg = if is_dark {
            Color32::from_rgba_unmultiplied(41, 38, 34, 200)
        } else {
            Color32::from_rgba_unmultiplied(240, 235, 226, 220)
        };
        painter.rect_filled(header_rect, 8.0, header_bg);

        // Title icon + text
        let title_text = if !entity.title.is_empty() {
            format!("⚡ {}", entity.title)
        } else {
            "⚡ Flowchart".to_string()
        };
        painter.text(
            header_rect.left_center() + Vec2::new(10.0 * scale, 0.0),
            egui::Align2::LEFT_CENTER,
            &title_text,
            egui::FontId::proportional((12.0 * scale).clamp(10.0, 14.0)),
            palette.text,
        );

        // Edit Code Pill Button (Top Right of header)
        let edit_btn_w = 46.0 * scale;
        let edit_btn_rect = Rect::from_min_size(
            header_rect.right_center() + Vec2::new(-edit_btn_w - 32.0 * scale, -9.0 * scale),
            Vec2::new(edit_btn_w, 18.0 * scale),
        );
        painter.rect_filled(edit_btn_rect, 4.0, palette.accent_soft);
        painter.text(
            edit_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Edit",
            egui::FontId::proportional((10.0 * scale).max(9.0)),
            palette.accent,
        );

        // Delete '×' Button
        let close_center = header_rect.right_center() + Vec2::new(-14.0 * scale, 0.0);
        painter.text(
            close_center,
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional((14.0 * scale).clamp(12.0, 16.0)),
            palette.text_muted,
        );

        // 3. Build or Obtain Flowchart Graph
        let graph = if let Some(g) = &entity.graph {
            g.clone()
        } else {
            let (ast, _) = Parser::from_str(&entity.dsl_source).parse();
            FlowchartGraph::from_ast(&ast)
        };

        if graph.nodes.is_empty() {
            painter.text(
                card_rect.center() + Vec2::new(0.0, header_h * 0.5),
                egui::Align2::CENTER_CENTER,
                "Double click or press 'Edit' to write flowchart DSL",
                egui::FontId::proportional((13.0 * scale).clamp(11.0, 15.0)),
                palette.text_muted,
            );
            return (Some(card_rect), false);
        }

        // 4. Compute Layout
        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.compute_layout(&graph);

        // 5. Compute Scaling and Centering inside Flowchart Card Canvas
        let inner_rect = Rect::from_min_max(
            card_rect.min + Vec2::new(16.0 * scale, header_h + 12.0 * scale),
            card_rect.max - Vec2::new(16.0 * scale, 16.0 * scale),
        );

        let layout_w = layout.total_width.max(100.0);
        let layout_h = layout.total_height.max(100.0);
        let fit_scale = (inner_rect.width() / layout_w)
            .min(inner_rect.height() / layout_h)
            .clamp(0.4, 1.25);

        let content_origin = inner_rect.min
            + Vec2::new(
                (inner_rect.width() - layout_w * fit_scale) * 0.5,
                (inner_rect.height() - layout_h * fit_scale) * 0.5,
            );

        let to_card_screen = |lx: f32, ly: f32| -> Pos2 {
            Pos2::new(
                content_origin.x + lx * fit_scale,
                content_origin.y + ly * fit_scale,
            )
        };

        // 6. Route and Render Edges
        let router = OrthogonalRouter {
            corner_radius: 6.0,
            ..Default::default()
        };
        let routed_edges = router.route_all(&graph, &layout.node_bounds);

        let wire_color = if is_dark {
            Color32::from_rgb(220, 215, 205)
        } else {
            Color32::from_rgb(50, 48, 44)
        };
        let wire_stroke = Stroke::new((1.6 * scale * fit_scale).clamp(1.2, 2.4), wire_color);

        for edge in &routed_edges {
            if edge.points.len() < 2 {
                continue;
            }

            // Draw line segments
            for pair in edge.points.windows(2) {
                let p1 = to_card_screen(pair[0][0], pair[0][1]);
                let p2 = to_card_screen(pair[1][0], pair[1][1]);
                painter.line_segment([p1, p2], wire_stroke);
            }

            // Draw Arrowhead at final target
            let last_idx = edge.points.len() - 1;
            let p_prev = to_card_screen(edge.points[last_idx - 1][0], edge.points[last_idx - 1][1]);
            let p_target = to_card_screen(edge.points[last_idx][0], edge.points[last_idx][1]);
            draw_arrowhead(painter, p_prev, p_target, 7.0 * fit_scale, wire_color);

            // Draw Edge Label badge
            if let Some(lbl) = &edge.label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    let lbl_pos = to_card_screen(edge.label_pos[0], edge.label_pos[1]);
                    let font_id =
                        egui::FontId::proportional((10.0 * scale * fit_scale).clamp(8.0, 12.0));
                    let text_color = palette.text;
                    let badge_bg = if is_dark {
                        Color32::from_rgb(45, 42, 38)
                    } else {
                        Color32::from_rgb(245, 242, 236)
                    };
                    let badge_w = (trimmed.len() as f32) * 7.0 * fit_scale + 12.0;
                    let badge_rect =
                        Rect::from_center_size(lbl_pos, Vec2::new(badge_w, 15.0 * fit_scale));

                    painter.rect_filled(badge_rect, 3.0, badge_bg);
                    painter.rect_stroke(badge_rect, 3.0, Stroke::new(0.8_f32, palette.border));
                    painter.text(
                        lbl_pos,
                        egui::Align2::CENTER_CENTER,
                        trimmed,
                        font_id,
                        text_color,
                    );
                }
            }
        }

        // 7. Render ISO 5807 Nodes
        for node in &graph.nodes {
            if let Some(nb) = layout.node_bounds.get(&node.id) {
                let node_screen_min = to_card_screen(nb.x, nb.y);
                let node_screen_size = Vec2::new(nb.width * fit_scale, nb.height * fit_scale);
                let node_rect = Rect::from_min_size(node_screen_min, node_screen_size);

                render_node_vector(
                    painter,
                    node.kind,
                    node_rect,
                    &node.label,
                    palette,
                    is_dark,
                    fit_scale,
                );
            }
        }

        // 8. Resize Handle at bottom right
        if is_selected {
            let handle_rect = Rect::from_min_size(
                card_rect.right_bottom() - Vec2::new(14.0, 14.0),
                Vec2::new(14.0, 14.0),
            );
            painter.rect_filled(handle_rect, 2.0, palette.accent);
        }

        (Some(card_rect), false)
    }
}

/// Renders crisp ISO 5807 vector shapes for each block kind.
fn render_node_vector(
    painter: &egui::Painter,
    kind: BlockKind,
    rect: Rect,
    label: &str,
    palette: &Palette,
    is_dark: bool,
    scale: f32,
) {
    let stroke_color = if is_dark {
        Color32::from_rgb(220, 215, 205)
    } else {
        Color32::from_rgb(45, 42, 38)
    };
    let stroke = Stroke::new((1.6 * scale).clamp(1.2, 2.2), stroke_color);

    let default_bg = if is_dark {
        Color32::from_rgb(43, 40, 36)
    } else {
        Color32::from_rgb(255, 254, 252)
    };

    let decision_bg = if is_dark {
        Color32::from_rgb(52, 46, 36)
    } else {
        Color32::from_rgb(254, 248, 235)
    };

    let terminal_bg = if is_dark {
        Color32::from_rgb(48, 44, 38)
    } else {
        Color32::from_rgb(243, 239, 232)
    };

    match kind {
        BlockKind::Start | BlockKind::End => {
            // Stadium / pill shape
            let rx = rect.height() * 0.5;
            painter.rect_filled(rect, rx, terminal_bg);
            painter.rect_stroke(rect, rx, stroke);
        }
        BlockKind::Decision => {
            // Diamond polygon
            let p_top = Pos2::new(rect.center().x, rect.min.y);
            let p_right = Pos2::new(rect.max.x, rect.center().y);
            let p_bottom = Pos2::new(rect.center().x, rect.max.y);
            let p_left = Pos2::new(rect.min.x, rect.center().y);

            let points = vec![p_top, p_right, p_bottom, p_left];
            painter.add(egui::Shape::convex_polygon(
                points.clone(),
                decision_bg,
                stroke,
            ));
        }
        BlockKind::Input | BlockKind::Output => {
            // Parallelogram
            let skew = 12.0 * scale;
            let p1 = Pos2::new(rect.min.x + skew, rect.min.y);
            let p2 = Pos2::new(rect.max.x, rect.min.y);
            let p3 = Pos2::new(rect.max.x - skew, rect.max.y);
            let p4 = Pos2::new(rect.min.x, rect.max.y);

            let points = vec![p1, p2, p3, p4];
            painter.add(egui::Shape::convex_polygon(points, default_bg, stroke));
        }
        BlockKind::Subroutine => {
            // Rect with double vertical bars
            painter.rect_filled(rect, 4.0, default_bg);
            painter.rect_stroke(rect, 4.0, stroke);
            let bar_offset = 10.0 * scale;
            painter.line_segment(
                [
                    Pos2::new(rect.min.x + bar_offset, rect.min.y),
                    Pos2::new(rect.min.x + bar_offset, rect.max.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(rect.max.x - bar_offset, rect.min.y),
                    Pos2::new(rect.max.x - bar_offset, rect.max.y),
                ],
                stroke,
            );
        }
        BlockKind::Connector => {
            // Circular badge
            let r = rect.width().min(rect.height()) * 0.5;
            painter.circle_filled(rect.center(), r, terminal_bg);
            painter.circle_stroke(rect.center(), r, stroke);
        }
        BlockKind::LoopFor | BlockKind::LoopWhile | BlockKind::LoopRepeat => {
            // Preparation hexagon
            let indent = 12.0 * scale;
            let p1 = Pos2::new(rect.min.x + indent, rect.min.y);
            let p2 = Pos2::new(rect.max.x - indent, rect.min.y);
            let p3 = Pos2::new(rect.max.x, rect.center().y);
            let p4 = Pos2::new(rect.max.x - indent, rect.max.y);
            let p5 = Pos2::new(rect.min.x + indent, rect.max.y);
            let p6 = Pos2::new(rect.min.x, rect.center().y);

            let points = vec![p1, p2, p3, p4, p5, p6];
            painter.add(egui::Shape::convex_polygon(points, default_bg, stroke));
        }
        _ => {
            // Standard Process rounded rectangle
            painter.rect_filled(rect, 5.0, default_bg);
            painter.rect_stroke(rect, 5.0, stroke);
        }
    }

    // Label Text
    let font_size = (11.0 * scale).clamp(9.0, 13.0);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        palette.text,
    );
}

/// Draw a vector directional arrowhead at the target port.
fn draw_arrowhead(painter: &egui::Painter, from: Pos2, to: Pos2, size: f32, color: Color32) {
    let dir = (to - from).normalized();
    if dir.x.is_nan() || dir.y.is_nan() {
        return;
    }
    let normal = Vec2::new(-dir.y, dir.x);
    let p_base = to - dir * size;
    let p_left = p_base + normal * (size * 0.5);
    let p_right = p_base - normal * (size * 0.5);

    let points = vec![to, p_left, p_right];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}
