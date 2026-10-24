//! AutoCAD-Style Standalone Window Application for Inkk.
//! Provides an independent native OS window with professional CAD dark workspace,
//! custom vector icons, AutoCAD Ribbon, ViewCube, UCS icon, and Command Line.

use eframe::egui::{
    self, Color32, FontId, Frame, Layout, Painter, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};
use std::f32::consts::PI;
use std::path::PathBuf;

use super::cad_icons::CadIcon;
use super::engine::GeometryEngine;
use super::export::SketchpadExporter;
use super::model::{
    GeoCircle, GeoLine, GeoMeasurement, GeoPoint, GeoPolygon, MeasureKind, PointConstraint,
    SketchpadModel, SketchpadTool,
};
use super::presets_3d::Presets3D;

pub const CAD_BG: Color32 = Color32::from_rgb(24, 25, 29);
pub const CAD_GRID: Color32 = Color32::from_rgb(38, 40, 48);
pub const CAD_RIBBON: Color32 = Color32::from_rgb(32, 34, 40);
pub const CAD_STATUS_BG: Color32 = Color32::from_rgb(20, 21, 25);
pub const CAD_ACCENT_CYAN: Color32 = Color32::from_rgb(0, 229, 255);
pub const CAD_GREEN_SNAP: Color32 = Color32::from_rgb(0, 230, 118);
pub const CAD_TEXT_WHITE: Color32 = Color32::from_rgb(230, 235, 245);
pub const CAD_BORDER: Color32 = Color32::from_rgb(50, 53, 62);

pub struct CadStandaloneApp {
    pub model: SketchpadModel,
    pub active_tool: SketchpadTool,
    pub line_is_dashed: bool,
    pub current_color: Color32,
    pub command_line_text: String,
    pub prompt_text: String,
    pub status_notification: Option<(String, std::time::Instant)>,

    // CAD Canvas settings
    pub show_grid: bool,
    pub ortho_mode: bool,
    pub osnap_enabled: bool,
    pub crosshair_size: f32,

    // Mouse & Navigation
    pub cursor_world_pos: Pos2,
    pub dragging_point_id: Option<String>,
    pub is_orbiting_3d: bool,
    pub last_mouse_pos: Option<Pos2>,
    pub pending_start_id: Option<String>,
    pub pending_poly_ids: Vec<String>,
}

impl Default for CadStandaloneApp {
    fn default() -> Self {
        let mut model = SketchpadModel::new();
        Presets3D::load_sphere_with_inscribed_pyramid(&mut model);
        GeometryEngine::update_geometry(&mut model);

        Self {
            model,
            active_tool: SketchpadTool::Select,
            line_is_dashed: false,
            current_color: CAD_ACCENT_CYAN,
            command_line_text: String::new(),
            prompt_text: "Command: Ready. Select a CAD tool or click & drag objects.".to_string(),
            status_notification: None,

            show_grid: true,
            ortho_mode: false,
            osnap_enabled: true,
            crosshair_size: 24.0,

            cursor_world_pos: Pos2::ZERO,
            dragging_point_id: None,
            is_orbiting_3d: false,
            last_mouse_pos: None,
            pending_start_id: None,
            pending_poly_ids: Vec::new(),
        }
    }
}

impl CadStandaloneApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Path to shared IPC exchange file between CAD window and main whiteboard
    pub fn exchange_svg_path() -> PathBuf {
        std::env::temp_dir().join("inkk_cad_exchange.svg")
    }

    pub fn insert_to_whiteboard(&mut self) {
        let svg = SketchpadExporter::to_svg(&self.model, 800.0, 600.0, false);
        let path = Self::exchange_svg_path();
        if std::fs::write(&path, &svg).is_ok() {
            // Also copy PNG image to clipboard if possible
            if let Some((png_bytes, w, h)) = SketchpadExporter::to_raster_png(&self.model, 800.0, 600.0, false) {
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let img_data = arboard::ImageData {
                        width: w as usize,
                        height: h as usize,
                        bytes: std::borrow::Cow::Owned(png_bytes),
                    };
                    let _ = clip.set_image(img_data);
                }
            }
            self.set_notification("CAD Drawing sent to Whiteboard! (Saved to exchange & clipboard)");
            self.prompt_text = "Command: WHITEBOARD_EXPORT Done. Open Inkk Whiteboard to place.".to_string();
        } else {
            self.set_notification("Failed to write CAD exchange file.");
        }
    }

    pub fn set_notification(&mut self, msg: impl Into<String>) {
        self.status_notification = Some((msg.into(), std::time::Instant::now()));
    }
}

