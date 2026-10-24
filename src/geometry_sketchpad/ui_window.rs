//! UI Window for Inkk Geometry Sketchpad (GSP) & 3D Spatial Studio.
//! Provides interactive Euclidean constructions, 3D model manipulation,
//! real-time measurements, action buttons, dynamic exploration, and whiteboard export.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use std::f32::consts::PI;

use super::engine::GeometryEngine;
use super::export::SketchpadExporter;
use super::model::{
    ActionKind, GeoCircle, GeoLine, GeoMeasurement, GeoPoint, GeoPolygon, MeasureKind,
    PointConstraint, SketchpadModel, SketchpadTool,
};
use super::presets_3d::Presets3D;
use crate::theme::Palette;

pub struct GeometrySketchpadState {
    pub is_open: bool,
    pub model: SketchpadModel,
    pub active_tool: SketchpadTool,
    pub line_is_dashed: bool,
    pub current_color: Color32,
    pub function_input: String,
    pub status_msg: String,
    pub req_insert_whiteboard: Option<String>,
    pub req_save_svg: Option<String>,
    pub req_save_png: Option<String>,

    // Dragging & Interaction
    pub dragging_point_id: Option<String>,
    pub is_rotating_3d_mouse: bool,
    pub last_mouse_pos: Option<Pos2>,
    pub pending_start_point_id: Option<String>,
    pub pending_poly_ids: Vec<String>,
}

impl Default for GeometrySketchpadState {
    fn default() -> Self {
        let mut model = SketchpadModel::new();
        // Load default classic 3D Sphere with Inscribed Pyramid (User Image 2!)
        Presets3D::load_sphere_with_inscribed_pyramid(&mut model);
        GeometryEngine::update_geometry(&mut model);

        Self {
            is_open: false,
            model,
            active_tool: SketchpadTool::Select,
            line_is_dashed: false,
            current_color: Color32::from_rgb(25, 60, 180),
            function_input: "sin(x)".to_string(),
            status_msg: "Sketchpad Ready — Select a tool or drag points in 2D/3D".to_string(),
            req_insert_whiteboard: None,
            req_save_svg: None,
            req_save_png: None,
            dragging_point_id: None,
            is_rotating_3d_mouse: false,
            last_mouse_pos: None,
            pending_start_point_id: None,
            pending_poly_ids: Vec::new(),
        }
    }
}

