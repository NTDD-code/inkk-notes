//! Vector icon rendering for Inkk based on 24x24 SVG specifications.
//!
//! Provides pixel-perfect vector icons for:
//! - Save
//! - Pages
//! - Open
//! - MathStudio
//! - Matrix
//! - Coordinate Axes
//! - Data Table
//!
//! All UI labels and tooltips in Inkk are in English.

use eframe::egui::{Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use std::sync::OnceLock;

pub const SAVE_PATH: &str =
    "M5.5 3.5H16L20.5 8V18.5C20.5 19.6 19.6 20.5 18.5 20.5H5.5C4.4 20.5 3.5 19.6 3.5 18.5V5.5C3.5 4.4 4.4 3.5 5.5 3.5Z M7.5 3.5V8H14.5V3.5 M7 20.5V14H17V20.5";

pub const PAGES_PATH: &str =
    "M6 3.5H14L19 8.5V18.5C19 19.6 18.1 20.5 17 20.5H7C5.9 20.5 5 19.6 5 18.5V5C5 4.2 5.4 3.5 6 3.5Z M14 3.5V8.5H19 M8.5 13H15.5 M8.5 16.5H13";

pub const OPEN_PATH: &str =
    "M3.5 18V6.5C3.5 5.7 4.2 5 5 5H9L11 7.5H17.5C18.3 7.5 19 8.2 19 9V10.8 M4.2 19H18.2C18.9 19 19.5 18.6 19.7 17.9L21.4 12.2C21.6 11.5 21.1 10.8 20.3 10.8H7.2C6.5 10.8 6 11.2 5.8 11.8Z";

pub const MATH_STUDIO_PATH: &str = r#"<rect x="9" y="3.5" width="5" height="8" rx="1.5"/> M11.5 11.5V14 M14 6C18.5 7 20 13.5 16 18 M8 16.5H14 M5 20.5H19"#;

pub const MATRIX_PATH: &str =
    "M7.5 3.5H4.5V20.5H7.5 M16.5 3.5H19.5V20.5H16.5 M9.5 9.5H11 M13 9.5H14.5 M9.5 14.5H11 M13 14.5H14.5";

pub const AXES_PATH: &str =
    "M5 3.5V19H20.5 M3 5.5L5 3.5L7 5.5 M18.5 17L20.5 19L18.5 21 M8 16C10 16 11 8 14 8C16.5 8 17 11 19.5 11";

pub const TABLE_PATH: &str = r#"<rect x="3.5" y="4.5" width="17" height="15" rx="2.5"/> M3.5 10H20.5 M3.5 15H20.5 M9.5 10V19.5"#;

pub const FLOWCHART_PATH: &str = r#"<rect x="7" y="3" width="10" height="5" rx="1.5"/> M12 8V11 M12 11L7 14.5L12 18L17 14.5Z M12 18V21 M17 14.5H20V21"#;

pub const PDF_PATH: &str =
    "M6 3.5H14L19 8.5V18.5C19 19.6 18.1 20.5 17 20.5H7C5.9 20.5 5 19.6 5 18.5V5C5 4.2 5.4 3.5 6 3.5Z M14 3.5V8.5H19 M12 11.5V17 M9.5 14.5L12 17L14.5 14.5";

pub const LIBRARY_PATH: &str =
    "M4 4.5V19.5C4 19.5 7 18 12 19.5C17 18 20 19.5 20 19.5V4.5C20 4.5 17 3 12 4.5C7 3 4 4.5 4 4.5Z M12 4.5V19.5";

pub const AUTOSAVE_PATH: &str =
    "M12 2.5L4.5 5.5V11.5C4.5 16 8 20 12 21.5C16 20 19.5 16 19.5 11.5V5.5L12 2.5Z M9 12L11 14L15.5 9.5";

// Shape vector paths (24x24 viewBox)
pub const SHAPE_LINE_PATH: &str = "M5 19L19 5";
pub const SHAPE_ARROW_PATH: &str = "M5 19L19 5 M13 5H19V11";
pub const SHAPE_DOUBLE_ARROW_PATH: &str = "M6 18L18 6 M12 6H18V12 M6 12V18H12";
pub const SHAPE_RECTANGLE_PATH: &str = r#"<rect x="4.5" y="4.5" width="15" height="15" rx="1.5"/>"#;
pub const SHAPE_CIRCLE_PATH: &str =
    "M12 4.5C16.14 4.5 19.5 7.86 19.5 12C19.5 16.14 16.14 19.5 12 19.5C7.86 19.5 4.5 16.14 4.5 12C4.5 7.86 7.86 4.5 12 4.5Z";
pub const SHAPE_TRIANGLE_PATH: &str = "M12 4.5L19.5 19.5H4.5Z";
pub const SHAPE_PENTAGON_PATH: &str = "M12 4L19.5 9.5L16.5 19.5H7.5L4.5 9.5Z";
pub const SHAPE_HEXAGON_PATH: &str = "M12 4L19 8V16L12 20L5 16V8Z";
pub const SHAPE_STAR_PATH: &str =
    "M12 3.5L14.6 8.8L20.5 9.6L16.2 13.8L17.2 19.6L12 16.8L6.8 19.6L7.8 13.8L3.5 9.6L9.4 8.8Z";

// Graph Coordinate Axes vector paths (24x24 viewBox)
pub const GRAPH_QUADRANT1_PATH: &str =
    "M5 4.5V19.5H20 M2.5 7.5L5 4.5L7.5 7.5 M17 17L20 19.5L17 22";
pub const GRAPH_2D_PATH: &str =
    "M4 12H20 M12 4V20 M10 6.5L12 4L14 6.5 M17.5 10L20 12L17.5 14";
pub const GRAPH_3D_PATH: &str =
    "M4.5 12H19.5 M12 4.5V19.5 M6.5 17.5L17.5 6.5 M10 6.5L12 4.5L14 6.5 M17.5 10L19.5 12L17.5 14 M15 6L17.5 6.5L17 9";

// 3D Solids vector paths (24x24 viewBox)
pub const SHAPE_CUBE_3D_PATH: &str =
    "M12 3L20 7.5V16.5L12 21L4 16.5V7.5Z M12 3V12 M20 7.5L12 12L4 7.5 M12 12V21";
pub const SHAPE_PYRAMID_3D_PATH: &str =
    "M12 3.5L4.5 17.5L11 20.5L19.5 17.5Z M12 3.5L11 20.5 M4.5 17.5L19.5 17.5";
pub const SHAPE_SPHERE_3D_PATH: &str =
    "M12 3.5C16.69 3.5 20.5 7.31 20.5 12C20.5 16.69 16.69 20.5 12 20.5C7.31 20.5 3.5 16.69 3.5 12C3.5 7.31 7.31 3.5 12 3.5Z M3.5 12C3.5 15 20.5 15 20.5 12C20.5 9 3.5 9 3.5 12 M12 3.5C15 3.5 15 20.5 12 20.5C9 20.5 9 3.5 12 3.5";

// Geometry Sketchpad Studio icon (Compass & Geometry)
pub const SKETCHPAD_PATH: &str =
    "M12 3.5L6.5 20.5 M12 3.5L17.5 20.5 M8.5 14.5H15.5 M12 2V5 M9 18C10.5 19 13.5 19 15 18";

// Page and document action vector paths (24x24 viewBox)
pub const MOVE_UP_PATH: &str = "M6 14L12 8L18 14";
pub const MOVE_DOWN_PATH: &str = "M6 10L12 16L18 10";
pub const DUPLICATE_PATH: &str =
    "M8 4.5H16C17.1 4.5 18 5.4 18 6.5V14.5 M6 8.5H14C15.1 8.5 16 9.4 16 10.5V18.5C16 19.6 15.1 20.5 14 20.5H6C4.9 20.5 4 19.6 4 18.5V10.5C4 9.4 4.9 8.5 6 8.5Z";
pub const DELETE_PATH: &str =
    "M4.5 6.5H19.5 M9.5 6.5V4.5H14.5V6.5 M6.5 6.5L7.5 19.5C7.5 20.1 8 20.5 8.5 20.5H15.5C16 20.5 16.5 20.1 16.5 19.5L17.5 6.5 M10 10.5V16.5 M14 10.5V16.5";
pub const ADD_PATH: &str = "M12 5V19 M5 12H19";
pub const CLOSE_PATH: &str = "M6 6L18 18 M18 6L6 18";

// Tools, History, Canvas, and Stencil vector paths (24x24 viewBox)
pub const UNDO_PATH: &str = "M9 14L4 9L9 4 M4 9H15C17.8 9 20 11.2 20 14C20 16.8 17.8 19 15 19H8";
pub const REDO_PATH: &str = "M15 14L20 9L15 4 M20 9H9C6.2 9 4 11.2 4 14C4 16.8 6.2 19 9 19H16";
pub const PEN_PATH: &str = "M18 2.5L21.5 6L7.5 20H4V16.5L18 2.5Z M15 5.5L18.5 9";
pub const HIGHLIGHTER_PATH: &str = "M15 3.5L20.5 9L14 15.5L8.5 10L15 3.5Z M6.5 12L3.5 19L10.5 16Z";
pub const ERASER_STROKE_PATH: &str = "M18 4L20 6L9 17L5 17L5 13L16 2Z M3 21H21";
pub const ERASER_POINT_PATH: &str = "M6 6L18 18 M18 6L6 18 M12 3V5 M12 19V21 M3 12H5 M19 12H21";
pub const LASSO_PATH: &str = "M7 4C4 6 3 11 5 15C7 19 13 21 17 19C21 17 21 12 19 8C17 4 11 2 7 4Z M14 9L18 9 M16 7L16 11";
pub const PAN_PATH: &str = "M7 11V6C7 5 8 4 9 4C10 4 11 5 11 6V10 M11 6C11 5 12 4 13 4C14 4 15 5 15 6V10 M15 7C15 6 16 5 17 5C18 5 19 6 19 7V12C19 16.5 15.5 20 11 20C7 20 5 17 5 14V10C5 9 6 8 7 8V11";
pub const RULER_PATH: &str = "M3 8H21V14H3Z M7 8V11 M11 8V12 M15 8V11 M19 8V12";
pub const PROTRACTOR_PATH: &str = "M3 17H21C21 12 17 8 12 8C7 8 3 12 3 17Z M12 14V17 M12 8V11";
pub const PICTURE_PATH: &str = "M4 5H20V19H4Z M8.5 10C9.3 10 10 9.3 10 8.5C10 7.7 9.3 7 8.5 7C7.7 7 7 7.7 7 8.5C7 9.3 7.7 10 8.5 10Z M4 17L9 12L13 16L16 13L20 17";
pub const SUN_PATH: &str = "M12 7C9.2 7 7 9.2 7 12C7 14.8 9.2 17 12 17C14.8 17 17 14.8 17 12C17 9.2 14.8 7 12 7Z M12 2V4 M12 20V22 M2 12H4 M20 12H22 M4.9 4.9L6.3 6.3 M17.7 17.7L19.1 19.1 M4.9 19.1L6.3 17.7 M17.7 6.3L19.1 4.9";
pub const MOON_PATH: &str = "M12 3C7 3 3 7 3 12C3 17 7 21 12 21C16 21 19.3 18.5 20.6 15C19.7 15.3 18.9 15.5 18 15.5C13.9 15.5 10.5 12.1 10.5 8C10.5 6 11.3 4.2 12.5 3C12.3 3 12.2 3 12 3Z";
pub const PREV_PATH: &str = "M15 18L9 12L15 6";
pub const NEXT_PATH: &str = "M9 18L15 12L9 6";
pub const EXTEND_PATH: &str = "M12 4V16 M7 11L12 16L17 11 M5 20H19";
pub const FIT_PATH: &str = "M4 9V4H9 M15 4H20V9 M20 15V20H15 M9 20H4V15";
pub const CHECK_PATH: &str = "M4 12L9 17L20 6";
pub const WARN_PATH: &str = "M12 9V13 M12 17H12.01 M10.3 3.9L1.8 18C1.4 18.7 1.9 19.5 2.7 19.5H21.3C22.1 19.5 22.6 18.7 22.2 18L13.7 3.9C13.3 3.2 12.3 3.2 12 3.9Z";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VectorIcon {
    Library,
    Save,
    Pages,
    Open,
    MathStudio,
    Matrix,
    Axes,
    Table,
    Flowchart,
    Pdf,
    Autosave,
    Sketchpad,
    MoveUp,
    MoveDown,
    Duplicate,
    Delete,
    Add,
    Close,
    Undo,
    Redo,
    Pen,
    Highlighter,
    EraserStroke,
    EraserPoint,
    Lasso,
    Pan,
    Ruler,
    Protractor,
    Picture,
    Sun,
    Moon,
    Prev,
    Next,
    Extend,
    Fit,
    Check,
    Warn,
    // Shapes & Graphs
    ShapeLine,
    ShapeArrow,
    ShapeDoubleArrow,
    ShapeRectangle,
    ShapeCircle,
    ShapeTriangle,
    ShapePentagon,
    ShapeHexagon,
    ShapeStar,
    GraphQuadrant1,
    Graph2D,
    Graph3D,
    ShapeCube3D,
    ShapePyramid3D,
    ShapeSphere3D,
}

impl VectorIcon {
    pub const ALL: [VectorIcon; 52] = [
        VectorIcon::Library,
        VectorIcon::Save,
        VectorIcon::Pages,
        VectorIcon::Open,
        VectorIcon::MathStudio,
        VectorIcon::Matrix,
        VectorIcon::Axes,
        VectorIcon::Table,
        VectorIcon::Flowchart,
        VectorIcon::Pdf,
        VectorIcon::Autosave,
        VectorIcon::Sketchpad,
        VectorIcon::MoveUp,
        VectorIcon::MoveDown,
        VectorIcon::Duplicate,
        VectorIcon::Delete,
        VectorIcon::Add,
        VectorIcon::Close,
        VectorIcon::Undo,
        VectorIcon::Redo,
        VectorIcon::Pen,
        VectorIcon::Highlighter,
        VectorIcon::EraserStroke,
        VectorIcon::EraserPoint,
        VectorIcon::Lasso,
        VectorIcon::Pan,
        VectorIcon::Ruler,
        VectorIcon::Protractor,
        VectorIcon::Picture,
        VectorIcon::Sun,
        VectorIcon::Moon,
        VectorIcon::Prev,
        VectorIcon::Next,
        VectorIcon::Extend,
        VectorIcon::Fit,
        VectorIcon::Check,
        VectorIcon::Warn,
        VectorIcon::ShapeLine,
        VectorIcon::ShapeArrow,
        VectorIcon::ShapeDoubleArrow,
        VectorIcon::ShapeRectangle,
        VectorIcon::ShapeCircle,
        VectorIcon::ShapeTriangle,
        VectorIcon::ShapePentagon,
        VectorIcon::ShapeHexagon,
        VectorIcon::ShapeStar,
        VectorIcon::GraphQuadrant1,
        VectorIcon::Graph2D,
        VectorIcon::Graph3D,
        VectorIcon::ShapeCube3D,
        VectorIcon::ShapePyramid3D,
        VectorIcon::ShapeSphere3D,
    ];

    pub fn raw_svg(&self) -> &'static str {
        match self {
            VectorIcon::Library => LIBRARY_PATH,
            VectorIcon::Save => SAVE_PATH,
            VectorIcon::Pages => PAGES_PATH,
            VectorIcon::Open => OPEN_PATH,
            VectorIcon::MathStudio => MATH_STUDIO_PATH,
            VectorIcon::Matrix => MATRIX_PATH,
            VectorIcon::Axes => AXES_PATH,
            VectorIcon::Table => TABLE_PATH,
            VectorIcon::Flowchart => FLOWCHART_PATH,
            VectorIcon::Pdf => PDF_PATH,
            VectorIcon::Autosave => AUTOSAVE_PATH,
            VectorIcon::Sketchpad => SKETCHPAD_PATH,
            VectorIcon::MoveUp => MOVE_UP_PATH,
            VectorIcon::MoveDown => MOVE_DOWN_PATH,
            VectorIcon::Duplicate => DUPLICATE_PATH,
            VectorIcon::Delete => DELETE_PATH,
            VectorIcon::Add => ADD_PATH,
            VectorIcon::Close => CLOSE_PATH,
            VectorIcon::Undo => UNDO_PATH,
            VectorIcon::Redo => REDO_PATH,
            VectorIcon::Pen => PEN_PATH,
            VectorIcon::Highlighter => HIGHLIGHTER_PATH,
            VectorIcon::EraserStroke => ERASER_STROKE_PATH,
            VectorIcon::EraserPoint => ERASER_POINT_PATH,
            VectorIcon::Lasso => LASSO_PATH,
            VectorIcon::Pan => PAN_PATH,
            VectorIcon::Ruler => RULER_PATH,
            VectorIcon::Protractor => PROTRACTOR_PATH,
            VectorIcon::Picture => PICTURE_PATH,
            VectorIcon::Sun => SUN_PATH,
            VectorIcon::Moon => MOON_PATH,
            VectorIcon::Prev => PREV_PATH,
            VectorIcon::Next => NEXT_PATH,
            VectorIcon::Extend => EXTEND_PATH,
            VectorIcon::Fit => FIT_PATH,
            VectorIcon::Check => CHECK_PATH,
            VectorIcon::Warn => WARN_PATH,
            VectorIcon::ShapeLine => SHAPE_LINE_PATH,
            VectorIcon::ShapeArrow => SHAPE_ARROW_PATH,
            VectorIcon::ShapeDoubleArrow => SHAPE_DOUBLE_ARROW_PATH,
            VectorIcon::ShapeRectangle => SHAPE_RECTANGLE_PATH,
            VectorIcon::ShapeCircle => SHAPE_CIRCLE_PATH,
            VectorIcon::ShapeTriangle => SHAPE_TRIANGLE_PATH,
            VectorIcon::ShapePentagon => SHAPE_PENTAGON_PATH,
            VectorIcon::ShapeHexagon => SHAPE_HEXAGON_PATH,
            VectorIcon::ShapeStar => SHAPE_STAR_PATH,
            VectorIcon::GraphQuadrant1 => GRAPH_QUADRANT1_PATH,
            VectorIcon::Graph2D => GRAPH_2D_PATH,
            VectorIcon::Graph3D => GRAPH_3D_PATH,
            VectorIcon::ShapeCube3D => SHAPE_CUBE_3D_PATH,
            VectorIcon::ShapePyramid3D => SHAPE_PYRAMID_3D_PATH,
            VectorIcon::ShapeSphere3D => SHAPE_SPHERE_3D_PATH,
        }
    }

    pub fn label_en(&self) -> &'static str {
        match self {
            VectorIcon::Library => "Library",
            VectorIcon::Save => "Save",
            VectorIcon::Pages => "Pages",
            VectorIcon::Open => "Open",
            VectorIcon::MathStudio => "MathStudio",
            VectorIcon::Matrix => "Matrix",
            VectorIcon::Axes => "Coordinate Axes",
            VectorIcon::Table => "Data Table",
            VectorIcon::Flowchart => "Flowchart",
            VectorIcon::Pdf => "PDF",
            VectorIcon::Autosave => "Autosave",
            VectorIcon::Sketchpad => "Sketchpad",
            VectorIcon::MoveUp => "Move Up",
            VectorIcon::MoveDown => "Move Down",
            VectorIcon::Duplicate => "Duplicate",
            VectorIcon::Delete => "Delete",
            VectorIcon::Add => "Add",
            VectorIcon::Close => "Close",
            VectorIcon::Undo => "Undo",
            VectorIcon::Redo => "Redo",
            VectorIcon::Pen => "Pen",
            VectorIcon::Highlighter => "Highlighter",
            VectorIcon::EraserStroke => "Stroke Eraser",
            VectorIcon::EraserPoint => "Point Eraser",
            VectorIcon::Lasso => "Lasso",
            VectorIcon::Pan => "Pan",
            VectorIcon::Ruler => "Ruler",
            VectorIcon::Protractor => "Protractor",
            VectorIcon::Picture => "Picture",
            VectorIcon::Sun => "Light Mode",
            VectorIcon::Moon => "Dark Mode",
            VectorIcon::Prev => "Previous",
            VectorIcon::Next => "Next",
            VectorIcon::Extend => "Extend",
            VectorIcon::Fit => "Fit",
            VectorIcon::Check => "Check",
            VectorIcon::Warn => "Warning",
            VectorIcon::ShapeLine => "Line",
            VectorIcon::ShapeArrow => "Arrow",
            VectorIcon::ShapeDoubleArrow => "Double Arrow",
            VectorIcon::ShapeRectangle => "Rectangle",
            VectorIcon::ShapeCircle => "Circle",
            VectorIcon::ShapeTriangle => "Triangle",
            VectorIcon::ShapePentagon => "Pentagon",
            VectorIcon::ShapeHexagon => "Hexagon",
            VectorIcon::ShapeStar => "Star",
            VectorIcon::GraphQuadrant1 => "1st Quadrant Axes",
            VectorIcon::Graph2D => "2D Cartesian Axes",
            VectorIcon::Graph3D => "3D Coordinate Axes",
            VectorIcon::ShapeCube3D => "3D Cube",
            VectorIcon::ShapePyramid3D => "3D Pyramid",
            VectorIcon::ShapeSphere3D => "3D Sphere",
        }
    }

    pub fn tooltip_en(&self) -> &'static str {
        match self {
            VectorIcon::Library => "Notebook Library & Bookshelf (Ctrl+L)",
            VectorIcon::Save => "Save notebook as .inkk (Ctrl+S)",
            VectorIcon::Pages => "Show page overview & previews",
            VectorIcon::Open => "Open notebook file (Ctrl+O)",
            VectorIcon::MathStudio => "MathStudio & MATLAB IDE (Ctrl+M)",
            VectorIcon::Matrix => "Insert Matrix (MATLAB code or UI)",
            VectorIcon::Axes => "Insert Coordinate Axes (1D / 2D / 3D)",
            VectorIcon::Table => "Insert Data Table (UI / CSV)",
            VectorIcon::Flowchart => "Flowchart Studio & Algorithm Editor (Ctrl+F)",
            VectorIcon::Pdf => "Export Notebook to PDF (Ctrl+Shift+E)",
            VectorIcon::Autosave => "Autosave & Crash Recovery Settings",
            VectorIcon::Sketchpad => "Geometry Sketchpad (GSP) & 3D Spatial Studio (Ctrl+G)",
            VectorIcon::MoveUp => "Move page earlier in document",
            VectorIcon::MoveDown => "Move page later in document",
            VectorIcon::Duplicate => "Duplicate page",
            VectorIcon::Delete => "Delete page",
            VectorIcon::Add => "Add page",
            VectorIcon::Close => "Close",
            VectorIcon::Undo => "Undo (Ctrl+Z)",
            VectorIcon::Redo => "Redo (Ctrl+Y)",
            VectorIcon::Pen => "Pen (1)",
            VectorIcon::Highlighter => "Highlighter (2)",
            VectorIcon::EraserStroke => "Stroke Eraser (5)",
            VectorIcon::EraserPoint => "Point Eraser (6)",
            VectorIcon::Lasso => "Lasso Select (7 / L)",
            VectorIcon::Pan => "Pan Canvas (8 / Space)",
            VectorIcon::Ruler => "Straight Ruler (R)",
            VectorIcon::Protractor => "Circular Protractor (Shift+R)",
            VectorIcon::Picture => "Insert Image from file (or Ctrl+V to paste)",
            VectorIcon::Sun => "Switch to light theme",
            VectorIcon::Moon => "Switch to dark theme",
            VectorIcon::Prev => "Previous page (PgUp)",
            VectorIcon::Next => "Next page (PgDn)",
            VectorIcon::Extend => "Extend page by 600 px",
            VectorIcon::Fit => "Fit page to window",
            VectorIcon::Check => "Success / Confirmed",
            VectorIcon::Warn => "Warning / Alert",
            VectorIcon::ShapeLine => "Draw straight line",
            VectorIcon::ShapeArrow => "Draw arrow",
            VectorIcon::ShapeDoubleArrow => "Draw double-ended arrow",
            VectorIcon::ShapeRectangle => "Draw rectangle / square",
            VectorIcon::ShapeCircle => "Draw ellipse / circle",
            VectorIcon::ShapeTriangle => "Draw triangle",
            VectorIcon::ShapePentagon => "Draw regular pentagon",
            VectorIcon::ShapeHexagon => "Draw regular hexagon",
            VectorIcon::ShapeStar => "Draw five-pointed star",
            VectorIcon::GraphQuadrant1 => "Draw 1st Quadrant axes (L-shape)",
            VectorIcon::Graph2D => "Draw 2D Cartesian axes (4 quadrants)",
            VectorIcon::Graph3D => "Draw 3D Coordinate axes (X, Y, Z)",
            VectorIcon::ShapeCube3D => "Draw 3D Cube (with hidden edges)",
            VectorIcon::ShapePyramid3D => "Draw 3D Pyramid (with hidden edges)",
            VectorIcon::ShapeSphere3D => "Draw 3D Sphere (with latitude/longitude)",
        }
    }

    pub fn parsed(&self) -> &'static ParsedSvg {
        match self {
            VectorIcon::Library => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(LIBRARY_PATH))
            }
            VectorIcon::Save => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SAVE_PATH))
            }
            VectorIcon::Pages => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PAGES_PATH))
            }
            VectorIcon::Open => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(OPEN_PATH))
            }
            VectorIcon::MathStudio => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(MATH_STUDIO_PATH))
            }
            VectorIcon::Matrix => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(MATRIX_PATH))
            }
            VectorIcon::Axes => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(AXES_PATH))
            }
            VectorIcon::Table => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(TABLE_PATH))
            }
            VectorIcon::Flowchart => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(FLOWCHART_PATH))
            }
            VectorIcon::Pdf => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PDF_PATH))
            }
            VectorIcon::Autosave => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(AUTOSAVE_PATH))
            }
            VectorIcon::Sketchpad => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SKETCHPAD_PATH))
            }
            VectorIcon::MoveUp => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(MOVE_UP_PATH))
            }
            VectorIcon::MoveDown => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(MOVE_DOWN_PATH))
            }
            VectorIcon::Duplicate => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(DUPLICATE_PATH))
            }
            VectorIcon::Delete => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(DELETE_PATH))
            }
            VectorIcon::Add => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(ADD_PATH))
            }
            VectorIcon::Close => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(CLOSE_PATH))
            }
            VectorIcon::Undo => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(UNDO_PATH))
            }
            VectorIcon::Redo => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(REDO_PATH))
            }
            VectorIcon::Pen => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PEN_PATH))
            }
            VectorIcon::Highlighter => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(HIGHLIGHTER_PATH))
            }
            VectorIcon::EraserStroke => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(ERASER_STROKE_PATH))
            }
            VectorIcon::EraserPoint => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(ERASER_POINT_PATH))
            }
            VectorIcon::Lasso => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(LASSO_PATH))
            }
            VectorIcon::Pan => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PAN_PATH))
            }
            VectorIcon::Ruler => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(RULER_PATH))
            }
            VectorIcon::Protractor => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PROTRACTOR_PATH))
            }
            VectorIcon::Picture => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PICTURE_PATH))
            }
            VectorIcon::Sun => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SUN_PATH))
            }
            VectorIcon::Moon => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(MOON_PATH))
            }
            VectorIcon::Prev => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(PREV_PATH))
            }
            VectorIcon::Next => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(NEXT_PATH))
            }
            VectorIcon::Extend => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(EXTEND_PATH))
            }
            VectorIcon::Fit => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(FIT_PATH))
            }
            VectorIcon::Check => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(CHECK_PATH))
            }
            VectorIcon::Warn => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(WARN_PATH))
            }
            VectorIcon::ShapeLine => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_LINE_PATH))
            }
            VectorIcon::ShapeArrow => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_ARROW_PATH))
            }
            VectorIcon::ShapeDoubleArrow => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_DOUBLE_ARROW_PATH))
            }
            VectorIcon::ShapeRectangle => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_RECTANGLE_PATH))
            }
            VectorIcon::ShapeCircle => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_CIRCLE_PATH))
            }
            VectorIcon::ShapeTriangle => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_TRIANGLE_PATH))
            }
            VectorIcon::ShapePentagon => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_PENTAGON_PATH))
            }
            VectorIcon::ShapeHexagon => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_HEXAGON_PATH))
            }
            VectorIcon::ShapeStar => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_STAR_PATH))
            }
            VectorIcon::GraphQuadrant1 => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(GRAPH_QUADRANT1_PATH))
            }
            VectorIcon::Graph2D => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(GRAPH_2D_PATH))
            }
            VectorIcon::Graph3D => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(GRAPH_3D_PATH))
            }
            VectorIcon::ShapeCube3D => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_CUBE_3D_PATH))
            }
            VectorIcon::ShapePyramid3D => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_PYRAMID_3D_PATH))
            }
            VectorIcon::ShapeSphere3D => {
                static CACHE: OnceLock<ParsedSvg> = OnceLock::new();
                CACHE.get_or_init(|| ParsedSvg::parse(SHAPE_SPHERE_3D_PATH))
            }
        }
    }

    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32, base_stroke_width: f32) {
        self.parsed()
            .render(painter, rect, color, base_stroke_width);
    }
}