impl eframe::App for CadStandaloneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Continuous 3D rotation update
        if self.model.camera3d.is_auto_rotating || self.model.is_animating_points {
            GeometryEngine::update_geometry(&mut self.model);
            ctx.request_repaint();
        }

        // Top CAD Ribbon panel
        egui::TopBottomPanel::top("cad_ribbon")
            .frame(Frame::none().fill(CAD_RIBBON))
            .show(ctx, |ui| {
                self.render_cad_ribbon(ui);
            });

        // Bottom AutoCAD Command Line & Status Bar
        egui::TopBottomPanel::bottom("cad_command_bar")
            .frame(Frame::none().fill(CAD_STATUS_BG))
            .show(ctx, |ui| {
                self.render_cad_bottom_bar(ui);
            });

        // Central Drawing Canvas
        egui::CentralPanel::default()
            .frame(Frame::none().fill(CAD_BG))
            .show(ctx, |ui| {
                self.render_cad_canvas(ui);
            });
    }
}

impl CadStandaloneApp {
    fn render_cad_ribbon(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.add_space(6.0);

            // Group: DRAW
            ui.group(|ui| {
                ui.label(egui::RichText::new("DRAW").size(10.0).color(Color32::from_rgb(140, 145, 160)));
                ui.horizontal(|ui| {
                    if self.cad_ribbon_button(ui, "SELECT", self.active_tool == SketchpadTool::Select, CadIcon::draw_select) {
                        self.active_tool = SketchpadTool::Select;
                        self.prompt_text = "Command: SELECT. Drag points or select objects.".to_string();
                    }
                    if self.cad_ribbon_button(ui, "POINT", self.active_tool == SketchpadTool::Point, CadIcon::draw_point) {
                        self.active_tool = SketchpadTool::Point;
                        self.prompt_text = "Command: POINT. Specify point position:".to_string();
                    }
                    if self.cad_ribbon_button(ui, "LINE", self.active_tool == SketchpadTool::Segment, CadIcon::draw_line) {
                        self.active_tool = SketchpadTool::Segment;
                        self.line_is_dashed = false;
                        self.prompt_text = "Command: LINE. Specify first point:".to_string();
                    }
                    if self.cad_ribbon_button(ui, "DASHED", self.active_tool == SketchpadTool::DashedSegment, CadIcon::draw_dashed_line) {
                        self.active_tool = SketchpadTool::DashedSegment;
                        self.line_is_dashed = true;
                        self.prompt_text = "Command: DASHED_LINE. Specify first point (hidden edge):".to_string();
                    }
                    if self.cad_ribbon_button(ui, "RAY", self.active_tool == SketchpadTool::Ray, CadIcon::draw_ray) {
                        self.active_tool = SketchpadTool::Ray;
                        self.prompt_text = "Command: RAY. Specify origin point:".to_string();
                    }
                    if self.cad_ribbon_button(ui, "XLINE", self.active_tool == SketchpadTool::Line, CadIcon::draw_xline) {
                        self.active_tool = SketchpadTool::Line;
                        self.prompt_text = "Command: XLINE (Infinite Line). Specify two points:".to_string();
                    }
                    if self.cad_ribbon_button(ui, "CIRCLE", self.active_tool == SketchpadTool::Compass, CadIcon::draw_circle) {
                        self.active_tool = SketchpadTool::Compass;
                        self.prompt_text = "Command: CIRCLE. Specify center point:".to_string();
                    }
                    if self.cad_ribbon_button(ui, "POLYGON", self.active_tool == SketchpadTool::Polygon, CadIcon::draw_polygon) {
                        self.active_tool = SketchpadTool::Polygon;
                        self.pending_poly_ids.clear();
                        self.prompt_text = "Command: POLYGON. Click vertices, click start point to close:".to_string();
                    }
                });
            });

            // Group: 3D SOLIDS
            ui.group(|ui| {
                ui.label(egui::RichText::new("3D SOLIDS").size(10.0).color(Color32::from_rgb(140, 145, 160)));
                ui.horizontal(|ui| {
                    if self.cad_ribbon_button(ui, "CUBE 3D", false, CadIcon::draw_cube3d) {
                        Presets3D::load_cube_3d(&mut self.model);
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: 3D_CUBE. Loaded ABCD.A'B'C' with dashed hidden edges.".to_string();
                    }
                    if self.cad_ribbon_button(ui, "PYRAMID", false, CadIcon::draw_pyramid3d) {
                        Presets3D::load_square_pyramid_3d(&mut self.model);
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: 3D_PYRAMID. Loaded S.ABCD with dashed hidden edges.".to_string();
                    }
                    if self.cad_ribbon_button(ui, "SPHERE", false, CadIcon::draw_sphere3d) {
                        Presets3D::load_sphere_with_inscribed_pyramid(&mut self.model);
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: GSP_SPHERE_PYRAMID. Loaded Sphere with Inscribed Pyramid (GSP Image 2).".to_string();
                    }
                    if self.cad_ribbon_button(ui, "PRISM", false, CadIcon::draw_cube3d) {
                        Presets3D::load_triangular_prism_3d(&mut self.model);
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: 3D_PRISM. Loaded triangular prism.".to_string();
                    }
                });
            });

            // Group: MODIFY & OSNAP
            ui.group(|ui| {
                ui.label(egui::RichText::new("MODIFY").size(10.0).color(Color32::from_rgb(140, 145, 160)));
                ui.horizontal(|ui| {
                    if self.cad_ribbon_button(ui, "MOVE", false, CadIcon::draw_move) {
                        GeometryEngine::translate_selected(&mut self.model, Vec2::new(20.0, -20.0));
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: MOVE. Translated selected objects (+20, -20).".to_string();
                    }
                    if self.cad_ribbon_button(ui, "ROTATE", false, CadIcon::draw_rotate) {
                        let c = self.model.camera3d.center;
                        GeometryEngine::rotate_selected(&mut self.model, c, 60.0);
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: ROTATE. Rotated selected objects 60°.".to_string();
                    }
                    if self.cad_ribbon_button(ui, "MIRROR", false, CadIcon::draw_mirror) {
                        let c = self.model.camera3d.center;
                        GeometryEngine::reflect_selected_across_line(&mut self.model, c, c + Vec2::new(0.0, 100.0));
                        GeometryEngine::update_geometry(&mut self.model);
                        self.prompt_text = "Command: MIRROR. Mirrored across vertical axis.".to_string();
                    }
                    if self.cad_ribbon_button(ui, "MIDPOINT", false, CadIcon::draw_midpoint_snap) {
                        let sel_ids: Vec<String> = self.model.points.iter().filter(|p| p.is_selected).map(|p| p.id.clone()).collect();
                        if sel_ids.len() == 2 {
                            let id = self.model.gen_id("mid");
                            let mut pt = GeoPoint::new_2d(&id, format!("M{}", self.model.next_id), Pos2::ZERO);
                            pt.constraint = PointConstraint::Midpoint(sel_ids[0].clone(), sel_ids[1].clone());
                            self.model.points.push(pt);
                            GeometryEngine::update_geometry(&mut self.model);
                            self.prompt_text = "Command: MIDPOINT. Constructed midpoint between selected points.".to_string();
                        } else {
                            self.prompt_text = "Command: MIDPOINT. Select exactly 2 points first.".to_string();
                        }
                    }
                });
            });

            // Group: ANNOTATE & MEASURE
            ui.group(|ui| {
                ui.label(egui::RichText::new("ANNOTATE").size(10.0).color(Color32::from_rgb(140, 145, 160)));
                ui.horizontal(|ui| {
                    if self.cad_ribbon_button(ui, "DIM DIST", false, CadIcon::draw_dimension) {
                        let sel_ids: Vec<String> = self.model.points.iter().filter(|p| p.is_selected).map(|p| p.id.clone()).collect();
                        if sel_ids.len() == 2 {
                            let mid = self.model.gen_id("dim");
                            let y_pos = 25.0 + self.model.measurements.len() as f32 * 24.0;
                            self.model.measurements.push(GeoMeasurement {
                                id: mid,
                                kind: MeasureKind::Length,
                                target_ids: sel_ids,
                                cached_value: 0.0,
                                display_text: String::new(),
                                pos: Pos2::new(30.0, y_pos),
                            });
                            GeometryEngine::update_geometry(&mut self.model);
                            self.prompt_text = "Command: DIMLINEAR. Created distance dimension.".to_string();
                        } else {
                            self.prompt_text = "Command: DIMLINEAR. Select 2 points to dimension.".to_string();
                        }
                    }
                    if self.cad_ribbon_button(ui, "DIM ANGLE", false, CadIcon::draw_angle_dim) {
                        let sel_ids: Vec<String> = self.model.points.iter().filter(|p| p.is_selected).map(|p| p.id.clone()).collect();
                        if sel_ids.len() == 3 {
                            let mid = self.model.gen_id("angle");
                            let y_pos = 25.0 + self.model.measurements.len() as f32 * 24.0;
                            self.model.measurements.push(GeoMeasurement {
                                id: mid,
                                kind: MeasureKind::Angle,
                                target_ids: sel_ids,
                                cached_value: 0.0,
                                display_text: String::new(),
                                pos: Pos2::new(30.0, y_pos),
                            });
                            GeometryEngine::update_geometry(&mut self.model);
                            self.prompt_text = "Command: DIMANGULAR. Created angle dimension.".to_string();
                        } else {
                            self.prompt_text = "Command: DIMANGULAR. Select 3 points (vertex in middle).".to_string();
                        }
                    }
                    if self.cad_ribbon_button(ui, "AREA", false, CadIcon::draw_area) {
                        let poly_id_opt = self.model.polygons.first().map(|p| p.id.clone());
                        if let Some(poly_id) = poly_id_opt {
                            let mid = self.model.gen_id("area");
                            let y_pos = 25.0 + self.model.measurements.len() as f32 * 24.0;
                            self.model.measurements.push(GeoMeasurement {
                                id: mid,
                                kind: MeasureKind::Area,
                                target_ids: vec![poly_id],
                                cached_value: 0.0,
                                display_text: String::new(),
                                pos: Pos2::new(30.0, y_pos),
                            });
                            GeometryEngine::update_geometry(&mut self.model);
                            self.prompt_text = "Command: AREA. Measured polygon area.".to_string();
                        }
                    }
                });
            });

            // Group: 3D VIEW & ORBIT
            ui.group(|ui| {
                ui.label(egui::RichText::new("3D VIEW").size(10.0).color(Color32::from_rgb(140, 145, 160)));
                ui.horizontal(|ui| {
                    if self.cad_ribbon_button(ui, "3D ORBIT", self.active_tool == SketchpadTool::Rotate3D, CadIcon::draw_orbit3d) {
                        self.active_tool = SketchpadTool::Rotate3D;
                        self.prompt_text = "Command: 3DORBIT. Click & drag anywhere on canvas to rotate scene.".to_string();
                    }
                    let is_spinning = self.model.camera3d.is_auto_rotating;
                    let spin_lbl = if is_spinning { "STOP QUAY" } else { "QUAY 3D" };
                    if self.cad_ribbon_button(ui, spin_lbl, is_spinning, CadIcon::draw_orbit3d) {
                        self.model.camera3d.is_auto_rotating = !self.model.camera3d.is_auto_rotating;
                        self.prompt_text = if self.model.camera3d.is_auto_rotating {
                            "Command: 3DORBIT_CONTINUOUS Started.".to_string()
                        } else {
                            "Command: 3DORBIT_CONTINUOUS Stopped.".to_string()
                        };
                    }
                });
            });

            // Group: WHITEBOARD EXPORT
            ui.group(|ui| {
                ui.label(egui::RichText::new("OUTPUT").size(10.0).color(CAD_GREEN_SNAP));
                ui.horizontal(|ui| {
                    if self.cad_ribbon_button(ui, "TO WHITEBOARD", false, CadIcon::draw_insert_whiteboard) {
                        self.insert_to_whiteboard();
                    }
                });
            });
        });
    }

    fn cad_ribbon_button<F>(&mut self, ui: &mut Ui, label: &str, is_active: bool, draw_icon_fn: F) -> bool
    where
        F: FnOnce(&Painter, Rect, Stroke),
    {
        let button_size = Vec2::new(54.0, 48.0);
        let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());
        let is_hovered = response.hovered();

        let bg_col = if is_active {
            Color32::from_rgb(20, 60, 90)
        } else if is_hovered {
            Color32::from_rgb(45, 48, 58)
        } else {
            Color32::from_rgb(28, 30, 36)
        };

        let stroke_col = if is_active {
            CAD_ACCENT_CYAN
        } else if is_hovered {
            Color32::WHITE
        } else {
            Color32::from_rgb(180, 185, 200)
        };

        let painter = ui.painter();
        painter.rect_filled(rect, Rounding::same(3.0), bg_col);
        painter.rect_stroke(rect, Rounding::same(3.0), Stroke::new(1.0_f32, if is_active { CAD_ACCENT_CYAN } else { CAD_BORDER }));

        // Icon area in top 28px
        let icon_rect = Rect::from_min_size(rect.min + Vec2::new(0.0, 3.0), Vec2::new(rect.width(), 24.0));
        draw_icon_fn(painter, icon_rect, Stroke::new(1.6_f32, stroke_col));

        // Label in bottom 18px
        let label_pos = Pos2::new(rect.center().x, rect.bottom() - 6.0);
        painter.text(
            label_pos,
            egui::Align2::CENTER_BOTTOM,
            label,
            FontId::proportional(9.0),
            if is_active { CAD_ACCENT_CYAN } else { Color32::from_rgb(200, 205, 220) },
        );

        response.clicked()
    }

    fn render_cad_bottom_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            // Command prompt line
            ui.label(egui::RichText::new(&self.prompt_text).color(CAD_ACCENT_CYAN).monospace().size(12.0));

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                // Coordinates readout (AutoCAD style: X, Y, Z)
                let cx = (self.cursor_world_pos.x - self.model.camera3d.center.x) / 40.0;
                let cy = (self.model.camera3d.center.y - self.cursor_world_pos.y) / 40.0;
                let coords_str = format!("X: {:+7.2}  Y: {:+7.2}  Z: 0.00", cx, cy);
                ui.label(egui::RichText::new(coords_str).monospace().size(11.0).color(Color32::from_rgb(170, 175, 190)));
                ui.separator();

                // Quick CAD toggles
                let grid_lbl = if self.show_grid { "GRID: ON" } else { "GRID: OFF" };
                if ui.button(egui::RichText::new(grid_lbl).size(10.0)).clicked() {
                    self.show_grid = !self.show_grid;
                }

                let dash_lbl = if self.line_is_dashed { "DASHED: ON" } else { "DASHED: OFF" };
                if ui.button(egui::RichText::new(dash_lbl).size(10.0)).clicked() {
                    self.line_is_dashed = !self.line_is_dashed;
                }

                let orbit_lbl = if self.active_tool == SketchpadTool::Rotate3D { "ORBIT: ON" } else { "ORBIT: OFF" };
                if ui.button(egui::RichText::new(orbit_lbl).size(10.0)).clicked() {
                    self.active_tool = if self.active_tool == SketchpadTool::Rotate3D { SketchpadTool::Select } else { SketchpadTool::Rotate3D };
                }

                ui.separator();
                if let Some((notif, at)) = &self.status_notification {
                    if at.elapsed().as_secs_f32() < 4.0 {
                        ui.label(egui::RichText::new(notif).color(CAD_GREEN_SNAP).strong().size(11.0));
                    }
                }
            });
        });
    }

    fn render_cad_canvas(&mut self, ui: &mut Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let canvas_rect = response.rect;
        self.model.camera3d.center = canvas_rect.center();

        // Canvas Background: Deep charcoal
        painter.rect_filled(canvas_rect, 0.0, CAD_BG);

        // 1. AutoCAD Grid
        if self.show_grid {
            let grid_step = 40.0;
            let cx = self.model.camera3d.center.x;
            let cy = self.model.camera3d.center.y;

            let x_start = cx - ((cx - canvas_rect.left()) / grid_step).floor() * grid_step;
            let mut x = x_start;
            while x < canvas_rect.right() {
                painter.line_segment([Pos2::new(x, canvas_rect.top()), Pos2::new(x, canvas_rect.bottom())], Stroke::new(0.6_f32, CAD_GRID));
                x += grid_step;
            }

            let y_start = cy - ((cy - canvas_rect.top()) / grid_step).floor() * grid_step;
            let mut y = y_start;
            while y < canvas_rect.bottom() {
                painter.line_segment([Pos2::new(canvas_rect.left(), y), Pos2::new(canvas_rect.right(), y)], Stroke::new(0.6_f32, CAD_GRID));
                y += grid_step;
            }

            // Origin cross axis
            let axis_stroke = Stroke::new(1.2_f32, Color32::from_rgb(60, 65, 78));
            painter.line_segment([Pos2::new(canvas_rect.left(), cy), Pos2::new(canvas_rect.right(), cy)], axis_stroke);
            painter.line_segment([Pos2::new(cx, canvas_rect.top()), Pos2::new(cx, canvas_rect.bottom())], axis_stroke);
        }

        let get_pos = |id: &str, model: &SketchpadModel| -> Option<Pos2> {
            model.points.iter().find(|p| p.id == id).map(|p| p.pos)
        };

        // 2. Polygons
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
                    painter.line_segment([pts[i], pts[next]], Stroke::new(2.0_f32, poly.stroke_color));
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
                    let steps = 40;
                    for i in 0..steps {
                        if i % 2 == 0 {
                            let t1 = std::f32::consts::TAU * (i as f32 / steps as f32);
                            let t2 = std::f32::consts::TAU * ((i + 1) as f32 / steps as f32);
                            let p1 = Pos2::new(cp.x + r * t1.cos(), cp.y + r * t1.sin());
                            let p2 = Pos2::new(cp.x + r * t2.cos(), cp.y + r * t2.sin());
                            painter.line_segment([p1, p2], Stroke::new(1.8_f32, c.color));
                        }
                    }
                } else {
                    painter.circle_stroke(cp, r, Stroke::new(2.0_f32, c.color));
                }
            }
        }

        // 4. Lines (Solid & Dashed)
        for line in &self.model.lines {
            if line.is_hidden { continue; }
            if let (Some(p1), Some(p2)) = (get_pos(&line.p1, &self.model), get_pos(&line.p2, &self.model)) {
                let stroke = Stroke::new(line.width, line.color);
                if line.is_dashed {
                    let dir = p2 - p1;
                    let total_len = dir.length();
                    if total_len > 1.0 {
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
                } else {
                    painter.line_segment([p1, p2], stroke);
                }
            }
        }

        // 5. Points & CAD Grips
        for pt in &self.model.points {
            if pt.is_hidden { continue; }
            if pt.is_selected {
                // AutoCAD blue grip box around selected point
                painter.rect_filled(Rect::from_center_size(pt.pos, Vec2::splat(9.0)), 0.0, Color32::from_rgb(0, 120, 215));
                painter.rect_stroke(Rect::from_center_size(pt.pos, Vec2::splat(9.0)), 0.0, Stroke::new(1.2_f32, Color32::WHITE));
            } else {
                painter.circle_filled(pt.pos, 4.0, pt.color);
                painter.circle_stroke(pt.pos, 4.0, Stroke::new(1.0_f32, Color32::WHITE));
            }

            if !pt.label.is_empty() {
                painter.text(
                    pt.pos + Vec2::new(8.0, -8.0),
                    egui::Align2::LEFT_BOTTOM,
                    &pt.label,
                    FontId::proportional(15.0),
                    CAD_TEXT_WHITE,
                );
            }
        }

        // 6. Measurements
        for m in &self.model.measurements {
            let m_pos = canvas_rect.min + m.pos.to_vec2();
            painter.text(
                m_pos,
                egui::Align2::LEFT_TOP,
                &m.display_text,
                FontId::monospace(14.0),
                CAD_ACCENT_CYAN,
            );
        }

        // 7. AutoCAD UCS Coordinate Icon in bottom-left
        self.render_autocad_ucs_icon(&painter, canvas_rect);

        // 8. AutoCAD ViewCube in top-right
        self.render_autocad_viewcube(&painter, canvas_rect, ui);

        // 9. Interactive Pointer Handling & Crosshairs
        if let Some(mouse_pos) = response.hover_pos() {
            self.cursor_world_pos = mouse_pos;

            // Draw AutoCAD Crosshairs
            let ch_len = self.crosshair_size;
            let ch_stroke = Stroke::new(1.0_f32, Color32::from_rgb(180, 190, 210));
            painter.line_segment([Pos2::new(mouse_pos.x - ch_len, mouse_pos.y), Pos2::new(mouse_pos.x + ch_len, mouse_pos.y)], ch_stroke);
            painter.line_segment([Pos2::new(mouse_pos.x, mouse_pos.y - ch_len), Pos2::new(mouse_pos.x, mouse_pos.y + ch_len)], ch_stroke);
            painter.rect_stroke(Rect::from_center_size(mouse_pos, Vec2::splat(6.0)), 0.0, ch_stroke);

            // Find closest point
            let hit_pt_id = self.model.points.iter().find(|p| !p.is_hidden && (p.pos - mouse_pos).length_sq() < 100.0).map(|p| p.id.clone());

            // Snap indicator (AutoCAD green OSNAP box)
            if let Some(hid) = &hit_pt_id {
                if let Some(p) = self.model.points.iter().find(|p| p.id == *hid) {
                    painter.rect_stroke(Rect::from_center_size(p.pos, Vec2::splat(14.0)), 0.0, Stroke::new(1.8_f32, CAD_GREEN_SNAP));
                }
            }

            // Click handling
            if response.clicked() {
                match self.active_tool {
                    SketchpadTool::Select => {
                        let is_shift = ui.input(|i| i.modifiers.shift);
                        if let Some(hid) = &hit_pt_id {
                            for p in &mut self.model.points {
                                if p.id == *hid {
                                    p.is_selected = if is_shift { !p.is_selected } else { true };
                                } else if !is_shift {
                                    p.is_selected = false;
                                }
                            }
                        } else if !is_shift {
                            for p in &mut self.model.points {
                                p.is_selected = false;
                            }
                        }
                    }
                    SketchpadTool::Point => {
                        let pid = self.model.gen_id("pt");
                        let label = format!("P{}", self.model.next_id);
                        self.model.points.push(GeoPoint::new_2d(pid, label, mouse_pos));
                        self.prompt_text = "Command: POINT Created.".to_string();
                    }
                    SketchpadTool::Segment | SketchpadTool::DashedSegment | SketchpadTool::Ray | SketchpadTool::Line => {
                        if let Some(hid) = &hit_pt_id {
                            if let Some(start_id) = self.pending_start_id.take() {
                                if start_id != *hid {
                                    let lid = self.model.gen_id("line");
                                    let is_dash = self.active_tool == SketchpadTool::DashedSegment || self.line_is_dashed;
                                    let mut line = GeoLine::new(lid, start_id, hid, is_dash);
                                    if self.active_tool == SketchpadTool::Ray {
                                        line.kind = super::model::LineKind::Ray;
                                    } else if self.active_tool == SketchpadTool::Line {
                                        line.kind = super::model::LineKind::Line;
                                    }
                                    self.model.lines.push(line);
                                    self.prompt_text = "Command: LINE Completed segment.".to_string();
                                }
                            } else {
                                self.pending_start_id = Some(hid.clone());
                                self.prompt_text = "Command: LINE Specify second point:".to_string();
                            }
                        }
                    }
                    SketchpadTool::Compass => {
                        if let Some(hid) = &hit_pt_id {
                            if let Some(center_id) = self.pending_start_id.take() {
                                if center_id != *hid {
                                    let cid = self.model.gen_id("circ");
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
                                    self.prompt_text = "Command: CIRCLE Created.".to_string();
                                }
                            } else {
                                self.pending_start_id = Some(hid.clone());
                                self.prompt_text = "Command: CIRCLE Specify point on circumference:".to_string();
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
                                    fill_color: Some(Color32::from_rgba_unmultiplied(0, 229, 255, 60)),
                                    stroke_color: self.current_color,
                                    is_selected: false,
                                    is_hidden: false,
                                });
                                self.pending_poly_ids.clear();
                                self.prompt_text = "Command: POLYGON Closed and created.".to_string();
                            } else {
                                self.pending_poly_ids.push(hid.clone());
                                self.prompt_text = format!("Command: POLYGON {} vertices picked.", self.pending_poly_ids.len());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Dragging handling
            if response.drag_started() {
                if let Some(hid) = &hit_pt_id {
                    self.dragging_point_id = Some(hid.clone());
                } else {
                    self.is_orbiting_3d = true;
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
                    } else if self.is_orbiting_3d || self.active_tool == SketchpadTool::Rotate3D {
                        // 3D Orbit: yaw & pitch
                        self.model.camera3d.yaw += delta.x * 0.012;
                        self.model.camera3d.pitch = (self.model.camera3d.pitch - delta.y * 0.012).clamp(-PI * 0.48, PI * 0.48);
                        GeometryEngine::update_geometry(&mut self.model);
                    }
                }
                self.last_mouse_pos = Some(mouse_pos);
            }

            if response.drag_stopped() {
                self.dragging_point_id = None;
                self.is_orbiting_3d = false;
                self.last_mouse_pos = None;
            }
        }
    }

    /// Renders AutoCAD UCS (User Coordinate System) Icon at lower-left
    fn render_autocad_ucs_icon(&self, painter: &Painter, rect: Rect) {
        let ucs_origin = Pos2::new(rect.left() + 50.0, rect.bottom() - 50.0);
        let arm_len = 34.0;

        let (sin_y, cos_y) = self.model.camera3d.yaw.sin_cos();
        let (sin_p, cos_p) = self.model.camera3d.pitch.sin_cos();

        let project_dir = |dx: f32, dy: f32, dz: f32| -> Vec2 {
            let x1 = dx * cos_y + dz * sin_y;
            let y1 = dy;
            let z1 = -dx * sin_y + dz * cos_y;
            let y2 = y1 * cos_p - z1 * sin_p;
            Vec2::new(x1 * arm_len, -y2 * arm_len)
        };

        let vx = project_dir(1.0, 0.0, 0.0);
        let vy = project_dir(0.0, 1.0, 0.0);
        let vz = project_dir(0.0, 0.0, 1.0);

        // X (Red), Y (Green), Z (Blue)
        let red = Color32::from_rgb(255, 60, 60);
        let green = Color32::from_rgb(40, 220, 60);
        let blue = Color32::from_rgb(50, 130, 255);

        painter.line_segment([ucs_origin, ucs_origin + vx], Stroke::new(2.2_f32, red));
        painter.line_segment([ucs_origin, ucs_origin + vy], Stroke::new(2.2_f32, green));
        painter.line_segment([ucs_origin, ucs_origin + vz], Stroke::new(2.2_f32, blue));

        painter.text(ucs_origin + vx + Vec2::new(5.0, 0.0), egui::Align2::LEFT_CENTER, "X", FontId::monospace(12.0), red);
        painter.text(ucs_origin + vy + Vec2::new(0.0, -5.0), egui::Align2::CENTER_BOTTOM, "Y", FontId::monospace(12.0), green);
        painter.text(ucs_origin + vz + Vec2::new(5.0, 0.0), egui::Align2::LEFT_CENTER, "Z", FontId::monospace(12.0), blue);
        painter.text(ucs_origin + Vec2::new(-20.0, 15.0), egui::Align2::LEFT_TOP, "WCS", FontId::monospace(10.0), Color32::from_rgb(140, 145, 160));
    }

    /// Renders AutoCAD ViewCube in top-right corner
    fn render_autocad_viewcube(&mut self, painter: &Painter, rect: Rect, ui: &mut Ui) {
        let cube_center = Pos2::new(rect.right() - 60.0, rect.top() + 60.0);
        let s = 36.0;

        // Background compass ring
        painter.circle_stroke(cube_center, s + 12.0, Stroke::new(1.0_f32, Color32::from_rgb(60, 65, 75)));
        painter.text(cube_center + Vec2::new(0.0, -s - 18.0), egui::Align2::CENTER_CENTER, "N", FontId::monospace(10.0), Color32::from_rgb(180, 190, 200));

        let cube_rect = Rect::from_center_size(cube_center, Vec2::splat(s * 2.0));
        let is_hovered = ui.rect_contains_pointer(cube_rect);

        // Cube face
        painter.rect_filled(cube_rect, Rounding::same(4.0), Color32::from_rgb(38, 42, 52));
        painter.rect_stroke(cube_rect, Rounding::same(4.0), Stroke::new(1.5_f32, if is_hovered { CAD_ACCENT_CYAN } else { Color32::from_rgb(80, 85, 100) }));

        // Current face label based on orientation
        let view_label = if self.model.camera3d.pitch > 1.2 {
            "TOP"
        } else if self.model.camera3d.pitch < -1.2 {
            "BOTTOM"
        } else if self.model.camera3d.yaw.abs() < 0.4 {
            "FRONT"
        } else if (self.model.camera3d.yaw - PI * 0.5).abs() < 0.4 {
            "RIGHT"
        } else {
            "ISO"
        };

        painter.text(cube_center, egui::Align2::CENTER_CENTER, view_label, FontId::monospace(13.0), Color32::WHITE);

        if is_hovered && ui.input(|i| i.pointer.primary_clicked()) {
            // Cycle views: TOP -> FRONT -> RIGHT -> ISO
            if view_label == "TOP" {
                self.model.camera3d.pitch = 0.0;
                self.model.camera3d.yaw = 0.0;
                self.prompt_text = "Command: VIEW FRONT (Oxz).".to_string();
            } else if view_label == "FRONT" {
                self.model.camera3d.pitch = 0.0;
                self.model.camera3d.yaw = PI * 0.5;
                self.prompt_text = "Command: VIEW RIGHT (Oyz).".to_string();
            } else if view_label == "RIGHT" {
                self.model.camera3d.pitch = 0.35;
                self.model.camera3d.yaw = -0.55;
                self.prompt_text = "Command: VIEW SW_ISOMETRIC.".to_string();
            } else {
                self.model.camera3d.pitch = PI * 0.5;
                self.model.camera3d.yaw = 0.0;
                self.prompt_text = "Command: VIEW TOP (Oxy).".to_string();
            }
            GeometryEngine::update_geometry(&mut self.model);
        }
    }
}
