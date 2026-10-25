use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::collections::{HashMap, HashSet};

use crate::math_renderer::MathRenderer;
use crate::model::{
    ImageEntity, Notebook, PaperType, PenType, Point, ShapeType, Stroke as NoteStroke, TextBox,
    ToolMode,
};
use crate::renderer::NoteRenderer;
use crate::ruler::{RulerKind, RulerState};
use crate::storage::NotebookStorage;
use crate::stroke_algorithm::StabilizerPipeline;
use crate::theme::{self, Palette};
use crate::ui_math_studio::MathStudioState;
use crate::vector_icon::VectorIcon;
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

const SHAPES: [(ShapeType, VectorIcon, &str); 15] = [
    (ShapeType::Line, VectorIcon::ShapeLine, "Line"),
    (ShapeType::Arrow, VectorIcon::ShapeArrow, "Arrow"),
    (ShapeType::DoubleArrow, VectorIcon::ShapeDoubleArrow, "Double arrow"),
    (ShapeType::Rectangle, VectorIcon::ShapeRectangle, "Rectangle"),
    (ShapeType::Circle, VectorIcon::ShapeCircle, "Circle"),
    (ShapeType::Triangle, VectorIcon::ShapeTriangle, "Triangle"),
    (ShapeType::Pentagon, VectorIcon::ShapePentagon, "Pentagon"),
    (ShapeType::Hexagon, VectorIcon::ShapeHexagon, "Hexagon"),
    (ShapeType::Star, VectorIcon::ShapeStar, "Star"),
    (ShapeType::GraphQuadrant1, VectorIcon::GraphQuadrant1, "1st Quadrant Axes (L-shape)"),
    (ShapeType::Graph2D, VectorIcon::Graph2D, "2D Cartesian Axes (4 Quadrants)"),
    (ShapeType::Graph3D, VectorIcon::Graph3D, "3D Coordinate Axes (X, Y, Z)"),
    (ShapeType::Cube3D, VectorIcon::ShapeCube3D, "3D Cube (with hidden edges)"),
    (ShapeType::Pyramid3D, VectorIcon::ShapePyramid3D, "3D Pyramid (with hidden edges)"),
    (ShapeType::Sphere3D, VectorIcon::ShapeSphere3D, "3D Sphere (with latitude/longitude)"),
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
        let p2 = Pos2::new(
            stroke.points.last().unwrap().x,
            stroke.points.last().unwrap().y,
        );
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

#[cfg(target_os = "windows")]
fn get_clipboard_files_windows() -> Vec<std::path::PathBuf> {
    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn GetClipboardData(uFormat: u32) -> *mut std::ffi::c_void;
        fn IsClipboardFormatAvailable(uFormat: u32) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn DragQueryFileW(
            hDrop: *mut std::ffi::c_void,
            iFile: u32,
            lpszFile: *mut u16,
            cch: u32,
        ) -> u32;
    }

    const CF_HDROP: u32 = 15;
    let mut paths = Vec::new();

    unsafe {
        if IsClipboardFormatAvailable(CF_HDROP) == 0 {
            return paths;
        }
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return paths;
        }
        let hdrop = GetClipboardData(CF_HDROP);
        if !hdrop.is_null() {
            let count = DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0);
            for i in 0..count {
                let len = DragQueryFileW(hdrop, i, std::ptr::null_mut(), 0);
                if len > 0 {
                    let mut buf: Vec<u16> = vec![0; (len + 1) as usize];
                    DragQueryFileW(hdrop, i, buf.as_mut_ptr(), len + 1);
                    if let Some(null_pos) = buf.iter().position(|&c| c == 0) {
                        buf.truncate(null_pos);
                    }
                    use std::os::windows::ffi::OsStringExt;
                    let os_str = std::ffi::OsString::from_wide(&buf);
                    paths.push(std::path::PathBuf::from(os_str));
                }
            }
        }
        CloseClipboard();
    }

    paths
}

#[cfg(not(target_os = "windows"))]
fn get_clipboard_files_windows() -> Vec<std::path::PathBuf> {
    Vec::new()
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
    custom_swatches: Vec<Color32>,

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
    selected_flowchart_ids: HashSet<String>,
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
    initial_flowchart_rects: Vec<(String, f32, f32, f32, f32)>,
    internal_clipboard: Option<crate::model::InkkClipboardBundle>,

    // Image textures cache
    image_textures: HashMap<String, egui::TextureHandle>,

    // MathStudio MATLAB-style IDE & Engine
    pub math_studio: MathStudioState,
    pub matrix_dialog: crate::ui_dialogs::MatrixDialogState,
    pub axes_dialog: crate::ui_dialogs::AxesDialogState,
    pub table_dialog: crate::ui_dialogs::TableDialogState,

    // Flowchart Studio & Interactive Builder
    pub flowchart_editor: crate::flowchart_ui::FlowchartEditorState,
    pub flowchart_state: crate::flowchart_ui::FlowchartUiState,


    // Splash screen & Photoshop-style startup cover (bia.png)
    pub splash_screen: crate::splash_screen::SplashScreenState,

    // Keyboard Shortcuts manager & interactive dialog
    pub shortcut_manager: crate::shortcuts::ShortcutManager,
    pub shortcuts_dialog: crate::shortcuts::ShortcutsDialogState,

    // Step-by-step interactive onboarding tour
    pub onboarding_tour: crate::onboarding_tour::OnboardingTourState,

    // Fullscreen view state
    pub is_fullscreen: bool,

    // Auto-Updater for market release updates
    pub updater: crate::updater::UpdaterState,

    // Notebook Library & Bookshelf Manager
    pub library: crate::library::LibraryState,

    // Autosave & Crash Recovery System
    pub autosave: crate::autosave::AutosaveManager,
    last_total_strokes: usize,

    // PDF Export modal state & quality presets (Low, Medium, High)
    pub show_pdf_export_dialog: bool,
    pub pdf_export_quality: crate::pdf_export::PdfQualityPreset,
    pub pdf_include_patterns: bool,
    pub pdf_force_light: bool,

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

    // Mouse auxiliary state
    secondary_drag_start: Option<Pos2>,
    secondary_drag_is_item: bool,
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
            custom_swatches: vec![
                Color32::from_rgb(163, 35, 141), // Plum Orchid (#A3238D)
                Color32::from_rgb(43, 108, 176), // Deep Azure
                Color32::from_rgb(217, 119, 6),  // Warm Ochre
                Color32::from_rgb(5, 150, 105),  // Emerald Green
                Color32::from_rgb(124, 58, 237), // Royal Violet
                Color32::from_rgb(220, 38, 38),  // Carmine Red
            ],

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
            selected_flowchart_ids: HashSet::new(),
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
            initial_flowchart_rects: Vec::new(),
            internal_clipboard: None,
            image_textures: HashMap::new(),
            math_studio: MathStudioState::default(),
            matrix_dialog: crate::ui_dialogs::MatrixDialogState::default(),
            axes_dialog: crate::ui_dialogs::AxesDialogState::default(),
            table_dialog: crate::ui_dialogs::TableDialogState::default(),
            flowchart_editor: crate::flowchart_ui::FlowchartEditorState::default(),
            flowchart_state: crate::flowchart_ui::FlowchartUiState::default(),
            splash_screen: crate::splash_screen::SplashScreenState::default(),
            shortcut_manager: crate::shortcuts::ShortcutManager::new(),
            shortcuts_dialog: crate::shortcuts::ShortcutsDialogState::default(),
            onboarding_tour: crate::onboarding_tour::OnboardingTourState::new(),
            is_fullscreen: false,
            updater: crate::updater::UpdaterState::new(),
            library: crate::library::LibraryState::default(),
            autosave: crate::autosave::AutosaveManager::new(),
            last_total_strokes: 0,

            show_pdf_export_dialog: false,
            pdf_export_quality: crate::pdf_export::PdfQualityPreset::High,
            pdf_include_patterns: true,
            pdf_force_light: true,

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
            secondary_drag_start: None,
            secondary_drag_is_item: false,
        }
    }
}

