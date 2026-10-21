use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::collections::{HashMap, HashSet};

use crate::model::{
    ImageEntity, Notebook, PaperType, PenType, Point, ShapeType, Stroke as NoteStroke, TextBox, ToolMode,
};
use crate::renderer::NoteRenderer;
use crate::ruler::{RulerKind, RulerState};
use crate::math_renderer::MathRenderer;
use crate::storage::NotebookStorage;
use crate::stroke_algorithm::StabilizerPipeline;
use crate::theme::{self, Palette};
use crate::ui_math_studio::MathStudioState;
use crate::widgets;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
}

fn get_handle_positions(r: Rect) -> [(SelectionHandle, Pos2); 8] {
    [
        (SelectionHandle::TopLeft, r.left_top()),
        (SelectionHandle::TopRight, r.right_top()),
        (SelectionHandle::BottomLeft, r.left_bottom()),
        (SelectionHandle::BottomRight, r.right_bottom()),
        (SelectionHandle::Top, r.center_top()),
        (SelectionHandle::Bottom, r.center_bottom()),
        (SelectionHandle::Left, r.left_center()),
        (SelectionHandle::Right, r.right_center()),
    ]
}

fn hit_handle(cursor: Pos2, r: Rect, radius: f32) -> Option<SelectionHandle> {
    for (handle, pos) in get_handle_positions(r) {
        if cursor.distance(pos) <= radius {
            return Some(handle);
        }
    }
    None
}

const SHAPES: [(ShapeType, &str, &str); 9] = [
    (ShapeType::Line, "↗", "Line"),
    (ShapeType::Arrow, "➡", "Arrow"),
    (ShapeType::DoubleArrow, "↔", "Double arrow"),
    (ShapeType::Rectangle, "☐", "Rectangle"),
    (ShapeType::Circle, "○", "Circle"),
    (ShapeType::Triangle, "⏶", "Triangle"),
    (ShapeType::Pentagon, "⬟", "Pentagon"),
    (ShapeType::Hexagon, "⬣", "Hexagon"),
    (ShapeType::Star, "☆", "Star"),
];

const PAPERS: [PaperType; 5] = [
    PaperType::Ruled,
    PaperType::Grid,
    PaperType::DotGrid,
    PaperType::Cornell,
    PaperType::Blank,
];

/// Check if eraser at world_pos (radius) hits this stroke's visual geometry.
fn stroke_hit_test(stroke: &NoteStroke, world_pos: Pos2, radius: f32) -> bool {
    let r_sq = radius * radius;

    // For shapes: test against the geometric outline, not just 2 corner points
    if stroke.shape_type != ShapeType::None && stroke.points.len() >= 2 {
        let p1 = Pos2::new(stroke.points[0].x, stroke.points[0].y);
        let p2 = Pos2::new(stroke.points.last().unwrap().x, stroke.points.last().unwrap().y);
        let rect = Rect::from_two_pos(p1, p2);

        match stroke.shape_type {
            ShapeType::None => {}
            ShapeType::Line | ShapeType::Arrow | ShapeType::DoubleArrow => {
                // Distance from point to line segment
                return point_to_segment_dist_sq(world_pos, p1, p2) <= r_sq;
            }
            ShapeType::Rectangle => {
                // Sample rectangle edges
                let corners = [
                    rect.left_top(),
                    rect.right_top(),
                    rect.right_bottom(),
                    rect.left_bottom(),
                ];
                for i in 0..4 {
                    let a = corners[i];
                    let b = corners[(i + 1) % 4];
                    if point_to_segment_dist_sq(world_pos, a, b) <= r_sq {
                        return true;
                    }
                }
                // Also check if filled and cursor inside
                if stroke.is_filled && rect.contains(world_pos) {
                    return true;
                }
            }
            ShapeType::Circle => {
                let center = rect.center();
                let rx = rect.width() / 2.0;
                let ry = rect.height() / 2.0;
                let dx = world_pos.x - center.x;
                let dy = world_pos.y - center.y;
                // Ellipse implicit: (dx/rx)^2 + (dy/ry)^2 = 1
                let normalized = (dx / rx).powi(2) + (dy / ry).powi(2);
                if stroke.is_filled {
                    // Inside or on boundary
                    return normalized <= 1.0 + radius / rx.min(ry);
                } else {
                    // Near circumference
                    return (normalized.sqrt() - 1.0).abs() * rx.min(ry) <= radius;
                }
            }
            _ => {
                // Triangle, Pentagon, Hexagon, Star: sample vertices
                // Fallback: check bounding box + any corner point
                if rect.expand(radius).contains(world_pos) {
                    return true;
                }
            }
        }
        return false;
    }

    // For ink strokes: check any point within radius
    stroke.points.iter().any(|pt| {
        let dx = pt.x - world_pos.x;
        let dy = pt.y - world_pos.y;
        dx * dx + dy * dy <= r_sq
    })
}

/// Squared distance from point p to line segment (a, b).
fn point_to_segment_dist_sq(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_sq();
    if ab_len_sq < 1e-6 {
        return ap.length_sq();
    }
    let t = (ap.x * ab.x + ap.y * ab.y) / ab_len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length_sq()
}

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

pub struct InkkApp {
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
    selected_image_ids: HashSet<String>,
    selected_plot_ids: HashSet<String>,
    selected_matrix_ids: HashSet<String>,
    selected_grid_ids: HashSet<String>,
    lasso_points: Vec<Pos2>,
    is_dragging_selection: bool,
    drag_last_pos_world: Pos2,
    active_resize_handle: Option<SelectionHandle>,
    initial_selection_rect: Option<Rect>,
    initial_stroke_points: Vec<(String, Vec<Point>)>,
    initial_image_rects: Vec<(String, f32, f32, f32, f32)>,
    initial_plot_rects: Vec<(String, f32, f32, f32, f32)>,
    initial_matrix_rects: Vec<(String, f32, f32, f32, f32)>,
    initial_grid_rects: Vec<(String, f32, f32, f32, f32)>,

    // Image textures cache
    image_textures: HashMap<String, egui::TextureHandle>,

    // MathStudio MATLAB-style IDE & Engine
    pub math_studio: MathStudioState,

    // UI & Theme state
    is_dark_mode: bool,
    show_previews_drawer: bool,
    current_file_path: Option<std::path::PathBuf>,
    /// (message, shown at, is_error)
    status_message: Option<(String, std::time::Instant, bool)>,
    /// True while a primary-button gesture that started on the canvas is in progress.
    /// Prevents drags that begin on toolbars/panels from inking, erasing or panning.
    canvas_gesture: bool,
    /// Stencil tools: Straight Ruler & Circular Protractor
    pub ruler: RulerState,

    // TextLayer state
    editing_text_id: Option<String>,
    text_font_size: f32,
    text_is_bold: bool,
    text_is_italic: bool,
    // Drag-move state
    dragging_text_id: Option<String>,
    drag_text_start_pointer: Pos2,
    drag_text_start_pos: Pos2,
    // Resize state
    resizing_text_id: Option<String>,
    resize_start_pointer: Pos2,
    resize_start_w: f32,
}

impl Default for InkkApp {
    fn default() -> Self {
        Self {
            notebook: Notebook::default(),
            current_stroke: None,
            redo_stack: Vec::new(),

            tool: ToolMode::Pen,
            pen_type: PenType::Ballpoint,
            current_shape: ShapeType::Rectangle,
            is_shape_filled: false,
            color: theme::DEFAULT_INK_LIGHT,
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
            selected_image_ids: HashSet::new(),
            selected_plot_ids: HashSet::new(),
            selected_matrix_ids: HashSet::new(),
            selected_grid_ids: HashSet::new(),
            lasso_points: Vec::new(),
            is_dragging_selection: false,
            drag_last_pos_world: Pos2::ZERO,
            active_resize_handle: None,
            initial_selection_rect: None,
            initial_stroke_points: Vec::new(),
            initial_image_rects: Vec::new(),
            initial_plot_rects: Vec::new(),
            initial_matrix_rects: Vec::new(),
            initial_grid_rects: Vec::new(),
            image_textures: HashMap::new(),
            math_studio: MathStudioState::default(),

            is_dark_mode: false,
            show_previews_drawer: false,
            current_file_path: None,
            status_message: None,
            canvas_gesture: false,
            ruler: RulerState::default(),

            editing_text_id: None,
            text_font_size: 18.0,
            text_is_bold: false,
            text_is_italic: false,
            dragging_text_id: None,
            drag_text_start_pointer: Pos2::ZERO,
            drag_text_start_pos: Pos2::ZERO,
            resizing_text_id: None,
            resize_start_pointer: Pos2::ZERO,
            resize_start_w: 0.0,
        }
    }
}

