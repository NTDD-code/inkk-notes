use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::collections::HashSet;

use crate::model::{
    Notebook, PaperType, PenType, Point, ShapeType, Stroke as NoteStroke, ToolMode,
};
use crate::renderer::NoteRenderer;
use crate::storage::NotebookStorage;
use crate::stroke_algorithm::StabilizerPipeline;

fn point_in_polygon(p: Pos2, poly: &[Pos2]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let pi = poly[i];
        let pj = poly[j];
        if ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub struct HandNoteApp {
    notebook: Notebook,
    current_stroke: Option<NoteStroke>,
    redo_stack: Vec<NoteStroke>,

    // Tool state
    tool: ToolMode,
    pen_type: PenType,
    current_shape: ShapeType,
    is_shape_filled: bool,
    color: Color32,
    size: f32,
    opacity: f32,

    // C++ One-Euro Stabilizer Pipeline & Timestamp Tracker
    stabilizer: StabilizerPipeline,
    app_start_time: std::time::Instant,
    current_pressure: f32,
    last_pointer_time: std::time::Instant,
    last_pointer_pos_world: Pos2,

    // Viewport transform
    scale: f32,
    pan: Vec2,

    // Selection state
    selected_stroke_ids: HashSet<String>,
    lasso_points: Vec<Pos2>,
    is_dragging_selection: bool,
    drag_last_pos_world: Pos2,

    // UI & Theme state
    is_dark_mode: bool,
    show_previews_drawer: bool,
    current_file_path: Option<std::path::PathBuf>,
    status_message: Option<(String, std::time::Instant)>,
}

impl Default for HandNoteApp {
    fn default() -> Self {
        Self {
            notebook: Notebook::default(),
            current_stroke: None,
            redo_stack: Vec::new(),

            tool: ToolMode::Pen,
            pen_type: PenType::Ballpoint,
            current_shape: ShapeType::Rectangle,
            is_shape_filled: false,
            color: Color32::from_rgb(0, 0, 0),
            size: 3.0,
            opacity: 1.0,

            stabilizer: StabilizerPipeline::new(),
            app_start_time: std::time::Instant::now(),
            current_pressure: 0.5,
            last_pointer_time: std::time::Instant::now(),
            last_pointer_pos_world: Pos2::ZERO,

            scale: 0.8,
            pan: Vec2::new(140.0, 50.0),

            selected_stroke_ids: HashSet::new(),
            lasso_points: Vec::new(),
            is_dragging_selection: false,
            drag_last_pos_world: Pos2::ZERO,

            is_dark_mode: false,
            show_previews_drawer: false,
            current_file_path: None,
            status_message: None,
        }
    }
}

impl HandNoteApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();
        cc.egui_ctx.set_visuals(if app.is_dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        app
    }

    fn to_world(&self, screen: Pos2) -> Pos2 {
        Pos2::new(
            (screen.x - self.pan.x) / self.scale,
            (screen.y - self.pan.y) / self.scale,
        )
    }

    fn to_screen(&self, world: Pos2) -> Pos2 {
        Pos2::new(
            world.x * self.scale + self.pan.x,
            world.y * self.scale + self.pan.y,
        )
    }

    fn fit_to_window(&mut self, available_size: Vec2) {
        let page = self.notebook.current_page();
        let scale_x = (available_size.x - 80.0) / page.width;
        let scale_y = (available_size.y - 120.0) / page.height;
        let fit_scale = scale_x.min(scale_y).clamp(0.2, 1.4);

        self.scale = fit_scale;
        self.pan = Vec2::new(
            (available_size.x - page.width * fit_scale) * 0.5,
            ((available_size.y - page.height * fit_scale) * 0.5).max(30.0),
        );
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), std::time::Instant::now()));
    }

    fn get_selection_bounds(&self) -> Option<Rect> {
        if self.selected_stroke_ids.is_empty() {
            return None;
        }
        let page = self.notebook.current_page();
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut found = false;

        for stroke in &page.strokes {
            if self.selected_stroke_ids.contains(&stroke.id) {
                for pt in &stroke.points {
                    found = true;
                    if pt.x < min_x { min_x = pt.x; }
                    if pt.x > max_x { max_x = pt.x; }
                    if pt.y < min_y { min_y = pt.y; }
                    if pt.y > max_y { max_y = pt.y; }
                }
            }
        }

        if found {
            let min_x = min_x.min(max_x);
            let max_x = max_x.max(min_x + 2.0);
            let min_y = min_y.min(max_y);
            let max_y = max_y.max(min_y + 2.0);
            Some(Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y)))
        } else {
            None
        }
    }

    fn delete_selected_strokes(&mut self) {
        if self.selected_stroke_ids.is_empty() {
            return;
        }
        let count = self.selected_stroke_ids.len();
        let page = self.notebook.current_page_mut();
        page.strokes.retain(|s| !self.selected_stroke_ids.contains(&s.id));
        self.selected_stroke_ids.clear();
        self.set_status(&format!("Deleted {} stroke(s)", count));
    }

    fn render_top_toolbar(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        egui::Frame::window(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🖊 HandNote").strong());
                ui.separator();

                // Tools selection
                let is_shape_tool = matches!(self.tool, ToolMode::Shape(_));
                let tools = [
                    (ToolMode::Pen, "✏ Pen"),
                    (ToolMode::Shape(self.current_shape), "📐 Shape"),
                    (ToolMode::Highlighter, "🖍 Highlighter"),
                    (ToolMode::EraserStroke, "🧹 Stroke Eraser"),
                    (ToolMode::EraserPoint, "✂ Point Eraser"),
                    (ToolMode::LassoSelect, "⭕ Lasso Select"),
                    (ToolMode::Pan, "✋ Pan"),
                ];

                for (mode, label) in tools {
                    let is_active = match (self.tool, mode) {
                        (ToolMode::Shape(_), ToolMode::Shape(_)) => true,
                        (a, b) => a == b,
                    };
                    if ui.selectable_label(is_active, label).clicked() {
                        self.tool = mode;
                        if mode != ToolMode::LassoSelect {
                            self.selected_stroke_ids.clear();
                        }
                    }
                }

                ui.separator();

                // Pen subtypes (Ballpoint, Fountain, Calligraphy, Pencil)
                if self.tool == ToolMode::Pen {
                    ui.selectable_value(&mut self.pen_type, PenType::Ballpoint, "Ballpoint");
                    ui.selectable_value(&mut self.pen_type, PenType::Fountain, "✒ Fountain");
                    ui.selectable_value(&mut self.pen_type, PenType::Calligraphy, "Calligraphy");
                    ui.selectable_value(&mut self.pen_type, PenType::Pencil, "Pencil");
                    ui.separator();
                } else if is_shape_tool {
                    egui::ComboBox::from_id_salt("shape_select")
                        .selected_text(format!("{:?}", self.current_shape))
                        .show_ui(ui, |ui| {
                            let shapes = [
                                ShapeType::Line,
                                ShapeType::Arrow,
                                ShapeType::DoubleArrow,
                                ShapeType::Rectangle,
                                ShapeType::Circle,
                                ShapeType::Triangle,
                                ShapeType::Pentagon,
                                ShapeType::Hexagon,
                                ShapeType::Star,
                            ];
                            for s in shapes {
                                if ui.selectable_label(self.current_shape == s, format!("{:?}", s)).clicked() {
                                    self.current_shape = s;
                                    self.tool = ToolMode::Shape(s);
                                }
                            }
                        });
                    ui.checkbox(&mut self.is_shape_filled, "Filled");
                    ui.separator();
                }

                // Quick Color Swatches
                let quick_colors = [
                    Color32::from_rgb(0, 0, 0),         // Black
                    Color32::from_rgb(248, 250, 252),   // White
                    Color32::from_rgb(37, 99, 235),     // Blue
                    Color32::from_rgb(220, 38, 38),     // Red
                    Color32::from_rgb(22, 163, 74),     // Green
                    Color32::from_rgb(234, 179, 8),     // Yellow
                    Color32::from_rgb(147, 51, 234),    // Purple
                ];

                for c in quick_colors {
                    let is_cur = self.color == c;
                    if ui.radio(is_cur, "").clicked() {
                        self.color = c;
                    }
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
                    ui.painter().circle_filled(rect.center(), 7.0, c);
                    ui.painter().circle_stroke(rect.center(), 7.0, Stroke::new(1.0_f32, Color32::from_gray(140)));
                }

                ui.color_edit_button_srgba(&mut self.color);

                ui.separator();

                // Stroke Size Slider
                ui.add(egui::Slider::new(&mut self.size, 1.0..=30.0).text("Size").suffix("px"));

                ui.separator();

                // Paper Style Selector
                let cur_paper = self.notebook.current_page().paper_type;
                let mut selected_paper = cur_paper;
                egui::ComboBox::from_id_salt("paper_combo")
                    .selected_text(cur_paper.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected_paper, PaperType::Ruled, PaperType::Ruled.name());
                        ui.selectable_value(&mut selected_paper, PaperType::Grid, PaperType::Grid.name());
                        ui.selectable_value(&mut selected_paper, PaperType::DotGrid, PaperType::DotGrid.name());
                        ui.selectable_value(&mut selected_paper, PaperType::Cornell, PaperType::Cornell.name());
                        ui.selectable_value(&mut selected_paper, PaperType::Blank, PaperType::Blank.name());
                    });

                if selected_paper != cur_paper {
                    self.notebook.current_page_mut().paper_type = selected_paper;
                }

                ui.separator();

                // Undo / Redo
                if ui.button("⮌ Undo").clicked() {
                    self.undo();
                }
                if ui.button("⮎ Redo").clicked() {
                    self.redo();
                }

                ui.separator();

                // Theme Toggle (Light / Dark)
                let theme_label = if self.is_dark_mode { "☀ Light" } else { "🌙 Dark" };
                if ui.button(theme_label).clicked() {
                    self.is_dark_mode = !self.is_dark_mode;
                    ctx.set_visuals(if self.is_dark_mode {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    });

                    // Smart default color adjustment when switching theme
                    if self.is_dark_mode && self.color == Color32::from_rgb(0, 0, 0) {
                        self.color = Color32::from_rgb(248, 250, 252);
                    } else if !self.is_dark_mode && self.color == Color32::from_rgb(248, 250, 252) {
                        self.color = Color32::from_rgb(0, 0, 0);
                    }
                }

                // Selection Actions Bar (if strokes are selected)
                if !self.selected_stroke_ids.is_empty() {
                    ui.separator();
                    ui.label(egui::RichText::new(format!("Selected: {}", self.selected_stroke_ids.len())).color(Color32::from_rgb(59, 130, 246)).strong());
                    if ui.button("🗑 Delete").clicked() {
                        self.delete_selected_strokes();
                    }
                    if ui.button("✕ Deselect").clicked() {
                        self.selected_stroke_ids.clear();
                    }
                }
            });
        });
    }

    fn render_bottom_bar(&mut self, ui: &mut Ui, available_size: Vec2) {
        ui.horizontal(|ui| {
            // Notebook Title
            ui.text_edit_singleline(&mut self.notebook.title);
            ui.separator();

            // Page Navigation
            let cur_idx = self.notebook.current_page_index + 1;
            let total = self.notebook.pages.len();

            if ui.button("‹ Prev").clicked() {
                self.notebook.prev_page();
                self.selected_stroke_ids.clear();
            }

            // Previews drawer toggle button
            if ui.selectable_label(self.show_previews_drawer, format!("📑 Page {} / {}", cur_idx, total)).clicked() {
                self.show_previews_drawer = !self.show_previews_drawer;
            }

            if ui.button("Next ›").clicked() {
                self.notebook.next_page();
                self.selected_stroke_ids.clear();
            }

            if ui.button("➕ Add Page").clicked() {
                self.notebook.add_page();
                self.selected_stroke_ids.clear();
            }

            // Extend Page Button (+600px)
            if ui.button("♾️ Extend (+600px)").clicked() {
                self.notebook.current_page_mut().extend_height(600.0);
                self.set_status("Extended page +600px!");
            }

            ui.separator();

            // Zoom Controls
            if ui.button("−").clicked() {
                self.scale = (self.scale * 0.85).clamp(0.2, 5.0);
            }
            ui.label(format!("{}%", (self.scale * 100.0).round() as i32));
            if ui.button("＋").clicked() {
                self.scale = (self.scale * 1.15).clamp(0.2, 5.0);
            }
            if ui.button("⊞ Fit").clicked() {
                self.fit_to_window(available_size);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Save Button
                if ui.button("💾 Save (.handnote)").clicked() {
                    self.prompt_save_file();
                }

                // Open Button
                if ui.button("📂 Open").clicked() {
                    self.prompt_open_file();
                }

                if let Some((msg, time)) = &self.status_message {
                    if time.elapsed().as_secs() < 4 {
                        ui.label(egui::RichText::new(msg).color(Color32::from_rgb(34, 197, 94)).strong());
                    }
                }
            });
        });
    }

    fn render_previews_drawer(&mut self, ctx: &egui::Context) {
        if !self.show_previews_drawer {
            return;
        }

        egui::SidePanel::right("previews_drawer")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📑 Page Previews");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            self.show_previews_drawer = false;
                        }
                    });
                });

                ui.separator();

                if ui.button("➕ Add New Page").clicked() {
                    self.notebook.add_page();
                    self.selected_stroke_ids.clear();
                }

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let total_pages = self.notebook.pages.len();
                    let cur_idx = self.notebook.current_page_index;

                    for idx in 0..total_pages {
                        let is_active = idx == cur_idx;
                        ui.group(|ui| {
                            let label = if is_active {
                                format!("● Page {} (Active)", idx + 1)
                            } else {
                                format!("Page {}", idx + 1)
                            };

                            if ui.selectable_label(is_active, label).clicked() {
                                self.notebook.current_page_index = idx;
                                self.selected_stroke_ids.clear();
                            }

                            // Thumbnail Mini Canvas Preview
                            let thumb_size = Vec2::new(200.0, 140.0);
                            let (rect, _) = ui.allocate_exact_size(thumb_size, Sense::hover());
                            let painter = ui.painter().with_clip_rect(rect);

                            // Background of thumbnail
                            let thumb_bg = if self.is_dark_mode {
                                Color32::from_rgb(35, 40, 48)
                            } else {
                                Color32::WHITE
                            };
                            painter.rect_filled(rect, 4.0, thumb_bg);
                            painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, Color32::from_gray(180)));

                            let page = &self.notebook.pages[idx];
                            let thumb_scale = (thumb_size.x / page.width).min(thumb_size.y / page.height) * 0.95;
                            let thumb_pan = rect.min + Vec2::new(
                                (thumb_size.x - page.width * thumb_scale) * 0.5,
                                (thumb_size.y - page.height * thumb_scale) * 0.5,
                            );

                            let to_thumb = |p: Pos2| Pos2::new(p.x * thumb_scale + thumb_pan.x, p.y * thumb_scale + thumb_pan.y);

                            // Render mini strokes (highlighters underneath regular ink)
                            NoteRenderer::render_strokes(&painter, &page.strokes, thumb_scale, &to_thumb);

                            ui.horizontal(|ui| {
                                if ui.button("📑 Duplicate").clicked() {
                                    self.notebook.current_page_index = idx;
                                    self.notebook.duplicate_current_page();
                                    self.selected_stroke_ids.clear();
                                }
                                if ui.button("🗑️ Delete").clicked() {
                                    if total_pages > 1 {
                                        self.notebook.current_page_index = idx;
                                        self.notebook.delete_current_page();
                                        self.selected_stroke_ids.clear();
                                    }
                                }
                            });
                        });
                        ui.add_space(8.0);
                    }
                });
            });
    }

    fn undo(&mut self) {
        let page = self.notebook.current_page_mut();
        if let Some(popped) = page.strokes.pop() {
            self.redo_stack.push(popped);
        }
        self.selected_stroke_ids.clear();
    }

    fn redo(&mut self) {
        if let Some(stroke) = self.redo_stack.pop() {
            let page = self.notebook.current_page_mut();
            page.strokes.push(stroke);
        }
        self.selected_stroke_ids.clear();
    }

    fn prompt_save_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Notebook")
            .add_filter("HandNote Files", &["handnote"])
            .set_file_name(&format!("{}.handnote", self.notebook.title))
            .save_file()
        {
            if let Err(e) = NotebookStorage::save(&path, &self.notebook) {
                self.set_status(&format!("Save error: {}", e));
            } else {
                self.current_file_path = Some(path);
                self.set_status("Notebook saved successfully! ✓");
            }
        }
    }

    fn prompt_open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Notebook")
            .add_filter("HandNote Files", &["handnote"])
            .pick_file()
        {
            match NotebookStorage::load(&path) {
                Ok(nb) => {
                    self.notebook = nb;
                    self.current_file_path = Some(path);
                    self.selected_stroke_ids.clear();
                    self.set_status("Notebook loaded! ✓");
                }
                Err(e) => {
                    self.set_status(&format!("Open error: {}", e));
                }
            }
        }
    }

    // --- ERASERS ---

    fn erase_stroke_at(&mut self, world_pos: Pos2, radius: f32) {
        let r_sq = radius * radius;
        let page = self.notebook.current_page_mut();
        page.strokes.retain(|stroke| {
            !stroke.points.iter().any(|pt| {
                let dx = pt.x - world_pos.x;
                let dy = pt.y - world_pos.y;
                dx * dx + dy * dy <= r_sq
            })
        });
    }

    fn erase_point_at(&mut self, world_pos: Pos2, radius: f32) {
        let r_sq = radius * radius;
        let page = self.notebook.current_page_mut();
        let mut new_strokes = Vec::new();

        for stroke in &page.strokes {
            let mut sub_segments = Vec::new();
            let mut cur_seg = Vec::new();

            for pt in &stroke.points {
                let dx = pt.x - world_pos.x;
                let dy = pt.y - world_pos.y;
                if dx * dx + dy * dy <= r_sq {
                    if !cur_seg.is_empty() {
                        sub_segments.push(cur_seg);
                        cur_seg = Vec::new();
                    }
                } else {
                    cur_seg.push(*pt);
                }
            }
            if !cur_seg.is_empty() {
                sub_segments.push(cur_seg);
            }

            for seg in sub_segments {
                let mut s = stroke.clone();
                s.id = format!("stroke_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
                s.points = seg;
                new_strokes.push(s);
            }
        }

        page.strokes = new_strokes;
    }
}