#[derive(Clone, Debug)]
pub struct RectSpec {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rx: f32,
}

#[derive(Clone, Debug)]
pub struct Subpath {
    pub points: Vec<Pos2>,
    pub closed: bool,
}

#[derive(Clone, Debug)]
pub struct ParsedSvg {
    pub rects: Vec<RectSpec>,
    pub subpaths: Vec<Subpath>,
}

#[derive(Debug, PartialEq)]
enum Token {
    Cmd(char),
    Num(f32),
}

fn tokenize_path(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() || b == b',' {
            i += 1;
            continue;
        }
        if b.is_ascii_alphabetic() {
            tokens.push(Token::Cmd(b as char));
            i += 1;
            continue;
        }
        if b == b'-' || b == b'+' || b == b'.' || b.is_ascii_digit() {
            let start = i;
            if b == b'-' || b == b'+' {
                i += 1;
            }
            let mut has_dot = b == b'.';
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_digit() {
                    i += 1;
                } else if c == b'.' && !has_dot {
                    has_dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            if let Ok(val) = std::str::from_utf8(&bytes[start..i])
                .unwrap_or("0")
                .parse::<f32>()
            {
                tokens.push(Token::Num(val));
            }
            continue;
        }
        i += 1;
    }
    tokens
}

fn parse_attr(tag: &str, attr: &str) -> Option<f32> {
    let pattern = format!("{}=\"", attr);
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')? + start;
    tag[start..end].trim().parse::<f32>().ok()
}