impl InkkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();
        theme::apply(&cc.egui_ctx, app.is_dark_mode);
        app
    }

    fn palette(&self) -> Palette {
        Palette::get(self.is_dark_mode)
    }

    fn clear_selection(&mut self) {
        self.selected_stroke_ids.clear();
        self.selected_image_ids.clear();
        self.selected_plot_ids.clear();
        self.selected_matrix_ids.clear();
        self.selected_grid_ids.clear();
        self.active_resize_handle = None;
        self.initial_selection_rect = None;
        self.initial_stroke_points.clear();
        self.initial_image_rects.clear();
        self.initial_plot_rects.clear();
        self.initial_matrix_rects.clear();
        self.initial_grid_rects.clear();
    }

    fn set_tool(&mut self, mode: ToolMode) {
        self.tool = mode;
        if mode != ToolMode::LassoSelect {
            self.clear_selection();
        }
        if mode != ToolMode::Text {
            self.editing_text_id = None;
            self.dragging_text_id = None;
        }
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.is_dark_mode = !self.is_dark_mode;
        theme::apply(ctx, self.is_dark_mode);

        // Intelligently transform all strokes across all notebook pages
        self.adapt_notebook_theme(self.is_dark_mode);

        // Adapt the active pen color so it matches the new theme
        if self.is_dark_mode {
            self.color = theme::adapt_color32_to_dark(self.color);
        } else {
            self.color = theme::adapt_color32_to_light(self.color);
        }
    }

    fn adapt_notebook_theme(&mut self, is_dark: bool) {
        for page in &mut self.notebook.pages {
            for stroke in &mut page.strokes {
                if is_dark {
                    let orig = stroke.original_color.unwrap_or(stroke.color);
                    stroke.original_color = Some(orig);
                    stroke.color = theme::adapt_color_to_dark(orig);
                } else if let Some(orig) = stroke.original_color.take() {
                    stroke.color = orig;
                } else {
                    stroke.color = theme::adapt_color_to_light(stroke.color);
                }
            }

            for tb in &mut page.text_boxes {
                if is_dark {
                    let orig = tb.original_color.unwrap_or(tb.color);
                    tb.original_color = Some(orig);
                    tb.color = theme::adapt_color_to_dark(orig);
                } else if let Some(orig) = tb.original_color.take() {
                    tb.color = orig;
                } else {
                    tb.color = theme::adapt_color_to_light(tb.color);
                }
            }
        }

        for stroke in &mut self.redo_stack {
            if is_dark {
                let orig = stroke.original_color.unwrap_or(stroke.color);
                stroke.original_color = Some(orig);
                stroke.color = theme::adapt_color_to_dark(orig);
            } else if let Some(orig) = stroke.original_color.take() {
                stroke.color = orig;
            } else {
                stroke.color = theme::adapt_color_to_light(stroke.color);
            }
        }

        if let Some(ref mut stroke) = self.current_stroke {
            if is_dark {
                let orig = stroke.original_color.unwrap_or(stroke.color);
                stroke.original_color = Some(orig);
                stroke.color = theme::adapt_color_to_dark(orig);
            } else if let Some(orig) = stroke.original_color.take() {
                stroke.color = orig;
            } else {
                stroke.color = theme::adapt_color_to_light(stroke.color);
            }
        }
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
        self.status_message = Some((msg.to_string(), std::time::Instant::now(), false));
    }

    fn set_error(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), std::time::Instant::now(), true));
    }

    fn hit_test_clickable_card(&self, world: Pos2) -> Option<(&'static str, String)> {
        let page = self.notebook.current_page();
        if let Some(p) = page.plots.iter().rev().find(|p| Rect::from_min_size(Pos2::new(p.x, p.y), Vec2::new(p.width, p.height)).contains(world)) {
            return Some(("plot", p.id.clone()));
        }
        if let Some(m) = page.matrices.iter().rev().find(|m| Rect::from_min_size(Pos2::new(m.x, m.y), Vec2::new(m.width, m.height)).contains(world)) {
            return Some(("matrix", m.id.clone()));
        }
        if let Some(g) = page.grids.iter().rev().find(|g| Rect::from_min_size(Pos2::new(g.x, g.y), Vec2::new(g.total_width(), g.total_height())).contains(world)) {
            return Some(("grid", g.id.clone()));
        }
        if let Some(i) = page.images.iter().rev().find(|i| Rect::from_min_size(Pos2::new(i.x, i.y), Vec2::new(i.width, i.height)).contains(world)) {
            return Some(("image", i.id.clone()));
        }
        None
    }

    fn get_selection_bounds(&self) -> Option<Rect> {
        if self.selected_stroke_ids.is_empty()
            && self.selected_image_ids.is_empty()
            && self.selected_plot_ids.is_empty()
            && self.selected_matrix_ids.is_empty()
            && self.selected_grid_ids.is_empty()
        {
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

        for img in &page.images {
            if self.selected_image_ids.contains(&img.id) {
                found = true;
                if img.x < min_x { min_x = img.x; }
                if img.y < min_y { min_y = img.y; }
                let right = img.x + img.width;
                let bottom = img.y + img.height;
                if right > max_x { max_x = right; }
                if bottom > max_y { max_y = bottom; }
            }
        }

        for plot in &page.plots {
            if self.selected_plot_ids.contains(&plot.id) {
                found = true;
                if plot.x < min_x { min_x = plot.x; }
                if plot.y < min_y { min_y = plot.y; }
                let right = plot.x + plot.width;
                let bottom = plot.y + plot.height;
                if right > max_x { max_x = right; }
                if bottom > max_y { max_y = bottom; }
            }
        }

        for matrix in &page.matrices {
            if self.selected_matrix_ids.contains(&matrix.id) {
                found = true;
                if matrix.x < min_x { min_x = matrix.x; }
                if matrix.y < min_y { min_y = matrix.y; }
                let right = matrix.x + matrix.width;
                let bottom = matrix.y + matrix.height;
                if right > max_x { max_x = right; }
                if bottom > max_y { max_y = bottom; }
            }
        }

        for grid in &page.grids {
            if self.selected_grid_ids.contains(&grid.id) {
                found = true;
                if grid.x < min_x { min_x = grid.x; }
                if grid.y < min_y { min_y = grid.y; }
                let right = grid.x + grid.total_width();
                let bottom = grid.y + grid.total_height();
                if right > max_x { max_x = right; }
                if bottom > max_y { max_y = bottom; }
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

    fn delete_selected(&mut self) {
        if self.selected_stroke_ids.is_empty()
            && self.selected_image_ids.is_empty()
            && self.selected_plot_ids.is_empty()
            && self.selected_matrix_ids.is_empty()
            && self.selected_grid_ids.is_empty()
        {
            return;
        }
        let stroke_count = self.selected_stroke_ids.len();
        let image_count = self.selected_image_ids.len();
        let math_count = self.selected_plot_ids.len() + self.selected_matrix_ids.len() + self.selected_grid_ids.len();
        let page = self.notebook.current_page_mut();
        page.strokes.retain(|s| !self.selected_stroke_ids.contains(&s.id));
        page.images.retain(|i| !self.selected_image_ids.contains(&i.id));
        page.plots.retain(|p| !self.selected_plot_ids.contains(&p.id));
        page.matrices.retain(|m| !self.selected_matrix_ids.contains(&m.id));
        page.grids.retain(|g| !self.selected_grid_ids.contains(&g.id));
        self.clear_selection();
        self.set_status(&format!("Deleted {} stroke(s), {} image(s), {} math item(s)", stroke_count, image_count, math_count));
    }

    fn prepare_selection_drag(&mut self) {
        if let Some(bounds) = self.get_selection_bounds() {
            self.initial_selection_rect = Some(bounds);
            let page = self.notebook.current_page();
            self.initial_stroke_points = page
                .strokes
                .iter()
                .filter(|s| self.selected_stroke_ids.contains(&s.id))
                .map(|s| (s.id.clone(), s.points.clone()))
                .collect();
            self.initial_image_rects = page
                .images
                .iter()
                .filter(|i| self.selected_image_ids.contains(&i.id))
                .map(|i| (i.id.clone(), i.x, i.y, i.width, i.height))
                .collect();
            self.initial_plot_rects = page
                .plots
                .iter()
                .filter(|p| self.selected_plot_ids.contains(&p.id))
                .map(|p| (p.id.clone(), p.x, p.y, p.width, p.height))
                .collect();
            self.initial_matrix_rects = page
                .matrices
                .iter()
                .filter(|m| self.selected_matrix_ids.contains(&m.id))
                .map(|m| (m.id.clone(), m.x, m.y, m.width, m.height))
                .collect();
            self.initial_grid_rects = page
                .grids
                .iter()
                .filter(|g| self.selected_grid_ids.contains(&g.id))
                .map(|g| (g.id.clone(), g.x, g.y, g.total_width(), g.total_height()))
                .collect();
        }
    }

    fn prompt_insert_image(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                    let w = dyn_img.width() as f32;
                    let h = dyn_img.height() as f32;
                    self.insert_image_bytes(bytes, w, h);
                } else {
                    self.set_error("Could not decode selected image file.");
                }
            }
        }
    }

    fn insert_image_bytes(&mut self, data: Vec<u8>, width: f32, height: f32) {
        let max_dim = 450.0f32;
        let (w, h) = if width > max_dim || height > max_dim {
            let aspect = width / height.max(1.0);
            if aspect > 1.0 {
                (max_dim, (max_dim / aspect).max(20.0))
            } else {
                ((max_dim * aspect).max(20.0), max_dim)
            }
        } else {
            (width.max(20.0), height.max(20.0))
        };

        let page = self.notebook.current_page_mut();
        let place_x = ((page.width - w) * 0.5).max(40.0);
        let place_y = 120.0f32;

        let new_img = ImageEntity::new(place_x, place_y, w, h, data);
        let new_id = new_img.id.clone();
        page.images.push(new_img);

        self.clear_selection();
        self.selected_image_ids.insert(new_id);
        self.set_tool(ToolMode::LassoSelect);
        self.set_status("Image inserted & selected! 🖼 (Drag handles to resize)");
    }

    fn paste_from_clipboard(&mut self) {
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(img_data) = clip.get_image() {
                let width = img_data.width as u32;
                let height = img_data.height as u32;
                if width > 0 && height > 0 {
                    let mut png_bytes = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
                    use image::ImageEncoder;
                    if encoder
                        .write_image(&img_data.bytes, width, height, image::ExtendedColorType::Rgba8)
                        .is_ok()
                    {
                        self.insert_image_bytes(png_bytes, width as f32, height as f32);
                        return;
                    }
                }
            }
        }
    }

    /// Main toolbar: brand, history, tools, colors, size, theme.
    fn render_top_toolbar(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let p = self.palette();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.label(
                egui::RichText::new(format!("{} Inkk", theme::icon::NOTEBOOK))
                    .strong()
                    .size(15.0)
                    .color(p.text),
            );
            widgets::divider(ui, &p);

            let can_undo = !self.notebook.current_page().strokes.is_empty();
            let can_redo = !self.redo_stack.is_empty();
            if ui
                .add_enabled_ui(can_undo, |ui| widgets::tool_button(ui, &p, theme::icon::UNDO, false, "Undo (Ctrl+Z)"))
                .inner
                .clicked()
            {
                self.undo();
            }
            if ui
                .add_enabled_ui(can_redo, |ui| widgets::tool_button(ui, &p, theme::icon::REDO, false, "Redo (Ctrl+Y)"))
                .inner
                .clicked()
            {
                self.redo();
            }
            widgets::divider(ui, &p);

            let shape_icon = SHAPES
                .iter()
                .find(|(s, _, _)| *s == self.current_shape)
                .map_or("☐", |(_, i, _)| *i);
            let tools = [
                (ToolMode::Pen, theme::icon::PEN, "Pen (1)"),
                (ToolMode::Highlighter, theme::icon::HIGHLIGHTER, "Highlighter (2)"),
                (ToolMode::Shape(self.current_shape), shape_icon, "Shapes (3)"),
                (ToolMode::Text, "T", "Text Box (4 / T)"),
                (ToolMode::EraserStroke, theme::icon::ERASER_STROKE, "Stroke eraser (5)"),
                (ToolMode::EraserPoint, theme::icon::ERASER_POINT, "Point eraser (6)"),
                (ToolMode::LassoSelect, theme::icon::LASSO, "Lasso select (7)"),
                (ToolMode::Pan, theme::icon::PAN, "Pan (8)"),
            ];
            for (mode, icon, tip) in tools {
                let active = match (self.tool, mode) {
                    (ToolMode::Shape(_), ToolMode::Shape(_)) => true,
                    (a, b) => a == b,
                };
                if widgets::tool_button(ui, &p, icon, active, tip).clicked() {
                    self.set_tool(mode);
                }
            }
            widgets::divider(ui, &p);

            // Stencil Tools: Straight Ruler & Circular Protractor
            let ruler_active = self.ruler.kind == RulerKind::Ruler;
            if widgets::tool_button(ui, &p, theme::icon::RULER, ruler_active, "Straight Ruler (R)").clicked() {
                self.ruler.toggle_ruler();
                let is_on = self.ruler.kind == RulerKind::Ruler;
                self.set_status(if is_on { "Straight Ruler ON 📏 (drag body to move, dial to rotate)" } else { "Ruler OFF" });
            }
            let protractor_active = self.ruler.kind == RulerKind::Protractor;
            if widgets::tool_button(ui, &p, theme::icon::PROTRACTOR, protractor_active, "Circular Protractor (Shift+R)").clicked() {
                self.ruler.toggle_protractor();
                let is_on = self.ruler.kind == RulerKind::Protractor;
                self.set_status(if is_on { "Protractor ON 📐 (drag body to move, dial to rotate)" } else { "Protractor OFF" });
            }
            widgets::divider(ui, &p);

            // Picture Tool: Insert Image from File
            if widgets::tool_button(ui, &p, theme::icon::PICTURE, false, "Insert Image from file (or Ctrl+V to paste)").clicked() {
                self.prompt_insert_image();
            }
            widgets::divider(ui, &p);

            // Math Studio: MATLAB IDE & Plotting Engine
            let math_active = self.math_studio.is_open;
            if widgets::tool_button(ui, &p, "🔬", math_active, "Math Studio & MATLAB IDE (M / Ctrl+M)").clicked() {
                self.math_studio.toggle();
                let is_on = self.math_studio.is_open;
                self.set_status(if is_on { "Math Studio OPEN 🔬 [MATLAB Engine Ready]" } else { "Math Studio CLOSED" });
            }
            widgets::divider(ui, &p);

            // Picking a color also switches back to an inking tool or updates text color
            for (c, name) in theme::ink_swatches(self.is_dark_mode) {
                if widgets::color_swatch(ui, &p, c, self.color == c, name).clicked() {
                    self.color = c;
                    if let Some(ref sel_id) = self.editing_text_id {
                        let c_arr = [c.r(), c.g(), c.b(), c.a()];
                        for tb in &mut self.notebook.current_page_mut().text_boxes {
                            if tb.id == *sel_id {
                                tb.color = c_arr;
                            }
                        }
                    } else if matches!(self.tool, ToolMode::EraserStroke | ToolMode::EraserPoint | ToolMode::Pan) {
                        self.set_tool(ToolMode::Pen);
                    }
                }
            }
            ui.color_edit_button_srgba(&mut self.color).on_hover_text("Custom color");
            widgets::divider(ui, &p);

            // Live size preview dot next to the slider
            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), (self.size * 0.5).clamp(1.5, 12.0), self.color);
            ui.painter().circle_stroke(dot_rect.center(), 12.5, Stroke::new(1.0_f32, p.border));
            ui.add(egui::Slider::new(&mut self.size, 1.0..=30.0).suffix(" px").show_value(true))
                .on_hover_text("Stroke size ( [ / ] )");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (icon, tip) = if self.is_dark_mode {
                    (theme::icon::SUN, "Switch to light theme")
                } else {
                    (theme::icon::MOON, "Switch to dark theme")
                };
                if widgets::tool_button(ui, &p, icon, false, tip).clicked() {
                    self.toggle_theme(ctx);
                }
            });
        });
    }
    /// Second toolbar row with options for the active tool and the page.
    fn render_context_bar(&mut self, ui: &mut Ui) {
        let p = self.palette();
        ui.horizontal_wrapped(|ui| {
            match self.tool {
                ToolMode::Pen => {
                    widgets::caption(ui, &p, "Pen");
                    for (pen, label) in [
                        (PenType::Ballpoint, "Ballpoint"),
                        (PenType::Fountain, "Fountain"),
                        (PenType::Calligraphy, "Calligraphy"),
                        (PenType::Pencil, "Pencil"),
                    ] {
                        if widgets::chip(ui, &p, label, self.pen_type == pen).clicked() {
                            self.pen_type = pen;
                        }
                    }
                }
                ToolMode::Shape(_) => {
                    widgets::caption(ui, &p, "Shape");
                    for (shape, icon, name) in SHAPES {
                        if widgets::tool_button(ui, &p, icon, self.current_shape == shape, name).clicked() {
                            self.current_shape = shape;
                            self.tool = ToolMode::Shape(shape);
                        }
                    }
                    widgets::divider(ui, &p);
                    if widgets::chip(ui, &p, "Filled", self.is_shape_filled).clicked() {
                        self.is_shape_filled = !self.is_shape_filled;
                    }
                }
                ToolMode::Highlighter => {
                    widgets::caption(ui, &p, "Highlighter draws beneath ink so text stays readable");
                }
                ToolMode::EraserStroke => {
                    widgets::caption(ui, &p, "Removes whole strokes it touches · size slider sets radius");
                }
                ToolMode::EraserPoint => {
                    widgets::caption(ui, &p, "Cuts strokes where it passes · size slider sets radius");
                }
                ToolMode::LassoSelect => {
                    widgets::caption(ui, &p, "Draw around strokes or click one · drag to move · Del to delete");
                }
                ToolMode::Pan => {
                    widgets::caption(ui, &p, "Drag to move the page · Ctrl + wheel to zoom");
                }
                ToolMode::Text => {
                    widgets::caption(ui, &p, "Text");
                    for fs in [14.0, 18.0, 24.0, 32.0] {
                        let label = format!("{:.0}pt", fs);
                        let is_active = (self.text_font_size - fs).abs() < 1.0;
                        if widgets::chip(ui, &p, &label, is_active).clicked() {
                            self.text_font_size = fs;
                            if let Some(ref id) = self.editing_text_id {
                                for tb in &mut self.notebook.current_page_mut().text_boxes {
                                    if tb.id == *id {
                                        tb.font_size = fs;
                                    }
                                }
                            }
                        }
                    }
                    widgets::divider(ui, &p);
                    if widgets::chip(ui, &p, "B", self.text_is_bold).clicked() {
                        self.text_is_bold = !self.text_is_bold;
                        if let Some(ref id) = self.editing_text_id {
                            for tb in &mut self.notebook.current_page_mut().text_boxes {
                                if tb.id == *id {
                                    tb.is_bold = self.text_is_bold;
                                }
                            }
                        }
                    }
                    if widgets::chip(ui, &p, "I", self.text_is_italic).clicked() {
                        self.text_is_italic = !self.text_is_italic;
                        if let Some(ref id) = self.editing_text_id {
                            for tb in &mut self.notebook.current_page_mut().text_boxes {
                                if tb.id == *id {
                                    tb.is_italic = self.text_is_italic;
                                }
                            }
                        }
                    }
                    widgets::divider(ui, &p);
                    if self.editing_text_id.is_some() {
                        if widgets::tool_button(ui, &p, "🗑", false, "Delete Text Box").clicked() {
                            if let Some(ref id) = self.editing_text_id.take() {
                                self.notebook.current_page_mut().text_boxes.retain(|tb| tb.id != *id);
                                self.set_status("Deleted text box");
                            }
                        }
                        if widgets::tool_button(ui, &p, "✓", false, "Done Editing").clicked() {
                            self.editing_text_id = None;
                        }
                    } else {
                        widgets::caption(ui, &p, "Click canvas to type · Drag top strip to move · right edge to resize");
                    }
                }
            }

            if matches!(self.tool, ToolMode::Pen | ToolMode::Highlighter | ToolMode::Shape(_)) {
                widgets::divider(ui, &p);
                widgets::caption(ui, &p, "Opacity");
                ui.add(
                    egui::Slider::new(&mut self.opacity, 0.1..=1.0)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let cur_paper = self.notebook.current_page().paper_type;
                let mut selected_paper = cur_paper;
                egui::ComboBox::from_id_salt("paper_combo")
                    .selected_text(cur_paper.name())
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        for paper in PAPERS {
                            ui.selectable_value(&mut selected_paper, paper, paper.name());
                        }
                    });
                if selected_paper != cur_paper {
                    self.notebook.current_page_mut().paper_type = selected_paper;
                }
                widgets::caption(ui, &p, "Paper");
            });
        });
    }

    /// Status bar: title, page navigation, zoom, file actions.
    fn render_bottom_bar(&mut self, ui: &mut Ui, available_size: Vec2) {
        let p = self.palette();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add(
                egui::TextEdit::singleline(&mut self.notebook.title)
                    .desired_width(180.0)
                    .hint_text("Notebook title"),
            )
            .on_hover_text("Notebook title");
            widgets::divider(ui, &p);

            let cur_idx = self.notebook.current_page_index;
            let total = self.notebook.pages.len();
            if ui.add_enabled_ui(cur_idx > 0, |ui| widgets::tool_button(ui, &p, theme::icon::PREV, false, "Previous page (PgUp)")).inner.clicked() {
                self.go_to_page(cur_idx.saturating_sub(1));
            }
            let pages_label = format!("{} Page {} / {}", theme::icon::PAGES, cur_idx + 1, total);
            if widgets::chip(ui, &p, &pages_label, self.show_previews_drawer)
                .on_hover_text("Show page previews")
                .clicked()
            {
                self.show_previews_drawer = !self.show_previews_drawer;
            }
            if ui.add_enabled_ui(cur_idx + 1 < total, |ui| widgets::tool_button(ui, &p, theme::icon::NEXT, false, "Next page (PgDn)")).inner.clicked() {
                self.go_to_page(cur_idx + 1);
            }
            if widgets::tool_button(ui, &p, theme::icon::ADD, false, "Add page after this one").clicked() {
                self.notebook.add_page();
                self.selected_stroke_ids.clear();
            }
            if widgets::tool_button(ui, &p, theme::icon::EXTEND, false, "Extend page by 600 px").clicked() {
                self.notebook.current_page_mut().extend_height(600.0);
                self.set_status("Page extended +600 px");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::primary_button(ui, &p, &format!("{} Save", theme::icon::SAVE))
                    .on_hover_text("Save as .inkk (Ctrl+S)")
                    .clicked()
                {
                    self.prompt_save_file();
                }
                if ui.button(format!("{} Open", theme::icon::OPEN)).on_hover_text("Open .inkk (Ctrl+O)").clicked() {
                    self.prompt_open_file();
                }
                widgets::divider(ui, &p);

                // Right-to-left: these appear as  [Fit] [−] 100% [+]
                if widgets::tool_button(ui, &p, "+", false, "Zoom in (Ctrl + wheel)").clicked() {
                    self.zoom_by(1.15, available_size);
                }
                let zoom_label = format!("{}%", (self.scale * 100.0).round() as i32);
                if ui
                    .add(egui::Button::new(egui::RichText::new(zoom_label).monospace()).frame(false).min_size(Vec2::new(48.0, 28.0)))
                    .on_hover_text("Reset to 100% (Ctrl+0)")
                    .clicked()
                {
                    self.zoom_by(1.0 / self.scale, available_size);
                }
                if widgets::tool_button(ui, &p, "−", false, "Zoom out (Ctrl + wheel)").clicked() {
                    self.zoom_by(0.85, available_size);
                }
                if widgets::tool_button(ui, &p, theme::icon::FIT, false, "Fit page to window").clicked() {
                    self.fit_to_window(available_size);
                }
            });
        });
    }

    fn go_to_page(&mut self, idx: usize) {
        if idx < self.notebook.pages.len() {
            self.notebook.current_page_index = idx;
            self.clear_selection();
        }
    }

    /// Zooms around the center of the canvas so the visible content stays in place.
    fn zoom_by(&mut self, factor: f32, available_size: Vec2) {
        let new_scale = (self.scale * factor).clamp(0.2, 5.0);
        let center = available_size * 0.5;
        self.pan = center - (center - self.pan) * (new_scale / self.scale);
        self.scale = new_scale;
    }
    /// Right drawer with clickable page thumbnails.
    fn render_previews_drawer(&mut self, ctx: &egui::Context) {
        if !self.show_previews_drawer {
            return;
        }
        let p = self.palette();
        let is_dark = self.is_dark_mode;

        egui::SidePanel::right("previews_drawer")
            .resizable(true)
            .default_width(240.0)
            .width_range(180.0..=420.0)
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(p.surface).inner_margin(12.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Pages").strong().size(15.0));
                    widgets::caption(ui, &p, format!("{}", self.notebook.pages.len()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::tool_button(ui, &p, theme::icon::CLOSE, false, "Close").clicked() {
                            self.show_previews_drawer = false;
                        }
                        if widgets::tool_button(ui, &p, theme::icon::ADD, false, "Add page").clicked() {
                            self.notebook.add_page();
                            self.clear_selection();
                        }
                    });
                });
                ui.add_space(6.0);

                // Page mutations are deferred until after the loop so indices stay valid.
                enum PageAction {
                    Select(usize),
                    Duplicate(usize),
                    Delete(usize),
                }
                let mut action = None;
                let total = self.notebook.pages.len();
                let cur_idx = self.notebook.current_page_index;

                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    let card_w = ui.available_width() - 4.0;
                    for idx in 0..total {
                        let page = &self.notebook.pages[idx];
                        let is_active = idx == cur_idx;

                        // Fit width; long (extended) pages show their top portion.
                        let thumb_scale = card_w / page.width;
                        let thumb_h = (page.height * thumb_scale).min(card_w * 1.414);
                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, thumb_h), Sense::click());
                        let resp = resp.on_hover_text(format!("Go to page {}", idx + 1));
                        if resp.clicked() {
                            action = Some(PageAction::Select(idx));
                        }

                        let painter = ui.painter_at(rect.expand(4.0));
                        let to_thumb = |pt: Pos2| rect.min + pt.to_vec2() * thumb_scale;
                        let page_rect = Rect::from_min_size(rect.min, Vec2::new(page.width, page.height) * thumb_scale);
                        NoteRenderer::render_background(
                            &painter.with_clip_rect(rect),
                            page_rect,
                            page.paper_type,
                            page.width,
                            page.height,
                            thumb_scale,
                            &to_thumb,
                            is_dark,
                        );
                        NoteRenderer::render_strokes(&painter.with_clip_rect(rect), &page.strokes, thumb_scale, &to_thumb);

                        let ring = if is_active {
                            Stroke::new(2.0_f32, p.accent)
                        } else if resp.hovered() {
                            Stroke::new(1.5_f32, p.text_muted)
                        } else {
                            Stroke::new(1.0_f32, p.border)
                        };
                        painter.rect_stroke(rect, theme::RADIUS_SMALL, ring);

                        ui.horizontal(|ui| {
                            let label = egui::RichText::new(format!("Page {}", idx + 1));
                            ui.label(if is_active { label.color(p.accent).strong() } else { label.color(p.text_muted) });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let can_delete = total > 1;
                                if ui
                                    .add_enabled_ui(can_delete, |ui| widgets::tool_button(ui, &p, theme::icon::DELETE, false, "Delete page"))
                                    .inner
                                    .clicked()
                                {
                                    action = Some(PageAction::Delete(idx));
                                }
                                if widgets::tool_button(ui, &p, theme::icon::DUPLICATE, false, "Duplicate page").clicked() {
                                    action = Some(PageAction::Duplicate(idx));
                                }
                            });
                        });
                        ui.add_space(10.0);
                    }
                });

                match action {
                    Some(PageAction::Select(idx)) => self.go_to_page(idx),
                    Some(PageAction::Duplicate(idx)) => {
                        self.go_to_page(idx);
                        self.notebook.duplicate_current_page();
                        self.set_status(&format!("Duplicated page {}", idx + 1));
                    }
                    Some(PageAction::Delete(idx)) => {
                        self.go_to_page(idx);
                        self.notebook.delete_current_page();
                        self.clear_selection();
                        self.set_status(&format!("Deleted page {}", idx + 1));
                    }
                    None => {}
                }
            });
    }

    fn undo(&mut self) {
        let page = self.notebook.current_page_mut();
        if let Some(popped) = page.strokes.pop() {
            self.redo_stack.push(popped);
        }
        self.clear_selection();
    }

    fn redo(&mut self) {
        if let Some(stroke) = self.redo_stack.pop() {
            let page = self.notebook.current_page_mut();
            page.strokes.push(stroke);
        }
        self.clear_selection();
    }

    fn prompt_save_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Notebook")
            .add_filter("Inkk Notebook (*.inkk, *.handnote)", &["inkk", "handnote"])
            .set_file_name(&format!("{}.inkk", self.notebook.title))
            .save_file()
        {
            if let Err(e) = NotebookStorage::save(&path, &self.notebook) {
                self.set_error(&format!("Save failed: {}", e));
            } else {
                self.current_file_path = Some(path);
                self.set_status("Notebook saved");
            }
        }
    }

    fn prompt_open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Notebook")
            .add_filter("Inkk Notebook (*.inkk, *.handnote)", &["inkk", "handnote"])
            .pick_file()
        {
            match NotebookStorage::load(&path) {
                Ok(mut nb) => {
                    if self.is_dark_mode {
                        for page in &mut nb.pages {
                            for stroke in &mut page.strokes {
                                let orig = stroke.original_color.unwrap_or(stroke.color);
                                stroke.original_color = Some(orig);
                                stroke.color = theme::adapt_color_to_dark(orig);
                            }
                        }
                    } else {
                        for page in &mut nb.pages {
                            for stroke in &mut page.strokes {
                                if let Some(orig) = stroke.original_color.take() {
                                    stroke.color = orig;
                                }
                            }
                        }
                    }
                    self.notebook = nb;
                    self.current_file_path = Some(path);
                    self.clear_selection();
                    self.set_status("Notebook opened");
                }
                Err(e) => {
                    self.set_error(&format!("Open failed: {}", e));
                }
            }
        }
    }

    // --- ERASERS ---

    fn erase_stroke_at(&mut self, world_pos: Pos2, radius: f32) {
        let page = self.notebook.current_page_mut();
        page.strokes.retain(|stroke| !stroke_hit_test(stroke, world_pos, radius));
        // Erase plot if eraser touches any part of its card
        page.plots.retain(|p| {
            let card_rect = Rect::from_min_size(Pos2::new(p.x, p.y), Vec2::new(p.width, p.height));
            !card_rect.expand(radius).contains(world_pos)
        });
        // Erase matrix if eraser touches any part of its card
        page.matrices.retain(|m| {
            let card_rect = Rect::from_min_size(Pos2::new(m.x, m.y), Vec2::new(m.width, m.height));
            !card_rect.expand(radius).contains(world_pos)
        });
        // Erase grid if eraser touches any part of its grid
        page.grids.retain(|g| {
            let grid_rect = Rect::from_min_size(Pos2::new(g.x, g.y), Vec2::new(g.total_width(), g.total_height()));
            !grid_rect.expand(radius).contains(world_pos)
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

impl InkkApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context, available_size: Vec2) {
        // Don't steal keys while the user is typing (e.g. the notebook title).
        if ctx.wants_keyboard_input() {
            return;
        }
        use egui::{Key, Modifiers};
        let pressed = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));

        if pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z) || pressed(Modifiers::COMMAND, Key::Y) {
            self.redo();
        }
        if pressed(Modifiers::COMMAND, Key::Z) {
            self.undo();
        }
        if pressed(Modifiers::COMMAND, Key::S) {
            self.prompt_save_file();
        }
        if pressed(Modifiers::COMMAND, Key::O) {
            self.prompt_open_file();
        }
        if pressed(Modifiers::COMMAND, Key::V) {
            self.paste_from_clipboard();
        }
        if pressed(Modifiers::COMMAND, Key::Num0) {
            self.zoom_by(1.0 / self.scale, available_size);
        }
        if pressed(Modifiers::NONE, Key::Delete) || pressed(Modifiers::NONE, Key::Backspace) {
            self.delete_selected();
        }
        if pressed(Modifiers::NONE, Key::Escape) {
            self.clear_selection();
            self.editing_text_id = None;
            self.notebook.current_page_mut().text_boxes.retain(|b| !b.text.trim().is_empty());
        }
        if pressed(Modifiers::NONE, Key::PageDown) {
            self.notebook.next_page();
            self.clear_selection();
        }
        if pressed(Modifiers::NONE, Key::PageUp) {
            self.notebook.prev_page();
            self.clear_selection();
        }
        if pressed(Modifiers::NONE, Key::OpenBracket) {
            self.size = (self.size - 1.0).max(1.0);
        }
        if pressed(Modifiers::NONE, Key::CloseBracket) {
            self.size = (self.size + 1.0).min(30.0);
        }

        if pressed(Modifiers::NONE, Key::T) {
            self.set_tool(ToolMode::Text);
        }

        // Number keys pick tools, matching the toolbar order
        let tool_keys = [
            (Key::Num1, ToolMode::Pen),
            (Key::Num2, ToolMode::Highlighter),
            (Key::Num3, ToolMode::Shape(self.current_shape)),
            (Key::Num4, ToolMode::Text),
            (Key::Num5, ToolMode::EraserStroke),
            (Key::Num6, ToolMode::EraserPoint),
            (Key::Num7, ToolMode::LassoSelect),
            (Key::Num8, ToolMode::Pan),
        ];
        for (key, mode) in tool_keys {
            if pressed(Modifiers::NONE, key) {
                self.set_tool(mode);
            }
        }

        if pressed(Modifiers::SHIFT, Key::R) {
            self.ruler.toggle_protractor();
            let is_on = self.ruler.kind == RulerKind::Protractor;
            self.set_status(if is_on { "Protractor ON 📐 (drag body to move, dial to rotate)" } else { "Protractor OFF" });
        } else if pressed(Modifiers::NONE, Key::R) {
            self.ruler.toggle_ruler();
            let is_on = self.ruler.kind == RulerKind::Ruler;
            self.set_status(if is_on { "Straight Ruler ON 📏 (drag body to move, dial to rotate)" } else { "Ruler OFF" });
        }

        if self.editing_text_id.is_none() && (pressed(Modifiers::NONE, Key::M) || pressed(Modifiers::COMMAND, Key::M)) {
            self.math_studio.toggle();
            let is_on = self.math_studio.is_open;
            self.set_status(if is_on { "Math Studio OPEN 🔬 [MATLAB Engine Ready]" } else { "Math Studio CLOSED" });
        }
    }

    /// Floating toast for status / error messages, fading out over `theme::TOAST_SECS`.
    fn render_toast(&self, ctx: &egui::Context, canvas_rect: Rect) {
        let Some((msg, at, is_error)) = &self.status_message else { return };
        let t = at.elapsed().as_secs_f32();
        let total = if *is_error { theme::TOAST_SECS * 2.0 } else { theme::TOAST_SECS };
        if t >= total {
            return;
        }
        let alpha = ((total - t) / 0.4).clamp(0.0, 1.0);
        let p = self.palette();
        let (icon, color) = if *is_error { (theme::icon::WARN, p.danger) } else { (theme::icon::CHECK, p.success) };

        egui::Area::new(egui::Id::new("toast"))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .fixed_pos(Pos2::new(canvas_rect.center().x, canvas_rect.bottom() - 16.0))
            .show(ctx, |ui| {
                ui.set_opacity(alpha);
                egui::Frame::popup(ui.style())
                    .rounding(egui::Rounding::same(18.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(icon).color(color));
                            ui.label(egui::RichText::new(msg).color(p.text));
                        });
                    });
            });
        ctx.request_repaint();
    }

    /// Draws a tool-specific cursor preview (brush size / eraser radius) on the canvas.
    fn render_cursor_preview(&self, ctx: &egui::Context, painter: &egui::Painter, cursor: Pos2) {
        let p = self.palette();
        match self.tool {
            ToolMode::Pen | ToolMode::Highlighter => {
                let mul = if self.tool == ToolMode::Highlighter { 2.5 } else { 0.5 };
                let r = (self.size * mul * self.scale).max(1.5);
                painter.circle_stroke(cursor, r + 1.0, Stroke::new(1.0_f32, p.text_muted.gamma_multiply(0.7)));
                ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
            }
            ToolMode::EraserStroke | ToolMode::EraserPoint => {
                // Matches the world-space radius used by erase_*_at (scale cancels out on screen)
                let r = if self.tool == ToolMode::EraserStroke { self.size * 5.0 } else { self.size * 3.5 };
                painter.circle(cursor, r, p.danger.gamma_multiply(0.08), Stroke::new(1.2_f32, p.danger.gamma_multiply(0.8)));
                ctx.set_cursor_icon(egui::CursorIcon::None);
            }
            ToolMode::Shape(_) => ctx.set_cursor_icon(egui::CursorIcon::Crosshair),
            ToolMode::LassoSelect => {
                let mut cursor_set = false;
                if let Some(bounds) = self.get_selection_bounds() {
                    let sb = Rect::from_min_max(self.to_screen(bounds.min), self.to_screen(bounds.max)).expand(8.0);
                    if let Some(handle) = hit_handle(cursor, sb, 8.0) {
                        let icon = match handle {
                            SelectionHandle::TopLeft | SelectionHandle::BottomRight => egui::CursorIcon::ResizeNwSe,
                            SelectionHandle::TopRight | SelectionHandle::BottomLeft => egui::CursorIcon::ResizeNeSw,
                            SelectionHandle::Top | SelectionHandle::Bottom => egui::CursorIcon::ResizeNorth,
                            SelectionHandle::Left | SelectionHandle::Right => egui::CursorIcon::ResizeEast,
                        };
                        ctx.set_cursor_icon(icon);
                        cursor_set = true;
                    } else if sb.contains(cursor) {
                        ctx.set_cursor_icon(egui::CursorIcon::Move);
                        cursor_set = true;
                    }
                }
                if !cursor_set {
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                }
            }
            ToolMode::Text => ctx.set_cursor_icon(egui::CursorIcon::Text),
            ToolMode::Pan => {
                let grabbing = ctx.input(|i| i.pointer.primary_down());
                ctx.set_cursor_icon(if grabbing { egui::CursorIcon::Grabbing } else { egui::CursorIcon::Grab });
            }
        }
    }
}