impl InkkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        // Check for unfinalized crash recovery session
        if let Some((recovered_nb, orig_path, meta)) = crate::autosave::AutosaveManager::check_recovery_available() {
            let time_str = meta.time_str.clone();
            let pages = meta.page_count;
            let strokes = meta.stroke_count;
            let title = meta.notebook_title.clone();
            app.notebook = recovered_nb;
            app.current_file_path = orig_path;
            app.last_total_strokes = strokes;
            app.autosave.recovery_restored_banner = Some(meta);
            app.set_status(&format!(
                "Restored uncommitted session '{}' ({} pages, {} strokes, saved at {})",
                title, pages, strokes, time_str
            ));
        }

        // Scan user documents & downloads for real notebooks immediately at startup
        app.library.scan(None);

        // Check for updates asynchronously in background on startup if configured
        if app.updater.config.check_on_startup {
            app.updater.check_for_updates(false);
        }

        theme::apply(&cc.egui_ctx, app.is_dark_mode);
        app
    }

    #[allow(dead_code)]
    pub fn mark_dirty(&mut self) {
        self.autosave.mark_dirty();
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
        self.selected_flowchart_ids.clear();
        self.active_resize_handle = None;
        self.initial_selection_rect = None;
        self.initial_stroke_points.clear();
        self.initial_image_rects.clear();
        self.initial_plot_rects.clear();
        self.initial_matrix_rects.clear();
        self.initial_grid_rects.clear();
        self.initial_flowchart_rects.clear();
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
        let pad = 8.0 / self.scale;
        if let Some(p) = page.plots.iter().rev().find(|p| {
            Rect::from_min_size(Pos2::new(p.x, p.y), Vec2::new(p.width, p.height))
                .expand(pad)
                .contains(world)
        }) {
            return Some(("plot", p.id.clone()));
        }
        if let Some(m) = page.matrices.iter().rev().find(|m| {
            Rect::from_min_size(Pos2::new(m.x, m.y), Vec2::new(m.width, m.height))
                .expand(pad)
                .contains(world)
        }) {
            return Some(("matrix", m.id.clone()));
        }
        if let Some(g) = page.grids.iter().rev().find(|g| {
            Rect::from_min_size(
                Pos2::new(g.x, g.y),
                Vec2::new(g.total_width(), g.total_height()),
            )
            .expand(pad)
            .contains(world)
        }) {
            return Some(("grid", g.id.clone()));
        }
        if let Some(f) = page.flowcharts.iter().rev().find(|f| {
            Rect::from_min_size(Pos2::new(f.x, f.y), Vec2::new(f.width, f.height))
                .expand(pad)
                .contains(world)
        }) {
            return Some(("flowchart", f.id.clone()));
        }
        if let Some(i) = page.images.iter().rev().find(|i| {
            Rect::from_min_size(Pos2::new(i.x, i.y), Vec2::new(i.width, i.height))
                .expand(pad)
                .contains(world)
        }) {
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
            && self.selected_flowchart_ids.is_empty()
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
                    if pt.x < min_x {
                        min_x = pt.x;
                    }
                    if pt.x > max_x {
                        max_x = pt.x;
                    }
                    if pt.y < min_y {
                        min_y = pt.y;
                    }
                    if pt.y > max_y {
                        max_y = pt.y;
                    }
                }
            }
        }

        for img in &page.images {
            if self.selected_image_ids.contains(&img.id) {
                found = true;
                if img.x < min_x {
                    min_x = img.x;
                }
                if img.y < min_y {
                    min_y = img.y;
                }
                let right = img.x + img.width;
                let bottom = img.y + img.height;
                if right > max_x {
                    max_x = right;
                }
                if bottom > max_y {
                    max_y = bottom;
                }
            }
        }

        for plot in &page.plots {
            if self.selected_plot_ids.contains(&plot.id) {
                found = true;
                if plot.x < min_x {
                    min_x = plot.x;
                }
                if plot.y < min_y {
                    min_y = plot.y;
                }
                let right = plot.x + plot.width;
                let bottom = plot.y + plot.height;
                if right > max_x {
                    max_x = right;
                }
                if bottom > max_y {
                    max_y = bottom;
                }
            }
        }

        for matrix in &page.matrices {
            if self.selected_matrix_ids.contains(&matrix.id) {
                found = true;
                if matrix.x < min_x {
                    min_x = matrix.x;
                }
                if matrix.y < min_y {
                    min_y = matrix.y;
                }
                let right = matrix.x + matrix.width;
                let bottom = matrix.y + matrix.height;
                if right > max_x {
                    max_x = right;
                }
                if bottom > max_y {
                    max_y = bottom;
                }
            }
        }

        for grid in &page.grids {
            if self.selected_grid_ids.contains(&grid.id) {
                found = true;
                if grid.x < min_x {
                    min_x = grid.x;
                }
                if grid.y < min_y {
                    min_y = grid.y;
                }
                let right = grid.x + grid.total_width();
                let bottom = grid.y + grid.total_height();
                if right > max_x {
                    max_x = right;
                }
                if bottom > max_y {
                    max_y = bottom;
                }
            }
        }

        for flow in &page.flowcharts {
            if self.selected_flowchart_ids.contains(&flow.id) {
                found = true;
                if flow.x < min_x {
                    min_x = flow.x;
                }
                if flow.y < min_y {
                    min_y = flow.y;
                }
                let right = flow.x + flow.width;
                let bottom = flow.y + flow.height;
                if right > max_x {
                    max_x = right;
                }
                if bottom > max_y {
                    max_y = bottom;
                }
            }
        }

        if found {
            let min_x = min_x.min(max_x);
            let max_x = max_x.max(min_x + 2.0);
            let min_y = min_y.min(max_y);
            let max_y = max_y.max(min_y + 2.0);
            Some(Rect::from_min_max(
                Pos2::new(min_x, min_y),
                Pos2::new(max_x, max_y),
            ))
        } else {
            None
        }
    }

    fn copy_selected(&mut self) -> bool {
        let has_sel = !self.selected_stroke_ids.is_empty()
            || !self.selected_image_ids.is_empty()
            || !self.selected_plot_ids.is_empty()
            || !self.selected_matrix_ids.is_empty()
            || !self.selected_grid_ids.is_empty()
            || !self.selected_flowchart_ids.is_empty();

        let page = self.notebook.current_page();

        let (strokes, images, plots, matrices, grids, flowcharts) = if has_sel {
            (
                page.strokes.iter().filter(|s| self.selected_stroke_ids.contains(&s.id)).cloned().collect::<Vec<_>>(),
                page.images.iter().filter(|i| self.selected_image_ids.contains(&i.id)).cloned().collect::<Vec<_>>(),
                page.plots.iter().filter(|p| self.selected_plot_ids.contains(&p.id)).cloned().collect::<Vec<_>>(),
                page.matrices.iter().filter(|m| self.selected_matrix_ids.contains(&m.id)).cloned().collect::<Vec<_>>(),
                page.grids.iter().filter(|g| self.selected_grid_ids.contains(&g.id)).cloned().collect::<Vec<_>>(),
                page.flowcharts.iter().filter(|f| self.selected_flowchart_ids.contains(&f.id)).cloned().collect::<Vec<_>>(),
            )
        } else {
            if page.strokes.is_empty()
                && page.images.is_empty()
                && page.plots.is_empty()
                && page.matrices.is_empty()
                && page.grids.is_empty()
                && page.flowcharts.is_empty()
            {
                self.set_status("Nothing to copy. Draw strokes or use Lasso (7) to select.");
                return false;
            }
            (
                page.strokes.clone(),
                page.images.clone(),
                page.plots.clone(),
                page.matrices.clone(),
                page.grids.clone(),
                page.flowcharts.clone(),
            )
        };

        let stroke_count = strokes.len();
        let total_count = stroke_count + images.len() + plots.len() + matrices.len() + grids.len() + flowcharts.len();
        if total_count == 0 {
            self.set_status("Nothing to copy. Select strokes with Lasso (7) first.");
            return false;
        }

        let bundle = crate::model::InkkClipboardBundle {
            inkk_signature: "inkk_clipboard_v1".to_string(),
            strokes,
            images,
            plots,
            matrices,
            grids,
            flowcharts,
        };

        if let Ok(json_str) = serde_json::to_string(&bundle) {
            if let Ok(mut clip) = arboard::Clipboard::new() {
                let _ = clip.set_text(json_str);
            }
        }

        self.internal_clipboard = Some(bundle);
        self.set_status(&format!("Copied {} item(s) ({} stroke(s)) to clipboard.", total_count, stroke_count));
        true
    }

    fn cut_selected(&mut self) {
        if self.copy_selected() {
            self.delete_selected();
            self.set_status("Cut selected item(s) to clipboard.");
        }
    }

    fn duplicate_selected(&mut self) {
        if self.copy_selected() {
            self.paste_from_clipboard();
        }
    }

    fn select_all(&mut self) {
        let (stroke_ids, img_ids, plot_ids, mat_ids, grid_ids, flow_ids) = {
            let page = self.notebook.current_page();
            (
                page.strokes.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
                page.images.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
                page.plots.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
                page.matrices.iter().map(|m| m.id.clone()).collect::<Vec<_>>(),
                page.grids.iter().map(|g| g.id.clone()).collect::<Vec<_>>(),
                page.flowcharts.iter().map(|f| f.id.clone()).collect::<Vec<_>>(),
            )
        };
        let stroke_count = stroke_ids.len();
        let total_count = stroke_count
            + img_ids.len()
            + plot_ids.len()
            + mat_ids.len()
            + grid_ids.len()
            + flow_ids.len();
        if total_count == 0 {
            self.set_status("Page is empty - nothing to select.");
            return;
        }
        self.clear_selection();
        self.selected_stroke_ids.extend(stroke_ids);
        self.selected_image_ids.extend(img_ids);
        self.selected_plot_ids.extend(plot_ids);
        self.selected_matrix_ids.extend(mat_ids);
        self.selected_grid_ids.extend(grid_ids);
        self.selected_flowchart_ids.extend(flow_ids);
        self.set_tool(ToolMode::LassoSelect);
        self.set_status(&format!("Selected all {} items ({} strokes) on page.", total_count, stroke_count));
    }

    fn paste_bundle(&mut self, bundle: crate::model::InkkClipboardBundle) {
        let total_count = bundle.strokes.len()
            + bundle.images.len()
            + bundle.plots.len()
            + bundle.matrices.len()
            + bundle.grids.len()
            + bundle.flowcharts.len();
        if total_count == 0 {
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for s in &bundle.strokes {
            for pt in &s.points {
                min_x = min_x.min(pt.x);
                min_y = min_y.min(pt.y);
                max_x = max_x.max(pt.x);
                max_y = max_y.max(pt.y);
            }
        }
        for i in &bundle.images {
            min_x = min_x.min(i.x);
            min_y = min_y.min(i.y);
            max_x = max_x.max(i.x + i.width);
            max_y = max_y.max(i.y + i.height);
        }
        for p in &bundle.plots {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x + p.width);
            max_y = max_y.max(p.y + p.height);
        }
        for m in &bundle.matrices {
            min_x = min_x.min(m.x);
            min_y = min_y.min(m.y);
            max_x = max_x.max(m.x + m.width);
            max_y = max_y.max(m.y + m.height);
        }
        for g in &bundle.grids {
            min_x = min_x.min(g.x);
            min_y = min_y.min(g.y);
            let grid_w = g.cols as f32 * g.col_width;
            let grid_h = g.rows as f32 * g.row_height;
            max_x = max_x.max(g.x + grid_w);
            max_y = max_y.max(g.y + grid_h);
        }
        for f in &bundle.flowcharts {
            min_x = min_x.min(f.x);
            min_y = min_y.min(f.y);
            max_x = max_x.max(f.x + f.width);
            max_y = max_y.max(f.y + f.height);
        }

        let bundle_center = if min_x < max_x && min_y < max_y {
            Pos2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
        } else {
            Pos2::new(100.0, 100.0)
        };

        let (page_w, page_h) = {
            let page = self.notebook.current_page();
            (page.width, page.height)
        };
        let target_pos = if self.last_pointer_pos_world != Pos2::ZERO
            && self.last_pointer_pos_world.x >= 0.0
            && self.last_pointer_pos_world.x <= page_w
            && self.last_pointer_pos_world.y >= 0.0
            && self.last_pointer_pos_world.y <= page_h
        {
            self.last_pointer_pos_world
        } else {
            bundle_center + Vec2::new(30.0, 30.0)
        };

        let offset = target_pos - bundle_center;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        self.clear_selection();

        let mut pasted_strokes = 0;
        {
            let page = self.notebook.current_page_mut();
            for (idx, mut s) in bundle.strokes.into_iter().enumerate() {
                s.id = format!("stroke_{}_{}", ts, idx);
                for pt in &mut s.points {
                    pt.x += offset.x;
                    pt.y += offset.y;
                }
                let id = s.id.clone();
                page.strokes.push(s);
                self.selected_stroke_ids.insert(id);
                pasted_strokes += 1;
            }

            for (idx, mut img) in bundle.images.into_iter().enumerate() {
                img.id = format!("img_{}_{}", ts, idx);
                img.x += offset.x;
                img.y += offset.y;
                let id = img.id.clone();
                page.images.push(img);
                self.selected_image_ids.insert(id);
            }

            for (idx, mut p) in bundle.plots.into_iter().enumerate() {
                p.id = format!("plot_{}_{}", ts, idx);
                p.x += offset.x;
                p.y += offset.y;
                let id = p.id.clone();
                page.plots.push(p);
                self.selected_plot_ids.insert(id);
            }

            for (idx, mut m) in bundle.matrices.into_iter().enumerate() {
                m.id = format!("mat_{}_{}", ts, idx);
                m.x += offset.x;
                m.y += offset.y;
                let id = m.id.clone();
                page.matrices.push(m);
                self.selected_matrix_ids.insert(id);
            }

            for (idx, mut g) in bundle.grids.into_iter().enumerate() {
                g.id = format!("grid_{}_{}", ts, idx);
                g.x += offset.x;
                g.y += offset.y;
                let id = g.id.clone();
                page.grids.push(g);
                self.selected_grid_ids.insert(id);
            }

            for (idx, mut f) in bundle.flowcharts.into_iter().enumerate() {
                f.id = format!("flow_{}_{}", ts, idx);
                f.x += offset.x;
                f.y += offset.y;
                let id = f.id.clone();
                page.flowcharts.push(f);
                self.selected_flowchart_ids.insert(id);
            }
        }

        if let Some(bounds) = self.get_selection_bounds() {
            self.notebook.current_page_mut().check_auto_expand(bounds.max.y + 100.0);
        }

        self.set_tool(ToolMode::LassoSelect);
        self.autosave.mark_dirty();
        self.set_status(&format!(
            "Pasted {} item(s) ({} stroke(s)) onto canvas.",
            total_count, pasted_strokes
        ));
    }

    fn center_page_in_view(&mut self, available_size: Vec2) {
        let page = self.notebook.current_page();
        let target_x = ((available_size.x - page.width * self.scale) * 0.5).max(40.0);
        let target_y = ((available_size.y - page.height * self.scale) * 0.5).max(40.0);
        self.pan = Vec2::new(target_x, target_y);
        self.set_status("Canvas centered on page.");
    }

    fn delete_selected(&mut self) {
        if self.selected_stroke_ids.is_empty()
            && self.selected_image_ids.is_empty()
            && self.selected_plot_ids.is_empty()
            && self.selected_matrix_ids.is_empty()
            && self.selected_grid_ids.is_empty()
            && self.selected_flowchart_ids.is_empty()
        {
            return;
        }
        let stroke_count = self.selected_stroke_ids.len();
        let image_count = self.selected_image_ids.len();
        let math_count = self.selected_plot_ids.len()
            + self.selected_matrix_ids.len()
            + self.selected_grid_ids.len();
        let flow_count = self.selected_flowchart_ids.len();
        let page = self.notebook.current_page_mut();
        page.strokes
            .retain(|s| !self.selected_stroke_ids.contains(&s.id));
        page.images
            .retain(|i| !self.selected_image_ids.contains(&i.id));
        page.plots
            .retain(|p| !self.selected_plot_ids.contains(&p.id));
        page.matrices
            .retain(|m| !self.selected_matrix_ids.contains(&m.id));
        page.grids
            .retain(|g| !self.selected_grid_ids.contains(&g.id));
        page.flowcharts
            .retain(|f| !self.selected_flowchart_ids.contains(&f.id));
        self.clear_selection();
        self.set_status(&format!(
            "Deleted {} stroke(s), {} image(s), {} math/flowchart item(s)",
            stroke_count,
            image_count,
            math_count + flow_count
        ));
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
            self.initial_flowchart_rects = page
                .flowcharts
                .iter()
                .filter(|f| self.selected_flowchart_ids.contains(&f.id))
                .map(|f| (f.id.clone(), f.x, f.y, f.width, f.height))
                .collect();
        }
    }

    fn apply_selection_delta(&mut self, delta: Vec2) {
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
        for flow in &mut page.flowcharts {
            if self.selected_flowchart_ids.contains(&flow.id) {
                flow.x += delta.x;
                flow.y += delta.y;
            }
        }
    }

    fn finish_selection_drag(&mut self) {
        self.is_dragging_selection = false;
        self.active_resize_handle = None;
        self.initial_selection_rect = None;
        self.initial_stroke_points.clear();
        self.initial_image_rects.clear();
        self.initial_plot_rects.clear();
        self.initial_matrix_rects.clear();
        self.initial_grid_rects.clear();
        self.initial_flowchart_rects.clear();

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
            self.set_status("Page auto-expanded!");
        }
    }

    fn get_svg_font_db() -> std::sync::Arc<resvg::usvg::fontdb::Database> {
        static FONT_DB: std::sync::OnceLock<std::sync::Arc<resvg::usvg::fontdb::Database>> =
            std::sync::OnceLock::new();
        FONT_DB
            .get_or_init(|| {
                let mut db = resvg::usvg::fontdb::Database::new();
                db.load_system_fonts();
                std::sync::Arc::new(db)
            })
            .clone()
    }

    fn decode_svg_data(bytes: &[u8]) -> Option<(Vec<u8>, f32, f32)> {
        let opt = resvg::usvg::Options {
            fontdb: Self::get_svg_font_db(),
            ..Default::default()
        };
        let tree = resvg::usvg::Tree::from_data(bytes, &opt).ok()?;
        let size = tree.size();
        let width = size.width();
        let height = size.height();
        if width <= 0.0 || height <= 0.0 {
            return None;
        }

        let max_render_dim = 1200.0f32;
        let scale = if width > max_render_dim || height > max_render_dim {
            (max_render_dim / width).min(max_render_dim / height)
        } else if width < 100.0 || height < 100.0 {
            (400.0 / width).min(400.0 / height).max(1.0)
        } else {
            1.0
        };

        let render_w = ((width * scale).round() as u32).max(1);
        let render_h = ((height * scale).round() as u32).max(1);
        let mut pixmap = resvg::tiny_skia::Pixmap::new(render_w, render_h)?;
        let transform = resvg::tiny_skia::Transform::from_scale(
            render_w as f32 / width,
            render_h as f32 / height,
        );
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        let png_bytes = pixmap.encode_png().ok()?;
        Some((png_bytes, width, height))
    }

    fn decode_image_or_svg(bytes: &[u8]) -> Option<(Vec<u8>, f32, f32)> {
        // 1. Try standard raster image decoding with image crate (png, jpeg, webp, bmp, gif, etc.)
        if let Ok(dyn_img) = image::load_from_memory(bytes) {
            let w = dyn_img.width() as f32;
            let h = dyn_img.height() as f32;
            return Some((bytes.to_vec(), w, h));
        }

        // 2. Try SVG vector rendering via resvg
        if let Some(res) = Self::decode_svg_data(bytes) {
            return Some(res);
        }

        None
    }

    fn prompt_insert_image(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "All Images & Vectors",
                &[
                    "png", "jpg", "jpeg", "webp", "bmp", "svg", "gif", "ico", "tiff",
                ],
            )
            .add_filter("Vector Graphics (*.svg)", &["svg"])
            .add_filter(
                "Raster Images",
                &["png", "jpg", "jpeg", "webp", "bmp", "gif", "ico", "tiff"],
            )
            .pick_file()
        {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(&bytes) {
                    self.insert_image_bytes(img_bytes, w, h);
                } else {
                    self.set_error("Could not decode selected image or SVG file.");
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

        // Position image in the center of the user's visible viewport or near last cursor
        let (place_x, place_y) = if self.last_pointer_pos_world != Pos2::ZERO
            && self.last_pointer_pos_world.x >= 0.0
            && self.last_pointer_pos_world.x <= page.width
            && self.last_pointer_pos_world.y >= 0.0
            && self.last_pointer_pos_world.y <= page.height
        {
            let px = (self.last_pointer_pos_world.x - w * 0.5).clamp(20.0, (page.width - w).max(20.0));
            let py = (self.last_pointer_pos_world.y - h * 0.5).clamp(20.0, (page.height - h).max(20.0));
            (px, py)
        } else {
            let view_center_world_x = (-self.pan.x + 450.0) / self.scale;
            let view_center_world_y = (-self.pan.y + 350.0) / self.scale;
            let px = (view_center_world_x - w * 0.5).clamp(20.0, (page.width - w).max(20.0));
            let py = (view_center_world_y - h * 0.5).clamp(20.0, (page.height - h).max(20.0));
            (px, py)
        };

        let _ = page.check_auto_expand(place_y + h + 80.0);

        let new_img = ImageEntity::new(place_x, place_y, w, h, data);
        let new_id = new_img.id.clone();
        page.images.push(new_img);

        self.clear_selection();
        self.selected_image_ids.insert(new_id);
        self.set_tool(ToolMode::LassoSelect);
        self.autosave.mark_dirty();
        self.set_status("Image inserted & selected! (Drag handles to resize)");
    }

    fn paste_from_clipboard(&mut self) {
        // 0. Check for Inkk stroke/bundle clipboard (from system clipboard or internal buffer)
        let mut bundle_from_clip = None;
        if let Ok(mut clip) = arboard::Clipboard::new() {
            if let Ok(text) = clip.get_text() {
                if text.contains("\"inkk_signature\":\"inkk_clipboard_v1\"")
                    || text.contains("\"inkk_signature\": \"inkk_clipboard_v1\"")
                {
                    if let Ok(bundle) = serde_json::from_str::<crate::model::InkkClipboardBundle>(&text) {
                        bundle_from_clip = Some(bundle);
                    }
                }
            }
        }

        if let Some(bundle) = bundle_from_clip.or_else(|| self.internal_clipboard.clone()) {
            self.paste_bundle(bundle);
            return;
        }

        // 1. Check Windows Explorer copied files (CF_HDROP)
        let explorer_files = get_clipboard_files_windows();
        if !explorer_files.is_empty() {
            let mut pasted_count = 0;
            for file_path in explorer_files {
                if file_path.is_file() {
                    if let Ok(bytes) = std::fs::read(&file_path) {
                        if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(&bytes) {
                            self.insert_image_bytes(img_bytes, w, h);
                            pasted_count += 1;
                        }
                    }
                }
            }
            if pasted_count > 0 {
                self.set_status(&format!("Pasted {} image(s) from copied file(s)!", pasted_count));
                return;
            }
        }

        // 2. Try raw image data from clipboard with retry (Snipping tool, copy image from web/paint)
        let mut clip = None;
        for _ in 0..5 {
            if let Ok(c) = arboard::Clipboard::new() {
                clip = Some(c);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let mut clip = match clip {
            Some(c) => c,
            None => {
                self.set_error("Could not access system clipboard (locked by another application).");
                return;
            }
        };

        if let Ok(img_data) = clip.get_image() {
            let width = img_data.width as u32;
            let height = img_data.height as u32;
            if width > 0 && height > 0 {
                if let Some(rgba_img) =
                    image::RgbaImage::from_raw(width, height, img_data.bytes.into_owned())
                {
                    let mut png_bytes = Vec::new();
                    let mut cursor = std::io::Cursor::new(&mut png_bytes);
                    if rgba_img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                        self.insert_image_bytes(png_bytes, width as f32, height as f32);
                        self.set_status("Image pasted from clipboard!");
                        return;
                    }
                }
            }
        }

        // 3. Try text / file paths / SVG markup from clipboard
        if let Ok(text) = clip.get_text() {
            let trimmed = text.trim();
            // Check if it's SVG XML text
            if (trimmed.starts_with("<svg") || trimmed.starts_with("<?xml"))
                && trimmed.contains("</svg>")
            {
                if let Some((png_bytes, w, h)) = Self::decode_svg_data(trimmed.as_bytes()) {
                    self.insert_image_bytes(png_bytes, w, h);
                    self.set_status("SVG Vector pasted & rendered!");
                    return;
                }
            }

            // Check if it's a file path or list of paths
            for line in trimmed.lines() {
                let p_str = line.trim().trim_matches('"').trim_matches('\'');
                let path = std::path::Path::new(p_str);
                if path.is_file() {
                    if let Ok(bytes) = std::fs::read(path) {
                        if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(&bytes) {
                            self.insert_image_bytes(img_bytes, w, h);
                            self.set_status("Image from file path pasted!");
                            return;
                        }
                    }
                }
            }
        }

        self.set_error(
            "Clipboard has no image or supported file (Copy an image or image file and press Ctrl+V).",
        );
    }

    /// Main toolbar: brand, history, tools, colors, size, theme.
    fn render_top_toolbar(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let p = self.palette();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = theme::SPACE_XS;

            // Brand Wordmark in Georgia Serif (restrained, no distracting sticker)
            ui.label(
                egui::RichText::new("Inkk")
                    .font(theme::font_serif(16.5))
                    .strong()
                    .color(p.text),
            );
            widgets::divider(ui, &p);

            // History: Always reserved slots, dimmed when disabled (zero layout jump)
            let can_undo = !self.notebook.current_page().strokes.is_empty();
            let can_redo = !self.redo_stack.is_empty();
            if ui
                .add_enabled_ui(can_undo, |ui| {
                    widgets::tool_button(ui, &p, VectorIcon::Undo, false, "Undo (Ctrl+Z)")
                })
                .inner
                .clicked()
            {
                self.undo();
            }
            if ui
                .add_enabled_ui(can_redo, |ui| {
                    widgets::tool_button(ui, &p, VectorIcon::Redo, false, "Redo (Ctrl+Y)")
                })
                .inner
                .clicked()
            {
                self.redo();
            }
            widgets::divider(ui, &p);

            // Document Actions: Library, Save, Pages, Open, Export to PDF
            let docs_resp = ui.scope(|ui| {
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Library,
                    self.library.is_open,
                    "Notebook Library & Bookshelf (Ctrl+L)",
                )
                .clicked()
                {
                    if self.library.is_open {
                        self.library.is_open = false;
                    } else {
                        self.library.open(self.current_file_path.as_deref());
                    }
                }
                if widgets::tool_button(ui, &p, VectorIcon::Save, false, "Save (Ctrl+S)").clicked() {
                    self.prompt_save_file();
                }
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Pages,
                    self.show_previews_drawer,
                    "Show page previews",
                )
                .clicked()
                {
                    self.show_previews_drawer = !self.show_previews_drawer;
                }
                if widgets::tool_button(ui, &p, VectorIcon::Open, false, "Open (Ctrl+O)").clicked() {
                    self.prompt_open_file();
                }
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Pdf,
                    false,
                    "Export Notebook to PDF (Ctrl+Shift+E)",
                )
                .clicked()
                {
                    self.prompt_export_pdf();
                }
            });
            self.onboarding_tour.rect_docs = docs_resp.response.rect;
            widgets::divider(ui, &p);

            let tools_resp = ui.scope(|ui| {
                let shape_vicon = self.current_shape.vector_icon();
                let tools: [(ToolMode, widgets::ToolIcon<'static>, &str); 8] = [
                    (ToolMode::Pen, VectorIcon::Pen.into(), "Pen (1)"),
                    (
                        ToolMode::Highlighter,
                        VectorIcon::Highlighter.into(),
                        "Highlighter (2)",
                    ),
                    (
                        ToolMode::Shape(self.current_shape),
                        shape_vicon.into(),
                        "Shapes (3)",
                    ),
                    (ToolMode::Text, "T".into(), "Text Box (4 / T)"),
                    (
                        ToolMode::EraserStroke,
                        VectorIcon::EraserStroke.into(),
                        "Stroke eraser (5)",
                    ),
                    (
                        ToolMode::EraserPoint,
                        VectorIcon::EraserPoint.into(),
                        "Point eraser (6)",
                    ),
                    (
                        ToolMode::LassoSelect,
                        VectorIcon::Lasso.into(),
                        "Lasso select (7)",
                    ),
                    (ToolMode::Pan, VectorIcon::Pan.into(), "Pan (8)"),
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
            });
            self.onboarding_tour.rect_tools = tools_resp.response.rect;
            widgets::divider(ui, &p);

            // Stencil Tools: Straight Ruler & Circular Protractor
            let stencils_resp = ui.scope(|ui| {
                let ruler_active = self.ruler.kind == RulerKind::Ruler;
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Ruler,
                    ruler_active,
                    "Straight Ruler (R)",
                )
                .clicked()
                {
                    self.ruler.toggle_ruler();
                    let is_on = self.ruler.kind == RulerKind::Ruler;
                    self.set_status(if is_on {
                        "Straight Ruler ON (drag body to move, dial to rotate)"
                    } else {
                        "Ruler OFF"
                    });
                }
                let protractor_active = self.ruler.kind == RulerKind::Protractor;
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Protractor,
                    protractor_active,
                    "Circular Protractor (Shift+R)",
                )
                .clicked()
                {
                    self.ruler.toggle_protractor();
                    let is_on = self.ruler.kind == RulerKind::Protractor;
                    self.set_status(if is_on {
                        "Protractor ON (drag body to move, dial to rotate)"
                    } else {
                        "Protractor OFF"
                    });
                }
            });
            self.onboarding_tour.rect_stencils = stencils_resp.response.rect;
            widgets::divider(ui, &p);

            // Picture Tool: Insert Image from File
            if widgets::tool_button(
                ui,
                &p,
                VectorIcon::Picture,
                false,
                "Insert Image from file (or Ctrl+V to paste)",
            )
            .clicked()
            {
                self.prompt_insert_image();
            }
            widgets::divider(ui, &p);

            // Math Studio & Mathematical Entities Tools
            let math_resp = ui.scope(|ui| {
                let math_active = self.math_studio.is_open;
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::MathStudio,
                    math_active,
                    "Math Studio & MATLAB IDE (M / Ctrl+M)",
                )
                .clicked()
                {
                    self.math_studio.toggle();
                    let is_on = self.math_studio.is_open;
                    self.set_status(if is_on {
                        "Math Studio OPEN [MATLAB Engine Ready]"
                    } else {
                        "Math Studio CLOSED"
                    });
                }
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Matrix,
                    self.matrix_dialog.is_open,
                    "Insert Matrix (MATLAB code or UI)",
                )
                .clicked()
                {
                    self.matrix_dialog.open_new();
                }
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Axes,
                    self.axes_dialog.is_open,
                    "Insert Coordinate Axes (1D / 2D / 3D)",
                )
                .clicked()
                {
                    self.axes_dialog.open_new();
                }
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Table,
                    self.table_dialog.is_open,
                    "Insert Data Table (UI / CSV)",
                )
                .clicked()
                {
                    self.table_dialog.open_new();
                }
                let flowchart_active = self.flowchart_state.is_panel_open || self.flowchart_editor.is_open;
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Flowchart,
                    flowchart_active,
                    "Flowchart Shapes Menu & Builder (Ctrl+F)",
                )
                .clicked()
                {
                    self.flowchart_state.is_panel_open = !self.flowchart_state.is_panel_open;
                    let is_on = self.flowchart_state.is_panel_open;
                    self.set_status(if is_on {
                        "Flowchart Shapes Menu OPEN"
                    } else {
                        "Flowchart Shapes Menu CLOSED"
                    });
                }
            });
            self.onboarding_tour.rect_math = math_resp.response.rect;
            widgets::divider(ui, &p);

            // Bespoke Ink Palette Popover (Archival Inks, Custom Swatches, 2D Spectrum & Density)
            let old_color = self.color;
            widgets::ink_palette_button(
                ui,
                &p,
                &mut self.color,
                &mut self.custom_swatches,
                &mut self.opacity,
                self.is_dark_mode,
            );
            if self.color != old_color {
                if let Some(ref sel_id) = self.editing_text_id {
                    let c_arr = [
                        self.color.r(),
                        self.color.g(),
                        self.color.b(),
                        self.color.a(),
                    ];
                    for tb in &mut self.notebook.current_page_mut().text_boxes {
                        if tb.id == *sel_id {
                            tb.color = c_arr;
                        }
                    }
                } else if matches!(
                    self.tool,
                    ToolMode::EraserStroke | ToolMode::EraserPoint | ToolMode::Pan
                ) {
                    self.set_tool(ToolMode::Pen);
                }
            }

            // Live size & dilution preview dot next to the slider
            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
            let alpha_u8 = ((self.opacity).clamp(0.1, 1.0) * 255.0) as u8;
            let preview_color = Color32::from_rgba_unmultiplied(
                self.color.r(),
                self.color.g(),
                self.color.b(),
                alpha_u8,
            );
            ui.painter().circle_filled(
                dot_rect.center(),
                (self.size * 0.45).clamp(1.5, 9.5),
                preview_color,
            );
            ui.painter()
                .circle_stroke(dot_rect.center(), 10.5, Stroke::new(1.0_f32, p.border));
            ui.scope(|ui| {
                ui.spacing_mut().slider_width = 55.0;
                ui.add(
                    egui::Slider::new(&mut self.size, 1.0..=30.0)
                        .suffix(" pt")
                        .show_value(true),
                )
                .on_hover_text("Stroke thickness ( [ / ] )");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (icon, tip) = if self.is_dark_mode {
                    (VectorIcon::Sun, "Switch to light theme")
                } else {
                    (VectorIcon::Moon, "Switch to dark theme")
                };
                if widgets::tool_button(ui, &p, icon, false, tip).clicked() {
                    self.toggle_theme(ctx);
                }
                if widgets::tool_button(
                    ui,
                    &p,
                    VectorIcon::Autosave,
                    self.autosave.show_settings_dialog,
                    "Autosave & Crash Recovery Settings",
                )
                .clicked()
                {
                    self.autosave.show_settings_dialog = !self.autosave.show_settings_dialog;
                }
            });
        });
    }
    /// Second toolbar row with options for the active tool and the page.
    fn render_context_bar(&mut self, ui: &mut Ui) {
        let p = self.palette();
        ui.horizontal(|ui| {
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
                    widgets::caption(ui, &p, "Lines");
                    for (shape, icon, name) in &SHAPES[0..3] {
                        if widgets::tool_button(ui, &p, *icon, self.current_shape == *shape, *name)
                            .clicked()
                        {
                            self.current_shape = *shape;
                            self.tool = ToolMode::Shape(*shape);
                        }
                    }
                    widgets::divider(ui, &p);

                    widgets::caption(ui, &p, "Basic");
                    for (shape, icon, name) in &SHAPES[3..9] {
                        if widgets::tool_button(ui, &p, *icon, self.current_shape == *shape, *name)
                            .clicked()
                        {
                            self.current_shape = *shape;
                            self.tool = ToolMode::Shape(*shape);
                        }
                    }
                    widgets::divider(ui, &p);

                    widgets::caption(ui, &p, "Graphs");
                    for (shape, icon, name) in &SHAPES[9..12] {
                        if widgets::tool_button(ui, &p, *icon, self.current_shape == *shape, *name)
                            .clicked()
                        {
                            self.current_shape = *shape;
                            self.tool = ToolMode::Shape(*shape);
                        }
                    }
                    widgets::divider(ui, &p);

                    widgets::caption(ui, &p, "3D Solids");
                    for (shape, icon, name) in &SHAPES[12..15] {
                        if widgets::tool_button(ui, &p, *icon, self.current_shape == *shape, *name)
                            .clicked()
                        {
                            self.current_shape = *shape;
                            self.tool = ToolMode::Shape(*shape);
                        }
                    }
                    widgets::divider(ui, &p);

                    if widgets::chip(ui, &p, "Filled", self.is_shape_filled).clicked() {
                        self.is_shape_filled = !self.is_shape_filled;
                    }

                }
                ToolMode::Highlighter => {
                    widgets::caption(
                        ui,
                        &p,
                        "Highlighter draws beneath ink so text stays readable",
                    );
                }
                ToolMode::EraserStroke => {
                    widgets::caption(
                        ui,
                        &p,
                        "Removes whole strokes it touches - size slider sets radius",
                    );
                }
                ToolMode::EraserPoint => {
                    widgets::caption(
                        ui,
                        &p,
                        "Cuts strokes where it passes - size slider sets radius",
                    );
                }
                ToolMode::LassoSelect => {
                    widgets::caption(
                        ui,
                        &p,
                        "Draw around strokes or click one - drag to move - Del to delete",
                    );
                }
                ToolMode::Pan => {
                    widgets::caption(ui, &p, "Drag to move the page - Ctrl + wheel to zoom");
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
                        if widgets::tool_button(ui, &p, VectorIcon::Delete, false, "Delete Text Box").clicked()
                        {
                            if let Some(ref id) = self.editing_text_id.take() {
                                self.notebook
                                    .current_page_mut()
                                    .text_boxes
                                    .retain(|tb| tb.id != *id);
                                self.set_status("Deleted text box");
                            }
                        }
                        if widgets::tool_button(ui, &p, VectorIcon::Close, false, "Done Editing").clicked() {
                            self.editing_text_id = None;
                        }
                    } else {
                        widgets::caption(
                            ui,
                            &p,
                            "Click canvas to type - Drag top strip to move - right edge to resize",
                        );
                    }
                }
            }

            if matches!(
                self.tool,
                ToolMode::Pen | ToolMode::Highlighter | ToolMode::Shape(_)
            ) {
                widgets::divider(ui, &p);
                widgets::caption(ui, &p, "Opacity");
                widgets::smart_opacity_control(ui, &p, &mut self.opacity, self.color);
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
            ui.spacing_mut().item_spacing.x = theme::SPACE_SM;

            // Click-to-edit Notebook Title in Georgia Serif (no ugly raw input box)
            let title_edit = ui.add(
                egui::TextEdit::singleline(&mut self.notebook.title)
                    .font(theme::font_serif(15.0))
                    .text_color(p.text)
                    .frame(false)
                    .desired_width(190.0)
                    .hint_text("Untitled Notebook"),
            );
            if title_edit.hovered() || title_edit.has_focus() {
                let r = title_edit.rect;
                ui.painter().line_segment(
                    [Pos2::new(r.min.x, r.max.y), Pos2::new(r.max.x, r.max.y)],
                    Stroke::new(
                        1.0_f32,
                        if title_edit.has_focus() {
                            p.accent
                        } else {
                            p.border
                        },
                    ),
                );
            }
            title_edit.on_hover_text("Click to rename notebook");
            widgets::divider(ui, &p);

            let cur_idx = self.notebook.current_page_index;
            let total = self.notebook.pages.len();
            if ui
                .add_enabled_ui(cur_idx > 0, |ui| {
                    widgets::tool_button(ui, &p, VectorIcon::Prev, false, "Previous page (PgUp)")
                })
                .inner
                .clicked()
            {
                self.go_to_page(cur_idx.saturating_sub(1));
            }
            let pages_label = format!("Page {} / {}", cur_idx + 1, total);
            if widgets::vector_chip(
                ui,
                &p,
                VectorIcon::Pages,
                &pages_label,
                self.show_previews_drawer,
            )
            .on_hover_text("Show page previews")
            .clicked()
            {
                self.show_previews_drawer = !self.show_previews_drawer;
            }
            if ui
                .add_enabled_ui(cur_idx + 1 < total, |ui| {
                    widgets::tool_button(ui, &p, VectorIcon::Next, false, "Next page (PgDn)")
                })
                .inner
                .clicked()
            {
                self.go_to_page(cur_idx + 1);
            }
            let can_move_cur_up = cur_idx > 0;
            if ui
                .add_enabled_ui(can_move_cur_up, |ui| {
                    widgets::tool_button(ui, &p, VectorIcon::MoveUp, false, "Move current page earlier (Ctrl+Shift+PgUp)")
                })
                .inner
                .clicked()
            {
                if self.notebook.move_page_up(cur_idx) {
                    self.set_status(&format!("Moved page to position {}", cur_idx + 1));
                    self.autosave.mark_dirty();
                }
            }
            let can_move_cur_down = cur_idx + 1 < total;
            if ui
                .add_enabled_ui(can_move_cur_down, |ui| {
                    widgets::tool_button(ui, &p, VectorIcon::MoveDown, false, "Move current page later (Ctrl+Shift+PgDn)")
                })
                .inner
                .clicked()
            {
                if self.notebook.move_page_down(cur_idx) {
                    self.set_status(&format!("Moved page to position {}", cur_idx + 2));
                    self.autosave.mark_dirty();
                }
            }
            if widgets::tool_button(ui, &p, VectorIcon::Add, false, "Add page after this one")
                .clicked()
            {
                self.notebook.add_page();
                self.selected_stroke_ids.clear();
                self.autosave.mark_dirty();
            }
            if widgets::tool_button(ui, &p, VectorIcon::Extend, false, "Extend page by 600 px")
                .clicked()
            {
                self.notebook.current_page_mut().extend_height(600.0);
                self.set_status("Page extended +600 px");
                self.autosave.mark_dirty();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::primary_icon_button(ui, &p, VectorIcon::Save, "Save")
                    .on_hover_text("Save notebook (Ctrl+S)")
                    .clicked()
                {
                    self.prompt_save_file();
                }
                widgets::divider(ui, &p);

                // Autosave Status Pill
                let autosave_label = if self.autosave.enabled {
                    format!("Autosave: {}", self.autosave.interval.short_label())
                } else {
                    "Autosave: OFF".to_string()
                };
                if widgets::vector_chip(
                    ui,
                    &p,
                    VectorIcon::Autosave,
                    &autosave_label,
                    self.autosave.show_settings_dialog,
                )
                .on_hover_text("Autosave & Crash Recovery Settings - Click to configure")
                .clicked()
                {
                    self.autosave.show_settings_dialog = !self.autosave.show_settings_dialog;
                }
                widgets::divider(ui, &p);

                // Right-to-left: these appear as  [Fit] [-] 100% [+]
                if widgets::tool_button(ui, &p, "+", false, "Zoom in (Ctrl + wheel)").clicked() {
                    self.zoom_by(1.15, available_size);
                }
                let zoom_label = format!("{}%", (self.scale * 100.0).round() as i32);
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(zoom_label).monospace())
                            .frame(false)
                            .min_size(Vec2::new(48.0, 28.0)),
                    )
                    .on_hover_text("Reset to 100% (Ctrl+0)")
                    .clicked()
                {
                    self.zoom_by(1.0 / self.scale, available_size);
                }
                if widgets::tool_button(ui, &p, "-", false, "Zoom out (Ctrl + wheel)").clicked() {
                    self.zoom_by(0.85, available_size);
                }
                if widgets::tool_button(ui, &p, VectorIcon::Fit, false, "Fit page to window")
                    .clicked()
                {
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
            .default_width(260.0)
            .width_range(230.0..=460.0)
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(p.surface)
                    .inner_margin(12.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Pages").strong().size(15.0));
                    widgets::caption(ui, &p, format!("{}", self.notebook.pages.len()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::tool_button(ui, &p, VectorIcon::Close, false, "Close")
                            .clicked()
                        {
                            self.show_previews_drawer = false;
                        }
                        if widgets::tool_button(ui, &p, VectorIcon::Add, false, "Add page")
                            .clicked()
                        {
                            self.notebook.add_page();
                            self.clear_selection();
                        }
                    });
                });
                ui.add_space(6.0);

                // Page mutations are deferred until after the loop so indices stay valid.
                enum PageAction {
                    Select(usize),
                    MoveUp(usize),
                    MoveDown(usize),
                    Duplicate(usize),
                    Delete(usize),
                }
                let mut action = None;
                let total = self.notebook.pages.len();
                let cur_idx = self.notebook.current_page_index;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let card_w = ui.available_width() - 4.0;
                        for idx in 0..total {
                            let page = &self.notebook.pages[idx];
                            let is_active = idx == cur_idx;

                            // Fit width; long (extended) pages show their top portion.
                            let thumb_scale = card_w / page.width;
                            let thumb_h = (page.height * thumb_scale).min(card_w * 1.414);
                            let (rect, resp) =
                                ui.allocate_exact_size(Vec2::new(card_w, thumb_h), Sense::click());
                            let resp = resp.on_hover_text(format!("Go to page {}", idx + 1));
                            if resp.clicked() {
                                action = Some(PageAction::Select(idx));
                            }

                            let painter = ui.painter_at(rect.expand(4.0));
                            let to_thumb = |pt: Pos2| rect.min + pt.to_vec2() * thumb_scale;
                            let page_rect = Rect::from_min_size(
                                rect.min,
                                Vec2::new(page.width, page.height) * thumb_scale,
                            );
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
                            NoteRenderer::render_strokes(
                                &painter.with_clip_rect(rect),
                                &page.strokes,
                                thumb_scale,
                                &to_thumb,
                            );

                            let ring = if is_active {
                                Stroke::new(2.0_f32, p.accent)
                            } else if resp.hovered() {
                                Stroke::new(1.5_f32, p.text_muted)
                            } else {
                                Stroke::new(1.0_f32, p.border)
                            };
                            painter.rect_stroke(rect, theme::RADIUS_SMALL, ring);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                                let label = egui::RichText::new(format!("Page {}", idx + 1)).size(13.0);
                                ui.label(if is_active {
                                    label.color(p.accent).strong()
                                } else {
                                    label.color(p.text_muted)
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
                                        let can_delete = total > 1;
                                        if ui
                                            .add_enabled_ui(can_delete, |ui| {
                                                widgets::small_tool_button(
                                                    ui,
                                                    &p,
                                                    VectorIcon::Delete,
                                                    false,
                                                    "Delete page",
                                                )
                                            })
                                            .inner
                                            .clicked()
                                        {
                                            action = Some(PageAction::Delete(idx));
                                        }
                                        if widgets::small_tool_button(
                                            ui,
                                            &p,
                                            VectorIcon::Duplicate,
                                            false,
                                            "Duplicate page",
                                        )
                                        .clicked()
                                        {
                                            action = Some(PageAction::Duplicate(idx));
                                        }
                                        let can_move_down = idx + 1 < total;
                                        if ui
                                            .add_enabled_ui(can_move_down, |ui| {
                                                widgets::small_tool_button(
                                                    ui,
                                                    &p,
                                                    VectorIcon::MoveDown,
                                                    false,
                                                    "Move page down (later in document)",
                                                )
                                            })
                                            .inner
                                            .clicked()
                                        {
                                            action = Some(PageAction::MoveDown(idx));
                                        }
                                        let can_move_up = idx > 0;
                                        if ui
                                            .add_enabled_ui(can_move_up, |ui| {
                                                widgets::small_tool_button(
                                                    ui,
                                                    &p,
                                                    VectorIcon::MoveUp,
                                                    false,
                                                    "Move page up (earlier in document)",
                                                )
                                            })
                                            .inner
                                            .clicked()
                                        {
                                            action = Some(PageAction::MoveUp(idx));
                                        }
                                    },
                                );
                            });
                            ui.add_space(10.0);
                        }
                    });

                match action {
                    Some(PageAction::Select(idx)) => self.go_to_page(idx),
                    Some(PageAction::MoveUp(idx)) => {
                        if self.notebook.move_page_up(idx) {
                            self.set_status(&format!("Moved page {} earlier to position {}", idx + 1, idx));
                            self.autosave.mark_dirty();
                        }
                    }
                    Some(PageAction::MoveDown(idx)) => {
                        if self.notebook.move_page_down(idx) {
                            self.set_status(&format!("Moved page {} later to position {}", idx + 1, idx + 2));
                            self.autosave.mark_dirty();
                        }
                    }
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
            self.autosave.mark_dirty();
        }
        self.clear_selection();
    }

    fn redo(&mut self) {
        if let Some(stroke) = self.redo_stack.pop() {
            let page = self.notebook.current_page_mut();
            page.strokes.push(stroke);
            self.autosave.mark_dirty();
        }
        self.clear_selection();
    }

    fn prompt_save_file(&mut self) {
        if let Some(ref path) = self.current_file_path.clone() {
            if let Err(e) = NotebookStorage::save(path, &self.notebook) {
                self.set_error(&format!("Save failed: {}", e));
            } else {
                self.autosave.mark_clean();
                self.set_status("Notebook saved");
            }
            return;
        }
        self.prompt_save_as_file();
    }

    fn prompt_save_as_file(&mut self) {
        let default_name = if self.notebook.title.trim().is_empty() {
            "notebook.handnote".to_string()
        } else {
            format!("{}.handnote", self.notebook.title.trim())
        };
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Notebook As")
            .add_filter("Inkk Notebook (*.handnote, *.inkk)", &["handnote", "inkk"])
            .set_file_name(&default_name)
            .save_file()
        {
            if let Err(e) = NotebookStorage::save(&path, &self.notebook) {
                self.set_error(&format!("Save failed: {}", e));
            } else {
                self.current_file_path = Some(path);
                self.autosave.mark_clean();
                self.set_status("Notebook saved");
            }
        }
    }

    fn prompt_export_pdf(&mut self) {
        self.show_pdf_export_dialog = true;
    }

    fn execute_export_pdf(&mut self) {
        let suggested_name = if self.notebook.title.trim().is_empty() {
            "notebook.pdf".to_string()
        } else {
            format!("{}.pdf", self.notebook.title.trim())
        };
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export Notebook to PDF")
            .add_filter("PDF Document (*.pdf)", &["pdf"])
            .set_file_name(&suggested_name)
            .save_file()
        {
            let options = self.pdf_export_quality.to_options(self.pdf_include_patterns, self.pdf_force_light);
            let quality_lbl = self.pdf_export_quality.label();
            match crate::pdf_export::PdfExporter::export_notebook(&self.notebook, &options) {
                Ok(pdf_bytes) => {
                    if let Err(e) = crate::pdf_export::PdfExporter::save_to_file(&pdf_bytes, &path) {
                        self.set_error(&format!("PDF export failed: {}", e));
                    } else {
                        let count = self.notebook.pages.len();
                        self.set_status(&format!(
                            "Exported {} page(s) to PDF ({}) successfully!",
                            count, quality_lbl
                        ));
                    }
                }
                Err(e) => {
                    self.set_error(&format!("PDF export failed: {}", e));
                }
            }
        }
    }

    fn render_pdf_export_dialog(&mut self, ctx: &egui::Context, p: &crate::theme::Palette) {
        if !self.show_pdf_export_dialog {
            return;
        }

        let mut is_open = self.show_pdf_export_dialog;
        let mut trigger_export = false;
        let mut cancel_clicked = false;

        egui::Window::new("Export Notebook to PDF")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_size([460.0, 390.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                ui.label(
                    egui::RichText::new("Choose export quality and print settings for your document:")
                        .size(11.5)
                        .color(p.text_muted),
                );

                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Resolution & Quality Preset")
                        .font(crate::theme::font_serif(13.0))
                        .strong()
                        .color(p.text),
                );

                // 3 Quality Preset Cards: Low, Medium, High
                let presets = [
                    (
                        crate::pdf_export::PdfQualityPreset::Low,
                        "Low Quality",
                        "120 DPI - Compact (~1.2 MB)",
                        "Fast export, minimal file size. Great for email and quick web previews.",
                    ),
                    (
                        crate::pdf_export::PdfQualityPreset::Medium,
                        "Medium Quality",
                        "240 DPI - Standard (~3.5 MB)",
                        "High fidelity and balanced file size. Ideal for digital notes and reading.",
                    ),
                    (
                        crate::pdf_export::PdfQualityPreset::High,
                        "High Quality (Ultra-HD)",
                        "380+ DPI - Print-Ready (~8.5 MB)",
                        "Maximum sharpness with sub-pixel Bezier smoothing. Razor-sharp vector print.",
                    ),
                ];

                for (preset, title, sub, desc) in presets {
                    let is_selected = self.pdf_export_quality == preset;
                    let (card_rect, card_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 52.0),
                        egui::Sense::click(),
                    );

                    if card_resp.clicked() {
                        self.pdf_export_quality = preset;
                    }

                    if ui.is_rect_visible(card_rect) {
                        let painter = ui.painter();
                        let bg = if is_selected {
                            p.accent_soft
                        } else if card_resp.hovered() {
                            p.hover
                        } else {
                            p.surface_alt
                        };
                        let border_stroke = if is_selected {
                            Stroke::new(1.5_f32, p.accent)
                        } else {
                            Stroke::new(1.0_f32, p.border)
                        };

                        painter.rect(card_rect, egui::Rounding::same(crate::theme::RADIUS_MD), bg, border_stroke);

                        // Radio dot on left
                        let radio_c = Pos2::new(card_rect.min.x + 18.0, card_rect.center().y);
                        painter.circle_stroke(radio_c, 7.0, Stroke::new(1.5_f32, if is_selected { p.accent } else { p.border }));
                        if is_selected {
                            painter.circle_filled(radio_c, 4.0, p.accent);
                        }

                        // Title & Subtext
                        let text_x = card_rect.min.x + 36.0;
                        painter.text(
                            Pos2::new(text_x, card_rect.min.y + 10.0),
                            egui::Align2::LEFT_TOP,
                            title,
                            egui::FontId::proportional(13.0),
                            if is_selected { p.accent } else { p.text },
                        );

                        painter.text(
                            Pos2::new(card_rect.max.x - 12.0, card_rect.min.y + 10.0),
                            egui::Align2::RIGHT_TOP,
                            sub,
                            egui::FontId::monospace(10.0),
                            p.text_muted,
                        );

                        painter.text(
                            Pos2::new(text_x, card_rect.min.y + 30.0),
                            egui::Align2::LEFT_TOP,
                            desc,
                            egui::FontId::proportional(10.5),
                            p.text_muted,
                        );
                    }
                }

                ui.add_space(4.0);
                ui.separator();

                // Options checkboxes
                ui.checkbox(&mut self.pdf_include_patterns, "Include paper rulings, grid, and margin lines");
                ui.checkbox(&mut self.pdf_force_light, "Force light paper background (Ink-saving print)");

                ui.add_space(8.0);

                // Footer row with Page Count and Action Buttons
                ui.horizontal(|ui| {
                    let page_count = self.notebook.pages.len();
                    ui.label(
                        egui::RichText::new(format!("{} page(s) - Ready", page_count))
                            .size(11.0)
                            .color(p.text_muted),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let export_btn = egui::Button::new(
                            egui::RichText::new("Export PDF...").color(p.on_accent).strong(),
                        )
                        .fill(p.accent)
                        .rounding(egui::Rounding::same(crate::theme::RADIUS_SM));

                        if ui.add(export_btn).clicked() {
                            trigger_export = true;
                        }

                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
            });

        if cancel_clicked {
            is_open = false;
        }
        self.show_pdf_export_dialog = is_open;

        if trigger_export {
            self.show_pdf_export_dialog = false;
            self.execute_export_pdf();
        }
    }

    pub fn open_notebook_from_path(&mut self, path: &std::path::Path) -> Result<(), String> {
        // Auto-save current file before switching if it was saved before
        if let Some(ref cur_path) = self.current_file_path {
            let _ = NotebookStorage::save(cur_path, &self.notebook);
        }

        let mut nb = NotebookStorage::load(path)?;
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
        self.current_file_path = Some(path.to_path_buf());
        self.clear_selection();
        self.set_status(&format!(
            "Opened '{}'",
            path.file_stem().and_then(|s| s.to_str()).unwrap_or("notebook")
        ));
        self.library.scan(self.current_file_path.as_deref());
        Ok(())
    }

    fn handle_library_action(&mut self, action: crate::library::LibraryAction) {
        match action {
            crate::library::LibraryAction::Open(path) => {
                if let Err(e) = self.open_notebook_from_path(&path) {
                    self.set_error(&format!("Failed to open notebook: {}", e));
                } else {
                    self.library.is_open = false;
                }
            }
            crate::library::LibraryAction::CreateNew { title, paper_type } => {
                // Auto-save current
                if let Some(ref cur_path) = self.current_file_path {
                    let _ = NotebookStorage::save(cur_path, &self.notebook);
                }

                let safe_title = if title.trim().is_empty() {
                    "Untitled Notebook".to_string()
                } else {
                    title.trim().to_string()
                };

                let mut new_nb = crate::model::Notebook::new_with_paper(&safe_title, paper_type);
                new_nb.title = safe_title.clone();

                let file_name = format!("{}.handnote", safe_title.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_"));
                let target_path = self.library.library_dir.join(file_name);

                if let Err(e) = NotebookStorage::save(&target_path, &new_nb) {
                    self.set_error(&format!("Failed to create notebook: {}", e));
                } else {
                    self.notebook = new_nb;
                    self.current_file_path = Some(target_path.clone());
                    self.clear_selection();
                    self.library.scan(Some(&target_path));
                    self.library.is_open = false;
                    self.set_status(&format!("Created and opened '{}'", safe_title));
                }
            }
            crate::library::LibraryAction::Duplicate(path) => {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Notebook");
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("handnote");
                let new_filename = format!("{} (Copy).{}", stem, ext);
                let target_path = path.with_file_name(new_filename);

                if let Err(e) = std::fs::copy(&path, &target_path) {
                    self.set_error(&format!("Failed to duplicate: {}", e));
                } else {
                    self.library.scan(self.current_file_path.as_deref());
                    self.set_status("Notebook duplicated");
                }
            }
            crate::library::LibraryAction::Rename { path, new_title } => {
                let clean_title = new_title.trim();
                if clean_title.is_empty() {
                    return;
                }
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("handnote");
                let safe_filename = format!("{}.{}", clean_title.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "_"), ext);
                let new_path = path.with_file_name(safe_filename);

                if let Err(e) = std::fs::rename(&path, &new_path) {
                    self.set_error(&format!("Failed to rename: {}", e));
                } else {
                    if let Ok(mut nb) = NotebookStorage::load(&new_path) {
                        nb.title = clean_title.to_string();
                        let _ = NotebookStorage::save(&new_path, &nb);
                    }
                    if self.current_file_path.as_deref() == Some(&path) {
                        self.current_file_path = Some(new_path);
                        self.notebook.title = clean_title.to_string();
                    }
                    self.library.scan(self.current_file_path.as_deref());
                    self.set_status(&format!("Renamed to '{}'", clean_title));
                }
            }
            crate::library::LibraryAction::Delete(path) => {
                let is_current = self.current_file_path.as_deref() == Some(&path);
                if let Err(e) = std::fs::remove_file(&path) {
                    self.set_error(&format!("Failed to delete: {}", e));
                } else {
                    if is_current {
                        self.notebook = crate::model::Notebook::new("Quick Notes");
                        self.current_file_path = None;
                        self.clear_selection();
                    }
                    self.library.scan(self.current_file_path.as_deref());
                    self.set_status("Notebook deleted");
                }
            }
            crate::library::LibraryAction::ExportPdf(path) => {
                if let Ok(nb) = NotebookStorage::load(&path) {
                    let options = self.pdf_export_quality.to_options(self.pdf_include_patterns, self.pdf_force_light);
                    let target_pdf = path.with_extension("pdf");
                    match crate::pdf_export::PdfExporter::export_notebook(&nb, &options) {
                        Ok(bytes) => {
                            if let Err(e) = crate::pdf_export::PdfExporter::save_to_file(&bytes, &target_pdf) {
                                self.set_error(&format!("Failed to save PDF: {}", e));
                            } else {
                                self.set_status(&format!("Exported PDF to '{}'", target_pdf.display()));
                            }
                        }
                        Err(e) => {
                            self.set_error(&format!("PDF export failed: {}", e));
                        }
                    }
                }
            }
            crate::library::LibraryAction::OpenFolderInExplorer => {
                let _ = std::process::Command::new("explorer")
                    .arg(&self.library.library_dir)
                    .spawn();
            }
            crate::library::LibraryAction::ImportFile => {
                self.prompt_open_file();
            }
        }
    }

    fn prompt_open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Notebook")
            .add_filter("Inkk Notebook (*.inkk, *.handnote)", &["inkk", "handnote"])
            .pick_file()
        {
            if let Err(e) = self.open_notebook_from_path(&path) {
                self.set_error(&format!("Open failed: {}", e));
            }
        }
    }

    pub fn recover_last_session(&mut self) {
        if let Some((recovered_nb, orig_path, meta)) = crate::autosave::AutosaveManager::check_recovery_available() {
            let title = meta.notebook_title.clone();
            let pages = meta.page_count;
            let strokes = meta.stroke_count;
            self.notebook = recovered_nb;
            self.current_file_path = orig_path;
            self.last_total_strokes = strokes;
            self.autosave.recovery_restored_banner = Some(meta);
            self.set_status(&format!("Recovered session '{}' ({} pages, {} strokes)", title, pages, strokes));
        } else {
            self.set_status("No uncommitted recovery session found.");
        }
    }

    // --- ERASERS ---

    fn erase_stroke_at(&mut self, world_pos: Pos2, radius: f32) {
        let page = self.notebook.current_page_mut();
        let old_len = page.strokes.len();
        page.strokes
            .retain(|stroke| !stroke_hit_test(stroke, world_pos, radius));
        if page.strokes.len() != old_len {
            self.autosave.mark_dirty();
        }
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
                s.id = format!(
                    "stroke_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                );
                s.points = seg;
                new_strokes.push(s);
            }
        }

        page.strokes = new_strokes;
        self.autosave.mark_dirty();
    }
}