impl eframe::App for HandNoteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keep theme visuals synchronized
        if ctx.style().visuals.dark_mode != self.is_dark_mode {
            ctx.set_visuals(if self.is_dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            });
        }

        // Previews Drawer
        self.render_previews_drawer(ctx);

        // Top Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.render_top_toolbar(ui, ctx);
        });

        // Bottom Panel
        let available_size = ctx.screen_rect().size();
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.render_bottom_bar(ui, available_size);
        });

        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            self.undo();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.redo();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            if !self.selected_stroke_ids.is_empty() {
                self.delete_selected_strokes();
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_stroke_ids.clear();
        }

        // Central Canvas Area
        let desk_color = if self.is_dark_mode {
            Color32::from_rgb(20, 23, 28)
        } else {
            Color32::from_rgb(238, 242, 246)
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(desk_color))
            .show(ctx, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());
                let painter = ui.painter_at(canvas_rect);

                // Handle Wheel Zoom & Pan
                if response.hovered() {
                    let scroll_delta = ui.input(|i| i.raw_scroll_delta);
                    if scroll_delta != Vec2::ZERO {
                        if ui.input(|i| i.modifiers.ctrl) {
                            let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 0.9 };
                            if let Some(cursor) = response.hover_pos() {
                                let new_scale = (self.scale * zoom_factor).clamp(0.2, 5.0);
                                self.pan = cursor.to_vec2() - (cursor.to_vec2() - self.pan) * (new_scale / self.scale);
                                self.scale = new_scale;
                            }
                        } else {
                            self.pan += scroll_delta;
                        }
                    }
                }

                let page = self.notebook.current_page();
                let page_w = page.width;
                let page_h = page.height;
                let paper_type = page.paper_type;

                let page_top_left = self.to_screen(Pos2::new(0.0, 0.0));
                let page_bottom_right = self.to_screen(Pos2::new(page_w, page_h));
                let page_screen_rect = Rect::from_min_max(page_top_left, page_bottom_right);

                let is_dark = self.is_dark_mode;
                let scale = self.scale;
                let to_screen = |p: Pos2| self.to_screen(p);

                // 1. Render Desk & Page Paper Sheet
                NoteRenderer::render_background(
                    &painter,
                    page_screen_rect,
                    paper_type,
                    page_w,
                    page_h,
                    scale,
                    &to_screen,
                    is_dark,
                );

                // 2. Render Existing Strokes and Active Stroke with Layering:
                // Strictly clipped to page_screen_rect so NO ink ever renders outside the paper sheet!
                let page_painter = painter.with_clip_rect(page_screen_rect);
                let page = self.notebook.current_page();

                // Layer 1: Highlighters (saved)
                for stroke in &page.strokes {
                    if stroke.pen_type == PenType::Highlighter {
                        NoteRenderer::render_single_stroke(&page_painter, stroke, scale, &to_screen);
                    }
                }
                // If active stroke is highlighter, render here
                if let Some(ref cur) = self.current_stroke {
                    if cur.pen_type == PenType::Highlighter {
                        NoteRenderer::render_single_stroke(&page_painter, cur, scale, &to_screen);
                    }
                }

                // Layer 2: Normal ink strokes (ballpoint, calligraphy, pencil)
                for stroke in &page.strokes {
                    if stroke.pen_type != PenType::Highlighter {
                        NoteRenderer::render_single_stroke(&page_painter, stroke, scale, &to_screen);
                    }
                }
                // If active stroke is normal ink, render here
                if let Some(ref cur) = self.current_stroke {
                    if cur.pen_type != PenType::Highlighter {
                        NoteRenderer::render_single_stroke(&page_painter, cur, scale, &to_screen);
                    }
                }

                // 3. Render Lasso Selection Visuals
                // 3a. In-progress Lasso Path
                if self.tool == ToolMode::LassoSelect && self.lasso_points.len() >= 2 {
                    let screen_pts: Vec<Pos2> = self.lasso_points.iter().map(|p| self.to_screen(*p)).collect();
                    let n = screen_pts.len();
                    for i in 0..n - 1 {
                        painter.line_segment(
                            [screen_pts[i], screen_pts[i + 1]],
                            Stroke::new(1.8_f32, Color32::from_rgb(59, 130, 246)),
                        );
                    }
                    if n >= 3 {
                        painter.line_segment(
                            [screen_pts[n - 1], screen_pts[0]],
                            Stroke::new(1.8_f32, Color32::from_rgb(59, 130, 246)),
                        );
                    }
                }

                // 3b. Selection Bounding Box & Handles
                if let Some(world_bounds) = self.get_selection_bounds() {
                    let p_min = self.to_screen(world_bounds.min);
                    let p_max = self.to_screen(world_bounds.max);
                    let screen_bounds = Rect::from_min_max(p_min, p_max).expand(8.0);

                    // Selection dashed/accent box
                    let box_color = Color32::from_rgb(59, 130, 246);
                    painter.rect_stroke(screen_bounds, 4.0, Stroke::new(1.5_f32, box_color));

                    // Corner & Edge Handles
                    let handle_radius = 4.0;
                    let handle_fill = if self.is_dark_mode { Color32::from_rgb(30, 35, 45) } else { Color32::WHITE };
                    let handle_stroke = Stroke::new(1.5_f32, box_color);

                    let handles = [
                        screen_bounds.left_top(),
                        screen_bounds.right_top(),
                        screen_bounds.left_bottom(),
                        screen_bounds.right_bottom(),
                        Pos2::new(screen_bounds.center().x, screen_bounds.top()),
                        Pos2::new(screen_bounds.center().x, screen_bounds.bottom()),
                        Pos2::new(screen_bounds.left(), screen_bounds.center().y),
                        Pos2::new(screen_bounds.right(), screen_bounds.center().y),
                    ];
                    for h in handles {
                        painter.circle(h, handle_radius, handle_fill, handle_stroke);
                    }

                    // Selection info badge
                    let badge_text = format!("{} selected | Drag to move | [Del] to delete", self.selected_stroke_ids.len());
                    let badge_pos = Pos2::new(screen_bounds.left(), (screen_bounds.top() - 18.0).max(10.0));
                    painter.text(
                        badge_pos,
                        egui::Align2::LEFT_BOTTOM,
                        badge_text,
                        egui::FontId::proportional(12.0),
                        box_color,
                    );
                }

                // 4. Robust & Zero-Threshold Inking / Selection Pointer Tracking
                let pointer = ui.input(|i| i.pointer.clone());
                let pointer_pos = pointer.interact_pos().or(pointer.latest_pos());
                let is_primary_down = pointer.primary_down();
                let is_primary_pressed = pointer.primary_pressed();

                // Detect hardware pressure from Wacom / Stylus touch events
                let mut hardware_force: Option<f32> = None;
                ui.input(|i| {
                    for ev in &i.events {
                        if let egui::Event::Touch { force, .. } = ev {
                            if let Some(f) = force {
                                hardware_force = Some(*f);
                            }
                        }
                    }
                });

                // Middle mouse drag or Pan tool
                if (self.tool == ToolMode::Pan && is_primary_down) || pointer.middle_down() {
                    self.pan += pointer.delta();
                } else if is_primary_pressed && response.hovered() {
                    if let Some(cursor) = pointer_pos {
                        let world = self.to_world(cursor);
                        match self.tool {
                            ToolMode::Pan => {}
                            ToolMode::EraserStroke => {
                                if world.x >= 0.0 && world.x <= page_w && world.y >= 0.0 {
                                    self.erase_stroke_at(world, (self.size * 5.0) / self.scale);
                                }
                            }
                            ToolMode::EraserPoint => {
                                if world.x >= 0.0 && world.x <= page_w && world.y >= 0.0 {
                                    self.erase_point_at(world, (self.size * 3.5) / self.scale);
                                }
                            }
                            ToolMode::LassoSelect => {
                                let world_bounds = self.get_selection_bounds();
                                let hit_existing_selection = if let Some(wb) = world_bounds {
                                    let sb = Rect::from_min_max(self.to_screen(wb.min), self.to_screen(wb.max)).expand(12.0);
                                    sb.contains(cursor)
                                } else {
                                    false
                                };

                                if hit_existing_selection {
                                    self.is_dragging_selection = true;
                                    self.drag_last_pos_world = world;
                                } else {
                                    self.selected_stroke_ids.clear();
                                    self.is_dragging_selection = false;
                                    self.lasso_points = vec![world];
                                }
                            }
                            ToolMode::Pen | ToolMode::Highlighter | ToolMode::Shape(_) => {
                                // Strictly forbid starting a stroke outside the paper page boundaries!
                                if world.x >= 0.0 && world.x <= page_w && world.y >= 0.0 {
                                    let pen = if self.tool == ToolMode::Highlighter {
                                        PenType::Highlighter
                                    } else {
                                        self.pen_type
                                    };

                                    let shape = match self.tool {
                                        ToolMode::Shape(s) => s,
                                        _ => ShapeType::None,
                                    };

                                    let pressure = if let Some(f) = hardware_force {
                                        self.current_pressure = f.clamp(0.05, 1.0);
                                        self.current_pressure
                                    } else {
                                        0.5
                                    };

                                    self.last_pointer_time = std::time::Instant::now();
                                    self.last_pointer_pos_world = world;

                                    // Reset One-Euro stabilizer pipeline (from C++ StabilizerPipeline::Reset)
                                    self.stabilizer.set_dpi_scale(ctx.pixels_per_point());
                                    self.stabilizer.reset(world.x, world.y, pressure);

                                    let _timestamp_sec = self.app_start_time.elapsed().as_secs_f64();
                                    let initial_pt = Point::with_thickness(
                                        world.x,
                                        world.y,
                                        pressure,
                                        crate::stroke_algorithm::compute_thickness(pen, self.size, 0.0, pressure),
                                    );

                                    let c = [self.color.r(), self.color.g(), self.color.b(), self.color.a()];
                                    self.current_stroke = Some(NoteStroke {
                                        id: format!("stroke_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                                        pen_type: pen,
                                        shape_type: shape,
                                        is_filled: self.is_shape_filled,
                                        color: c,
                                        size: self.size,
                                        opacity: self.opacity,
                                        points: vec![initial_pt],
                                    });
                                    self.redo_stack.clear();
                                }
                            }
                        }
                    }
                } else if is_primary_down {
                    if let Some(cursor) = pointer_pos {
                        let world = self.to_world(cursor);
                        match self.tool {
                            ToolMode::EraserStroke => {
                                if world.x >= 0.0 && world.x <= page_w && world.y >= 0.0 {
                                    self.erase_stroke_at(world, (self.size * 5.0) / self.scale);
                                }
                            }
                            ToolMode::EraserPoint => {
                                if world.x >= 0.0 && world.x <= page_w && world.y >= 0.0 {
                                    self.erase_point_at(world, (self.size * 3.5) / self.scale);
                                }
                            }
                            ToolMode::LassoSelect => {
                                if self.is_dragging_selection {
                                    let delta = world - self.drag_last_pos_world;
                                    self.drag_last_pos_world = world;
                                    let page = self.notebook.current_page_mut();
                                    for stroke in &mut page.strokes {
                                        if self.selected_stroke_ids.contains(&stroke.id) {
                                            for pt in &mut stroke.points {
                                                pt.x += delta.x;
                                                pt.y += delta.y;
                                            }
                                        }
                                    }
                                } else {
                                    if let Some(last) = self.lasso_points.last() {
                                        let d_sq = (world.x - last.x).powi(2) + (world.y - last.y).powi(2);
                                        if d_sq > 2.0 {
                                            self.lasso_points.push(world);
                                        }
                                    } else {
                                        self.lasso_points.push(world);
                                    }
                                }
                            }
                            ToolMode::Shape(_) => {
                                if let Some(ref mut stroke) = self.current_stroke {
                                    let bounded_x = world.x.clamp(0.0, page_w);
                                    let bounded_y = world.y.max(0.0);
                                    if stroke.points.len() < 2 {
                                        stroke.points.push(Point::new(bounded_x, bounded_y, 0.5));
                                    } else {
                                        stroke.points[1] = Point::new(bounded_x, bounded_y, 0.5);
                                    }
                                }
                            }
                            ToolMode::Pen | ToolMode::Highlighter => {
                                if let Some(ref mut stroke) = self.current_stroke {
                                    // Restrict inking points to paper sheet width [0.0, page_w] and y >= 0.0
                                    let bounded_x = world.x.clamp(0.0, page_w);
                                    let bounded_y = world.y.max(0.0);

                                    let raw_pressure = if let Some(f) = hardware_force {
                                        self.current_pressure = f.clamp(0.05, 1.0);
                                        self.current_pressure
                                    } else {
                                        // Speed-based dynamic tapering fallback
                                        let dt = self.last_pointer_time.elapsed().as_secs_f32().max(0.001);
                                        let dist = (bounded_x - self.last_pointer_pos_world.x).hypot(bounded_y - self.last_pointer_pos_world.y);
                                        let speed = (dist * self.scale) / dt;
                                        let target = (0.90 - (speed / 1600.0) * 0.50).clamp(0.20, 1.0);
                                        self.current_pressure = self.current_pressure * 0.65 + target * 0.35;
                                        self.current_pressure
                                    };
                                    self.last_pointer_time = std::time::Instant::now();
                                    self.last_pointer_pos_world = Pos2::new(bounded_x, bounded_y);

                                    let timestamp_sec = self.app_start_time.elapsed().as_secs_f64();
                                    // === ONE-EURO ADAPTIVE FILTERING & CORNER-DETECTION PIPELINE ===
                                    if let Some(smoothed_pt) = self.stabilizer.process_point(
                                        bounded_x,
                                        bounded_y,
                                        raw_pressure,
                                        self.size,
                                        stroke.pen_type,
                                        timestamp_sec,
                                    ) {
                                        stroke.points.push(smoothed_pt);
                                    }

                                    // Dynamically auto-expand page downwards when writing near bottom
                                    if bounded_y > self.notebook.current_page().height - 200.0 {
                                        if self.notebook.current_page_mut().check_auto_expand(bounded_y) {
                                            self.set_status("Page auto-expanded! ♾️");
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Pointer released or up
                    if self.is_dragging_selection {
                        self.is_dragging_selection = false;
                        let page = self.notebook.current_page_mut();
                        let mut max_y = 0.0f32;
                        for stroke in &page.strokes {
                            if self.selected_stroke_ids.contains(&stroke.id) {
                                for pt in &stroke.points {
                                    if pt.y > max_y {
                                        max_y = pt.y;
                                    }
                                }
                            }
                        }
                        if page.check_auto_expand(max_y) {
                            self.set_status("Page auto-expanded! ♾️");
                        }
                    } else if !self.lasso_points.is_empty() {
                        if self.lasso_points.len() >= 3 {
                            let mut selected = HashSet::new();
                            for stroke in &self.notebook.current_page().strokes {
                                if stroke.points.iter().any(|pt| point_in_polygon(Pos2::new(pt.x, pt.y), &self.lasso_points)) {
                                    selected.insert(stroke.id.clone());
                                }
                            }
                            let count = selected.len();
                            self.selected_stroke_ids = selected;
                            if count > 0 {
                                self.set_status(&format!("Selected {} stroke(s)", count));
                            }
                        } else {
                            // Single click: hit test nearby stroke
                            let click_pos = self.lasso_points[0];
                            let hit_dist = 14.0 / self.scale;
                            let mut hit_id = None;
                            for stroke in self.notebook.current_page().strokes.iter().rev() {
                                if stroke.points.iter().any(|pt| {
                                    let dx = pt.x - click_pos.x;
                                    let dy = pt.y - click_pos.y;
                                    dx * dx + dy * dy <= hit_dist * hit_dist
                                }) {
                                    hit_id = Some(stroke.id.clone());
                                    break;
                                }
                            }
                            if let Some(id) = hit_id {
                                self.selected_stroke_ids.insert(id);
                                self.set_status("Selected 1 stroke");
                            } else {
                                self.selected_stroke_ids.clear();
                            }
                        }
                        self.lasso_points.clear();
                    }

                    if let Some(mut stroke) = self.current_stroke.take() {
                        if !stroke.points.is_empty() {
                            // Inertia end point flush (from C++ StabilizerPipeline::FlushEndPoint)
                            if stroke.shape_type == ShapeType::None {
                                if let Some(end_pt) = self.stabilizer.flush_end_point(self.size, stroke.pen_type) {
                                    stroke.points.push(end_pt);
                                }
                            }
                            let mut max_y = 0.0f32;
                            for p in &stroke.points {
                                if p.y > max_y {
                                    max_y = p.y;
                                }
                            }

                            let expanded = self.notebook.current_page_mut().check_auto_expand(max_y);
                            if expanded {
                                self.set_status("Page auto-expanded! ♾️");
                            }
                            self.notebook.current_page_mut().strokes.push(stroke);
                        }
                    }
                }
            });
    }
}