impl GeometrySketchpadState {
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            GeometryEngine::update_geometry(&mut self.model);
        }
    }

    /// Renders the Geometry Sketchpad window.
    pub fn render(&mut self, ctx: &egui::Context, palette: &Palette, is_dark: bool) {
        if !self.is_open {
            return;
        }

        // Keep updating continuously if 3D auto-rotate is running or animating
        if self.model.camera3d.is_auto_rotating || self.model.is_animating_points {
            GeometryEngine::update_geometry(&mut self.model);
            ctx.request_repaint();
        }

        let mut is_open_flag = self.is_open;
        egui::Window::new("📐 Geometry Sketchpad (GSP) & 3D Spatial Studio")
            .open(&mut is_open_flag)
            .default_size(Vec2::new(880.0, 640.0))
            .min_size(Vec2::new(620.0, 480.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_menu_bar(ui, is_dark);
                ui.separator();
                ui.horizontal(|ui| {
                    self.render_left_toolbar(ui, palette);
                    ui.separator();
                    self.render_canvas_area(ui, palette, is_dark);
                });
            });

        self.is_open = is_open_flag;
    }

    fn render_menu_bar(&mut self, ui: &mut Ui, is_dark: bool) {
        egui::menu::bar(ui, |ui| {
            // File Menu
            ui.menu_button("File", |ui| {
                if ui.button("📄 New Blank Sketch").clicked() {
                    self.model.clear();
                    self.status_msg = "New sketch created".to_string();
                    ui.close_menu();
                }
                ui.separator();
                ui.label(egui::RichText::new("3D Spatial Presets").small().strong());
                if ui.button("🌐 Sphere + Inscribed Pyramid (GSP Classic)").clicked() {
                    Presets3D::load_sphere_with_inscribed_pyramid(&mut self.model);
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Loaded 3D Sphere with Inscribed Pyramid".to_string();
                    ui.close_menu();
                }
                if ui.button("📦 3D Cube (ABCD.A'B'C')").clicked() {
                    Presets3D::load_cube_3d(&mut self.model);
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Loaded 3D Cube".to_string();
                    ui.close_menu();
                }
                if ui.button("🔺 3D Square Pyramid (S.ABCD)").clicked() {
                    Presets3D::load_square_pyramid_3d(&mut self.model);
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Loaded 3D Square Pyramid".to_string();
                    ui.close_menu();
                }
                if ui.button("📐 3D Triangular Prism").clicked() {
                    Presets3D::load_triangular_prism_3d(&mut self.model);
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Loaded 3D Triangular Prism".to_string();
                    ui.close_menu();
                }
                if ui.button("🥫 3D Cylinder").clicked() {
                    Presets3D::load_cylinder_3d(&mut self.model);
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Loaded 3D Cylinder".to_string();
                    ui.close_menu();
                }
                if ui.button("🍦 3D Cone").clicked() {
                    Presets3D::load_cone_3d(&mut self.model);
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Loaded 3D Cone".to_string();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("📥 Insert into Whiteboard").clicked() {
                    let svg = SketchpadExporter::to_svg(&self.model, 700.0, 500.0, is_dark);
                    self.req_insert_whiteboard = Some(svg);
                    self.status_msg = "Exported to whiteboard!".to_string();
                    ui.close_menu();
                }
                if ui.button("💾 Save as SVG...").clicked() {
                    let svg = SketchpadExporter::to_svg(&self.model, 700.0, 500.0, is_dark);
                    self.req_save_svg = Some(svg);
                    ui.close_menu();
                }
                if ui.button("🖼️ Save as PNG...").clicked() {
                    let svg = SketchpadExporter::to_svg(&self.model, 700.0, 500.0, is_dark);
                    self.req_save_png = Some(svg);
                    ui.close_menu();
                }
            });

            // Edit Menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Select All Points").clicked() {
                    for p in &mut self.model.points {
                        p.is_selected = true;
                    }
                    ui.close_menu();
                }
                if ui.button("Deselect All").clicked() {
                    for p in &mut self.model.points {
                        p.is_selected = false;
                    }
                    ui.close_menu();
                }
                if ui.button("Delete Selected Points").clicked() {
                    self.model.points.retain(|p| !p.is_selected);
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Clear All Traces").clicked() {
                    self.model.clear_traces();
                    ui.close_menu();
                }
            });

            // Construct Menu (Euclidean Classical Geometry)
            ui.menu_button("Construct", |ui| {
                let sel_ids: Vec<String> = self.model.points.iter().filter(|p| p.is_selected).map(|p| p.id.clone()).collect();
                if sel_ids.len() == 2 {
                    if ui.button("Midpoint of Selected Points").clicked() {
                        let id = self.model.gen_id("mid");
                        let mut pt = GeoPoint::new_2d(&id, format!("M{}", self.model.next_id), Pos2::ZERO);
                        pt.constraint = PointConstraint::Midpoint(sel_ids[0].clone(), sel_ids[1].clone());
                        self.model.points.push(pt);
                        GeometryEngine::update_geometry(&mut self.model);
                        self.status_msg = "Constructed Midpoint".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Segment connecting Points").clicked() {
                        let lid = self.model.gen_id("line");
                        self.model.lines.push(GeoLine::new(lid, &sel_ids[0], &sel_ids[1], self.line_is_dashed));
                        self.status_msg = "Constructed Segment".to_string();
                        ui.close_menu();
                    }
                    if ui.button("Circle (Center & Radius Point)").clicked() {
                        let cid = self.model.gen_id("circ");
                        self.model.circles.push(GeoCircle {
                            id: cid,
                            center: sel_ids[0].clone(),
                            radius_pt: Some(sel_ids[1].clone()),
                            fixed_radius: None,
                            is_dashed: self.line_is_dashed,
                            color: self.current_color,
                            is_selected: false,
                            is_hidden: false,
                        });
                        self.status_msg = "Constructed Circle".to_string();
                        ui.close_menu();
                    }
                } else {
                    ui.label(egui::RichText::new("Select 2 points to construct Midpoint, Segment, or Circle").weak());
                }

                if sel_ids.len() >= 3 {
                    if ui.button("Construct Polygon from Selected Points").clicked() {
                        let pid = self.model.gen_id("poly");
                        self.model.polygons.push(GeoPolygon {
                            id: pid,
                            vertices: sel_ids.clone(),
                            fill_color: Some(Color32::from_rgba_unmultiplied(200, 225, 255, 100)),
                            stroke_color: self.current_color,
                            is_selected: false,
                            is_hidden: false,
                        });
                        self.status_msg = "Constructed Polygon".to_string();
                        ui.close_menu();
                    }
                }
            });

            // Transform Menu
            ui.menu_button("Transform", |ui| {
                if ui.button("Translate (+20px, -20px)").clicked() {
                    GeometryEngine::translate_selected(&mut self.model, Vec2::new(20.0, -20.0));
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Rotate (60°)").clicked() {
                    let center = self.model.camera3d.center;
                    GeometryEngine::rotate_selected(&mut self.model, center, 60.0);
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Rotate (90°)").clicked() {
                    let center = self.model.camera3d.center;
                    GeometryEngine::rotate_selected(&mut self.model, center, 90.0);
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Dilate (1.5x)").clicked() {
                    let center = self.model.camera3d.center;
                    GeometryEngine::dilate_selected(&mut self.model, center, 1.5);
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Dilate (0.67x)").clicked() {
                    let center = self.model.camera3d.center;
                    GeometryEngine::dilate_selected(&mut self.model, center, 0.67);
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Reflect across Vertical Axis").clicked() {
                    let center = self.model.camera3d.center;
                    GeometryEngine::reflect_selected_across_line(&mut self.model, center, center + Vec2::new(0.0, 100.0));
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
            });

            // Measure Menu
            ui.menu_button("Measure", |ui| {
                let sel_ids: Vec<String> = self.model.points.iter().filter(|p| p.is_selected).map(|p| p.id.clone()).collect();
                if sel_ids.len() == 2 {
                    if ui.button("Measure Distance / Length").clicked() {
                        let mid = self.model.gen_id("measure");
                        let y_offset = 20.0 + self.model.measurements.len() as f32 * 25.0;
                        self.model.measurements.push(GeoMeasurement {
                            id: mid,
                            kind: MeasureKind::Length,
                            target_ids: sel_ids.clone(),
                            cached_value: 0.0,
                            display_text: String::new(),
                            pos: Pos2::new(30.0, y_offset),
                        });
                        GeometryEngine::update_geometry(&mut self.model);
                        ui.close_menu();
                    }
                    if ui.button("Measure Slope").clicked() {
                        let mid = self.model.gen_id("slope");
                        let y_offset = 20.0 + self.model.measurements.len() as f32 * 25.0;
                        self.model.measurements.push(GeoMeasurement {
                            id: mid,
                            kind: MeasureKind::Slope,
                            target_ids: sel_ids.clone(),
                            cached_value: 0.0,
                            display_text: String::new(),
                            pos: Pos2::new(30.0, y_offset),
                        });
                        GeometryEngine::update_geometry(&mut self.model);
                        ui.close_menu();
                    }
                } else if sel_ids.len() == 3 {
                    if ui.button("Measure Angle").clicked() {
                        let mid = self.model.gen_id("angle");
                        let y_offset = 20.0 + self.model.measurements.len() as f32 * 25.0;
                        self.model.measurements.push(GeoMeasurement {
                            id: mid,
                            kind: MeasureKind::Angle,
                            target_ids: sel_ids.clone(),
                            cached_value: 0.0,
                            display_text: String::new(),
                            pos: Pos2::new(30.0, y_offset),
                        });
                        GeometryEngine::update_geometry(&mut self.model);
                        ui.close_menu();
                    }
                } else if sel_ids.len() == 1 {
                    if ui.button("Measure Coordinates").clicked() {
                        let mid = self.model.gen_id("coord");
                        let y_offset = 20.0 + self.model.measurements.len() as f32 * 25.0;
                        self.model.measurements.push(GeoMeasurement {
                            id: mid,
                            kind: MeasureKind::Coordinates,
                            target_ids: sel_ids.clone(),
                            cached_value: 0.0,
                            display_text: String::new(),
                            pos: Pos2::new(30.0, y_offset),
                        });
                        GeometryEngine::update_geometry(&mut self.model);
                        ui.close_menu();
                    }
                } else {
                    ui.label(egui::RichText::new("Select 1, 2, or 3 points to measure").weak());
                }
            });

            // 3D View Menu
            ui.menu_button("3D View", |ui| {
                if ui.button("Quay 🔄 (Toggle 3D Rotation)").clicked() {
                    self.model.camera3d.is_auto_rotating = !self.model.camera3d.is_auto_rotating;
                    ui.close_menu();
                }
                if ui.button("Top-Down View (Oxy)").clicked() {
                    self.model.camera3d.pitch = PI * 0.5;
                    self.model.camera3d.yaw = 0.0;
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Front View (Oxz)").clicked() {
                    self.model.camera3d.pitch = 0.0;
                    self.model.camera3d.yaw = 0.0;
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Side View (Oyz)").clicked() {
                    self.model.camera3d.pitch = 0.0;
                    self.model.camera3d.yaw = PI * 0.5;
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
                if ui.button("Reset ISO 3D View").clicked() {
                    self.model.camera3d.pitch = 0.35;
                    self.model.camera3d.yaw = -0.55;
                    GeometryEngine::update_geometry(&mut self.model);
                    ui.close_menu();
                }
            });

            // Graph & Algebra Menu
            ui.menu_button("Graph", |ui| {
                ui.checkbox(&mut self.model.show_axes, "Show Coordinate Axes (Oxy)");
                ui.separator();
                ui.label("Plot function y = f(x):");
                ui.text_edit_singleline(&mut self.function_input);
                if ui.button("+ Add Function Curve").clicked() {
                    let fid = self.model.gen_id("fn");
                    self.model.functions.push(super::model::GeoFunctionGraph {
                        id: fid,
                        expr: self.function_input.clone(),
                        color: Color32::from_rgb(200, 30, 120),
                    });
                    self.status_msg = format!("Plotted y = {}", self.function_input);
                    ui.close_menu();
                }
            });

            // Action: Whiteboard Insert Shortcut
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("📥 Insert into Whiteboard").strong().color(Color32::from_rgb(0, 150, 80))).clicked() {
                    let svg = SketchpadExporter::to_svg(&self.model, 700.0, 500.0, is_dark);
                    self.req_insert_whiteboard = Some(svg);
                    self.status_msg = "Successfully placed geometric sketch on Whiteboard!".to_string();
                }
            });
        });
    }

    fn render_left_toolbar(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.vertical(|ui| {
            ui.set_width(46.0);

            let cad_btn = |ui: &mut Ui, tool: SketchpadTool, cur_tool: SketchpadTool, tip: &str, draw_fn: fn(&Painter, Rect, Stroke)| -> bool {
                let is_sel = tool == cur_tool;
                let (rect, response) = ui.allocate_exact_size(Vec2::new(38.0, 34.0), Sense::click());
                let is_hovered = response.hovered();
                let bg = if is_sel {
                    palette.accent.gamma_multiply(0.35)
                } else if is_hovered {
                    palette.accent.gamma_multiply(0.15)
                } else {
                    palette.surface
                };
                let painter = ui.painter();
                painter.rect_filled(rect, Rounding::same(3.0), bg);
                painter.rect_stroke(rect, Rounding::same(3.0), Stroke::new(1.0_f32, if is_sel { palette.accent } else { palette.border }));
                let icon_stroke = Stroke::new(1.6_f32, if is_sel { palette.accent } else { palette.text });
                draw_fn(painter, rect, icon_stroke);
                response.on_hover_text(tip).clicked()
            };

            if cad_btn(ui, SketchpadTool::Select, self.active_tool, "Selection / Drag & Rotate 3D", super::cad_icons::CadIcon::draw_select) {
                self.active_tool = SketchpadTool::Select;
                self.status_msg = "Pointer Tool: Drag points freely or drag empty space to rotate 3D".to_string();
            }
            if cad_btn(ui, SketchpadTool::Point, self.active_tool, "Point: Click canvas to create point", super::cad_icons::CadIcon::draw_point) {
                self.active_tool = SketchpadTool::Point;
                self.status_msg = "Point Tool: Click on canvas to add a new point".to_string();
            }
            if cad_btn(ui, SketchpadTool::Compass, self.active_tool, "Circle: Center & Radius", super::cad_icons::CadIcon::draw_circle) {
                self.active_tool = SketchpadTool::Compass;
                self.status_msg = "Circle Tool: Click center point then radius point to draw circle".to_string();
            }
            if cad_btn(ui, SketchpadTool::Segment, self.active_tool, "Line Segment", super::cad_icons::CadIcon::draw_line) {
                self.active_tool = SketchpadTool::Segment;
                self.line_is_dashed = false;
                self.status_msg = "Segment Tool: Click point 1 then point 2".to_string();
            }
            if cad_btn(ui, SketchpadTool::DashedSegment, self.active_tool, "Dashed Segment (Hidden 3D edges)", super::cad_icons::CadIcon::draw_dashed_line) {
                self.active_tool = SketchpadTool::DashedSegment;
                self.line_is_dashed = true;
                self.status_msg = "Dashed Segment Tool: Click 2 points to connect with dashed line".to_string();
            }
            if cad_btn(ui, SketchpadTool::Ray, self.active_tool, "Ray", super::cad_icons::CadIcon::draw_ray) {
                self.active_tool = SketchpadTool::Ray;
                self.status_msg = "Ray Tool: Click origin then directional point".to_string();
            }
            if cad_btn(ui, SketchpadTool::Line, self.active_tool, "Infinite Line", super::cad_icons::CadIcon::draw_xline) {
                self.active_tool = SketchpadTool::Line;
                self.status_msg = "Infinite Line Tool: Click 2 points to draw full line".to_string();
            }
            if cad_btn(ui, SketchpadTool::Polygon, self.active_tool, "Polygon", super::cad_icons::CadIcon::draw_polygon) {
                self.active_tool = SketchpadTool::Polygon;
                self.pending_poly_ids.clear();
                self.status_msg = "Polygon Tool: Click points sequentially, click first point to close".to_string();
            }
            if cad_btn(ui, SketchpadTool::Rotate3D, self.active_tool, "3D Orbit", super::cad_icons::CadIcon::draw_orbit3d) {
                self.active_tool = SketchpadTool::Rotate3D;
                self.status_msg = "Rotate 3D: Click & drag on canvas to pitch & yaw".to_string();
            }

            ui.separator();
            // Solid/Dashed toggle chip
            let dash_text = if self.line_is_dashed { "Dashed" } else { "Solid" };
            if ui.add_sized([42.0, 22.0], egui::Button::new(egui::RichText::new(dash_text).small())).clicked() {
                self.line_is_dashed = !self.line_is_dashed;
            }

            // Trace toggle
            if ui.add_sized([42.0, 22.0], egui::Button::new(egui::RichText::new("Trace").small())).on_hover_text("Toggle Trace (Locus) on selected point").clicked() {
                for p in &mut self.model.points {
                    if p.is_selected {
                        p.trace_enabled = !p.trace_enabled;
                    }
                }
                self.status_msg = "Toggled trace for selected points".to_string();
            }
        });
    }

    fn render_canvas_area(&mut self, ui: &mut Ui, palette: &Palette, is_dark: bool) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let canvas_rect = response.rect;

        // Keep camera center synced to middle of canvas
        self.model.camera3d.center = canvas_rect.center();

        let bg_color = if is_dark {
            Color32::from_rgb(28, 28, 30)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        painter.rect_filled(canvas_rect, Rounding::ZERO, bg_color);

        // 1. Grid & Axes (Oxy)
        if self.model.show_axes {
            let cx = self.model.camera3d.center.x;
            let cy = self.model.camera3d.center.y;
            let axis_col = if is_dark { Color32::from_rgb(60, 60, 65) } else { Color32::from_rgb(215, 220, 225) };
            painter.line_segment([Pos2::new(canvas_rect.left(), cy), Pos2::new(canvas_rect.right(), cy)], Stroke::new(1.2_f32, axis_col));
            painter.line_segment([Pos2::new(cx, canvas_rect.top()), Pos2::new(cx, canvas_rect.bottom())], Stroke::new(1.2_f32, axis_col));
            // Axis arrow tips
            painter.line_segment([Pos2::new(canvas_rect.right() - 8.0, cy - 4.0), Pos2::new(canvas_rect.right(), cy)], Stroke::new(1.2_f32, axis_col));
            painter.line_segment([Pos2::new(canvas_rect.right() - 8.0, cy + 4.0), Pos2::new(canvas_rect.right(), cy)], Stroke::new(1.2_f32, axis_col));
            painter.line_segment([Pos2::new(cx - 4.0, canvas_rect.top() + 8.0), Pos2::new(cx, canvas_rect.top())], Stroke::new(1.2_f32, axis_col));
            painter.line_segment([Pos2::new(cx + 4.0, canvas_rect.top() + 8.0), Pos2::new(cx, canvas_rect.top())], Stroke::new(1.2_f32, axis_col));
        }

        // 2. Polygons (Fills & Outlines)
        let get_pos = |id: &str, model: &SketchpadModel| -> Option<Pos2> {
            model.points.iter().find(|p| p.id == id).map(|p| p.pos)
        };

        for poly in &self.model.polygons {
            if poly.is_hidden { continue; }
            let pts: Vec<Pos2> = poly.vertices.iter().filter_map(|vid| get_pos(vid, &self.model)).collect();
            if pts.len() >= 3 {
                if let Some(fc) = poly.fill_color {
                    let mut mesh = egui::Mesh::default();
                    let c_idx = mesh.vertices.len() as u32;
                    let center_pt = Pos2::new(
                        pts.iter().map(|p| p.x).sum::<f32>() / pts.len() as f32,
                        pts.iter().map(|p| p.y).sum::<f32>() / pts.len() as f32,
                    );
                    mesh.colored_vertex(center_pt, fc);
                    for p in &pts {
                        mesh.colored_vertex(*p, fc);
                    }
                    for i in 0..pts.len() {
                        let i1 = c_idx + 1 + i as u32;
                        let i2 = if i + 1 == pts.len() { c_idx + 1 } else { c_idx + 2 + i as u32 };
                        mesh.add_triangle(c_idx, i1, i2);
                    }
                    painter.add(egui::Shape::Mesh(mesh));
                }
                for i in 0..pts.len() {
                    let next = (i + 1) % pts.len();
                    painter.line_segment([pts[i], pts[next]], Stroke::new(1.8_f32, poly.stroke_color));
                }
            }
        }

        // 3. Circles
        for c in &self.model.circles {
            if c.is_hidden { continue; }
            if let Some(cp) = get_pos(&c.center, &self.model) {
                let r = if let Some(rid) = &c.radius_pt {
                    get_pos(rid, &self.model).map(|rp| (rp - cp).length()).unwrap_or(50.0)
                } else {
                    c.fixed_radius.unwrap_or(50.0)
                };
                if c.is_dashed {
                    Self::draw_dashed_circle(&painter, cp, r, Stroke::new(1.8_f32, c.color));
                } else {
                    painter.circle_stroke(cp, r, Stroke::new(1.8_f32, c.color));
                }
            }
        }

        // 4. Lines (Solid & Dashed)
        for line in &self.model.lines {
            if line.is_hidden { continue; }
            if let (Some(p1), Some(p2)) = (get_pos(&line.p1, &self.model), get_pos(&line.p2, &self.model)) {
                let stroke = Stroke::new(line.width, line.color);
                if line.is_dashed {
                    Self::draw_dashed_line(&painter, p1, p2, stroke);
                } else {
                    painter.line_segment([p1, p2], stroke);
                }
            }
        }

        // 5. Function plots
        for f in &self.model.functions {
            let cx = self.model.camera3d.center.x;
            let cy = self.model.camera3d.center.y;
            let scale = 40.0;
            let stroke = Stroke::new(2.2_f32, f.color);
            let steps = 140;
            let mut prev_pt: Option<Pos2> = None;
            for i in 0..=steps {
                let px = canvas_rect.left() + (i as f32 / steps as f32) * canvas_rect.width();
                let math_x = (px - cx) / scale;
                if let Some(math_y) = GeometryEngine::eval_func(&f.expr, math_x) {
                    let py = cy - math_y * scale;
                    if py >= canvas_rect.top() - 20.0 && py <= canvas_rect.bottom() + 20.0 {
                        let cur_pt = Pos2::new(px, py);
                        if let Some(prv) = prev_pt {
                            painter.line_segment([prv, cur_pt], stroke);
                        }
                        prev_pt = Some(cur_pt);
                    } else {
                        prev_pt = None;
                    }
                } else {
                    prev_pt = None;
                }
            }
        }

        // 6. Trace Points (Locus trails)
        for w in self.model.traces.windows(2) {
            painter.line_segment([w[0].pos, w[1].pos], Stroke::new(1.5_f32, Color32::from_rgb(220, 50, 50)));
        }

        // 7. Points & Labels
        let text_color = if is_dark { Color32::from_rgb(235, 235, 240) } else { Color32::from_rgb(20, 20, 30) };
        for pt in &self.model.points {
            if pt.is_hidden { continue; }
            let pt_color = if pt.is_selected { Color32::from_rgb(255, 180, 0) } else { pt.color };
            let radius = if pt.is_selected { 5.5 } else { 4.0 };
            painter.circle_filled(pt.pos, radius, pt_color);
            painter.circle_stroke(pt.pos, radius, Stroke::new(1.2_f32, Color32::WHITE));

            if !pt.label.is_empty() {
                painter.text(
                    pt.pos + Vec2::new(7.0, -7.0),
                    egui::Align2::LEFT_BOTTOM,
                    &pt.label,
                    egui::FontId::proportional(15.0),
                    text_color,
                );
            }
        }

        // 8. On-Canvas Interactive Action Buttons (as in User Image 2)
        let mut clicked_action: Option<ActionKind> = None;
        for btn in &self.model.action_buttons {
            let b_rect = Rect::from_min_size(canvas_rect.min + btn.pos.to_vec2(), Vec2::new(72.0, 26.0));
            let is_hovered = ui.rect_contains_pointer(b_rect);
            let btn_fill = if is_hovered {
                palette.accent.gamma_multiply(0.35)
            } else if is_dark {
                Color32::from_rgb(45, 45, 50)
            } else {
                Color32::from_rgb(235, 240, 248)
            };
            painter.rect_filled(b_rect, Rounding::same(4.0), btn_fill);
            painter.rect_stroke(b_rect, Rounding::same(4.0), Stroke::new(1.0_f32, palette.border));
            painter.text(
                b_rect.center(),
                egui::Align2::CENTER_CENTER,
                &btn.label,
                egui::FontId::proportional(13.0),
                palette.text,
            );
            if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
                clicked_action = Some(btn.kind);
            }
        }

        // 9. Measurements Display Overlay
        for m in &self.model.measurements {
            let m_pos = canvas_rect.min + m.pos.to_vec2();
            painter.text(
                m_pos,
                egui::Align2::LEFT_TOP,
                &m.display_text,
                egui::FontId::proportional(14.0),
                text_color,
            );
        }

        // 10. 3D Coordinate Gimbal Indicator (Bottom-left corner)
        Self::draw_3d_orientation_gimbal(&painter, canvas_rect, &self.model.camera3d);

        // --- Handle Canvas Pointer Events & Dragging ---
        if let Some(act) = clicked_action {
            match act {
                ActionKind::Toggle3DRotate => {
                    self.model.camera3d.is_auto_rotating = !self.model.camera3d.is_auto_rotating;
                    self.status_msg = if self.model.camera3d.is_auto_rotating {
                        "3D Auto-Rotation ON 🔄".to_string()
                    } else {
                        "3D Rotation PAUSED".to_string()
                    };
                }
                ActionKind::ViewPlaneOxy => {
                    self.model.camera3d.pitch = PI * 0.5;
                    self.model.camera3d.yaw = 0.0;
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Top View (Oxy)".to_string();
                }
                ActionKind::ViewPlaneOxz => {
                    self.model.camera3d.pitch = 0.0;
                    self.model.camera3d.yaw = 0.0;
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Front View (Oxz)".to_string();
                }
                ActionKind::ViewPlaneOyz => {
                    self.model.camera3d.pitch = 0.0;
                    self.model.camera3d.yaw = PI * 0.5;
                    GeometryEngine::update_geometry(&mut self.model);
                    self.status_msg = "Side View (Oyz)".to_string();
                }
                ActionKind::ToggleAnimation => {
                    self.model.is_animating_points = !self.model.is_animating_points;
                }
                ActionKind::Reset3D => {
                    self.model.camera3d.pitch = 0.35;
                    self.model.camera3d.yaw = -0.55;
                    GeometryEngine::update_geometry(&mut self.model);
                }
                ActionKind::ToggleHideSelected => {
                    for p in &mut self.model.points {
                        if p.is_selected { p.is_hidden = !p.is_hidden; }
                    }
                }
            }
        }

        if let Some(mouse_pos) = response.hover_pos() {
            // Find closest point under cursor
            let hit_pt_id = self.model.points.iter().find(|p| !p.is_hidden && (p.pos - mouse_pos).length_sq() < 120.0).map(|p| p.id.clone());

            // Click interaction
            if response.clicked() {
                match self.active_tool {
                    SketchpadTool::Select => {
                        if let Some(hid) = &hit_pt_id {
                            let is_shift = ui.input(|i| i.modifiers.shift);
                            for p in &mut self.model.points {
                                if p.id == *hid {
                                    p.is_selected = if is_shift { !p.is_selected } else { true };
                                } else if !is_shift {
                                    p.is_selected = false;
                                }
                            }
                        } else {
                            for p in &mut self.model.points {
                                p.is_selected = false;
                            }
                        }
                    }
                    SketchpadTool::Point => {
                        let pid = self.model.gen_id("pt");
                        let label = format!("P{}", self.model.next_id);
                        self.model.points.push(GeoPoint::new_2d(pid, label, mouse_pos));
                        self.status_msg = "Created Point".to_string();
                    }
                    SketchpadTool::Segment | SketchpadTool::DashedSegment | SketchpadTool::Ray | SketchpadTool::Line => {
                        if let Some(hid) = &hit_pt_id {
                            if let Some(start_id) = self.pending_start_point_id.take() {
                                if start_id != *hid {
                                    let lid = self.model.gen_id("l");
                                    let mut line = GeoLine::new(lid, start_id, hid, self.active_tool == SketchpadTool::DashedSegment || self.line_is_dashed);
                                    if self.active_tool == SketchpadTool::Ray {
                                        line.kind = super::model::LineKind::Ray;
                                    } else if self.active_tool == SketchpadTool::Line {
                                        line.kind = super::model::LineKind::Line;
                                    }
                                    self.model.lines.push(line);
                                    self.status_msg = "Connected segment".to_string();
                                }
                            } else {
                                self.pending_start_point_id = Some(hid.clone());
                                self.status_msg = "Click second point to finish line".to_string();
                            }
                        }
                    }
                    SketchpadTool::Compass => {
                        if let Some(hid) = &hit_pt_id {
                            if let Some(center_id) = self.pending_start_point_id.take() {
                                if center_id != *hid {
                                    let cid = self.model.gen_id("c");
                                    self.model.circles.push(GeoCircle {
                                        id: cid,
                                        center: center_id,
                                        radius_pt: Some(hid.clone()),
                                        fixed_radius: None,
                                        is_dashed: self.line_is_dashed,
                                        color: self.current_color,
                                        is_selected: false,
                                        is_hidden: false,
                                    });
                                    self.status_msg = "Created Circle".to_string();
                                }
                            } else {
                                self.pending_start_point_id = Some(hid.clone());
                                self.status_msg = "Click point on circle perimeter".to_string();
                            }
                        }
                    }
                    SketchpadTool::Polygon => {
                        if let Some(hid) = &hit_pt_id {
                            if !self.pending_poly_ids.is_empty() && self.pending_poly_ids[0] == *hid && self.pending_poly_ids.len() >= 3 {
                                let poly_id = self.model.gen_id("poly");
                                self.model.polygons.push(GeoPolygon {
                                    id: poly_id,
                                    vertices: self.pending_poly_ids.clone(),
                                    fill_color: Some(Color32::from_rgba_unmultiplied(200, 230, 255, 110)),
                                    stroke_color: self.current_color,
                                    is_selected: false,
                                    is_hidden: false,
                                });
                                self.pending_poly_ids.clear();
                                self.status_msg = "Polygon closed & created".to_string();
                            } else {
                                self.pending_poly_ids.push(hid.clone());
                                self.status_msg = format!("Polygon: {} vertices selected", self.pending_poly_ids.len());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Dragging interaction
            if response.drag_started() {
                if let Some(hid) = &hit_pt_id {
                    self.dragging_point_id = Some(hid.clone());
                } else {
                    self.is_rotating_3d_mouse = true;
                }
                self.last_mouse_pos = Some(mouse_pos);
            }

            if response.dragged() {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = mouse_pos - last_pos;
                    if let Some(drag_id) = &self.dragging_point_id {
                        if let Some(p) = self.model.points.iter_mut().find(|p| p.id == *drag_id) {
                            if let Some(p3d) = &mut p.pos3d {
                                p3d[0] += delta.x / self.model.camera3d.zoom;
                                p3d[1] -= delta.y / self.model.camera3d.zoom;
                            } else {
                                p.pos += delta;
                            }
                        }
                        GeometryEngine::update_geometry(&mut self.model);
                    } else if self.is_rotating_3d_mouse || self.active_tool == SketchpadTool::Rotate3D {
                        // Drag rotates 3D scene in pitch & yaw!
                        self.model.camera3d.yaw += delta.x * 0.012;
                        self.model.camera3d.pitch = (self.model.camera3d.pitch - delta.y * 0.012).clamp(-PI * 0.48, PI * 0.48);
                        GeometryEngine::update_geometry(&mut self.model);
                    }
                }
                self.last_mouse_pos = Some(mouse_pos);
            }

            if response.drag_stopped() {
                self.dragging_point_id = None;
                self.is_rotating_3d_mouse = false;
                self.last_mouse_pos = None;
            }
        }

        // Status bar on bottom of canvas
        painter.text(
            Pos2::new(canvas_rect.left() + 10.0, canvas_rect.bottom() - 14.0),
            egui::Align2::LEFT_BOTTOM,
            &self.status_msg,
            egui::FontId::proportional(12.0),
            palette.text.gamma_multiply(0.7),
        );
    }

    fn draw_dashed_line(painter: &Painter, p1: Pos2, p2: Pos2, stroke: Stroke) {
        let dir = p2 - p1;
        let total_len = dir.length();
        if total_len <= 1.0 { return; }
        let unit = dir / total_len;
        let dash_len = 6.0;
        let gap_len = 4.0;
        let mut dist = 0.0;
        while dist < total_len {
            let seg_start = p1 + unit * dist;
            let seg_end = p1 + unit * (dist + dash_len).min(total_len);
            painter.line_segment([seg_start, seg_end], stroke);
            dist += dash_len + gap_len;
        }
    }

    fn draw_dashed_circle(painter: &Painter, center: Pos2, r: f32, stroke: Stroke) {
        let steps = 40;
        for i in 0..steps {
            if i % 2 == 0 {
                let t1 = std::f32::consts::TAU * (i as f32 / steps as f32);
                let t2 = std::f32::consts::TAU * ((i + 1) as f32 / steps as f32);
                let p1 = Pos2::new(center.x + r * t1.cos(), center.y + r * t1.sin());
                let p2 = Pos2::new(center.x + r * t2.cos(), center.y + r * t2.sin());
                painter.line_segment([p1, p2], stroke);
            }
        }
    }

    fn draw_3d_orientation_gimbal(painter: &Painter, rect: Rect, cam: &super::model::Geo3DCamera) {
        let g_origin = Pos2::new(rect.left() + 45.0, rect.bottom() - 45.0);
        let len = 28.0;

        // X, Y, Z directions rotated by camera
        let (sin_y, cos_y) = cam.yaw.sin_cos();
        let (sin_p, cos_p) = cam.pitch.sin_cos();

        let project_dir = |dx: f32, dy: f32, dz: f32| -> Vec2 {
            let x1 = dx * cos_y + dz * sin_y;
            let y1 = dy;
            let z1 = -dx * sin_y + dz * cos_y;
            let y2 = y1 * cos_p - z1 * sin_p;
            Vec2::new(x1 * len, -y2 * len)
        };

        let vx = project_dir(1.0, 0.0, 0.0);
        let vy = project_dir(0.0, 1.0, 0.0);
        let vz = project_dir(0.0, 0.0, 1.0);

        painter.line_segment([g_origin, g_origin + vx], Stroke::new(1.8_f32, Color32::from_rgb(220, 50, 50)));
        painter.line_segment([g_origin, g_origin + vy], Stroke::new(1.8_f32, Color32::from_rgb(40, 180, 50)));
        painter.line_segment([g_origin, g_origin + vz], Stroke::new(1.8_f32, Color32::from_rgb(50, 100, 240)));

        painter.text(g_origin + vx, egui::Align2::CENTER_CENTER, "X", egui::FontId::proportional(11.0), Color32::from_rgb(220, 50, 50));
        painter.text(g_origin + vy, egui::Align2::CENTER_CENTER, "Y", egui::FontId::proportional(11.0), Color32::from_rgb(40, 180, 50));
        painter.text(g_origin + vz, egui::Align2::CENTER_CENTER, "Z", egui::FontId::proportional(11.0), Color32::from_rgb(50, 100, 240));
    }
}