impl InkkApp {
    fn handle_shortcuts(&mut self, ctx: &egui::Context, available_size: Vec2) {
        use egui::{Key, Modifiers};
        let pressed = |m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));

        // 1. Whiteboard Canvas Shortcuts: Copy, Cut, Paste, Duplicate, Select All
        // When user is not editing an inline text box, process these first!
        if self.editing_text_id.is_none() {
            let mut do_copy = false;
            let mut do_cut = false;
            let mut do_paste = false;
            let mut do_duplicate = false;
            let mut do_select_all = false;

            ctx.input_mut(|i| {
                i.events.retain(|event| {
                    match event {
                        egui::Event::Copy => {
                            do_copy = true;
                            false
                        }
                        egui::Event::Cut => {
                            do_cut = true;
                            false
                        }
                        egui::Event::Paste(_) => {
                            do_paste = true;
                            false
                        }
                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } => {
                            let ctrl = modifiers.ctrl || modifiers.command;
                            if ctrl && !modifiers.shift && !modifiers.alt {
                                match key {
                                    Key::C => {
                                        do_copy = true;
                                        false
                                    }
                                    Key::X => {
                                        do_cut = true;
                                        false
                                    }
                                    Key::V => {
                                        do_paste = true;
                                        false
                                    }
                                    Key::D => {
                                        do_duplicate = true;
                                        false
                                    }
                                    Key::A => {
                                        do_select_all = true;
                                        false
                                    }
                                    _ => true,
                                }
                            } else {
                                true
                            }
                        }
                        _ => true,
                    }
                });
            });

            if do_copy {
                self.copy_selected();
                return;
            }
            if do_cut {
                self.cut_selected();
                return;
            }
            if do_paste {
                self.paste_from_clipboard();
                return;
            }
            if do_duplicate {
                self.duplicate_selected();
                return;
            }
            if do_select_all {
                self.select_all();
                return;
            }
        }

        // Don't steal keys while the user is actively typing in a text widget.
        if ctx.wants_keyboard_input() {
            return;
        }

        // Mouse function buttons: Side buttons (Extra1 = Back / Undo, Extra2 = Forward / Redo)
        let (extra1, extra2, middle_clicked) = ctx.input(|i| {
            (
                i.pointer.button_clicked(egui::PointerButton::Extra1),
                i.pointer.button_clicked(egui::PointerButton::Extra2),
                i.pointer.button_clicked(egui::PointerButton::Middle),
            )
        });
        if extra1 {
            self.undo();
        }
        if extra2 {
            self.redo();
        }
        if middle_clicked {
            self.scale = 1.0;
            self.set_status("Scale: 100% (Middle click)");
        }

        if pressed(Modifiers::NONE, Key::F1)
            || self.shortcut_manager.is_triggered(ctx, "shortcuts_dialog")
        {
            self.shortcuts_dialog.open();
        }

        if self.shortcut_manager.is_triggered(ctx, "edit_redo")
            || pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
            || pressed(Modifiers::COMMAND, Key::Y)
        {
            self.redo();
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_undo")
            || pressed(Modifiers::COMMAND, Key::Z)
        {
            self.undo();
        }
        if self.shortcut_manager.is_triggered(ctx, "file_save")
            || pressed(Modifiers::COMMAND, Key::S)
        {
            self.prompt_save_file();
        }
        if self.shortcut_manager.is_triggered(ctx, "file_open")
            || pressed(Modifiers::COMMAND, Key::O)
        {
            self.prompt_open_file();
        }
        if self.shortcut_manager.is_triggered(ctx, "file_library")
            || pressed(Modifiers::COMMAND, Key::L)
        {
            if self.library.is_open {
                self.library.is_open = false;
            } else {
                self.library.open(self.current_file_path.as_deref());
            }
        }
        if self.shortcut_manager.is_triggered(ctx, "file_export_pdf")
            || pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::E)
        {
            self.prompt_export_pdf();
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_copy")
            || pressed(Modifiers::COMMAND, Key::C)
        {
            self.copy_selected();
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_cut")
            || pressed(Modifiers::COMMAND, Key::X)
        {
            self.cut_selected();
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_duplicate")
            || pressed(Modifiers::COMMAND, Key::D)
        {
            self.duplicate_selected();
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_select_all")
            || pressed(Modifiers::COMMAND, Key::A)
        {
            self.select_all();
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_paste")
            || pressed(Modifiers::COMMAND, Key::V)
        {
            self.paste_from_clipboard();
        }
        if self.shortcut_manager.is_triggered(ctx, "canvas_reset_zoom")
            || pressed(Modifiers::COMMAND, Key::Num0)
        {
            self.zoom_by(1.0 / self.scale, available_size);
        }
        if self.shortcut_manager.is_triggered(ctx, "canvas_center_page")
            || pressed(Modifiers::NONE, Key::Home)
        {
            self.center_page_in_view(available_size);
        }
        if self.shortcut_manager.is_triggered(ctx, "edit_delete")
            || pressed(Modifiers::NONE, Key::Delete)
            || pressed(Modifiers::NONE, Key::Backspace)
        {
            self.delete_selected();
        }
        if pressed(Modifiers::NONE, Key::Escape) {
            self.clear_selection();
            self.editing_text_id = None;
            self.notebook
                .current_page_mut()
                .text_boxes
                .retain(|b| !b.text.trim().is_empty());
        }
        if self.shortcut_manager.is_triggered(ctx, "nav_move_page_up")
            || pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::PageUp)
            || pressed(Modifiers::ALT, Key::ArrowUp)
        {
            let cur = self.notebook.current_page_index;
            if self.notebook.move_page_up(cur) {
                self.set_status(&format!("Moved page to position {}", cur + 1));
                self.autosave.mark_dirty();
            }
        }
        if self.shortcut_manager.is_triggered(ctx, "nav_move_page_down")
            || pressed(Modifiers::COMMAND | Modifiers::SHIFT, Key::PageDown)
            || pressed(Modifiers::ALT, Key::ArrowDown)
        {
            let cur = self.notebook.current_page_index;
            if self.notebook.move_page_down(cur) {
                self.set_status(&format!("Moved page to position {}", cur + 2));
                self.autosave.mark_dirty();
            }
        }
        if self.shortcut_manager.is_triggered(ctx, "nav_next_page")
            || pressed(Modifiers::NONE, Key::PageDown)
        {
            self.notebook.next_page();
            self.clear_selection();
        }
        if self.shortcut_manager.is_triggered(ctx, "nav_prev_page")
            || pressed(Modifiers::NONE, Key::PageUp)
        {
            self.notebook.prev_page();
            self.clear_selection();
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_brush_smaller")
            || pressed(Modifiers::NONE, Key::OpenBracket)
        {
            self.size = (self.size - 1.0).max(1.0);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_brush_bigger")
            || pressed(Modifiers::NONE, Key::CloseBracket)
        {
            self.size = (self.size + 1.0).min(30.0);
        }

        if self.shortcut_manager.is_triggered(ctx, "tool_pen") || pressed(Modifiers::NONE, Key::Num1) {
            self.set_tool(ToolMode::Pen);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_highlighter") || pressed(Modifiers::NONE, Key::Num2) {
            self.set_tool(ToolMode::Highlighter);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_shape") || pressed(Modifiers::NONE, Key::Num3) {
            self.set_tool(ToolMode::Shape(self.current_shape));
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_text")
            || pressed(Modifiers::NONE, Key::Num4)
            || pressed(Modifiers::NONE, Key::T)
        {
            self.set_tool(ToolMode::Text);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_eraser_stroke") || pressed(Modifiers::NONE, Key::Num5) {
            self.set_tool(ToolMode::EraserStroke);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_eraser_point") || pressed(Modifiers::NONE, Key::Num6) {
            self.set_tool(ToolMode::EraserPoint);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_lasso")
            || pressed(Modifiers::NONE, Key::Num7)
            || pressed(Modifiers::NONE, Key::L)
        {
            self.set_tool(ToolMode::LassoSelect);
        }
        if self.shortcut_manager.is_triggered(ctx, "tool_pan") || pressed(Modifiers::NONE, Key::Num8) {
            self.set_tool(ToolMode::Pan);
        }

        if self.shortcut_manager.is_triggered(ctx, "instrument_protractor") || pressed(Modifiers::SHIFT, Key::R) {
            self.ruler.toggle_protractor();
            let is_on = self.ruler.kind == RulerKind::Protractor;
            self.set_status(if is_on {
                "Protractor ON (drag body to move, dial to rotate)"
            } else {
                "Protractor OFF"
            });
        } else if self.shortcut_manager.is_triggered(ctx, "instrument_ruler") || pressed(Modifiers::NONE, Key::R) {
            self.ruler.toggle_ruler();
            let is_on = self.ruler.kind == RulerKind::Ruler;
            self.set_status(if is_on {
                "Straight Ruler ON (drag body to move, dial to rotate)"
            } else {
                "Ruler OFF"
            });
        }

        let ctrl_m = pressed(Modifiers::COMMAND, Key::M);
        let plain_m = pressed(Modifiers::NONE, Key::M)
            && !ctx.wants_keyboard_input()
            && self.editing_text_id.is_none();
        if self.shortcut_manager.is_triggered(ctx, "studio_math") || ctrl_m || plain_m {
            self.math_studio.toggle();
            let is_on = self.math_studio.is_open;
            self.set_status(if is_on {
                "Math Studio OPEN [MATLAB Engine Ready]"
            } else {
                "Math Studio CLOSED"
            });
        }

        let ctrl_f = pressed(Modifiers::COMMAND, Key::F);
        if self.shortcut_manager.is_triggered(ctx, "studio_flowchart") || ctrl_f {
            self.flowchart_state.is_panel_open = !self.flowchart_state.is_panel_open;
            let is_on = self.flowchart_state.is_panel_open;
            self.set_status(if is_on {
                "Flowchart Shapes Menu OPEN"
            } else {
                "Flowchart Shapes Menu CLOSED"
            });
        }

        // F11: Toggle Fullscreen
        let f11_pressed = pressed(Modifiers::NONE, Key::F11)
            || self.shortcut_manager.is_triggered(ctx, "view_fullscreen");
        if f11_pressed {
            self.toggle_fullscreen(ctx);
        }
    }

    /// Toggle borderless full screen mode on or off
    pub fn toggle_fullscreen(&mut self, ctx: &egui::Context) {
        let current = ctx.input(|i| i.viewport().fullscreen).unwrap_or(self.is_fullscreen);
        self.is_fullscreen = !current;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
        self.set_status(if self.is_fullscreen {
            "Fullscreen Mode ON (Press F11 to exit)"
        } else {
            "Fullscreen Mode OFF"
        });
    }

    /// Floating toast for status / error messages, fading out over `theme::TOAST_SECS`.
    fn render_toast(&self, ctx: &egui::Context, canvas_rect: Rect) {
        let Some((msg, at, is_error)) = &self.status_message else {
            return;
        };
        let t = at.elapsed().as_secs_f32();
        let total = if *is_error {
            theme::TOAST_SECS * 2.0
        } else {
            theme::TOAST_SECS
        };
        if t >= total {
            return;
        }
        let alpha = ((total - t) / 0.4).clamp(0.0, 1.0);
        let p = self.palette();
        let (v_icon, color) = if *is_error {
            (VectorIcon::Warn, p.danger)
        } else {
            (VectorIcon::Check, p.success)
        };

        egui::Area::new(egui::Id::new("toast"))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .fixed_pos(Pos2::new(
                canvas_rect.center().x,
                canvas_rect.bottom() - 16.0,
            ))
            .show(ctx, |ui| {
                ui.set_opacity(alpha);
                egui::Frame::popup(ui.style())
                    .rounding(egui::Rounding::same(18.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            v_icon.paint(ui.painter(), icon_rect, color, 1.8);
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
                let mul = if self.tool == ToolMode::Highlighter {
                    2.5
                } else {
                    0.5
                };
                let r = (self.size * mul * self.scale).max(1.5);
                painter.circle_stroke(
                    cursor,
                    r + 1.0,
                    Stroke::new(1.0_f32, p.text_muted.gamma_multiply(0.7)),
                );
                ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
            }
            ToolMode::EraserStroke | ToolMode::EraserPoint => {
                // Matches the world-space radius used by erase_*_at (scale cancels out on screen)
                let r = if self.tool == ToolMode::EraserStroke {
                    self.size * 5.0
                } else {
                    self.size * 3.5
                };
                painter.circle(
                    cursor,
                    r,
                    p.danger.gamma_multiply(0.08),
                    Stroke::new(1.2_f32, p.danger.gamma_multiply(0.8)),
                );
                ctx.set_cursor_icon(egui::CursorIcon::None);
            }
            ToolMode::Shape(_) => ctx.set_cursor_icon(egui::CursorIcon::Crosshair),
            ToolMode::LassoSelect => {
                let mut cursor_set = false;
                if let Some(bounds) = self.get_selection_bounds() {
                    let sb =
                        Rect::from_min_max(self.to_screen(bounds.min), self.to_screen(bounds.max))
                            .expand(8.0);
                    if let Some(handle) = hit_handle(cursor, sb, 16.0) {
                        let icon = match handle {
                            SelectionHandle::TopLeft | SelectionHandle::BottomRight => {
                                egui::CursorIcon::ResizeNwSe
                            }
                            SelectionHandle::TopRight | SelectionHandle::BottomLeft => {
                                egui::CursorIcon::ResizeNeSw
                            }
                            SelectionHandle::Top | SelectionHandle::Bottom => {
                                egui::CursorIcon::ResizeNorth
                            }
                            SelectionHandle::Left | SelectionHandle::Right => {
                                egui::CursorIcon::ResizeEast
                            }
                        };
                        ctx.set_cursor_icon(icon);
                        cursor_set = true;
                    } else {
                        let move_bar_rect = Rect::from_min_max(
                            Pos2::new(sb.left() - 8.0, sb.top() - 38.0),
                            Pos2::new(sb.right().max(sb.left() + 280.0) + 8.0, sb.top() + 6.0),
                        );
                        if sb.expand(16.0).contains(cursor) || move_bar_rect.contains(cursor) {
                            ctx.set_cursor_icon(egui::CursorIcon::Move);
                            cursor_set = true;
                        }
                    }
                }
                if !cursor_set {
                    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
                }
            }
            ToolMode::Text => ctx.set_cursor_icon(egui::CursorIcon::Text),
            ToolMode::Pan => {
                let grabbing = ctx.input(|i| i.pointer.primary_down());
                ctx.set_cursor_icon(if grabbing {
                    egui::CursorIcon::Grabbing
                } else {
                    egui::CursorIcon::Grab
                });
            }
        }
    }
}

impl eframe::App for InkkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let p = self.palette();
        let available_size = ctx.screen_rect().size();

        // Drag-and-drop of image files from Windows Explorer onto the window
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            let mut count = 0;
            for file in dropped {
                if let Some(path) = file.path {
                    if let Ok(bytes) = std::fs::read(&path) {
                        if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(&bytes) {
                            self.insert_image_bytes(img_bytes, w, h);
                            count += 1;
                        }
                    }
                } else if let Some(bytes) = file.bytes {
                    if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(&bytes) {
                        self.insert_image_bytes(img_bytes, w, h);
                        count += 1;
                    }
                }
            }
            if count > 0 {
                self.set_status(&format!("Inserted {} dropped image(s)!", count));
            }
        }

        // Track stroke changes to ensure dirty state is maintained
        let cur_strokes: usize = self.notebook.pages.iter().map(|p| p.strokes.len()).sum();
        if cur_strokes != self.last_total_strokes {
            self.autosave.mark_dirty();
            self.last_total_strokes = cur_strokes;
        }

        // Periodic background autosave check
        if self.autosave.should_autosave() {
            if let Err(e) = self.autosave.save_recovery(&self.notebook, self.current_file_path.as_deref()) {
                eprintln!("Autosave snapshot failed: {}", e);
            }
        }

        // Keep wake-up timer active if dirty and autosave is enabled
        if self.autosave.enabled && self.autosave.is_dirty {
            let elapsed = self.autosave.last_autosave_time.elapsed().as_secs();
            let remaining = self.autosave.interval.as_secs().saturating_sub(elapsed);
            ctx.request_repaint_after(std::time::Duration::from_secs(remaining.max(1)));
        }

        // Crash Recovery Notice Banner (only shown when splash screen is not active to prevent duplicate notification)
        let mut dismiss_recovery_banner = false;
        let mut save_as_from_recovery = false;
        if !self.splash_screen.is_active {
            if let Some(ref meta) = self.autosave.recovery_restored_banner {
                egui::TopBottomPanel::top("crash_recovery_banner")
                    .frame(
                        egui::Frame::side_top_panel(&ctx.style())
                            .fill(if self.is_dark_mode {
                                Color32::from_rgb(0x28, 0x22, 0x1E)
                            } else {
                                Color32::from_rgb(0xF5, 0xEF, 0xE6)
                            })
                            .inner_margin(egui::Margin::symmetric(14.0, 7.0))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0xB5, 0x41, 0x2E))),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Crash Recovery Active:")
                                    .font(theme::font_serif(13.0))
                                    .strong()
                                    .color(Color32::from_rgb(0xB5, 0x41, 0x2E)),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "Restored uncommitted session '{}' from {} ({} pages, {} strokes). All work safe!",
                                    meta.notebook_title, meta.time_str, meta.page_count, meta.stroke_count
                                ))
                                .color(p.text)
                                .size(11.5),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("Dismiss")
                                            .size(11.0)
                                            .color(p.text_muted),
                                    )
                                    .clicked()
                                {
                                    dismiss_recovery_banner = true;
                                }
                                if ui
                                    .button(
                                        egui::RichText::new("Save As...")
                                            .size(11.0)
                                            .color(p.accent)
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    save_as_from_recovery = true;
                                }
                            });
                        });
                    });
            }
        }
        if dismiss_recovery_banner {
            self.autosave.recovery_restored_banner = None;
        }
        if save_as_from_recovery {
            self.prompt_save_as_file();
            self.autosave.recovery_restored_banner = None;
        }

        let bar_frame = egui::Frame::side_top_panel(&ctx.style())
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .stroke(Stroke::new(1.0_f32, p.border));

        egui::TopBottomPanel::top("top_panel")
            .frame(bar_frame)
            .show(ctx, |ui| {
                self.render_top_toolbar(ui, ctx);
            });
        if matches!(
            self.tool,
            ToolMode::Pen | ToolMode::Shape(_) | ToolMode::Text
        ) {
            egui::TopBottomPanel::top("context_bar")
                .frame(
                    bar_frame
                        .fill(p.surface_alt)
                        .inner_margin(egui::Margin::symmetric(12.0, 4.0)),
                )
                .show(ctx, |ui| {
                    self.render_context_bar(ui);
                });
        }
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(bar_frame)
            .show(ctx, |ui| {
                self.render_bottom_bar(ui, available_size);
            });
        self.math_studio
            .render_panel(ctx, self.notebook.current_page_mut(), &p, self.is_dark_mode);

        if self.math_studio.req_open_matrix_dialog {
            self.math_studio.req_open_matrix_dialog = false;
            self.matrix_dialog.open_new();
        }
        if self.math_studio.req_open_axes_dialog {
            self.math_studio.req_open_axes_dialog = false;
            self.axes_dialog.open_new();
        }
        if self.math_studio.req_open_table_dialog {
            self.math_studio.req_open_table_dialog = false;
            self.table_dialog.open_new();
        }

        self.matrix_dialog.render(
            ctx,
            self.notebook.current_page_mut(),
            &mut self.math_studio.workspace,
            &p,
        );
        self.axes_dialog.render(
            ctx,
            self.notebook.current_page_mut(),
            &mut self.math_studio.workspace,
            &p,
        );
        self.table_dialog
            .render(ctx, self.notebook.current_page_mut(), &p);
        self.flowchart_editor.render_window(
            ctx,
            &mut self.notebook.current_page_mut().flowcharts,
            &p,
            self.is_dark_mode,
        );
        self.render_pdf_export_dialog(ctx, &p);
        self.autosave.render_dialog(ctx, &p, &self.notebook, self.current_file_path.as_deref());

        if let Some(act) = self.library.render(ctx, &p, self.is_dark_mode, self.current_file_path.as_deref()) {
            self.handle_library_action(act);
        }

        if let Some(svg) = self.flowchart_editor.req_bake_to_image.take() {
            if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(svg.as_bytes()) {
                self.insert_image_bytes(img_bytes, w, h);
                self.set_status("Flowchart converted and placed on whiteboard as an image!");
            } else {
                self.set_error("Could not render flowchart SVG into image.");
            }
        }

        if let Some(svg) = self.flowchart_editor.req_save_svg_file.take() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Scalable Vector Graphics (*.svg)", &["svg"])
                .set_file_name("flowchart.svg")
                .save_file()
            {
                if std::fs::write(&path, svg).is_ok() {
                    self.set_status(&format!("Saved flowchart SVG to {}", path.display()));
                } else {
                    self.set_error("Failed to write SVG file.");
                }
            }
        }



        // Flowchart Left Shapes Sidebar Panel
        let view_center_world = self.to_world(Pos2::new(available_size.x * 0.5, available_size.y * 0.5));
        let mut open_flow_editor = false;
        let mut req_bake_flow_svg = None;
        let is_dark_mode = self.is_dark_mode;
        crate::flowchart_ui::FlowchartPalette::render_side_panel(
            ctx,
            &mut self.flowchart_state,
            self.notebook.current_page_mut(),
            &p,
            is_dark_mode,
            &mut open_flow_editor,
            &mut req_bake_flow_svg,
            view_center_world,
        );
        if open_flow_editor {
            self.flowchart_editor.is_open = true;
        }
        if let Some(svg) = req_bake_flow_svg {
            if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(svg.as_bytes()) {
                self.insert_image_bytes(img_bytes, w, h);
                self.set_status("Flowchart converted and placed on whiteboard as an image!");
            }
        }

        self.render_previews_drawer(ctx);

        self.handle_shortcuts(ctx, available_size);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.desk))
            .show(ctx, |ui| {
                let canvas_rect = ui.available_rect_before_wrap();
                self.onboarding_tour.rect_canvas = canvas_rect;
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

                // Empty state watermark: understated Inkk mascot + Georgia serif welcome
                let is_page_empty = page.strokes.is_empty()
                    && page.images.is_empty()
                    && page.text_boxes.is_empty()
                    && page.matrices.is_empty()
                    && page.plots.is_empty()
                    && page.grids.is_empty()
                    && self.current_stroke.is_none();
                if is_page_empty {
                    let center = to_screen(Pos2::new(page_w * 0.5, (page_h * 0.38).min(380.0)));
                    let body_col = if is_dark {
                        Color32::from_rgba_unmultiplied(212, 98, 82, 50)
                    } else {
                        Color32::from_rgba_unmultiplied(185, 75, 60, 45)
                    };
                    let eye_col = if is_dark {
                        Color32::from_rgba_unmultiplied(237, 230, 220, 60)
                    } else {
                        Color32::from_rgba_unmultiplied(32, 30, 27, 50)
                    };
                    let blush_col = if is_dark {
                        Color32::from_rgba_unmultiplied(230, 140, 130, 70)
                    } else {
                        Color32::from_rgba_unmultiplied(235, 150, 140, 65)
                    };
                    widgets::draw_squid_mascot(&page_painter, center, 4.0 * scale.min(1.2), body_col, eye_col, blush_col);

                    let title_pos = center + Vec2::new(0.0, 36.0 * scale.min(1.2));
                    page_painter.text(
                        title_pos,
                        egui::Align2::CENTER_TOP,
                        "Inkk",
                        theme::font_serif(18.0 * scale.min(1.2)),
                        p.text_muted.gamma_multiply(0.4),
                    );

                    let sub_pos = title_pos + Vec2::new(0.0, 24.0 * scale.min(1.2));
                    page_painter.text(
                        sub_pos,
                        egui::Align2::CENTER_TOP,
                        "Pick up your pen or tap to write",
                        egui::FontId::proportional(12.5 * scale.min(1.2)),
                        p.text_muted.gamma_multiply(0.3),
                    );
                }

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

                // Layer 0.4: Flowchart Entities
                let hover_cursor_pos = ui.input(|i| i.pointer.hover_pos());
                for flow in &page.flowcharts {
                    let is_sel = self.selected_flowchart_ids.contains(&flow.id)
                        || self.flowchart_state.active_flowchart_id.as_deref() == Some(&flow.id);
                    crate::flowchart_ui::FlowchartRenderer::render(
                        &page_painter,
                        ui,
                        flow,
                        &mut self.flowchart_state,
                        scale,
                        &to_screen,
                        &p,
                        is_dark,
                        is_sel,
                        hover_cursor_pos,
                    );
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
                        painter.circle(h, 6.5, fill, Stroke::new(2.0_f32, stroke_color));
                    }

                    // Move Bar Header above the selection
                    let total_count = self.selected_stroke_ids.len()
                        + self.selected_image_ids.len()
                        + self.selected_plot_ids.len()
                        + self.selected_matrix_ids.len()
                        + self.selected_grid_ids.len();
                    let badge_text = if total_count == 1 {
                        if !self.selected_image_ids.is_empty() {
                            "Move - Drag corners to resize - Del: Delete".to_string()
                        } else if !self.selected_plot_ids.is_empty() {
                            "Plot - Drag to move - Del: Delete".to_string()
                        } else if !self.selected_matrix_ids.is_empty() {
                            "Matrix - Drag to move - Del: Delete".to_string()
                        } else if !self.selected_grid_ids.is_empty() {
                            "Table - Drag to move - Del: Delete".to_string()
                        } else {
                            "Strokes - Drag to move - Del: Delete".to_string()
                        }
                    } else {
                        format!("{} selected items - Drag to move - Del: Delete", total_count)
                    };
                    let galley = painter.layout_no_wrap(badge_text, egui::FontId::proportional(12.0), p.on_accent);
                    let badge_min = Pos2::new(r.left(), (r.top() - galley.size().y - 14.0).max(canvas_rect.top() + 6.0));
                    let badge_rect = Rect::from_min_size(badge_min, galley.size() + Vec2::new(20.0, 8.0));
                    painter.rect_filled(badge_rect, 6.0, accent);
                    painter.galley(badge_rect.min + Vec2::new(10.0, 4.0), galley, p.on_accent);
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
                                "Drag to move - Right edge to resize",
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
                            self.set_status("Page auto-expanded!");
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
                    if self.canvas_gesture {
                        // Auto-dismiss color and opacity popovers when user starts writing or touches the canvas
                        ctx.data_mut(|d| {
                            d.insert_temp(egui::Id::new("ink_palette_popover"), false);
                            d.insert_temp(egui::Id::new("smart_opacity_custom_popup"), false);
                        });
                    }
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

                // 1. Right mouse button gestures (Quick move object/card or Pan canvas)
                if pointer.secondary_pressed() && response.hovered() {
                    if let Some(cursor) = pointer_pos {
                        let world = self.to_world(cursor);
                        self.secondary_drag_start = Some(cursor);

                        let over_selection = self.get_selection_bounds().map(|b| b.expand(16.0 / self.scale).contains(world)).unwrap_or(false);
                        let hit_card = self.hit_test_clickable_card(world);

                        if over_selection {
                            self.secondary_drag_is_item = true;
                            self.is_dragging_selection = true;
                            self.active_resize_handle = None;
                            self.drag_last_pos_world = world;
                            self.prepare_selection_drag();
                        } else if let Some((kind, id)) = hit_card {
                            self.clear_selection();
                            match kind {
                                "plot" => { self.selected_plot_ids.insert(id); }
                                "matrix" => { self.selected_matrix_ids.insert(id); }
                                "grid" => { self.selected_grid_ids.insert(id); }
                                "flowchart" => { self.selected_flowchart_ids.insert(id); }
                                "image" => { self.selected_image_ids.insert(id); }
                                _ => {}
                            }
                            self.secondary_drag_is_item = true;
                            self.is_dragging_selection = true;
                            self.active_resize_handle = None;
                            self.drag_last_pos_world = world;
                            self.prepare_selection_drag();
                        } else {
                            self.secondary_drag_is_item = false;
                        }
                    }
                }

                if pointer.secondary_down() && self.secondary_drag_start.is_some() {
                    if let Some(cursor) = pointer_pos {
                        if self.secondary_drag_is_item && self.is_dragging_selection {
                            let world = self.to_world(cursor);
                            let delta = world - self.drag_last_pos_world;
                            self.drag_last_pos_world = world;
                            self.apply_selection_delta(delta);
                            ctx.set_cursor_icon(egui::CursorIcon::Move);
                        } else if !self.secondary_drag_is_item {
                            self.pan += pointer.delta();
                            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                        }
                    }
                }

                if pointer.secondary_released() {
                    if let Some(start_pos) = self.secondary_drag_start.take() {
                        let end_pos = pointer_pos.unwrap_or(start_pos);
                        let dist = start_pos.distance(end_pos);
                        if self.secondary_drag_is_item {
                            self.finish_selection_drag();
                            if dist > 8.0 {
                                self.set_status("Moved object");
                            }
                        }
                    }
                }

                let is_space_down = ctx.input(|i| i.key_down(egui::Key::Space));
                if is_space_down {
                    if is_primary_down {
                        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else {
                        ctx.set_cursor_icon(egui::CursorIcon::Grab);
                    }
                }

                // Middle mouse drag, Pan tool, or Space + Left drag (Space pan)
                if (self.tool == ToolMode::Pan && is_primary_down) || pointer.middle_down() || (is_space_down && is_primary_down) {
                    self.pan += pointer.delta();
                } else if !ruler_consumed && !is_space_down && is_primary_pressed && self.canvas_gesture {
                    if let Some(cursor) = pointer_pos {
                        let effective_cursor = self.ruler.snap_point(cursor).unwrap_or(cursor);
                        let world = self.to_world(effective_cursor);

                        // Check if user clicked on the close button or edit button of any plot, matrix, or table grid
                        let mut delete_plot_idx = None;
                        let mut delete_mat_idx = None;
                        let mut delete_grid_idx = None;
                        let mut edit_plot_idx = None;
                        let mut edit_mat_idx = None;
                        let mut edit_grid_idx = None;
                        {
                            let page = self.notebook.current_page();
                            let scale = self.scale;

                            // 1. Plot close and edit button
                            for (idx, plot) in page.plots.iter().enumerate() {
                                let card_min = self.to_screen(Pos2::new(plot.x, plot.y));
                                let card_size = Vec2::new(plot.width * scale, plot.height * scale);
                                let card_rect = Rect::from_min_size(card_min, card_size);
                                let btn_size = (20.0 * scale).clamp(16.0, 24.0);
                                let close_center = card_rect.right_top() + Vec2::new(-btn_size * 0.75, btn_size * 0.75);
                                let edit_center = close_center - Vec2::new(btn_size * 1.05, 0.0);
                                if cursor.distance(close_center) <= (btn_size * 0.85).max(16.0) {
                                    delete_plot_idx = Some(idx);
                                    break;
                                }
                                if cursor.distance(edit_center) <= (btn_size * 0.85).max(16.0) {
                                    edit_plot_idx = Some(idx);
                                    break;
                                }
                            }

                            // 2. Matrix close and edit button
                            if delete_plot_idx.is_none() && edit_plot_idx.is_none() {
                                for (idx, mat) in page.matrices.iter().enumerate() {
                                    let card_min = self.to_screen(Pos2::new(mat.x, mat.y));
                                    let card_size = Vec2::new(mat.width * scale, mat.height * scale);
                                    let card_rect = Rect::from_min_size(card_min, card_size);
                                    let btn_size = (18.0 * scale).clamp(14.0, 22.0);
                                    let close_center = card_rect.right_top() + Vec2::new(-btn_size * 0.65, btn_size * 0.65);
                                    let edit_center = close_center - Vec2::new(btn_size * 1.05, 0.0);
                                    if cursor.distance(close_center) <= (btn_size * 0.85).max(16.0) {
                                        delete_mat_idx = Some(idx);
                                        break;
                                    }
                                    if cursor.distance(edit_center) <= (btn_size * 0.85).max(16.0) {
                                        edit_mat_idx = Some(idx);
                                        break;
                                    }
                                }
                            }

                            // 3. Table grid close and edit button
                            if delete_plot_idx.is_none() && edit_plot_idx.is_none() && delete_mat_idx.is_none() && edit_mat_idx.is_none() {
                                for (idx, grid) in page.grids.iter().enumerate() {
                                    let origin = self.to_screen(Pos2::new(grid.x, grid.y));
                                    let total_w = grid.col_width * grid.cols as f32 * scale;
                                    let total_h = grid.row_height * grid.rows as f32 * scale;
                                    let grid_rect = Rect::from_min_size(origin, Vec2::new(total_w, total_h));
                                    let btn_size = (18.0 * scale).clamp(14.0, 22.0);
                                    let close_center = grid_rect.right_top() + Vec2::new(-btn_size * 0.75, btn_size * 0.75);
                                    let edit_center = close_center - Vec2::new(btn_size * 1.05, 0.0);
                                    if cursor.distance(close_center) <= (btn_size * 0.85).max(16.0) {
                                        delete_grid_idx = Some(idx);
                                        break;
                                    }
                                    if cursor.distance(edit_center) <= (btn_size * 0.85).max(16.0) {
                                        edit_grid_idx = Some(idx);
                                        break;
                                    }
                                }
                            }
                        }

                        if let Some(idx) = edit_plot_idx {
                            let plot = self.notebook.current_page().plots[idx].clone();
                            self.axes_dialog.open_edit(&plot);
                        } else if let Some(idx) = edit_mat_idx {
                            let mat = self.notebook.current_page().matrices[idx].clone();
                            self.matrix_dialog.open_edit(&mat);
                        } else if let Some(idx) = edit_grid_idx {
                            let grid = self.notebook.current_page().grids[idx].clone();
                            self.table_dialog.open_edit(&grid);
                        } else if let Some(idx) = delete_plot_idx {
                            self.notebook.current_page_mut().plots.remove(idx);
                            self.clear_selection();
                            self.set_status("Deleted plot");
                        } else if let Some(idx) = delete_mat_idx {
                            self.notebook.current_page_mut().matrices.remove(idx);
                            self.clear_selection();
                            self.set_status("Deleted matrix");
                        } else if let Some(idx) = delete_grid_idx {
                            self.notebook.current_page_mut().grids.remove(idx);
                            self.clear_selection();
                            self.set_status("Deleted table");
                        } else if let Some((port_node, port_side)) = self.flowchart_state.hovered_port {
                            // User clicked an anchor connection dot!
                            self.flowchart_state.active_port_menu = Some((port_node, port_side, cursor));
                            self.flowchart_state.selected_node_id = Some(port_node);
                            self.canvas_gesture = false;
                        } else if let Some(hit_node) = self.flowchart_state.hovered_node_id {
                            // User clicked on a flowchart node body directly on the whiteboard
                            let is_double_click = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));
                            if is_double_click {
                                self.flowchart_state.editing_node_id = Some(hit_node);
                                for flow in &self.notebook.current_page().flowcharts {
                                    if let Some(g) = &flow.graph {
                                        if let Some(n) = g.node(hit_node) {
                                            self.flowchart_state.edit_label_buffer = n.label.clone();
                                        }
                                    }
                                }
                            } else {
                                self.flowchart_state.selected_node_id = Some(hit_node);
                                self.flowchart_state.dragging_node = Some((hit_node, world));
                            }
                            self.flowchart_state.active_port_menu = None;
                            self.canvas_gesture = false;
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
                                    hit_handle_opt = hit_handle(cursor, sb, 16.0);
                                }

                                if let Some(handle) = hit_handle_opt {
                                    self.active_resize_handle = Some(handle);
                                    self.is_dragging_selection = true;
                                    self.drag_last_pos_world = world;
                                    self.prepare_selection_drag();
                                } else if let Some(wb) = self.get_selection_bounds() {
                                    let sb = Rect::from_min_max(self.to_screen(wb.min), self.to_screen(wb.max)).expand(8.0);
                                    let move_bar_rect = Rect::from_min_max(
                                        Pos2::new(sb.left() - 8.0, sb.top() - 38.0),
                                        Pos2::new(sb.right().max(sb.left() + 280.0) + 8.0, sb.top() + 6.0),
                                    );
                                    if sb.expand(16.0).contains(cursor) || move_bar_rect.contains(cursor) || wb.expand(16.0 / self.scale).contains(world) {
                                        self.active_resize_handle = None;
                                        self.is_dragging_selection = true;
                                        self.drag_last_pos_world = world;
                                        self.prepare_selection_drag();
                                    } else {
                                        // Clicked outside existing selection: start a fresh lasso gesture!
                                        self.clear_selection();
                                        self.is_dragging_selection = false;
                                        self.lasso_points = vec![world];
                                    }
                                } else {
                                    // No current selection: start a fresh lasso gesture!
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
                } else if !ruler_consumed && !is_space_down && is_primary_down {
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

                                            for (id, orig_x, orig_y, orig_w, orig_h) in &self.initial_flowchart_rects {
                                                if let Some(flow) = page.flowcharts.iter_mut().find(|f| f.id == *id) {
                                                    flow.x = new_min.x + (orig_x - orig.min.x) * scale_x;
                                                    flow.y = new_min.y + (orig_y - orig.min.y) * scale_y;
                                                    flow.width = (orig_w * scale_x).max(80.0);
                                                    flow.height = (orig_h * scale_y).max(60.0);
                                                }
                                            }
                                        }
                                    } else {
                                        let delta = world - self.drag_last_pos_world;
                                        self.drag_last_pos_world = world;
                                        self.apply_selection_delta(delta);
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
                                            self.set_status("Page auto-expanded!");
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
                                self.set_status("Page auto-expanded!");
                            }
                        }
                    }

                    if self.is_dragging_selection {
                        self.finish_selection_drag();
                    } else if !self.lasso_points.is_empty() {
                        // Compute bounding box and diagonal span of the lasso gesture
                        let mut l_min_x = f32::MAX;
                        let mut l_min_y = f32::MAX;
                        let mut l_max_x = f32::MIN;
                        let mut l_max_y = f32::MIN;
                        for pt in &self.lasso_points {
                            l_min_x = l_min_x.min(pt.x);
                            l_max_x = l_max_x.max(pt.x);
                            l_min_y = l_min_y.min(pt.y);
                            l_max_y = l_max_y.max(pt.y);
                        }
                        let lasso_bbox = Rect::from_min_max(Pos2::new(l_min_x, l_min_y), Pos2::new(l_max_x, l_max_y));
                        let gesture_diag = (lasso_bbox.width() * lasso_bbox.width() + lasso_bbox.height() * lasso_bbox.height()).sqrt();

                        // Treat as a real lasso loop only if it has >= 3 points and a movement span >= 10 world units
                        let is_lasso_loop = self.lasso_points.len() >= 3 && gesture_diag >= (10.0 / self.scale).max(6.0);

                        if is_lasso_loop {
                            let mut selected_strokes = HashSet::new();
                            let mut selected_images = HashSet::new();
                            let mut selected_plots = HashSet::new();
                            let mut selected_matrices = HashSet::new();
                            let mut selected_grids = HashSet::new();
                            let mut selected_flowcharts = HashSet::new();
                            let page = self.notebook.current_page();

                            // 1. Precise Stroke Enclosure Algorithm (enclosed majority + centroid)
                            for stroke in &page.strokes {
                                if stroke.points.is_empty() {
                                    continue;
                                }
                                let mut s_min_x = f32::MAX;
                                let mut s_min_y = f32::MAX;
                                let mut s_max_x = f32::MIN;
                                let mut s_max_y = f32::MIN;
                                let mut sum_x = 0.0f32;
                                let mut sum_y = 0.0f32;
                                for pt in &stroke.points {
                                    s_min_x = s_min_x.min(pt.x);
                                    s_max_x = s_max_x.max(pt.x);
                                    s_min_y = s_min_y.min(pt.y);
                                    s_max_y = s_max_y.max(pt.y);
                                    sum_x += pt.x;
                                    sum_y += pt.y;
                                }
                                let s_bbox = Rect::from_min_max(Pos2::new(s_min_x, s_min_y), Pos2::new(s_max_x, s_max_y));
                                if !lasso_bbox.intersects(s_bbox) {
                                    continue;
                                }

                                let n_pts = stroke.points.len();
                                let centroid = Pos2::new(sum_x / n_pts as f32, sum_y / n_pts as f32);

                                let pts_inside = stroke
                                    .points
                                    .iter()
                                    .filter(|pt| point_in_polygon(Pos2::new(pt.x, pt.y), &self.lasso_points))
                                    .count();

                                let is_selected = if n_pts <= 4 {
                                    // For small dots, punctuation, diacritics: centroid inside or >= 50% points inside
                                    pts_inside >= (n_pts + 1) / 2 || point_in_polygon(centroid, &self.lasso_points)
                                } else {
                                    let ratio = pts_inside as f32 / n_pts as f32;
                                    if ratio >= 0.55 {
                                        true
                                    } else if ratio >= 0.40 && point_in_polygon(centroid, &self.lasso_points) {
                                        true
                                    } else {
                                        false
                                    }
                                };

                                if is_selected {
                                    selected_strokes.insert(stroke.id.clone());
                                }
                            }

                            // 2. Precise Card Enclosure Algorithm (prevents accidental card selection when circling nearby/inside notes)
                            let card_encircled = |card_rect: Rect, lasso: &[Pos2], l_bbox: Rect| -> bool {
                                if card_rect.width() > l_bbox.width() * 1.25 || card_rect.height() > l_bbox.height() * 1.25 {
                                    return false;
                                }
                                let tl = card_rect.left_top();
                                let tr = card_rect.right_top();
                                let bl = card_rect.left_bottom();
                                let br = card_rect.right_bottom();
                                let center = card_rect.center();
                                let corners = [tl, tr, bl, br];
                                let corners_inside = corners.iter().filter(|c| point_in_polygon(**c, lasso)).count();
                                if corners_inside == 4 {
                                    true
                                } else if corners_inside >= 3 && point_in_polygon(center, lasso) {
                                    true
                                } else {
                                    false
                                }
                            };

                            for img in &page.images {
                                let rect = Rect::from_min_size(Pos2::new(img.x, img.y), Vec2::new(img.width, img.height));
                                if card_encircled(rect, &self.lasso_points, lasso_bbox) {
                                    selected_images.insert(img.id.clone());
                                }
                            }
                            for plot in &page.plots {
                                let rect = Rect::from_min_size(Pos2::new(plot.x, plot.y), Vec2::new(plot.width, plot.height));
                                if card_encircled(rect, &self.lasso_points, lasso_bbox) {
                                    selected_plots.insert(plot.id.clone());
                                }
                            }
                            for m in &page.matrices {
                                let rect = Rect::from_min_size(Pos2::new(m.x, m.y), Vec2::new(m.width, m.height));
                                if card_encircled(rect, &self.lasso_points, lasso_bbox) {
                                    selected_matrices.insert(m.id.clone());
                                }
                            }
                            for g in &page.grids {
                                let rect = Rect::from_min_size(Pos2::new(g.x, g.y), Vec2::new(g.total_width(), g.total_height()));
                                if card_encircled(rect, &self.lasso_points, lasso_bbox) {
                                    selected_grids.insert(g.id.clone());
                                }
                            }
                            for f in &page.flowcharts {
                                let rect = Rect::from_min_size(Pos2::new(f.x, f.y), Vec2::new(f.width, f.height));
                                if card_encircled(rect, &self.lasso_points, lasso_bbox) {
                                    selected_flowcharts.insert(f.id.clone());
                                }
                            }
                            let count = selected_strokes.len() + selected_images.len() + selected_plots.len() + selected_matrices.len() + selected_grids.len() + selected_flowcharts.len();
                            self.selected_stroke_ids = selected_strokes;
                            self.selected_image_ids = selected_images;
                            self.selected_plot_ids = selected_plots;
                            self.selected_matrix_ids = selected_matrices;
                            self.selected_grid_ids = selected_grids;
                            self.selected_flowchart_ids = selected_flowcharts;
                            if count > 0 {
                                self.set_status(&format!("{} item(s) selected (drag handles to resize, drag body to move)", count));
                            } else {
                                self.clear_selection();
                            }
                        } else {
                            // Single click: hit test cards first, then nearby stroke
                            let click_pos = self.lasso_points[0];
                            if let Some((kind, id)) = self.hit_test_clickable_card(click_pos) {
                                self.clear_selection();
                                match kind {
                                    "plot" => {
                                        self.selected_plot_ids.insert(id);
                                        self.set_status("Selected 1 plot (drag handles to resize)");
                                    }
                                    "matrix" => {
                                        self.selected_matrix_ids.insert(id);
                                        self.set_status("Selected 1 matrix (drag handles to resize)");
                                    }
                                    "grid" => {
                                        self.selected_grid_ids.insert(id);
                                        self.set_status("Selected 1 table grid (drag handles to resize)");
                                    }
                                    "flowchart" => {
                                        self.selected_flowchart_ids.insert(id.clone());
                                        self.flowchart_state.active_flowchart_id = Some(id);
                                        self.set_status("Selected flowchart on whiteboard");
                                    }
                                    "image" => {
                                        self.selected_image_ids.insert(id);
                                        self.set_status("Selected 1 image (drag handles to resize)");
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
                                self.set_status("Page auto-expanded!");
                            }
                            self.notebook.current_page_mut().strokes.push(stroke);
                            self.autosave.mark_dirty();
                        }
                    }
                }

                // Right-Click Context Menu for quick access & mouse shortcuts
                let mut action_copy = false;
                let mut action_cut = false;
                let mut action_duplicate = false;
                let mut action_select_all = false;
                let mut action_paste = false;
                let mut action_insert = false;
                let mut action_delete = false;
                let mut action_clear_sel = false;
                let mut action_move_page_up = false;
                let mut action_move_page_down = false;
                let mut action_center_page = false;
                let mut action_set_tool = None;
                // 1. Flowchart Node Dragging
                if let Some((drag_node_id, drag_start_world)) = self.flowchart_state.dragging_node {
                    if pointer.primary_down() {
                        if let Some(cursor) = pointer_pos {
                            let world = self.to_world(cursor);
                            let delta = world - drag_start_world;
                            self.flowchart_state.dragging_node = Some((drag_node_id, world));
                            let page_mut = self.notebook.current_page_mut();
                            for flow in &mut page_mut.flowcharts {
                                if let Some(graph) = &mut flow.graph {
                                    if let Some(n) = graph.node_mut(drag_node_id) {
                                        if let Some(ref mut pin) = n.pinned_pos {
                                            pin.0 = (pin.0 + delta.x).max(10.0);
                                            pin.1 = (pin.1 + delta.y).max(10.0);
                                        }
                                        let (_, _, max_x, max_y) = graph.compute_bounds();
                                        flow.width = (max_x + 60.0).max(400.0);
                                        flow.height = (max_y + 60.0).max(300.0);
                                        break;
                                    }
                                }
                            }
                        }
                    } else {
                        self.flowchart_state.dragging_node = None;
                    }
                }

                // 2. Flowchart Keyboard Delete
                let key_del = ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
                if key_del && self.flowchart_state.selected_node_id.is_some() && self.editing_text_id.is_none() {
                    let sel_id = self.flowchart_state.selected_node_id.take().unwrap();
                    let page_mut = self.notebook.current_page_mut();
                    for flow in &mut page_mut.flowcharts {
                        if let Some(graph) = &mut flow.graph {
                            graph.remove_node(sel_id);
                        }
                    }
                    self.set_status("Deleted flowchart block");
                }

                // 3. Flowchart Interactive Actions & Quick Picker Popover
                if self.flowchart_state.active_port_menu.is_some()
                    || self.flowchart_state.applied_label_change.is_some()
                    || self.flowchart_state.req_delete_node_id.is_some()
                {
                    let page_mut = self.notebook.current_page_mut();
                    let active_fid = self.flowchart_state.active_flowchart_id.clone();
                    if let Some(fid) = active_fid {
                        if let Some(flow) = page_mut.flowcharts.iter_mut().find(|f| f.id == fid) {
                            crate::flowchart_ui::FlowchartRenderer::handle_popover_and_actions(
                                ui,
                                flow,
                                &mut self.flowchart_state,
                                &p,
                                is_dark,
                            );
                        }
                    } else if let Some(flow) = page_mut.flowcharts.first_mut() {
                        crate::flowchart_ui::FlowchartRenderer::handle_popover_and_actions(
                            ui,
                            flow,
                            &mut self.flowchart_state,
                            &p,
                            is_dark,
                        );
                    }
                }

                let mut action_toggle_math = false;
                let mut action_toggle_ruler = false;
                let mut action_toggle_protractor = false;
                let mut action_open_matrix = false;
                let mut action_open_axes = false;
                let mut action_open_table = false;
                let mut action_bake_flowchart = false;
                let mut action_save_flowchart_svg = false;
                let mut action_undo = false;
                let mut action_redo = false;
                let mut action_reset_zoom = false;

                let has_selection = !self.selected_stroke_ids.is_empty()
                    || !self.selected_image_ids.is_empty()
                    || !self.selected_plot_ids.is_empty()
                    || !self.selected_matrix_ids.is_empty()
                    || !self.selected_grid_ids.is_empty();
                let cur_tool = self.tool;
                let is_math_open = self.math_studio.is_open;
                let is_ruler_on = self.ruler.kind == RulerKind::Ruler;
                let is_protractor_on = self.ruler.kind == RulerKind::Protractor;
                let p = self.palette();

                response.context_menu(|ui| {
                    ui.set_min_width(220.0);

                    ui.label(egui::RichText::new("Quick Actions").strong().color(p.accent));
                    ui.separator();

                    if has_selection {
                        if ui.button("Copy Selected (Ctrl+C)").clicked() {
                            action_copy = true;
                            ui.close_menu();
                        }
                        if ui.button("Cut Selected (Ctrl+X)").clicked() {
                            action_cut = true;
                            ui.close_menu();
                        }
                        if ui.button("Duplicate Selected (Ctrl+D)").clicked() {
                            action_duplicate = true;
                            ui.close_menu();
                        }
                    } else if !self.notebook.current_page().strokes.is_empty() {
                        if ui.button("Copy All Strokes (Ctrl+C)").clicked() {
                            action_copy = true;
                            ui.close_menu();
                        }
                    }

                    if ui.button("Paste to Whiteboard (Ctrl+V)").clicked() {
                        action_paste = true;
                        ui.close_menu();
                    }
                    if ui.button("Select All (Ctrl+A)").clicked() {
                        action_select_all = true;
                        ui.close_menu();
                    }
                    if ui.button("Insert Image from file...").clicked() {
                        action_insert = true;
                        ui.close_menu();
                    }

                    if has_selection {
                        if ui.button("Delete selected (Del)").clicked() {
                            action_delete = true;
                            ui.close_menu();
                        }
                        if ui.button("Deselect (Esc)").clicked() {
                            action_clear_sel = true;
                            ui.close_menu();
                        }
                    }

                    if !self.selected_flowchart_ids.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new("Flowchart Actions").small().color(p.accent));
                        if ui.button("Convert Flowchart to Image on Board").clicked() {
                            action_bake_flowchart = true;
                            ui.close_menu();
                        }
                        if ui.button("Save Flowchart as SVG file...").clicked() {
                            action_save_flowchart_svg = true;
                            ui.close_menu();
                        }
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Drawing Tools").small().color(p.text_muted));

                    if ui.selectable_label(cur_tool == ToolMode::Pen, "Pen (1)").clicked() {
                        action_set_tool = Some(ToolMode::Pen);
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur_tool == ToolMode::Highlighter, "Highlighter (2)").clicked() {
                        action_set_tool = Some(ToolMode::Highlighter);
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur_tool == ToolMode::EraserStroke, "Stroke Eraser (5)").clicked() {
                        action_set_tool = Some(ToolMode::EraserStroke);
                        ui.close_menu();
                    }
                    if ui.selectable_label(cur_tool == ToolMode::LassoSelect, "Select & Move (Lasso) (7 / L)").clicked() {
                        action_set_tool = Some(ToolMode::LassoSelect);
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Math & Tools").small().color(p.text_muted));

                    let math_title = if is_math_open { "Close Math Studio" } else { "Open Math Studio (Ctrl+M)" };
                    if ui.button(math_title).clicked() {
                        action_toggle_math = true;
                        ui.close_menu();
                    }

                    let ruler_title = if is_ruler_on { "Hide Ruler" } else { "Show Ruler (R)" };
                    if ui.button(ruler_title).clicked() {
                        action_toggle_ruler = true;
                        ui.close_menu();
                    }

                    let protractor_title = if is_protractor_on { "Hide Protractor" } else { "Show Protractor (Shift+R)" };
                    if ui.button(protractor_title).clicked() {
                        action_toggle_protractor = true;
                        ui.close_menu();
                    }

                    if ui.button("Insert Matrix (Code / UI)...").clicked() {
                        action_open_matrix = true;
                        ui.close_menu();
                    }
                    if ui.button("Insert Coordinate Axes (1D/2D/3D)...").clicked() {
                        action_open_axes = true;
                        ui.close_menu();
                    }
                    if ui.button("Insert Data Table...").clicked() {
                        action_open_table = true;
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Page & Position").small().color(p.text_muted));
                    let cur_idx = self.notebook.current_page_index;
                    let total_pages = self.notebook.pages.len();
                    if cur_idx > 0 {
                        if ui.button("Move Page Earlier (Alt+Up)").clicked() {
                            action_move_page_up = true;
                            ui.close_menu();
                        }
                    }
                    if cur_idx + 1 < total_pages {
                        if ui.button("Move Page Later (Alt+Down)").clicked() {
                            action_move_page_down = true;
                            ui.close_menu();
                        }
                    }
                    if ui.button("Center Page on Screen (Home)").clicked() {
                        action_center_page = true;
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Mouse & Shortcuts").small().color(p.text_muted));

                    if ui.button("Undo (Mouse 4 / Ctrl+Z)").clicked() {
                        action_undo = true;
                        ui.close_menu();
                    }
                    if ui.button("Redo (Mouse 5 / Ctrl+Y)").clicked() {
                        action_redo = true;
                        ui.close_menu();
                    }
                    if ui.button("100% Zoom (Ctrl+0 / Middle click)").clicked() {
                        action_reset_zoom = true;
                        ui.close_menu();
                    }
                    if ui.button("Keyboard Shortcuts (F1)...").clicked() {
                        self.shortcuts_dialog.open();
                        ui.close_menu();
                    }
                    if ui.button("Quick Tour (Walkthrough)...").clicked() {
                        self.onboarding_tour.start();
                        ui.close_menu();
                    }

                    ui.separator();
                    let fs_title = if self.is_fullscreen {
                        "Exit Fullscreen (F11)"
                    } else {
                        "Fullscreen (F11)"
                    };
                    if ui.button(fs_title).clicked() {
                        self.toggle_fullscreen(ctx);
                        ui.close_menu();
                    }
                    if ui.button("Check for Updates...").clicked() {
                        self.updater.open_dialog();
                        ui.close_menu();
                    }
                });

                if action_copy {
                    self.copy_selected();
                }
                if action_cut {
                    self.cut_selected();
                }
                if action_duplicate {
                    self.duplicate_selected();
                }
                if action_select_all {
                    self.select_all();
                }
                if action_paste {
                    self.paste_from_clipboard();
                }
                if action_insert {
                    self.prompt_insert_image();
                }
                if action_delete {
                    self.delete_selected();
                }
                if action_clear_sel {
                    self.clear_selection();
                }
                if action_move_page_up {
                    let cur = self.notebook.current_page_index;
                    if self.notebook.move_page_up(cur) {
                        self.set_status(&format!("Moved page to position {}", cur + 1));
                        self.autosave.mark_dirty();
                    }
                }
                if action_move_page_down {
                    let cur = self.notebook.current_page_index;
                    if self.notebook.move_page_down(cur) {
                        self.set_status(&format!("Moved page to position {}", cur + 2));
                        self.autosave.mark_dirty();
                    }
                }
                if action_center_page {
                    self.center_page_in_view(available_size);
                }
                if let Some(tool) = action_set_tool {
                    self.set_tool(tool);
                }
                if action_toggle_math {
                    self.math_studio.toggle();
                }
                if action_toggle_ruler {
                    self.ruler.toggle_ruler();
                }
                if action_toggle_protractor {
                    self.ruler.toggle_protractor();
                }
                if action_open_matrix {
                    self.matrix_dialog.open_new();
                }
                if action_open_axes {
                    self.axes_dialog.open_new();
                }
                if action_open_table {
                    self.table_dialog.open_new();
                }
                if action_bake_flowchart {
                    if let Some(fid) = self.selected_flowchart_ids.iter().next() {
                        if let Some(flow) = self.notebook.current_page().flowcharts.iter().find(|f| f.id == *fid) {
                            let graph = if let Some(g) = &flow.graph {
                                g.clone()
                            } else {
                                let (ast, _) = crate::flowchart_core::parser::Parser::from_str(&flow.dsl_source).parse();
                                crate::flowchart_core::ir::FlowchartGraph::from_ast(&ast)
                            };
                            let svg = crate::flowchart_core::export::FlowchartExporter::to_svg(&graph, None, None);
                            if let Some((img_bytes, w, h)) = Self::decode_image_or_svg(svg.as_bytes()) {
                                self.insert_image_bytes(img_bytes, w, h);
                                self.set_status("Flowchart converted and placed on whiteboard as an image!");
                            } else {
                                self.set_error("Could not render flowchart SVG into image.");
                            }
                        }
                    }
                }
                if action_save_flowchart_svg {
                    if let Some(fid) = self.selected_flowchart_ids.iter().next() {
                        if let Some(flow) = self.notebook.current_page().flowcharts.iter().find(|f| f.id == *fid) {
                            let graph = if let Some(g) = &flow.graph {
                                g.clone()
                            } else {
                                let (ast, _) = crate::flowchart_core::parser::Parser::from_str(&flow.dsl_source).parse();
                                crate::flowchart_core::ir::FlowchartGraph::from_ast(&ast)
                            };
                            let svg = crate::flowchart_core::export::FlowchartExporter::to_svg(&graph, None, None);
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Scalable Vector Graphics (*.svg)", &["svg"])
                                .set_file_name("flowchart.svg")
                                .save_file()
                            {
                                if std::fs::write(&path, svg).is_ok() {
                                    self.set_status(&format!("Saved flowchart SVG to {}", path.display()));
                                } else {
                                    self.set_error("Failed to write SVG file.");
                                }
                            }
                        }
                    }
                }
                if action_undo {
                    self.undo();
                }
                if action_redo {
                    self.redo();
                }
                if action_reset_zoom {
                    self.scale = 1.0;
                    self.set_status("Zoom: 100%");
                }
            });

        // Minimalist Centered Splash Dialog
        let recent_splash_items: Vec<crate::splash_screen::SplashRecentItem> = self
            .library
            .items
            .iter()
            .take(5)
            .map(|item| crate::splash_screen::SplashRecentItem {
                path: item.path.clone(),
                title: item.title.clone(),
                modified_str: item.format_modified(),
            })
            .collect();

        let has_recoverable_session = crate::autosave::AutosaveManager::check_recovery_available().is_some();

        if let Some(act) = self.splash_screen.render(ctx, &recent_splash_items, has_recoverable_session) {
            match act {
                crate::splash_screen::SplashAction::NewBlank => {
                    self.notebook.current_page_mut().paper_type = PaperType::Blank;
                    self.set_status("Started with Blank Canvas");
                }
                crate::splash_screen::SplashAction::NewRuled => {
                    self.notebook.current_page_mut().paper_type = PaperType::Ruled;
                    self.set_status("Started with Ruled Paper");
                }
                crate::splash_screen::SplashAction::NewGrid => {
                    self.notebook.current_page_mut().paper_type = PaperType::Grid;
                    self.set_status("Started with Grid Paper");
                }
                crate::splash_screen::SplashAction::NewCornell => {
                    self.notebook.current_page_mut().paper_type = PaperType::Cornell;
                    self.set_status("Started with Cornell Notes");
                }
                crate::splash_screen::SplashAction::OpenMathStudio => {
                    self.math_studio.is_open = true;
                    self.set_status("Math Studio OPEN");
                }
                crate::splash_screen::SplashAction::OpenFlowchart => {
                    self.flowchart_state.is_panel_open = true;
                    self.set_status("Flowchart Shapes Menu OPEN");
                }
                crate::splash_screen::SplashAction::OpenRecent(path) => {
                    if let Err(e) = self.open_notebook_from_path(&path) {
                        self.set_error(&format!("Could not open notebook: {}", e));
                    }
                }
                crate::splash_screen::SplashAction::OpenLibrary => {
                    self.library.is_open = true;
                }
                crate::splash_screen::SplashAction::RecoverLastSession => {
                    self.recover_last_session();
                    self.autosave.recovery_restored_banner = None;
                }
                crate::splash_screen::SplashAction::OpenFilePicker => {
                    self.prompt_open_file();
                }
                crate::splash_screen::SplashAction::OpenKeyboardShortcuts => {
                    self.shortcuts_dialog.open();
                }
                crate::splash_screen::SplashAction::StartOnboardingTour => {
                    self.onboarding_tour.start();
                }
            }
        }

        // Keyboard Shortcuts interactive configuration window
        self.shortcuts_dialog.render(ctx, &mut self.shortcut_manager);

        // Onboarding Tour interactive walkthrough overlay
        self.onboarding_tour.render(ctx);

        // Auto-Updater notification modal
        self.updater.render(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_svg_vector() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><circle cx="50" cy="50" r="40" stroke="green" stroke-width="4" fill="yellow" /></svg>"#;
        let res = InkkApp::decode_image_or_svg(svg.as_bytes());
        assert!(
            res.is_some(),
            "SVG should be successfully decoded to raster image bytes"
        );
        let (bytes, w, h) = res.unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(w, 100.0);
        assert_eq!(h, 100.0);
    }

    #[test]
    fn test_pointer_buttons() {
        let _ = egui::PointerButton::Primary;
        let _ = egui::PointerButton::Secondary;
        let _ = egui::PointerButton::Middle;
        let _ = egui::PointerButton::Extra1;
        let _ = egui::PointerButton::Extra2;
    }

    #[test]
    fn test_point_in_polygon_and_lasso_precision() {
        let lasso = vec![
            Pos2::new(100.0, 100.0),
            Pos2::new(200.0, 100.0),
            Pos2::new(200.0, 200.0),
            Pos2::new(100.0, 200.0),
        ];

        // Points inside
        assert!(point_in_polygon(Pos2::new(150.0, 150.0), &lasso));
        assert!(point_in_polygon(Pos2::new(110.0, 110.0), &lasso));

        // Points outside
        assert!(!point_in_polygon(Pos2::new(50.0, 50.0), &lasso));
        assert!(!point_in_polygon(Pos2::new(250.0, 150.0), &lasso));
        assert!(!point_in_polygon(Pos2::new(150.0, 250.0), &lasso));

        // Stroke 1: Fully enclosed in lasso (intended target)
        let stroke_target: Vec<Pos2> = (0..10).map(|i| Pos2::new(120.0 + i as f32 * 5.0, 150.0)).collect();
        let inside_count_target = stroke_target.iter().filter(|p| point_in_polygon(**p, &lasso)).count();
        let ratio_target = inside_count_target as f32 / stroke_target.len() as f32;
        assert_eq!(ratio_target, 1.0, "Encircled target stroke must have 100% points inside");
        assert!(ratio_target >= 0.55);

        // Stroke 2: Unrelated stroke passing nearby that only grazes 1 point into the lasso area
        let stroke_unrelated: Vec<Pos2> = (0..20).map(|i| Pos2::new(195.0 + i as f32 * 5.0, 150.0)).collect();
        // Only point at index 0 (195.0, 150.0) is inside, the remaining 19 are outside (200.0 .. 290.0)
        let inside_count_unrelated = stroke_unrelated.iter().filter(|p| point_in_polygon(**p, &lasso)).count();
        let ratio_unrelated = inside_count_unrelated as f32 / stroke_unrelated.len() as f32;
        assert_eq!(inside_count_unrelated, 1);
        assert_eq!(ratio_unrelated, 0.05);
        assert!(ratio_unrelated < 0.55, "Unrelated grazing stroke (5% inside) must be rejected by precision algorithm");
    }

    #[test]
    fn test_card_encircling_filter() {
        let lasso_bbox = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(200.0, 200.0));
        let lasso = vec![
            Pos2::new(100.0, 100.0),
            Pos2::new(200.0, 100.0),
            Pos2::new(200.0, 200.0),
            Pos2::new(100.0, 200.0),
        ];

        let card_encircled = |card_rect: Rect, l: &[Pos2], l_bbox: Rect| -> bool {
            if card_rect.width() > l_bbox.width() * 1.25 || card_rect.height() > l_bbox.height() * 1.25 {
                return false;
            }
            let corners = [
                card_rect.left_top(),
                card_rect.right_top(),
                card_rect.left_bottom(),
                card_rect.right_bottom(),
            ];
            let corners_inside = corners.iter().filter(|c| point_in_polygon(**c, l)).count();
            corners_inside == 4 || (corners_inside >= 3 && point_in_polygon(card_rect.center(), l))
        };

        // Case 1: Large table / plot (400x300) near or under a small lasso (100x100)
        let large_table = Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(400.0, 300.0));
        assert!(!card_encircled(large_table, &lasso, lasso_bbox), "Large background card must NEVER be selected when lassoing a small note");

        // Case 2: Card deliberately encircled by a larger lasso
        let small_card = Rect::from_min_size(Pos2::new(120.0, 120.0), Vec2::new(50.0, 50.0));
        assert!(card_encircled(small_card, &lasso, lasso_bbox), "Card fully surrounded by lasso must be selected");
    }
}