impl eframe::App for InkkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let p = self.palette();
        let available_size = ctx.screen_rect().size();

        let bar_frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .stroke(Stroke::new(1.0_f32, p.border));

        egui::TopBottomPanel::top("top_panel").frame(bar_frame).show(ctx, |ui| {
            self.render_top_toolbar(ui, ctx);
        });
        egui::TopBottomPanel::top("context_bar")
            .frame(bar_frame.fill(p.surface_alt).inner_margin(egui::Margin::symmetric(12.0, 4.0)))
            .show(ctx, |ui| {
                self.render_context_bar(ui);
            });
        egui::TopBottomPanel::bottom("bottom_panel").frame(bar_frame).show(ctx, |ui| {
            self.render_bottom_bar(ui, available_size);
        });
        self.math_studio.render_panel(ctx, self.notebook.current_page_mut(), &p, self.is_dark_mode);
        self.render_previews_drawer(ctx);

        self.handle_shortcuts(ctx, available_size);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.desk))
            .show(ctx, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(canvas_rect, Sense::click_and_drag());
                let painter = ui.painter_at(canvas_rect);

                // Handle Ruler & Protractor Gestures (move, rotate, wheel dial)
                let ruler_consumed = self.ruler.handle_events(ui, canvas_rect);
                if self.ruler.is_dragging || self.ruler.is_rotating {
                    ctx.request_repaint();
                }

                // Handle Wheel Zoom & Pan (only if ruler did not consume the wheel event)
                if !ruler_consumed && response.hovered() {
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
                let pan = self.pan;
                let to_screen = move |p: Pos2| Pos2::new(p.x * scale + pan.x, p.y * scale + pan.y);

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

                // 2. Render Existing Images, Strokes, Text with Layering:
                // Strictly clipped to page_screen_rect so NO ink ever renders outside the paper sheet!
                let page_painter = painter.with_clip_rect(page_screen_rect);
                let page = self.notebook.current_page();

                // Layer 0: Image Layer (Scrble Ink PictureLayer)
                for img in &page.images {
                    let tex_handle = self.image_textures.entry(img.id.clone()).or_insert_with(|| {
                        let color_img = match image::load_from_memory(&img.data) {
                            Ok(dyn_img) => {
                                let rgba = dyn_img.to_rgba8();
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [rgba.width() as usize, rgba.height() as usize],
                                    &rgba.into_raw(),
                                )
                            }
                            Err(_) => egui::ColorImage::example(),
                        };
                        ctx.load_texture(&img.id, color_img, egui::TextureOptions::LINEAR)
                    });

                    let img_screen_min = to_screen(Pos2::new(img.x, img.y));
                    let img_screen_size = Vec2::new(img.width * scale, img.height * scale);
                    let img_rect = Rect::from_min_size(img_screen_min, img_screen_size);
                    page_painter.image(
                        tex_handle.id(),
                        img_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                // Layer 0.1: Table Grids
                for grid in &page.grids {
                    MathRenderer::render_table_grid(&page_painter, grid, scale, &to_screen, &p, is_dark);
                }

                // Layer 0.2: 2D Plots & Mathematical Graphs
                for plot in &page.plots {
                    MathRenderer::render_plot(&page_painter, plot, scale, &to_screen, &p, is_dark);
                }

                // Layer 0.3: Matrix Entities
                for matrix in &page.matrices {
                    MathRenderer::render_matrix(&page_painter, matrix, scale, &to_screen, &p, is_dark);
                }

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

                // Layer 3: Text Layer (Scrble Ink TextLayer)
                for tb in &page.text_boxes {
                    if self.editing_text_id.as_deref() == Some(&tb.id) {
                        continue;
                    }
                    if tb.text.trim().is_empty() {
                        continue;
                    }

                    let tb_pos = to_screen(Pos2::new(tb.x, tb.y));
                    let scaled_font_size = (tb.font_size * scale).max(8.0);
                    let tb_screen_w = tb.width * scale;

                    let mut job = egui::text::LayoutJob::default();
                    let font_id = egui::FontId::proportional(scaled_font_size);
                    let color = Color32::from_rgba_unmultiplied(tb.color[0], tb.color[1], tb.color[2], tb.color[3]);
                    let format = egui::TextFormat {
                        font_id,
                        color,
                        italics: tb.is_italic,
                        ..Default::default()
                    };
                    job.append(&tb.text, 0.0, format);
                    job.wrap.max_width = tb_screen_w;

                    let galley = ui.fonts(|f| f.layout_job(job));
                    let galley_size = galley.size();

                    if self.tool == ToolMode::Text {
                        let box_rect = Rect::from_min_size(tb_pos, Vec2::new(tb_screen_w, (tb.height * scale).max(galley_size.y + 8.0)));
                        page_painter.rect_stroke(box_rect, 4.0, Stroke::new(1.0_f32, p.border.gamma_multiply(0.4)));
                    }

                    page_painter.galley(tb_pos + Vec2::new(2.0, 2.0), galley, color);
                }

                // === ELASTIC TAIL: Bắt dính ngòi mực vào đúng đầu chuột thời gian thực (giống Renderer.cpp) ===
                if let Some(ref cur) = self.current_stroke {
                    let pointer = ui.input(|i| i.pointer.clone());
                    if let Some(raw_cursor) = pointer.interact_pos().or(pointer.latest_pos()) {
                        let effective_tail = self.ruler.snap_point(raw_cursor).unwrap_or(raw_cursor);
                        NoteRenderer::render_elastic_tail(&page_painter, cur, effective_tail, scale, &to_screen);
                    }
                    // Yêu cầu egui repaint ngay lập tức trong frame tiếp theo để không rơi rụng vi điểm nào
                    ctx.request_repaint();
                }

                // 3. Lasso path & selection visuals
                let accent = p.accent;
                if self.tool == ToolMode::LassoSelect && self.lasso_points.len() >= 2 {
                    let screen_pts: Vec<Pos2> = self.lasso_points.iter().map(|pt| self.to_screen(*pt)).collect();
                    if screen_pts.len() >= 3 {
                        painter.add(egui::Shape::convex_polygon(screen_pts.clone(), p.accent_soft, Stroke::NONE));
                    }
                    let mut closed = screen_pts;
                    closed.push(closed[0]);
                    painter.add(egui::Shape::dashed_line(&closed, Stroke::new(1.5_f32, accent), 6.0, 4.0));
                }

                if let Some(world_bounds) = self.get_selection_bounds() {
                    let screen_bounds =
                        Rect::from_min_max(self.to_screen(world_bounds.min), self.to_screen(world_bounds.max)).expand(8.0);
                    painter.rect_filled(screen_bounds, 4.0, p.accent_soft.gamma_multiply(0.5));
                    let r = screen_bounds;
                    let outline = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
                    painter.add(egui::Shape::dashed_line(&outline, Stroke::new(1.5_f32, accent), 6.0, 4.0));

                    let handles = get_handle_positions(r);
                    for (handle, h) in handles {
                        let is_active = self.active_resize_handle == Some(handle);
                        let fill = if is_active { accent } else { p.surface };
                        let stroke_color = if is_active { p.on_accent } else { accent };
                        painter.circle(h, 5.0, fill, Stroke::new(1.5_f32, stroke_color));
                    }

                    // Pill badge above the selection
                    let total_count = self.selected_stroke_ids.len()
                        + self.selected_image_ids.len()
                        + self.selected_plot_ids.len()
                        + self.selected_matrix_ids.len()
                        + self.selected_grid_ids.len();
                    let badge_text = if total_count == 1 {
                        if !self.selected_image_ids.is_empty() {
                            "1 image selected · drag handles to resize · Del to delete".to_string()
                        } else if !self.selected_plot_ids.is_empty() {
                            "1 plot graph selected · drag handles to resize · Del to delete".to_string()
                        } else if !self.selected_matrix_ids.is_empty() {
                            "1 matrix selected · drag handles to resize · Del to delete".to_string()
                        } else if !self.selected_grid_ids.is_empty() {
                            "1 table grid selected · drag handles to resize · Del to delete".to_string()
                        } else {
                            "1 stroke selected · drag to move/resize · Del to delete".to_string()
                        }
                    } else {
                        format!("{} item(s) selected · drag to move/resize · Del to delete", total_count)
                    };
                    let galley = painter.layout_no_wrap(badge_text, egui::FontId::proportional(12.0), p.on_accent);
                    let badge_min = Pos2::new(r.left(), (r.top() - galley.size().y - 14.0).max(canvas_rect.top() + 6.0));
                    let badge_rect = Rect::from_min_size(badge_min, galley.size() + Vec2::new(16.0, 6.0));
                    painter.rect_filled(badge_rect, 10.0, accent);
                    painter.galley(badge_rect.min + Vec2::new(8.0, 3.0), galley, p.on_accent);
                }

                // Stencil Overlay (Glassmorphic Ruler / Protractor)
                self.ruler.render_overlay(&painter, &p, is_dark);

                // Inline Canvas-Embedded Text Box Editor (with Vietnamese IME and dynamic auto-expansion)
                let mut finish_text_editing = false;
                let mut expand_target = None;
                if let Some(ref edit_id) = self.editing_text_id.clone() {
                    let grip_px = 18.0f32;
                    let resize_strip_px = 7.0f32;
                    {
                        let page = self.notebook.current_page_mut();
                        if let Some(tb) = page.text_boxes.iter_mut().find(|b| b.id == *edit_id) {
                            let tb_screen_pos = to_screen(Pos2::new(tb.x, tb.y));
                            let scaled_fs = (tb.font_size * scale).max(9.0);
                            let tb_sw = (tb.width * scale).max(100.0);
                            let tb_color = Color32::from_rgba_unmultiplied(tb.color[0], tb.color[1], tb.color[2], tb.color[3]);

                            // Measure text height using LayoutJob
                            let mut job = egui::text::LayoutJob::default();
                            job.append(
                                &tb.text,
                                0.0,
                                egui::TextFormat {
                                    font_id: egui::FontId::proportional(scaled_fs),
                                    color: tb_color,
                                    italics: tb.is_italic,
                                    ..Default::default()
                                },
                            );
                            job.wrap.max_width = (tb_sw - resize_strip_px - 10.0).max(40.0);
                            let galley_h = ui.fonts(|f| f.layout_job(job)).size().y;
                            let tb_sh = (grip_px + galley_h.max(28.0) + 12.0).max(tb.height * scale);
                            let box_rect = Rect::from_min_size(tb_screen_pos, Vec2::new(tb_sw, tb_sh));

                            // Visual: Box background & border (selection box aesthetic)
                            let bg = if is_dark {
                                Color32::from_rgba_unmultiplied(26, 30, 42, 220)
                            } else {
                                Color32::from_rgba_unmultiplied(252, 252, 254, 230)
                            };
                            page_painter.rect_filled(box_rect, 4.0, bg);
                            page_painter.rect_stroke(box_rect, 4.0, Stroke::new(1.5_f32, p.accent));

                            // Visual: Top grip strip (drag handle)
                            let grip_rect = Rect::from_min_max(
                                box_rect.left_top(),
                                Pos2::new(box_rect.right(), box_rect.top() + grip_px),
                            );
                            page_painter.rect_filled(
                                grip_rect,
                                egui::Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 },
                                p.accent.gamma_multiply(0.18),
                            );
                            page_painter.text(
                                grip_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "⠿  drag to move  ↔ right edge to resize",
                                egui::FontId::proportional(9.0),
                                p.text_muted,
                            );

                            // Visual: Right edge resize strip
                            let right_strip = Rect::from_min_max(
                                Pos2::new(box_rect.right() - resize_strip_px, box_rect.top() + grip_px),
                                box_rect.right_bottom(),
                            );
                            page_painter.rect_filled(
                                right_strip,
                                egui::Rounding { nw: 0.0, ne: 0.0, sw: 0.0, se: 4.0 },
                                p.accent.gamma_multiply(0.15),
                            );

                            // Inline TextEdit directly on the canvas panel
                            let text_rect = Rect::from_min_max(
                                Pos2::new(box_rect.left() + 4.0, box_rect.top() + grip_px + 2.0),
                                Pos2::new(box_rect.right() - resize_strip_px - 2.0, box_rect.bottom() - 4.0),
                            );
                            let text_resp = ui.put(
                                text_rect,
                                egui::TextEdit::multiline(&mut tb.text)
                                    .font(egui::FontId::proportional(scaled_fs))
                                    .text_color(tb_color)
                                    .desired_width(text_rect.width())
                                    .hint_text("Type note here...")
                                    .frame(false),
                            );

                            tb.height = (tb_sh / scale).max(40.0);

                            // Scrble Ink _TextOverflow: Check if text reached bottom of page and auto-expand
                            if text_resp.changed() && (tb.y + tb.height > page_h - 150.0) {
                                expand_target = Some(tb.y + tb.height + 100.0);
                            }
                        }
                    } // mutable borrow of self.notebook ends here

                    if let Some(target_y) = expand_target {
                        if self.notebook.current_page_mut().check_auto_expand(target_y) {
                            self.set_status("Page auto-expanded! ♾️");
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        finish_text_editing = true;
                    }
                }
                if finish_text_editing {
                    self.editing_text_id = None;
                    self.notebook.current_page_mut().text_boxes.retain(|b| !b.text.trim().is_empty());
                }

                // 4. Robust & Zero-Threshold Inking / Selection Pointer Tracking
                let pointer = ui.input(|i| i.pointer.clone());
                let pointer_pos = pointer.interact_pos().or(pointer.latest_pos());
                let is_primary_pressed = pointer.primary_pressed();
                // Only treat a press as a canvas gesture if it started over the canvas,
                // so dragging toolbar sliders never inks, erases or pans.
                if is_primary_pressed {
                    self.canvas_gesture = response.hovered();
                } else if !pointer.primary_down() {
                    self.canvas_gesture = false;
                }
                let is_primary_down = pointer.primary_down() && self.canvas_gesture;

                if response.hovered() {
                    if let Some(cursor) = pointer.hover_pos() {
                        let effective_cursor = self.ruler.snap_point(cursor).unwrap_or(cursor);
                        self.render_cursor_preview(ctx, &painter, effective_cursor);
                    }
                }
                self.render_toast(ctx, canvas_rect);

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
                } else if !ruler_consumed && is_primary_pressed && self.canvas_gesture {
                    if let Some(cursor) = pointer_pos {
                        let effective_cursor = self.ruler.snap_point(cursor).unwrap_or(cursor);
                        let world = self.to_world(effective_cursor);

                        // Check if user clicked on the '✕' close button of any plot, matrix, or table grid
                        let mut delete_plot_idx = None;
                        let mut delete_mat_idx = None;
                        let mut delete_grid_idx = None;
                        {
                            let page = self.notebook.current_page();
                            let scale = self.scale;

                            // 1. Plot close button
                            for (idx, plot) in page.plots.iter().enumerate() {
                                let card_min = self.to_screen(Pos2::new(plot.x, plot.y));
                                let card_size = Vec2::new(plot.width * scale, plot.height * scale);
                                let card_rect = Rect::from_min_size(card_min, card_size);
                                let btn_size = (20.0 * scale).clamp(16.0, 24.0);
                                let close_center = card_rect.right_top() + Vec2::new(-btn_size * 0.75, btn_size * 0.75);
                                if cursor.distance(close_center) <= (btn_size * 0.85).max(16.0) {
                                    delete_plot_idx = Some(idx);
                                    break;
                                }
                            }

                            // 2. Matrix close button
                            if delete_plot_idx.is_none() {
                                for (idx, mat) in page.matrices.iter().enumerate() {
                                    let card_min = self.to_screen(Pos2::new(mat.x, mat.y));
                                    let card_size = Vec2::new(mat.width * scale, mat.height * scale);
                                    let card_rect = Rect::from_min_size(card_min, card_size);
                                    let btn_size = (18.0 * scale).clamp(14.0, 22.0);
                                    let close_center = card_rect.right_top() + Vec2::new(-btn_size * 0.65, btn_size * 0.65);
                                    if cursor.distance(close_center) <= (btn_size * 0.85).max(16.0) {
                                        delete_mat_idx = Some(idx);
                                        break;
                                    }
                                }
                            }

                            // 3. Table grid close button
                            if delete_plot_idx.is_none() && delete_mat_idx.is_none() {
                                for (idx, grid) in page.grids.iter().enumerate() {
                                    let origin = self.to_screen(Pos2::new(grid.x, grid.y));
                                    let total_w = grid.col_width * grid.cols as f32 * scale;
                                    let total_h = grid.row_height * grid.rows as f32 * scale;
                                    let grid_rect = Rect::from_min_size(origin, Vec2::new(total_w, total_h));
                                    let btn_size = (18.0 * scale).clamp(14.0, 22.0);
                                    let close_center = grid_rect.right_top() + Vec2::new(-btn_size * 0.75, btn_size * 0.75);
                                    if cursor.distance(close_center) <= (btn_size * 0.85).max(16.0) {
                                        delete_grid_idx = Some(idx);
                                        break;
                                    }
                                }
                            }
                        }

                        let closed_any = if let Some(idx) = delete_plot_idx {
                            self.notebook.current_page_mut().plots.remove(idx);
                            true
                        } else if let Some(idx) = delete_mat_idx {
                            self.notebook.current_page_mut().matrices.remove(idx);
                            true
                        } else if let Some(idx) = delete_grid_idx {
                            self.notebook.current_page_mut().grids.remove(idx);
                            true
                        } else {
                            false
                        };

                        if closed_any {
                            self.clear_selection();
                            self.set_status("Đã xoá mục toán học 🗑");
                        } else {
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
                                let mut hit_handle_opt = None;
                                if let Some(wb) = self.get_selection_bounds() {
                                    let sb = Rect::from_min_max(self.to_screen(wb.min), self.to_screen(wb.max)).expand(8.0);
                                    hit_handle_opt = hit_handle(cursor, sb, 9.0);
                                }

                                if let Some(handle) = hit_handle_opt {
                                    self.active_resize_handle = Some(handle);
                                    self.is_dragging_selection = true;
                                    self.drag_last_pos_world = world;
                                    self.prepare_selection_drag();
                                } else if let Some(wb) = self.get_selection_bounds() {
                                    let sb = Rect::from_min_max(self.to_screen(wb.min), self.to_screen(wb.max)).expand(8.0);
                                    if sb.contains(cursor) {
                                        self.active_resize_handle = None;
                                        self.is_dragging_selection = true;
                                        self.drag_last_pos_world = world;
                                        self.prepare_selection_drag();
                                    } else if let Some((kind, id)) = self.hit_test_clickable_card(world) {
                                        self.clear_selection();
                                        match kind {
                                            "plot" => { self.selected_plot_ids.insert(id); }
                                            "matrix" => { self.selected_matrix_ids.insert(id); }
                                            "grid" => { self.selected_grid_ids.insert(id); }
                                            "image" => { self.selected_image_ids.insert(id); }
                                            _ => {}
                                        }
                                        self.active_resize_handle = None;
                                        self.is_dragging_selection = true;
                                        self.drag_last_pos_world = world;
                                        self.prepare_selection_drag();
                                    } else {
                                        self.clear_selection();
                                        self.is_dragging_selection = false;
                                        self.lasso_points = vec![world];
                                    }
                                } else if let Some((kind, id)) = self.hit_test_clickable_card(world) {
                                    self.clear_selection();
                                    match kind {
                                        "plot" => { self.selected_plot_ids.insert(id); }
                                        "matrix" => { self.selected_matrix_ids.insert(id); }
                                        "grid" => { self.selected_grid_ids.insert(id); }
                                        "image" => { self.selected_image_ids.insert(id); }
                                        _ => {}
                                    }
                                    self.active_resize_handle = None;
                                    self.is_dragging_selection = true;
                                    self.drag_last_pos_world = world;
                                    self.prepare_selection_drag();
                                } else {
                                    self.clear_selection();
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
                                        crate::stroke_algorithm::compute_thickness(
                                            pen,
                                            self.size,
                                            0.0,
                                            crate::stroke_algorithm::pressure_curve(pressure),
                                        ),
                                    );

                                    let c = [self.color.r(), self.color.g(), self.color.b(), self.color.a()];
                                    let orig = if self.is_dark_mode {
                                        Some(theme::adapt_color_to_light(c))
                                    } else {
                                        None
                                    };
                                    self.current_stroke = Some(NoteStroke {
                                        id: format!("stroke_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
                                        pen_type: pen,
                                        shape_type: shape,
                                        is_filled: self.is_shape_filled,
                                        color: c,
                                        original_color: orig,
                                        size: self.size,
                                        opacity: self.opacity,
                                        points: vec![initial_pt],
                                    });
                                    self.redo_stack.clear();
                                }
                            }
                            ToolMode::Text => {
                                let mut text_handled = false;

                                // 1. If currently editing a text box, check if clicked on its grip or resize strip
                                if let Some(ref edit_id) = self.editing_text_id.clone() {
                                    let tb_info = {
                                        let page = self.notebook.current_page();
                                        page.text_boxes.iter().find(|b| b.id == *edit_id).map(|b| (b.x, b.y, b.width, b.height))
                                    };
                                    if let Some((tb_x, tb_y, tb_w, tb_h)) = tb_info {
                                        let grip_h_world = 18.0 / scale;
                                        let resize_w_world = 9.0 / scale;
                                        let grip_rect = Rect::from_min_max(
                                            Pos2::new(tb_x, tb_y),
                                            Pos2::new(tb_x + tb_w, tb_y + grip_h_world),
                                        );
                                        let resize_rect = Rect::from_min_max(
                                            Pos2::new(tb_x + tb_w - resize_w_world, tb_y + grip_h_world),
                                            Pos2::new(tb_x + tb_w + 4.0 / scale, tb_y + tb_h),
                                        );
                                        let full_box = Rect::from_min_max(
                                            Pos2::new(tb_x, tb_y),
                                            Pos2::new(tb_x + tb_w, tb_y + tb_h),
                                        );

                                        if resize_rect.contains(world) {
                                            self.resizing_text_id = Some(edit_id.clone());
                                            self.resize_start_pointer = cursor;
                                            self.resize_start_w = tb_w;
                                            text_handled = true;
                                        } else if grip_rect.contains(world) {
                                            self.dragging_text_id = Some(edit_id.clone());
                                            self.drag_text_start_pointer = cursor;
                                            self.drag_text_start_pos = Pos2::new(tb_x, tb_y);
                                            text_handled = true;
                                        } else if full_box.contains(world) {
                                            // Clicked inside the text body of currently editing box
                                            text_handled = true;
                                        } else {
                                            // Clicked outside: finish editing current box
                                            self.editing_text_id = None;
                                            self.notebook.current_page_mut().text_boxes.retain(|b| !b.text.trim().is_empty());
                                        }
                                    }
                                }

                                // 2. If not handled yet, check if clicked on another text box or empty canvas
                                if !text_handled {
                                    let clicked_tb = {
                                        let page = self.notebook.current_page();
                                        page.text_boxes.iter().rev()
                                            .find(|tb| {
                                                let hit_rect = Rect::from_min_size(
                                                    Pos2::new(tb.x, tb.y),
                                                    Vec2::new(tb.width, tb.height.max(30.0)),
                                                );
                                                hit_rect.contains(world)
                                            })
                                            .map(|tb| (tb.id.clone(), tb.font_size, tb.is_bold, tb.is_italic, tb.color))
                                    };

                                    if let Some((tb_id, fs, bold, italic, col)) = clicked_tb {
                                        self.editing_text_id = Some(tb_id);
                                        self.text_font_size = fs;
                                        self.text_is_bold = bold;
                                        self.text_is_italic = italic;
                                        self.color = Color32::from_rgba_unmultiplied(col[0], col[1], col[2], col[3]);
                                    } else if world.x >= 0.0 && world.x <= page_w && world.y >= 0.0 {
                                        let max_w = (page_w - world.x - 20.0).clamp(140.0, 420.0);
                                        let c = [self.color.r(), self.color.g(), self.color.b(), self.color.a()];
                                        let orig = if self.is_dark_mode {
                                            Some(theme::adapt_color_to_light(c))
                                        } else {
                                            None
                                        };
                                        let mut new_tb = TextBox::new(world.x, world.y, c, self.text_font_size);
                                        new_tb.width = max_w;
                                        new_tb.height = 50.0;
                                        new_tb.is_bold = self.text_is_bold;
                                        new_tb.is_italic = self.text_is_italic;
                                        new_tb.original_color = orig;
                                        let new_id = new_tb.id.clone();
                                        self.notebook.current_page_mut().text_boxes.push(new_tb);
                                        self.editing_text_id = Some(new_id);
                                    }
                                }
                            }
                        }
                    }
                }
                } else if !ruler_consumed && is_primary_down {
                    if let Some(cursor) = pointer_pos {
                        let effective_cursor = self.ruler.snap_point(cursor).unwrap_or(cursor);
                        let world = self.to_world(effective_cursor);

                        if let Some(ref drag_id) = self.dragging_text_id {
                            let delta = (cursor - self.drag_text_start_pointer) / self.scale;
                            let page = self.notebook.current_page_mut();
                            if let Some(tb) = page.text_boxes.iter_mut().find(|b| b.id == *drag_id) {
                                tb.x = (self.drag_text_start_pos.x + delta.x).clamp(0.0, page_w - 40.0);
                                tb.y = (self.drag_text_start_pos.y + delta.y).max(0.0);
                            }
                        }

                        if let Some(ref resize_id) = self.resizing_text_id.clone() {
                            let delta = (cursor - self.resize_start_pointer) / self.scale;
                            let page = self.notebook.current_page_mut();
                            if let Some(tb) = page.text_boxes.iter_mut().find(|b| b.id == *resize_id) {
                                tb.width = (self.resize_start_w + delta.x).clamp(80.0, page_w - tb.x - 10.0);
                            }
                        }

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
                                    if let Some(handle) = self.active_resize_handle {
                                        if let Some(orig) = self.initial_selection_rect {
                                            let init_w = orig.width().max(5.0);
                                            let init_h = orig.height().max(5.0);

                                            let (new_min, new_max) = match handle {
                                                SelectionHandle::BottomRight => (
                                                    orig.min,
                                                    Pos2::new(world.x.max(orig.min.x + 10.0), world.y.max(orig.min.y + 10.0)),
                                                ),
                                                SelectionHandle::BottomLeft => (
                                                    Pos2::new(world.x.min(orig.max.x - 10.0), orig.min.y),
                                                    Pos2::new(orig.max.x, world.y.max(orig.min.y + 10.0)),
                                                ),
                                                SelectionHandle::TopRight => (
                                                    Pos2::new(orig.min.x, world.y.min(orig.max.y - 10.0)),
                                                    Pos2::new(world.x.max(orig.min.x + 10.0), orig.max.y),
                                                ),
                                                SelectionHandle::TopLeft => (
                                                    Pos2::new(world.x.min(orig.max.x - 10.0), world.y.min(orig.max.y - 10.0)),
                                                    orig.max,
                                                ),
                                                SelectionHandle::Top => (
                                                    Pos2::new(orig.min.x, world.y.min(orig.max.y - 10.0)),
                                                    orig.max,
                                                ),
                                                SelectionHandle::Bottom => (
                                                    orig.min,
                                                    Pos2::new(orig.max.x, world.y.max(orig.min.y + 10.0)),
                                                ),
                                                SelectionHandle::Left => (
                                                    Pos2::new(world.x.min(orig.max.x - 10.0), orig.min.y),
                                                    orig.max,
                                                ),
                                                SelectionHandle::Right => (
                                                    orig.min,
                                                    Pos2::new(world.x.max(orig.min.x + 10.0), orig.max.y),
                                                ),
                                            };

                                            let new_w = (new_max.x - new_min.x).max(10.0);
                                            let new_h = (new_max.y - new_min.y).max(10.0);
                                            let scale_x = new_w / init_w;
                                            let scale_y = new_h / init_h;

                                            let page = self.notebook.current_page_mut();
                                            for (id, orig_pts) in &self.initial_stroke_points {
                                                if let Some(stroke) = page.strokes.iter_mut().find(|s| s.id == *id) {
                                                    for (i, orig_pt) in orig_pts.iter().enumerate() {
                                                        if i < stroke.points.len() {
                                                            stroke.points[i].x = new_min.x + (orig_pt.x - orig.min.x) * scale_x;
                                                            stroke.points[i].y = new_min.y + (orig_pt.y - orig.min.y) * scale_y;
                                                        }
                                                    }
                                                }
                                            }

                                            for (id, orig_x, orig_y, orig_w, orig_h) in &self.initial_image_rects {
                                                if let Some(img) = page.images.iter_mut().find(|i| i.id == *id) {
                                                    img.x = new_min.x + (orig_x - orig.min.x) * scale_x;
                                                    img.y = new_min.y + (orig_y - orig.min.y) * scale_y;
                                                    img.width = (orig_w * scale_x).max(10.0);
                                                    img.height = (orig_h * scale_y).max(10.0);
                                                }
                                            }

                                            for (id, orig_x, orig_y, orig_w, orig_h) in &self.initial_plot_rects {
                                                if let Some(plot) = page.plots.iter_mut().find(|p| p.id == *id) {
                                                    plot.x = new_min.x + (orig_x - orig.min.x) * scale_x;
                                                    plot.y = new_min.y + (orig_y - orig.min.y) * scale_y;
                                                    plot.width = (orig_w * scale_x).max(20.0);
                                                    plot.height = (orig_h * scale_y).max(20.0);
                                                }
                                            }

                                            for (id, orig_x, orig_y, orig_w, orig_h) in &self.initial_matrix_rects {
                                                if let Some(matrix) = page.matrices.iter_mut().find(|m| m.id == *id) {
                                                    matrix.x = new_min.x + (orig_x - orig.min.x) * scale_x;
                                                    matrix.y = new_min.y + (orig_y - orig.min.y) * scale_y;
                                                    matrix.width = (orig_w * scale_x).max(20.0);
                                                    matrix.height = (orig_h * scale_y).max(20.0);
                                                }
                                            }

                                            for (id, orig_x, orig_y, orig_w, orig_h) in &self.initial_grid_rects {
                                                if let Some(grid) = page.grids.iter_mut().find(|g| g.id == *id) {
                                                    grid.x = new_min.x + (orig_x - orig.min.x) * scale_x;
                                                    grid.y = new_min.y + (orig_y - orig.min.y) * scale_y;
                                                    let new_w = (orig_w * scale_x).max(20.0);
                                                    let new_h = (orig_h * scale_y).max(20.0);
                                                    grid.col_width = (new_w / (grid.cols.max(1) as f32)).max(10.0);
                                                    grid.row_height = (new_h / (grid.rows.max(1) as f32)).max(10.0);
                                                }
                                            }
                                        }
                                    } else {
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
                                        for img in &mut page.images {
                                            if self.selected_image_ids.contains(&img.id) {
                                                img.x += delta.x;
                                                img.y += delta.y;
                                            }
                                        }
                                        for plot in &mut page.plots {
                                            if self.selected_plot_ids.contains(&plot.id) {
                                                plot.x += delta.x;
                                                plot.y += delta.y;
                                            }
                                        }
                                        for matrix in &mut page.matrices {
                                            if self.selected_matrix_ids.contains(&matrix.id) {
                                                matrix.x += delta.x;
                                                matrix.y += delta.y;
                                            }
                                        }
                                        for grid in &mut page.grids {
                                            if self.selected_grid_ids.contains(&grid.id) {
                                                grid.x += delta.x;
                                                grid.y += delta.y;
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
                                        // Speed-based dynamic tapering - ULTRA RESPONSIVE (Scribble Ink level)
                                        let dt = self.last_pointer_time.elapsed().as_secs_f32().clamp(0.001, 0.05);
                                        let dist = (bounded_x - self.last_pointer_pos_world.x).hypot(bounded_y - self.last_pointer_pos_world.y);
                                        let speed = dist / dt; // world pixels per sec
                                        // Phản hồi TỨC THÌ, biến thiên mượt mà tự nhiên
                                        let target = (1.0 - (speed / 1800.0) * 0.45).clamp(0.42, 1.0);
                                        self.current_pressure = self.current_pressure * 0.68 + target * 0.32;
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

                                    // === SCRBLE INK: WritingArea Auto-Scroll (Tự động cuộn êm khi viết sát mép dưới) ===
                                    let screen_pos = self.to_screen(Pos2::new(bounded_x, bounded_y));
                                    let threshold_y = canvas_rect.max.y - 70.0;
                                    if screen_pos.y > threshold_y {
                                        let scroll_amount = ((screen_pos.y - threshold_y) * 0.18).clamp(1.0, 5.0);
                                        self.pan.y -= scroll_amount;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Pointer released or up
                    self.resizing_text_id = None;
                    if let Some(drag_id) = self.dragging_text_id.take() {
                        let page = self.notebook.current_page_mut();
                        if let Some(tb) = page.text_boxes.iter().find(|b| b.id == drag_id) {
                            if page.check_auto_expand(tb.y + tb.height) {
                                self.set_status("Page auto-expanded! ♾️");
                            }
                        }
                    }

                    if self.is_dragging_selection {
                        self.is_dragging_selection = false;
                        self.active_resize_handle = None;
                        self.initial_selection_rect = None;
                        self.initial_stroke_points.clear();
                        self.initial_image_rects.clear();
                        self.initial_plot_rects.clear();
                        self.initial_matrix_rects.clear();
                        self.initial_grid_rects.clear();

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
                        for img in &page.images {
                            if self.selected_image_ids.contains(&img.id) {
                                let bottom = img.y + img.height;
                                if bottom > max_y {
                                    max_y = bottom;
                                }
                            }
                        }
                        for plot in &page.plots {
                            if self.selected_plot_ids.contains(&plot.id) {
                                let bottom = plot.y + plot.height;
                                if bottom > max_y {
                                    max_y = bottom;
                                }
                            }
                        }
                        for m in &page.matrices {
                            if self.selected_matrix_ids.contains(&m.id) {
                                let bottom = m.y + m.height;
                                if bottom > max_y {
                                    max_y = bottom;
                                }
                            }
                        }
                        for g in &page.grids {
                            if self.selected_grid_ids.contains(&g.id) {
                                let bottom = g.y + g.total_height();
                                if bottom > max_y {
                                    max_y = bottom;
                                }
                            }
                        }
                        if page.check_auto_expand(max_y) {
                            self.set_status("Page auto-expanded! ♾️");
                        }
                    } else if !self.lasso_points.is_empty() {
                        if self.lasso_points.len() >= 3 {
                            let mut selected_strokes = HashSet::new();
                            let mut selected_images = HashSet::new();
                            let mut selected_plots = HashSet::new();
                            let mut selected_matrices = HashSet::new();
                            let mut selected_grids = HashSet::new();
                            let page = self.notebook.current_page();
                            for stroke in &page.strokes {
                                if stroke.points.iter().any(|pt| point_in_polygon(Pos2::new(pt.x, pt.y), &self.lasso_points)) {
                                    selected_strokes.insert(stroke.id.clone());
                                }
                            }
                            for img in &page.images {
                                let center = Pos2::new(img.x + img.width * 0.5, img.y + img.height * 0.5);
                                let tl = Pos2::new(img.x, img.y);
                                let br = Pos2::new(img.x + img.width, img.y + img.height);
                                if point_in_polygon(center, &self.lasso_points)
                                    || point_in_polygon(tl, &self.lasso_points)
                                    || point_in_polygon(br, &self.lasso_points)
                                {
                                    selected_images.insert(img.id.clone());
                                }
                            }
                            for plot in &page.plots {
                                let center = Pos2::new(plot.x + plot.width * 0.5, plot.y + plot.height * 0.5);
                                let tl = Pos2::new(plot.x, plot.y);
                                let br = Pos2::new(plot.x + plot.width, plot.y + plot.height);
                                if point_in_polygon(center, &self.lasso_points)
                                    || point_in_polygon(tl, &self.lasso_points)
                                    || point_in_polygon(br, &self.lasso_points)
                                {
                                    selected_plots.insert(plot.id.clone());
                                }
                            }
                            for m in &page.matrices {
                                let center = Pos2::new(m.x + m.width * 0.5, m.y + m.height * 0.5);
                                let tl = Pos2::new(m.x, m.y);
                                let br = Pos2::new(m.x + m.width, m.y + m.height);
                                if point_in_polygon(center, &self.lasso_points)
                                    || point_in_polygon(tl, &self.lasso_points)
                                    || point_in_polygon(br, &self.lasso_points)
                                {
                                    selected_matrices.insert(m.id.clone());
                                }
                            }
                            for g in &page.grids {
                                let center = Pos2::new(g.x + g.total_width() * 0.5, g.y + g.total_height() * 0.5);
                                let tl = Pos2::new(g.x, g.y);
                                let br = Pos2::new(g.x + g.total_width(), g.y + g.total_height());
                                if point_in_polygon(center, &self.lasso_points)
                                    || point_in_polygon(tl, &self.lasso_points)
                                    || point_in_polygon(br, &self.lasso_points)
                                {
                                    selected_grids.insert(g.id.clone());
                                }
                            }
                            let count = selected_strokes.len() + selected_images.len() + selected_plots.len() + selected_matrices.len() + selected_grids.len();
                            self.selected_stroke_ids = selected_strokes;
                            self.selected_image_ids = selected_images;
                            self.selected_plot_ids = selected_plots;
                            self.selected_matrix_ids = selected_matrices;
                            self.selected_grid_ids = selected_grids;
                            if count > 0 {
                                self.set_status(&format!("{} item(s) selected (drag handles to resize, drag body to move)", count));
                            }
                        } else {
                            // Single click: hit test cards first, then nearby stroke
                            let click_pos = self.lasso_points[0];
                            if let Some((kind, id)) = self.hit_test_clickable_card(click_pos) {
                                self.clear_selection();
                                match kind {
                                    "plot" => {
                                        self.selected_plot_ids.insert(id);
                                        self.set_status("Selected 1 plot 📈 (drag handles to resize)");
                                    }
                                    "matrix" => {
                                        self.selected_matrix_ids.insert(id);
                                        self.set_status("Selected 1 matrix 🔲 (drag handles to resize)");
                                    }
                                    "grid" => {
                                        self.selected_grid_ids.insert(id);
                                        self.set_status("Selected 1 table grid ▦ (drag handles to resize)");
                                    }
                                    "image" => {
                                        self.selected_image_ids.insert(id);
                                        self.set_status("Selected 1 image 🖼 (drag handles to resize)");
                                    }
                                    _ => {}
                                }
                            } else {
                                let hit_dist = 14.0 / self.scale;
                                let hit_id = self
                                    .notebook
                                    .current_page()
                                    .strokes
                                    .iter()
                                    .rev()
                                    .find(|stroke| {
                                        stroke.points.iter().any(|pt| {
                                            let dx = pt.x - click_pos.x;
                                            let dy = pt.y - click_pos.y;
                                            dx * dx + dy * dy <= hit_dist * hit_dist
                                        })
                                    })
                                    .map(|s| s.id.clone());

                                if let Some(id) = hit_id {
                                    self.clear_selection();
                                    self.selected_stroke_ids.insert(id);
                                    self.set_status("Selected 1 stroke");
                                } else {
                                    self.clear_selection();
                                }
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