fn parse_rect_tag(tag: &str) -> Option<RectSpec> {
    Some(RectSpec {
        x: parse_attr(tag, "x").unwrap_or(0.0),
        y: parse_attr(tag, "y").unwrap_or(0.0),
        width: parse_attr(tag, "width").unwrap_or(0.0),
        height: parse_attr(tag, "height").unwrap_or(0.0),
        rx: parse_attr(tag, "rx").unwrap_or(0.0),
    })
}

fn extract_rects_and_path(spec: &str) -> (Vec<RectSpec>, String) {
    let mut rects = Vec::new();
    let mut remaining = String::new();
    let mut cursor = spec;

    while let Some(start_idx) = cursor.find("<rect") {
        remaining.push_str(&cursor[..start_idx]);
        if let Some(end_idx) = cursor[start_idx..].find("/>") {
            let tag = &cursor[start_idx..start_idx + end_idx + 2];
            if let Some(rect) = parse_rect_tag(tag) {
                rects.push(rect);
            }
            cursor = &cursor[start_idx + end_idx + 2..];
        } else {
            break;
        }
    }
    remaining.push_str(cursor);
    (rects, remaining)
}

fn parse_path_commands(tokens: &[Token]) -> Vec<Subpath> {
    let mut subpaths = Vec::new();
    let mut current = Pos2::ZERO;
    let mut subpath_start = Pos2::ZERO;
    let mut cur_pts: Vec<Pos2> = Vec::new();
    let mut cur_cmd = ' ';

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            Token::Cmd(c) => {
                cur_cmd = c;
                i += 1;
                if cur_cmd == 'Z' || cur_cmd == 'z' {
                    if !cur_pts.is_empty() {
                        cur_pts.push(subpath_start);
                        subpaths.push(Subpath {
                            points: std::mem::take(&mut cur_pts),
                            closed: true,
                        });
                    }
                    current = subpath_start;
                }
            }
            Token::Num(_) => match cur_cmd {
                'M' | 'm' => {
                    if i + 1 < tokens.len() {
                        if let (Token::Num(x), Token::Num(y)) = (&tokens[i], &tokens[i + 1]) {
                            if !cur_pts.is_empty() {
                                subpaths.push(Subpath {
                                    points: std::mem::take(&mut cur_pts),
                                    closed: false,
                                });
                            }
                            let next_pt = if cur_cmd == 'm' {
                                Pos2::new(current.x + *x, current.y + *y)
                            } else {
                                Pos2::new(*x, *y)
                            };
                            current = next_pt;
                            subpath_start = current;
                            cur_pts.push(current);
                            i += 2;
                            cur_cmd = if cur_cmd == 'm' { 'l' } else { 'L' };
                            continue;
                        }
                    }
                    i += 1;
                }
                'L' | 'l' => {
                    if i + 1 < tokens.len() {
                        if let (Token::Num(x), Token::Num(y)) = (&tokens[i], &tokens[i + 1]) {
                            let next_pt = if cur_cmd == 'l' {
                                Pos2::new(current.x + *x, current.y + *y)
                            } else {
                                Pos2::new(*x, *y)
                            };
                            current = next_pt;
                            cur_pts.push(current);
                            i += 2;
                            continue;
                        }
                    }
                    i += 1;
                }
                'H' | 'h' => {
                    if let Token::Num(x) = tokens[i] {
                        let nx = if cur_cmd == 'h' { current.x + x } else { x };
                        current = Pos2::new(nx, current.y);
                        cur_pts.push(current);
                        i += 1;
                        continue;
                    }
                    i += 1;
                }
                'V' | 'v' => {
                    if let Token::Num(y) = tokens[i] {
                        let ny = if cur_cmd == 'v' { current.y + y } else { y };
                        current = Pos2::new(current.x, ny);
                        cur_pts.push(current);
                        i += 1;
                        continue;
                    }
                    i += 1;
                }
                'C' | 'c' => {
                    if i + 5 < tokens.len() {
                        if let (
                            Token::Num(x1),
                            Token::Num(y1),
                            Token::Num(x2),
                            Token::Num(y2),
                            Token::Num(x),
                            Token::Num(y),
                        ) = (
                            &tokens[i],
                            &tokens[i + 1],
                            &tokens[i + 2],
                            &tokens[i + 3],
                            &tokens[i + 4],
                            &tokens[i + 5],
                        ) {
                            let p0 = current;
                            let (p1, p2, p3) = if cur_cmd == 'c' {
                                (
                                    Pos2::new(current.x + *x1, current.y + *y1),
                                    Pos2::new(current.x + *x2, current.y + *y2),
                                    Pos2::new(current.x + *x, current.y + *y),
                                )
                            } else {
                                (Pos2::new(*x1, *y1), Pos2::new(*x2, *y2), Pos2::new(*x, *y))
                            };
                            let steps = 12;
                            for step in 1..=steps {
                                let t = step as f32 / steps as f32;
                                let it = 1.0 - t;
                                let it2 = it * it;
                                let it3 = it2 * it;
                                let t2 = t * t;
                                let t3 = t2 * t;
                                let px = it3 * p0.x
                                    + 3.0 * it2 * t * p1.x
                                    + 3.0 * it * t2 * p2.x
                                    + t3 * p3.x;
                                let py = it3 * p0.y
                                    + 3.0 * it2 * t * p1.y
                                    + 3.0 * it * t2 * p2.y
                                    + t3 * p3.y;
                                cur_pts.push(Pos2::new(px, py));
                            }
                            current = p3;
                            i += 6;
                            continue;
                        }
                    }
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            },
        }
    }

    if !cur_pts.is_empty() {
        subpaths.push(Subpath {
            points: cur_pts,
            closed: false,
        });
    }

    subpaths
}

