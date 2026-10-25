//! Interactive Canvas Renderer for Flowcharts.
//! Renders ISO 5807 blocks, orthogonal routed wires, ports, and interactive connection handles
//! directly on the whiteboard canvas.

use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use std::collections::HashMap;

use crate::flowchart_core::ast::BlockKind;
use crate::flowchart_core::ir::{FlowchartGraph, NodeId, PortSide};
use crate::flowchart_core::layout::LayoutEngine;
use crate::flowchart_core::parser::Parser;
use crate::flowchart_core::route::OrthogonalRouter;
use crate::flowchart_ui::palette::FlowchartUiState;
use crate::model::FlowchartEntity;
use crate::theme::Palette;
use crate::vector_icon::VectorIcon;

pub struct FlowchartRenderer;

impl FlowchartRenderer {
    /// Render an entire Flowchart entity on the whiteboard canvas with nodes, wires, and anchor connection dots.
    pub fn render(
        painter: &egui::Painter,
        ui: &mut egui::Ui,
        entity: &FlowchartEntity,
        state: &mut FlowchartUiState,
        scale: f32,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        palette: &Palette,
        is_dark: bool,
        is_selected: bool,
        pointer_pos: Option<Pos2>,
    ) -> Option<Rect> {
        let card_min = to_screen(Pos2::new(entity.x, entity.y));
        let card_size = Vec2::new(entity.width * scale, entity.height * scale);
        let card_rect = Rect::from_min_size(card_min, card_size);

        // 1. Draw Card Background Frame
        let bg_color = if is_dark {
            Color32::from_rgba_unmultiplied(31, 28, 25, 235)
        } else {
            Color32::from_rgba_unmultiplied(253, 252, 250, 245)
        };

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

        // Header Vector Icon
        let icon_box = Rect::from_min_size(
            header_rect.left_center() + Vec2::new(8.0 * scale, -9.0 * scale),
            Vec2::splat(18.0 * scale),
        );
        VectorIcon::Flowchart.paint(painter, icon_box, palette.accent, 1.8);

        // Title text
        let title_text = if !entity.title.is_empty() {
            entity.title.clone()
        } else {
            "Flowchart".to_string()
        };
        painter.text(
            header_rect.left_center() + Vec2::new(32.0 * scale, 0.0),
            egui::Align2::LEFT_CENTER,
            &title_text,
            egui::FontId::proportional((12.0 * scale).clamp(10.0, 14.0)),
            palette.text,
        );

        // Edit Code Pill Button
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

        // Close 'x' Button
        let close_center = header_rect.right_center() + Vec2::new(-14.0 * scale, 0.0);
        painter.text(
            close_center,
            egui::Align2::CENTER_CENTER,
            "x",
            egui::FontId::proportional((14.0 * scale).clamp(12.0, 16.0)),
            palette.text_muted,
        );

        // 3. Obtain Flowchart Graph
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
                "Select shapes from left panel to build flowchart",
                egui::FontId::proportional((13.0 * scale).clamp(11.0, 15.0)),
                palette.text_muted,
            );
            return Some(card_rect);
        }

        // 4. Compute layout for any unpinned nodes
        let layout_engine = LayoutEngine::new();
        let layout = layout_engine.compute_layout(&graph);

        let mut node_screen_rects: HashMap<NodeId, Rect> = HashMap::new();
        let mut layout_bounds: HashMap<NodeId, crate::flowchart_core::layout::Rect> = HashMap::new();

        for node in &graph.nodes {
            let (nx, ny) = if let Some(pin) = node.pinned_pos {
                pin
            } else if let Some(nb) = layout.node_bounds.get(&node.id) {
                (nb.x, nb.y)
            } else {
                (60.0, 60.0)
            };

            let node_screen_min = to_screen(Pos2::new(entity.x + nx, entity.y + ny));
            let node_screen_size = Vec2::new(node.width * scale, node.height * scale);
            let n_rect = Rect::from_min_size(node_screen_min, node_screen_size);

            node_screen_rects.insert(node.id, n_rect);
            layout_bounds.insert(
                node.id,
                crate::flowchart_core::layout::Rect::new(nx, ny, node.width, node.height),
            );
        }

        // 5. Route and Render Orthogonal Connecting Edges
        let router = OrthogonalRouter {
            corner_radius: 6.0,
            ..Default::default()
        };
        let routed_edges = router.route_all(&graph, &layout_bounds);

        let wire_color = if is_dark {
            Color32::from_rgb(220, 215, 205)
        } else {
            Color32::from_rgb(50, 48, 44)
        };
        let wire_stroke = Stroke::new((1.8 * scale).clamp(1.4, 2.5), wire_color);

        for edge in &routed_edges {
            if edge.points.len() < 2 {
                continue;
            }

            for pair in edge.points.windows(2) {
                let p1 = to_screen(Pos2::new(entity.x + pair[0][0], entity.y + pair[0][1]));
                let p2 = to_screen(Pos2::new(entity.x + pair[1][0], entity.y + pair[1][1]));
                painter.line_segment([p1, p2], wire_stroke);
            }

            let last_idx = edge.points.len() - 1;
            let p_prev = to_screen(Pos2::new(
                entity.x + edge.points[last_idx - 1][0],
                entity.y + edge.points[last_idx - 1][1],
            ));
            let p_target = to_screen(Pos2::new(
                entity.x + edge.points[last_idx][0],
                entity.y + edge.points[last_idx][1],
            ));
            draw_arrowhead(painter, p_prev, p_target, 7.5 * scale, wire_color);

            if let Some(lbl) = &edge.label {
                let trimmed = lbl.trim();
                if !trimmed.is_empty() {
                    let lbl_pos = to_screen(Pos2::new(
                        entity.x + edge.label_pos[0],
                        entity.y + edge.label_pos[1],
                    ));
                    let font_id = egui::FontId::proportional((10.0 * scale).clamp(8.0, 12.0));
                    let text_color = palette.text;
                    let badge_bg = if is_dark {
                        Color32::from_rgb(45, 42, 38)
                    } else {
                        Color32::from_rgb(245, 242, 236)
                    };
                    let badge_w = (trimmed.len() as f32) * 7.0 * scale + 12.0;
                    let badge_rect =
                        Rect::from_center_size(lbl_pos, Vec2::new(badge_w, 15.0 * scale));

                    painter.rect_filled(badge_rect, 3.0, badge_bg);
                    painter.rect_stroke(badge_rect, 3.0, Stroke::new(0.8_f32, palette.border));
                    painter.text(lbl_pos, egui::Align2::CENTER_CENTER, trimmed, font_id, text_color);
                }
            }
        }

        // 6. Test Hover for Nodes and Anchor Dots
        let mut hovered_node = None;
        let mut hovered_port = None;
        let dot_hit_radius = (14.0 * scale).clamp(12.0, 20.0);

        if let Some(cursor) = pointer_pos {
            // Anchor dots on candidate node
            let candidate_nodes: Vec<NodeId> = if let Some(sel) = state.selected_node_id {
                vec![sel]
            } else {
                graph.nodes.iter().map(|n| n.id).collect()
            };

            for &nid in &candidate_nodes {
                if let Some(n_rect) = node_screen_rects.get(&nid) {
                    let ports = [
                        (PortSide::Top, Pos2::new(n_rect.center().x, n_rect.top())),
                        (PortSide::Bottom, Pos2::new(n_rect.center().x, n_rect.bottom())),
                        (PortSide::Left, Pos2::new(n_rect.left(), n_rect.center().y)),
                        (PortSide::Right, Pos2::new(n_rect.right(), n_rect.center().y)),
                    ];

                    for (side, dot_pos) in ports {
                        if cursor.distance(dot_pos) <= dot_hit_radius {
                            hovered_port = Some((nid, side));
                            break;
                        }
                    }

                    if hovered_port.is_some() {
                        break;
                    }
                }
            }

            // Node body hover
            if hovered_port.is_none() {
                for node in graph.nodes.iter().rev() {
                    if let Some(n_rect) = node_screen_rects.get(&node.id) {
                        if n_rect.contains(cursor) {
                            hovered_node = Some(node.id);
                            break;
                        }
                    }
                }
            }
        }

        state.hovered_node_id = hovered_node;
        state.hovered_port = hovered_port;

        // 7. Render ISO 5807 Nodes & Interactive Connection Dots
        for node in &graph.nodes {
            if let Some(&node_rect) = node_screen_rects.get(&node.id) {
                render_node_vector(painter, node.kind, node_rect, &node.label, palette, is_dark, scale);

                let is_sel = state.selected_node_id == Some(node.id);
                let is_hov = state.hovered_node_id == Some(node.id);

                // Render selection outline and handles (Creately style)
                if is_sel || is_hov {
                    let sel_stroke = if is_sel {
                        Stroke::new(1.8_f32, Color32::from_rgb(33, 150, 243))
                    } else {
                        Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(33, 150, 243, 160))
                    };
                    painter.rect_stroke(node_rect.expand(2.0), 3.0, sel_stroke);

                    if is_sel {
                        let handle_size = Vec2::splat(6.0 * scale.clamp(0.8, 1.4));
                        let corners = [
                            node_rect.left_top(),
                            node_rect.right_top(),
                            node_rect.left_bottom(),
                            node_rect.right_bottom(),
                        ];
                        for c in corners {
                            let h_rect = Rect::from_center_size(c, handle_size);
                            painter.rect_filled(h_rect, 1.0, Color32::WHITE);
                            painter.rect_stroke(h_rect, 1.0, Stroke::new(1.2_f32, Color32::from_rgb(33, 150, 243)));
                        }
                    }

                    // 8. Render the 4 Connection Anchor Dots: Top, Bottom, Left, Right
                    let ports = [
                        (PortSide::Top, Pos2::new(node_rect.center().x, node_rect.top())),
                        (PortSide::Bottom, Pos2::new(node_rect.center().x, node_rect.bottom())),
                        (PortSide::Left, Pos2::new(node_rect.left(), node_rect.center().y)),
                        (PortSide::Right, Pos2::new(node_rect.right(), node_rect.center().y)),
                    ];

                    let dot_normal_r = 4.8 * scale.clamp(0.8, 1.3);
                    let dot_hover_r = 7.0 * scale.clamp(0.8, 1.3);

                    for (side, dot_pos) in ports {
                        let is_dot_hovered = state
                            .hovered_port
                            .map(|(nid, s)| nid == node.id && s == side)
                            .unwrap_or(false);

                        if is_dot_hovered {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            // Outer glowing halo
                            painter.circle_filled(
                                dot_pos,
                                dot_hover_r + 4.5,
                                Color32::from_rgba_unmultiplied(33, 150, 243, 75),
                            );
                            // Solid blue circular dot
                            painter.circle_filled(dot_pos, dot_hover_r, Color32::from_rgb(25, 118, 210));
                            // White border
                            painter.circle_stroke(dot_pos, dot_hover_r, Stroke::new(1.6_f32, Color32::WHITE));
                        } else {
                            // Solid blue circular dot
                            painter.circle_filled(dot_pos, dot_normal_r, Color32::from_rgb(25, 118, 210));
                            // White border
                            painter.circle_stroke(dot_pos, dot_normal_r, Stroke::new(1.2_f32, Color32::WHITE));
                        }
                    }
                }
            }
        }

        Some(card_rect)
    }

    /// Handle interactive Quick Picker Popover and node creation.
    pub fn handle_popover_and_actions(
        ui: &mut egui::Ui,
        entity: &mut FlowchartEntity,
        state: &mut FlowchartUiState,
        palette: &Palette,
        is_dark: bool,
    ) {
        if let Some((source_id, source_side, dot_screen_pos)) = state.active_port_menu {
            let mut choice: Option<BlockKind> = None;
            let mut close_menu = false;

            let popover_pos = dot_screen_pos + match source_side {
                PortSide::Right => Vec2::new(16.0, -40.0),
                PortSide::Bottom => Vec2::new(-60.0, 16.0),
                PortSide::Left => Vec2::new(-170.0, -40.0),
                PortSide::Top => Vec2::new(-60.0, -190.0),
            };

            egui::Area::new(egui::Id::new("flowchart_anchor_picker_area"))
                .fixed_pos(popover_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::menu(ui.style())
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(1.0_f32, palette.border))
                        .fill(if is_dark {
                            Color32::from_rgb(38, 35, 32)
                        } else {
                            Color32::from_rgb(255, 255, 255)
                        })
                        .show(ui, |ui| {
                            ui.set_min_width(175.0);
                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Add next shape")
                                        .strong()
                                        .size(12.5)
                                        .color(palette.accent),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("x").clicked() {
                                        close_menu = true;
                                    }
                                });
                            });

                            ui.separator();

                            let choices = [
                                (BlockKind::Process, VectorIcon::ShapeRectangle, "Process (Xử lý)"),
                                (BlockKind::Decision, VectorIcon::ShapeDiamond, "Decision (Điều kiện)"),
                                (BlockKind::Start, VectorIcon::ShapeOval, "Terminator (Đầu / Cuối)"),
                                (BlockKind::Input, VectorIcon::ShapeParallelogram, "Data I/O (Nhập / Xuất)"),
                                (BlockKind::Connector, VectorIcon::ShapeCircle, "Connector (Điểm nối)"),
                                (BlockKind::Subroutine, VectorIcon::ShapeSubroutine, "Subroutine (Khối con)"),
                            ];

                            for (k, icon, title) in choices {
                                ui.horizontal(|ui| {
                                    let icon_rect = ui.allocate_space(Vec2::splat(18.0)).1;
                                    icon.paint(ui.painter(), icon_rect, palette.accent, 1.5);
                                    if ui.button(title).clicked() {
                                        choice = Some(k);
                                    }
                                });
                            }
                            ui.add_space(4.0);
                        });
                });

            if close_menu {
                state.active_port_menu = None;
            } else if let Some(chosen_kind) = choice {
                if entity.graph.is_none() {
                    let (ast, _) = Parser::from_str(&entity.dsl_source).parse();
                    entity.graph = Some(FlowchartGraph::from_ast(&ast));
                }

                if let Some(graph) = entity.graph.as_mut() {
                    if let Some(src) = graph.node(source_id).cloned() {
                        let (src_x, src_y) = src.pinned_pos.unwrap_or((60.0, 60.0));
                        let (src_w, src_h) = (src.width, src.height);

                        let (new_w, new_h) = match chosen_kind {
                            BlockKind::Start | BlockKind::End => (110.0, 42.0),
                            BlockKind::Decision => (130.0, 70.0),
                            BlockKind::Input | BlockKind::Output => (130.0, 50.0),
                            BlockKind::Connector => (48.0, 48.0),
                            _ => (130.0, 50.0),
                        };

                        let gap = 70.0;
                        let (mut new_x, mut new_y) = match source_side {
                            PortSide::Right => (src_x + src_w + gap, src_y + (src_h - new_h) * 0.5),
                            PortSide::Bottom => (src_x + (src_w - new_w) * 0.5, src_y + src_h + gap),
                            PortSide::Left => (src_x - new_w - gap, src_y + (src_h - new_h) * 0.5),
                            PortSide::Top => (src_x + (src_w - new_w) * 0.5, src_y - new_h - gap),
                        };

                        // Handle coordinate bounds shift
                        if new_x < 20.0 || new_y < 20.0 {
                            let shift_x = (20.0 - new_x).max(0.0);
                            let shift_y = (20.0 - new_y).max(0.0);
                            graph.shift_all_nodes(shift_x, shift_y);
                            entity.x -= shift_x;
                            entity.y -= shift_y;
                            new_x += shift_x;
                            new_y += shift_y;
                        }

                        let new_node_id = graph.add_visual_node(
                            chosen_kind,
                            chosen_kind.default_label().to_string(),
                            (new_x, new_y),
                        );

                        let target_side = match source_side {
                            PortSide::Right => PortSide::Left,
                            PortSide::Bottom => PortSide::Top,
                            PortSide::Left => PortSide::Right,
                            PortSide::Top => PortSide::Bottom,
                        };

                        graph.add_edge(source_id, source_side, new_node_id, target_side, None);
                        state.selected_node_id = Some(new_node_id);

                        let (min_x, min_y, max_x, max_y) = graph.compute_bounds();
                        entity.width = (max_x - min_x + 90.0).max(400.0);
                        entity.height = (max_y - min_y + 90.0).max(300.0);
                    }
                }

                state.active_port_menu = None;
            }
        }
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
                points,
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
        BlockKind::Database => {
            painter.rect_filled(rect, 4.0, default_bg);
            painter.rect_stroke(rect, 4.0, stroke);
        }
        BlockKind::Document => {
            painter.rect_filled(rect, 4.0, default_bg);
            painter.rect_stroke(rect, 4.0, stroke);
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
