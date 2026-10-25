//! Flowchart Shapes Menu & Interactive Builder Palette.
//! Provides a dedicated sidebar panel for creating and managing ISO 5807 flowchart shapes.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use crate::flowchart_core::ast::BlockKind;
use crate::flowchart_core::ir::{FlowchartGraph, NodeId, PortSide};
use crate::model::{FlowchartEntity, Page};
use crate::theme::Palette;
use crate::vector_icon::VectorIcon;
use crate::widgets;

#[derive(Debug, Clone, Default)]
pub struct FlowchartUiState {
    pub selected_node_id: Option<NodeId>,
    pub hovered_node_id: Option<NodeId>,
    pub hovered_port: Option<(NodeId, PortSide)>,
    pub active_port_menu: Option<(NodeId, PortSide, Pos2)>,
    pub dragging_node: Option<(NodeId, Pos2)>,
    pub editing_node_id: Option<NodeId>,
    pub edit_label_buffer: String,
    pub applied_label_change: Option<(NodeId, String)>,
    pub req_delete_node_id: Option<NodeId>,
    pub is_panel_open: bool,
    pub active_flowchart_id: Option<String>,
}

pub struct FlowchartPalette;

impl FlowchartPalette {
    /// Render dedicated flowchart sidebar panel on the left.
    pub fn render_side_panel(
        ctx: &egui::Context,
        state: &mut FlowchartUiState,
        page: &mut Page,
        palette: &Palette,
        is_dark: bool,
        _open_code_editor: &mut bool,
        req_bake_svg: &mut Option<String>,
        view_center_world: Pos2,
    ) {
        if !state.is_panel_open {
            return;
        }

        egui::SidePanel::left("flowchart_shapes_panel")
            .resizable(true)
            .default_width(240.0)
            .width_range(210.0..=360.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Panel Header
                ui.horizontal(|ui| {
                    let icon_size = Vec2::splat(22.0);
                    let (rect, _) = ui.allocate_exact_size(icon_size, egui::Sense::hover());
                    VectorIcon::Flowchart.paint(ui.painter(), rect, palette.accent, 2.0);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Flowchart Shapes")
                            .strong()
                            .size(15.0)
                            .color(palette.text),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::tool_button(ui, palette, VectorIcon::Close, false, "Close Flowchart Panel")
                            .clicked()
                        {
                            state.is_panel_open = false;
                        }
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("STANDARD SHAPES")
                            .small()
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(4.0);

                    // Flowchart shape definitions
                    let shapes = [
                        (
                            BlockKind::Start,
                            VectorIcon::ShapeOval,
                            "Start / End",
                            "Terminal: start or end of flow",
                        ),
                        (
                            BlockKind::Process,
                            VectorIcon::ShapeRectangle,
                            "Process",
                            "Action, task or calculation",
                        ),
                        (
                            BlockKind::Decision,
                            VectorIcon::ShapeDiamond,
                            "Decision",
                            "Conditional branch (Yes / No)",
                        ),
                        (
                            BlockKind::Input,
                            VectorIcon::ShapeParallelogram,
                            "Data I/O",
                            "Input or output operation",
                        ),
                        (
                            BlockKind::Connector,
                            VectorIcon::ShapeCircle,
                            "Connector",
                            "Diagram connection point",
                        ),
                        (
                            BlockKind::Subroutine,
                            VectorIcon::ShapeSubroutine,
                            "Subroutine",
                            "Predefined process or call",
                        ),
                        (
                            BlockKind::LoopFor,
                            VectorIcon::ShapeHexagon,
                            "Preparation",
                            "Loop or initialization step",
                        ),
                        (
                            BlockKind::Database,
                            VectorIcon::ShapeDatabase,
                            "Database",
                            "Direct access data storage",
                        ),
                        (
                            BlockKind::Document,
                            VectorIcon::ShapeDocument,
                            "Document",
                            "Printed document or report",
                        ),
                    ];

                    for (kind, icon, label_title, label_desc) in shapes {
                        let btn_rect = ui.available_rect_before_wrap();
                        let item_h = 38.0;
                        let rect = Rect::from_min_size(btn_rect.min, Vec2::new(btn_rect.width(), item_h));
                        let resp = ui.allocate_rect(rect, egui::Sense::click());

                        let is_hovered = resp.hovered();
                        let bg = if is_hovered {
                            if is_dark {
                                Color32::from_rgb(48, 44, 38)
                            } else {
                                Color32::from_rgb(240, 235, 226)
                            }
                        } else {
                            Color32::TRANSPARENT
                        };

                        ui.painter().rect_filled(rect, 6.0, bg);
                        if is_hovered {
                            ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0_f32, palette.border));
                        }

                        // Icon on left
                        let icon_box = Rect::from_min_size(
                            rect.min + Vec2::new(8.0, 7.0),
                            Vec2::splat(24.0),
                        );
                        icon.paint(
                            ui.painter(),
                            icon_box,
                            if is_hovered { palette.accent } else { palette.text },
                            1.8,
                        );

                        // Label text (pure English)
                        ui.painter().text(
                            rect.min + Vec2::new(40.0, 11.0),
                            egui::Align2::LEFT_TOP,
                            label_title,
                            egui::FontId::proportional(12.5),
                            palette.text,
                        );
                        ui.painter().text(
                            rect.min + Vec2::new(40.0, 23.0),
                            egui::Align2::LEFT_TOP,
                            label_desc,
                            egui::FontId::proportional(10.0),
                            palette.text_muted,
                        );

                        ui.advance_cursor_after_rect(rect);

                        if resp.clicked() {
                            Self::add_shape_to_canvas(page, state, kind, view_center_world);
                        }
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(
                        egui::RichText::new("ACTIONS")
                            .small()
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.add_space(6.0);

                    // Add new flowchart diagram directly on whiteboard
                    if ui.button("+ Add Shape to Board").clicked() {
                        Self::add_shape_to_canvas(page, state, BlockKind::Process, view_center_world);
                    }

                    // Auto-align layout
                    if ui.button("Auto-align Flowchart").clicked() {
                        if let Some(fid) = &state.active_flowchart_id {
                            if let Some(flow) = page.flowcharts.iter_mut().find(|f| f.id == *fid) {
                                let graph = if let Some(g) = &flow.graph {
                                    g.clone()
                                } else {
                                    let (ast, _) = crate::flowchart_core::parser::Parser::from_str(&flow.dsl_source).parse();
                                    FlowchartGraph::from_ast(&ast)
                                };
                                let layout = crate::flowchart_core::layout::LayoutEngine::new().compute_layout(&graph);
                                if let Some(g) = &mut flow.graph {
                                    for node in &mut g.nodes {
                                        if let Some(nb) = layout.node_bounds.get(&node.id) {
                                            node.pinned_pos = Some((nb.x, nb.y));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Bake SVG to whiteboard image
                    if ui.button("Bake to Whiteboard Image").clicked() {
                        if let Some(fid) = &state.active_flowchart_id {
                            if let Some(flow) = page.flowcharts.iter().find(|f| f.id == *fid) {
                                let graph = if let Some(g) = &flow.graph {
                                    g.clone()
                                } else {
                                    let (ast, _) = crate::flowchart_core::parser::Parser::from_str(&flow.dsl_source).parse();
                                    FlowchartGraph::from_ast(&ast)
                                };
                                let svg = crate::flowchart_core::export::FlowchartExporter::to_svg(&graph, None, None);
                                *req_bake_svg = Some(svg);
                            }
                        }
                    }
                });
            });
    }

    /// Add a flowchart shape node to the active flowchart or create a new flowchart container.
    pub fn add_shape_to_canvas(
        page: &mut Page,
        state: &mut FlowchartUiState,
        kind: BlockKind,
        view_center_world: Pos2,
    ) {
        let label = kind.default_label().to_string();

        // 1. Try to find the active flowchart entity
        let mut target_flowchart = None;
        if let Some(fid) = &state.active_flowchart_id {
            if let Some(idx) = page.flowcharts.iter().position(|f| f.id == *fid) {
                target_flowchart = Some(idx);
            }
        }

        // If no active flowchart, use the newest flowchart on page or create a new one
        if target_flowchart.is_none() && !page.flowcharts.is_empty() {
            target_flowchart = Some(page.flowcharts.len() - 1);
            state.active_flowchart_id = Some(page.flowcharts.last().unwrap().id.clone());
        }

        let flow_idx = match target_flowchart {
            Some(idx) => idx,
            None => {
                let ent = FlowchartEntity {
                    title: "Flowchart".to_string(),
                    x: (view_center_world.x - 200.0).max(40.0),
                    y: (view_center_world.y - 150.0).max(40.0),
                    width: 450.0,
                    height: 320.0,
                    dsl_source: String::new(),
                    graph: Some(FlowchartGraph::new()),
                    ..Default::default()
                };
                let id = ent.id.clone();
                page.flowcharts.push(ent);
                state.active_flowchart_id = Some(id);
                page.flowcharts.len() - 1
            }
        };

        let flow = &mut page.flowcharts[flow_idx];
        if flow.graph.is_none() {
            let (ast, _) = crate::flowchart_core::parser::Parser::from_str(&flow.dsl_source).parse();
            flow.graph = Some(FlowchartGraph::from_ast(&ast));
        }

        let graph = flow.graph.as_mut().unwrap();

        // Determine node placement position
        let (node_x, node_y) = if graph.nodes.is_empty() {
            (50.0, 50.0)
        } else {
            let (_, _, max_x, _max_y) = graph.compute_bounds();
            (max_x + 60.0, 50.0)
        };

        let new_id = graph.add_visual_node(kind, label, (node_x, node_y));
        state.selected_node_id = Some(new_id);

        // Update container bounds
        let (min_x, min_y, max_x, max_y) = graph.compute_bounds();
        flow.width = (max_x - min_x + 90.0).max(400.0);
        flow.height = (max_y - min_y + 90.0).max(300.0);
    }
}