impl ParsedSvg {
    pub fn parse(svg_text: &str) -> Self {
        let (rects, path_str) = extract_rects_and_path(svg_text);
        let tokens = tokenize_path(&path_str);
        let subpaths = parse_path_commands(&tokens);
        Self { rects, subpaths }
    }

    pub fn render(
        &self,
        painter: &Painter,
        rect: Rect,
        stroke_color: Color32,
        base_stroke_width: f32,
    ) {
        let size = rect.width().min(rect.height());
        let scale = size / 24.0;
        let origin = Pos2::new(
            rect.min.x + (rect.width() - size) * 0.5,
            rect.min.y + (rect.height() - size) * 0.5,
        );
        let stroke_w = (base_stroke_width * scale).max(1.15);
        let stroke = Stroke::new(stroke_w, stroke_color);

        let map_pt =
            |p: Pos2| -> Pos2 { Pos2::new(origin.x + p.x * scale, origin.y + p.y * scale) };

        // Draw rects
        for r in &self.rects {
            let top_left = map_pt(Pos2::new(r.x, r.y));
            let r_rect =
                Rect::from_min_size(top_left, Vec2::new(r.width * scale, r.height * scale));
            painter.rect_stroke(r_rect, Rounding::same(r.rx * scale), stroke);
        }

        // Draw subpaths
        for subpath in &self.subpaths {
            if subpath.points.len() < 2 {
                continue;
            }
            let pts: Vec<Pos2> = subpath.points.iter().map(|&p| map_pt(p)).collect();
            if subpath.closed {
                painter.add(Shape::closed_line(pts, stroke));
            } else {
                painter.add(Shape::line(pts, stroke));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_ten_icons_parse() {
        assert_eq!(VectorIcon::ALL.len(), 52);
        for icon in VectorIcon::ALL {
            let parsed = icon.parsed();
            let total_geometry = parsed.rects.len() + parsed.subpaths.len();
            assert!(
                total_geometry > 0,
                "Icon {:?} produced empty geometry",
                icon
            );
        }
    }

    #[test]
    fn test_flowchart_icon_structure() {
        let parsed = VectorIcon::Flowchart.parsed();
        assert_eq!(parsed.rects.len(), 1);
        assert!(!parsed.subpaths.is_empty());
    }

    #[test]
    fn test_save_icon_structure() {
        let parsed = VectorIcon::Save.parsed();
        assert_eq!(parsed.rects.len(), 0);
        // Outer body (closed) + shutter (open) + label (open) = 3 subpaths
        assert_eq!(parsed.subpaths.len(), 3);
        assert!(parsed.subpaths[0].closed);
    }

    #[test]
    fn test_math_studio_icon_structure() {
        let parsed = VectorIcon::MathStudio.parsed();
        assert_eq!(parsed.rects.len(), 1);
        assert_eq!(parsed.rects[0].rx, 1.5);
        assert!(parsed.subpaths.len() >= 3);
    }

    #[test]
    fn test_table_icon_structure() {
        let parsed = VectorIcon::Table.parsed();
        assert_eq!(parsed.rects.len(), 1);
        assert_eq!(parsed.rects[0].rx, 2.5);
        assert_eq!(parsed.subpaths.len(), 3);
    }
}
