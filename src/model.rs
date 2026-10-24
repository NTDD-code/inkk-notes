use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub thickness: f32,
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = serde_json::Value::deserialize(deserializer)?;
        match val {
            serde_json::Value::Object(map) => {
                let x = map.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let y = map.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let pressure = map.get("pressure").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                let thickness = map.get("thickness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                Ok(Point {
                    x,
                    y,
                    pressure,
                    thickness,
                })
            }
            serde_json::Value::Array(arr) => {
                let x = arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let y = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let pressure = arr.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                let thickness = arr.get(3).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                Ok(Point {
                    x,
                    y,
                    pressure,
                    thickness,
                })
            }
            _ => Ok(Point::new(0.0, 0.0, 1.0)),
        }
    }
}

impl Point {
    pub fn new(x: f32, y: f32, pressure: f32) -> Self {
        Self {
            x,
            y,
            pressure,
            thickness: 0.0,
        }
    }

    pub fn with_thickness(x: f32, y: f32, pressure: f32, thickness: f32) -> Self {
        Self {
            x,
            y,
            pressure,
            thickness,
        }
    }

    #[allow(dead_code)]
    pub fn dist_sq(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PenType {
    #[serde(alias = "ballpoint", alias = "pen", alias = "Pen", alias = "BALLPOINT")]
    Ballpoint,
    #[serde(alias = "fountain", alias = "Fountain", alias = "FOUNTAIN")]
    Fountain,
    #[serde(alias = "calligraphy", alias = "Calligraphy", alias = "CALLIGRAPHY")]
    Calligraphy,
    #[serde(alias = "pencil", alias = "Pencil", alias = "PENCIL")]
    Pencil,
    #[serde(alias = "highlighter", alias = "Highlighter", alias = "HIGHLIGHTER")]
    Highlighter,
}

pub fn deserialize_pen_type<'de, D>(deserializer: D) -> Result<PenType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::Number(n) => match n.as_u64() {
            Some(1) => Ok(PenType::Fountain),
            Some(2) => Ok(PenType::Calligraphy),
            Some(3) => Ok(PenType::Pencil),
            Some(4) => Ok(PenType::Highlighter),
            _ => Ok(PenType::Ballpoint),
        },
        serde_json::Value::String(s) => {
            let lower = s.to_lowercase();
            match lower.as_str() {
                "fountain" => Ok(PenType::Fountain),
                "calligraphy" => Ok(PenType::Calligraphy),
                "pencil" => Ok(PenType::Pencil),
                "highlighter" => Ok(PenType::Highlighter),
                _ => Ok(PenType::Ballpoint),
            }
        }
        _ => Ok(PenType::Ballpoint),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    None,
    Line,
    Rectangle,
    Circle,
    Triangle,
    Arrow,
    DoubleArrow,
    Pentagon,
    Hexagon,
    Star,
    GraphQuadrant1,
    Graph2D,
    Graph3D,
    Cube3D,
    Pyramid3D,
    Sphere3D,
}

impl ShapeType {
    pub fn vector_icon(&self) -> crate::vector_icon::VectorIcon {
        match self {
            ShapeType::Line => crate::vector_icon::VectorIcon::ShapeLine,
            ShapeType::Arrow => crate::vector_icon::VectorIcon::ShapeArrow,
            ShapeType::DoubleArrow => crate::vector_icon::VectorIcon::ShapeDoubleArrow,
            ShapeType::Rectangle => crate::vector_icon::VectorIcon::ShapeRectangle,
            ShapeType::Circle => crate::vector_icon::VectorIcon::ShapeCircle,
            ShapeType::Triangle => crate::vector_icon::VectorIcon::ShapeTriangle,
            ShapeType::Pentagon => crate::vector_icon::VectorIcon::ShapePentagon,
            ShapeType::Hexagon => crate::vector_icon::VectorIcon::ShapeHexagon,
            ShapeType::Star => crate::vector_icon::VectorIcon::ShapeStar,
            ShapeType::GraphQuadrant1 => crate::vector_icon::VectorIcon::GraphQuadrant1,
            ShapeType::Graph2D => crate::vector_icon::VectorIcon::Graph2D,
            ShapeType::Graph3D => crate::vector_icon::VectorIcon::Graph3D,
            ShapeType::Cube3D => crate::vector_icon::VectorIcon::ShapeCube3D,
            ShapeType::Pyramid3D => crate::vector_icon::VectorIcon::ShapePyramid3D,
            ShapeType::Sphere3D => crate::vector_icon::VectorIcon::ShapeSphere3D,
            ShapeType::None => crate::vector_icon::VectorIcon::ShapeRectangle,
        }
    }
}

pub fn deserialize_shape_type<'de, D>(deserializer: D) -> Result<ShapeType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::Number(n) => match n.as_u64() {
            Some(1) => Ok(ShapeType::Line),
            Some(2) => Ok(ShapeType::Rectangle),
            Some(3) => Ok(ShapeType::Circle),
            Some(4) => Ok(ShapeType::Triangle),
            Some(5) => Ok(ShapeType::Arrow),
            Some(6) => Ok(ShapeType::DoubleArrow),
            Some(7) => Ok(ShapeType::Pentagon),
            Some(8) => Ok(ShapeType::Hexagon),
            Some(9) => Ok(ShapeType::Star),
            Some(10) => Ok(ShapeType::GraphQuadrant1),
            Some(11) => Ok(ShapeType::Graph2D),
            Some(12) => Ok(ShapeType::Graph3D),
            Some(13) => Ok(ShapeType::Cube3D),
            Some(14) => Ok(ShapeType::Pyramid3D),
            Some(15) => Ok(ShapeType::Sphere3D),
            _ => Ok(ShapeType::None),
        },
        serde_json::Value::String(s) => {
            let lower = s.to_lowercase();
            match lower.as_str() {
                "line" => Ok(ShapeType::Line),
                "rectangle" | "rect" => Ok(ShapeType::Rectangle),
                "circle" => Ok(ShapeType::Circle),
                "triangle" => Ok(ShapeType::Triangle),
                "arrow" => Ok(ShapeType::Arrow),
                "doublearrow" | "double_arrow" => Ok(ShapeType::DoubleArrow),
                "pentagon" => Ok(ShapeType::Pentagon),
                "hexagon" => Ok(ShapeType::Hexagon),
                "star" => Ok(ShapeType::Star),
                "graphquadrant1" | "quadrant1" | "axes1d" | "graph1d" => {
                    Ok(ShapeType::GraphQuadrant1)
                }
                "graph2d" | "axes2d" | "fourquadrants" => Ok(ShapeType::Graph2D),
                "graph3d" | "axes3d" | "threedimensional" => Ok(ShapeType::Graph3D),
                "cube" | "cube3d" | "box3d" => Ok(ShapeType::Cube3D),
                "pyramid" | "pyramid3d" | "tetrahedron" => Ok(ShapeType::Pyramid3D),
                "sphere" | "sphere3d" | "globe" => Ok(ShapeType::Sphere3D),
                _ => Ok(ShapeType::None),
            }
        }
        _ => Ok(ShapeType::None),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToolMode {
    Pen,
    Highlighter,
    Shape(ShapeType),
    Text,
    EraserStroke,
    EraserPoint,
    LassoSelect,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PaperType {
    Ruled,
    Grid,
    DotGrid,
    Cornell,
    Blank,
}

pub fn deserialize_paper_type<'de, D>(deserializer: D) -> Result<PaperType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::Number(n) => match n.as_u64() {
            Some(1) => Ok(PaperType::Grid),
            Some(2) => Ok(PaperType::DotGrid),
            Some(3) => Ok(PaperType::Cornell),
            Some(4) => Ok(PaperType::Blank),
            _ => Ok(PaperType::Ruled),
        },
        serde_json::Value::String(s) => {
            let lower = s.to_lowercase();
            match lower.as_str() {
                "grid" => Ok(PaperType::Grid),
                "dotgrid" | "dot_grid" | "dots" => Ok(PaperType::DotGrid),
                "cornell" => Ok(PaperType::Cornell),
                "blank" | "white" | "plain" => Ok(PaperType::Blank),
                _ => Ok(PaperType::Ruled),
            }
        }
        _ => Ok(PaperType::Ruled),
    }
}

impl PaperType {
    pub fn name(&self) -> &'static str {
        match self {
            PaperType::Ruled => "Ruled",
            PaperType::Grid => "Grid",
            PaperType::DotGrid => "Dot Grid",
            PaperType::Cornell => "Cornell Notes",
            PaperType::Blank => "Blank",
        }
    }
}

pub fn deserialize_color<'de, D>(deserializer: D) -> Result<[u8; 4], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    Ok(parse_color_value(&val))
}

pub fn parse_color_value(val: &serde_json::Value) -> [u8; 4] {
    match val {
        serde_json::Value::String(s) => parse_hex_color(s),
        serde_json::Value::Array(arr) => {
            if arr.len() == 4 {
                let r = parse_channel(&arr[0]);
                let g = parse_channel(&arr[1]);
                let b = parse_channel(&arr[2]);
                let a = parse_channel(&arr[3]);
                [r, g, b, a]
            } else if arr.len() == 3 {
                let r = parse_channel(&arr[0]);
                let g = parse_channel(&arr[1]);
                let b = parse_channel(&arr[2]);
                [r, g, b, 255]
            } else {
                [0, 0, 0, 255]
            }
        }
        serde_json::Value::Object(map) => {
            let r = map.get("r").map(parse_channel).unwrap_or(0);
            let g = map.get("g").map(parse_channel).unwrap_or(0);
            let b = map.get("b").map(parse_channel).unwrap_or(0);
            let a = map.get("a").map(parse_channel).unwrap_or(255);
            [r, g, b, a]
        }
        serde_json::Value::Number(num) => {
            if let Some(n) = num.as_u64() {
                if n <= 0xFFFFFF {
                    let r = ((n >> 16) & 0xFF) as u8;
                    let g = ((n >> 8) & 0xFF) as u8;
                    let b = (n & 0xFF) as u8;
                    [r, g, b, 255]
                } else {
                    let r = ((n >> 24) & 0xFF) as u8;
                    let g = ((n >> 16) & 0xFF) as u8;
                    let b = ((n >> 8) & 0xFF) as u8;
                    let a = (n & 0xFF) as u8;
                    [r, g, b, a]
                }
            } else {
                [0, 0, 0, 255]
            }
        }
        _ => [0, 0, 0, 255],
    }
}

fn parse_channel(val: &serde_json::Value) -> u8 {
    if let Some(u) = val.as_u64() {
        u.min(255) as u8
    } else if let Some(i) = val.as_i64() {
        i.clamp(0, 255) as u8
    } else if let Some(f) = val.as_f64() {
        if (0.0..=1.0).contains(&f) {
            (f * 255.0).round() as u8
        } else {
            f.clamp(0.0, 255.0).round() as u8
        }
    } else {
        0
    }
}

fn parse_hex_color(s: &str) -> [u8; 4] {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).unwrap_or(0) * 17;
            let g = u8::from_str_radix(&s[1..2], 16).unwrap_or(0) * 17;
            let b = u8::from_str_radix(&s[2..3], 16).unwrap_or(0) * 17;
            [r, g, b, 255]
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).unwrap_or(0) * 17;
            let g = u8::from_str_radix(&s[1..2], 16).unwrap_or(0) * 17;
            let b = u8::from_str_radix(&s[2..3], 16).unwrap_or(0) * 17;
            let a = u8::from_str_radix(&s[3..4], 16).unwrap_or(15) * 17;
            [r, g, b, a]
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            [r, g, b, 255]
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
            [r, g, b, a]
        }
        _ => [0, 0, 0, 255],
    }
}

fn default_stroke_id() -> String {
    format!(
        "stroke_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn default_pen_type() -> PenType {
    PenType::Ballpoint
}

fn default_shape_type() -> ShapeType {
    ShapeType::None
}

fn default_color() -> [u8; 4] {
    [0, 0, 0, 255]
}

fn default_stroke_size() -> f32 {
    3.0
}

fn default_opacity() -> f32 {
    1.0
}

fn default_paper_type() -> PaperType {
    PaperType::Ruled
}

fn default_page_width() -> f32 {
    1000.0
}

fn default_page_height() -> f32 {
    1400.0
}

fn default_page_id() -> String {
    format!(
        "page_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}

fn default_notebook_title() -> String {
    "My Notebook".to_string()
}

fn default_pages() -> Vec<Page> {
    vec![Page::new(PaperType::Ruled)]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    #[serde(default = "default_stroke_id")]
    pub id: String,
    #[serde(
        default = "default_pen_type",
        deserialize_with = "deserialize_pen_type"
    )]
    pub pen_type: PenType,
    #[serde(
        default = "default_shape_type",
        deserialize_with = "deserialize_shape_type"
    )]
    pub shape_type: ShapeType,
    #[serde(default)]
    pub is_filled: bool,
    #[serde(default = "default_color", deserialize_with = "deserialize_color")]
    pub color: [u8; 4],
    #[serde(default)]
    pub original_color: Option<[u8; 4]>,
    #[serde(default = "default_stroke_size")]
    pub size: f32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub points: Vec<Point>,
}

impl Stroke {
    #[allow(dead_code)]
    pub fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        if self.points.is_empty() {
            return None;
        }
        let mut min_x = self.points[0].x;
        let mut max_x = self.points[0].x;
        let mut min_y = self.points[0].y;
        let mut max_y = self.points[0].y;

        for p in &self.points[1..] {
            if p.x < min_x {
                min_x = p.x;
            }
            if p.x > max_x {
                max_x = p.x;
            }
            if p.y < min_y {
                min_y = p.y;
            }
            if p.y > max_y {
                max_y = p.y;
            }
        }
        Some((min_x, min_y, max_x, max_y))
    }
}

fn default_text_box_id() -> String {
    format!(
        "text_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn default_text_box_width() -> f32 {
    320.0
}

fn default_text_box_height() -> f32 {
    50.0
}

fn default_text_color() -> [u8; 4] {
    [15, 23, 42, 255]
}

fn default_font_size() -> f32 {
    18.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBox {
    #[serde(default = "default_text_box_id")]
    pub id: String,
    pub x: f32, // world coordinates
    pub y: f32,
    #[serde(default = "default_text_box_width")]
    pub width: f32,
    #[serde(default = "default_text_box_height")]
    pub height: f32,
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_text_color", deserialize_with = "deserialize_color")]
    pub color: [u8; 4],
    #[serde(default)]
    pub original_color: Option<[u8; 4]>,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub is_bold: bool,
    #[serde(default)]
    pub is_italic: bool,
}

impl TextBox {
    pub fn new(x: f32, y: f32, color: [u8; 4], font_size: f32) -> Self {
        Self {
            id: default_text_box_id(),
            x,
            y,
            width: 320.0,
            height: 50.0,
            text: String::new(),
            color,
            original_color: None,
            font_size,
            is_bold: false,
            is_italic: false,
        }
    }
}

fn default_image_id() -> String {
    format!(
        "img_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntity {
    #[serde(default = "default_image_id")]
    pub id: String,
    pub x: f32, // world coordinates on page
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub data: Vec<u8>, // PNG/JPEG encoded bytes
}

impl ImageEntity {
    pub fn new(x: f32, y: f32, width: f32, height: f32, data: Vec<u8>) -> Self {
        Self {
            id: default_image_id(),
            x,
            y,
            width,
            height,
            rotation: 0.0,
            data,
        }
    }

    #[allow(dead_code)]
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }
}

fn default_true() -> bool {
    true
}

fn default_plot_id() -> String {
    format!(
        "plot_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotCurveData {
    pub label: String,
    pub color: [u8; 4],
    pub points: Vec<[f64; 2]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotEntity {
    #[serde(default = "default_plot_id")]
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    #[serde(default)]
    pub curves: Vec<PlotCurveData>,
    #[serde(default = "default_true")]
    pub show_grid: bool,
    #[serde(default = "default_true")]
    pub show_axes: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub dimension: Option<String>,
    #[serde(default)]
    pub x_step: Option<f64>,
    #[serde(default)]
    pub y_step: Option<f64>,
    #[serde(default)]
    pub z_min: Option<f64>,
    #[serde(default)]
    pub z_max: Option<f64>,
    #[serde(default)]
    pub z_step: Option<f64>,
    #[serde(default)]
    pub source_code: Option<String>,
}

fn default_matrix_id() -> String {
    format!(
        "mat_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixEntity {
    #[serde(default = "default_matrix_id")]
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub label: String,
    pub data: Vec<Vec<f64>>,
    #[serde(default)]
    pub source_code: Option<String>,
}

fn default_grid_id() -> String {
    format!(
        "grid_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableGridEntity {
    #[serde(default = "default_grid_id")]
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub rows: usize,
    pub cols: usize,
    pub col_width: f32,
    pub row_height: f32,
    #[serde(default)]
    pub cells: Vec<Vec<String>>,
    #[serde(default)]
    pub source_code: Option<String>,
}

impl Default for PlotEntity {
    fn default() -> Self {
        Self {
            id: default_plot_id(),
            x: 50.0,
            y: 50.0,
            width: 450.0,
            height: 320.0,
            x_min: -5.0,
            x_max: 5.0,
            y_min: -5.0,
            y_max: 5.0,
            curves: Vec::new(),
            show_grid: true,
            show_axes: true,
            title: String::new(),
            dimension: None,
            x_step: None,
            y_step: None,
            z_min: None,
            z_max: None,
            z_step: None,
            source_code: None,
        }
    }
}

impl Default for MatrixEntity {
    fn default() -> Self {
        Self {
            id: default_matrix_id(),
            x: 60.0,
            y: 60.0,
            width: 220.0,
            height: 140.0,
            label: String::new(),
            data: Vec::new(),
            source_code: None,
        }
    }
}

impl Default for TableGridEntity {
    fn default() -> Self {
        Self {
            id: default_grid_id(),
            x: 60.0,
            y: 60.0,
            rows: 3,
            cols: 3,
            col_width: 100.0,
            row_height: 36.0,
            cells: Vec::new(),
            source_code: None,
        }
    }
}

impl TableGridEntity {
    pub fn total_width(&self) -> f32 {
        (self.cols.max(1) as f32) * self.col_width
    }

    pub fn total_height(&self) -> f32 {
        (self.rows.max(1) as f32) * self.row_height
    }
}

fn default_flowchart_id() -> String {
    format!(
        "flow_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartEntity {
    #[serde(default = "default_flowchart_id")]
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub dsl_source: String,
    #[serde(default)]
    pub graph: Option<crate::flowchart_core::ir::FlowchartGraph>,
    #[serde(default)]
    pub title: String,
}

impl Default for FlowchartEntity {
    fn default() -> Self {
        Self {
            id: default_flowchart_id(),
            x: 60.0,
            y: 60.0,
            width: 400.0,
            height: 300.0,
            dsl_source: String::new(),
            graph: None,
            title: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    #[serde(default = "default_page_id")]
    pub id: String,
    #[serde(
        default = "default_paper_type",
        deserialize_with = "deserialize_paper_type"
    )]
    pub paper_type: PaperType,
    #[serde(default = "default_page_width")]
    pub width: f32,
    #[serde(default = "default_page_height")]
    pub height: f32,
    #[serde(default)]
    pub strokes: Vec<Stroke>,
    #[serde(default)]
    pub text_boxes: Vec<TextBox>,
    #[serde(default)]
    pub images: Vec<ImageEntity>,
    #[serde(default)]
    pub plots: Vec<PlotEntity>,
    #[serde(default)]
    pub matrices: Vec<MatrixEntity>,
    #[serde(default)]
    pub grids: Vec<TableGridEntity>,
    #[serde(default)]
    pub flowcharts: Vec<FlowchartEntity>,
}

impl Page {
    pub fn new(paper_type: PaperType) -> Self {
        Self {
            id: format!(
                "page_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
            paper_type,
            width: 1000.0,
            height: 1400.0,
            strokes: Vec::new(),
            text_boxes: Vec::new(),
            images: Vec::new(),
            plots: Vec::new(),
            matrices: Vec::new(),
            grids: Vec::new(),
            flowcharts: Vec::new(),
        }
    }

    pub fn extend_height(&mut self, delta: f32) {
        self.height += delta;
    }

    pub fn check_auto_expand(&mut self, max_y: f32) -> bool {
        if max_y > self.height - 180.0 {
            self.height = (self.height + 500.0).max(max_y + 350.0);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    #[serde(default = "default_notebook_title")]
    pub title: String,
    #[serde(
        default = "default_paper_type",
        deserialize_with = "deserialize_paper_type"
    )]
    pub default_paper: PaperType,
    #[serde(default = "default_pages")]
    pub pages: Vec<Page>,
    #[serde(default)]
    pub current_page_index: usize,
}

impl Default for Notebook {
    fn default() -> Self {
        Self {
            title: "My Notebook".to_string(),
            default_paper: PaperType::Ruled,
            pages: vec![Page::new(PaperType::Ruled)],
            current_page_index: 0,
        }
    }
}

impl Notebook {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            default_paper: PaperType::Ruled,
            pages: vec![Page::new(PaperType::Ruled)],
            current_page_index: 0,
        }
    }

    pub fn new_with_paper(title: &str, paper: PaperType) -> Self {
        Self {
            title: title.to_string(),
            default_paper: paper,
            pages: vec![Page::new(paper)],
            current_page_index: 0,
        }
    }

    pub fn current_page(&self) -> &Page {
        &self.pages[self.current_page_index]
    }

    pub fn current_page_mut(&mut self) -> &mut Page {
        &mut self.pages[self.current_page_index]
    }

    pub fn add_page(&mut self) {
        let paper = self.current_page().paper_type;
        self.pages
            .insert(self.current_page_index + 1, Page::new(paper));
        self.current_page_index += 1;
    }

    pub fn duplicate_current_page(&mut self) {
        let mut clone = self.current_page().clone();
        clone.id = format!(
            "page_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        self.pages.insert(self.current_page_index + 1, clone);
        self.current_page_index += 1;
    }

    pub fn delete_current_page(&mut self) {
        if self.pages.len() <= 1 {
            self.current_page_mut().strokes.clear();
            return;
        }
        self.pages.remove(self.current_page_index);
        if self.current_page_index >= self.pages.len() {
            self.current_page_index = self.pages.len() - 1;
        }
    }

    pub fn next_page(&mut self) {
        if self.current_page_index + 1 < self.pages.len() {
            self.current_page_index += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.current_page_index > 0 {
            self.current_page_index -= 1;
        }
    }

    /// Move page at `idx` up (earlier in document), swapping with `idx - 1`.
    pub fn move_page_up(&mut self, idx: usize) -> bool {
        if idx > 0 && idx < self.pages.len() {
            self.pages.swap(idx, idx - 1);
            if self.current_page_index == idx {
                self.current_page_index = idx - 1;
            } else if self.current_page_index == idx - 1 {
                self.current_page_index = idx;
            }
            true
        } else {
            false
        }
    }

    /// Move page at `idx` down (later in document), swapping with `idx + 1`.
    pub fn move_page_down(&mut self, idx: usize) -> bool {
        if idx + 1 < self.pages.len() {
            self.pages.swap(idx, idx + 1);
            if self.current_page_index == idx {
                self.current_page_index = idx + 1;
            } else if self.current_page_index == idx + 1 {
                self.current_page_index = idx;
            }
            true
        } else {
            false
        }
    }

    /// Move page from index `from` to index `to`.
    pub fn move_page(&mut self, from: usize, to: usize) -> bool {
        if from >= self.pages.len() || to >= self.pages.len() || from == to {
            return false;
        }
        let page = self.pages.remove(from);
        self.pages.insert(to, page);
        if self.current_page_index == from {
            self.current_page_index = to;
        } else if from < self.current_page_index && to >= self.current_page_index {
            self.current_page_index -= 1;
        } else if from > self.current_page_index && to <= self.current_page_index {
            self.current_page_index += 1;
        }
        true
    }
}

/// Serialized clipboard bundle representing copied strokes and canvas elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InkkClipboardBundle {
    #[serde(default = "default_inkk_signature")]
    pub inkk_signature: String,
    #[serde(default)]
    pub strokes: Vec<Stroke>,
    #[serde(default)]
    pub images: Vec<ImageEntity>,
    #[serde(default)]
    pub plots: Vec<PlotEntity>,
    #[serde(default)]
    pub matrices: Vec<MatrixEntity>,
    #[serde(default)]
    pub grids: Vec<TableGridEntity>,
    #[serde(default)]
    pub flowcharts: Vec<FlowchartEntity>,
}

impl Default for InkkClipboardBundle {
    fn default() -> Self {
        Self {
            inkk_signature: default_inkk_signature(),
            strokes: Vec::new(),
            images: Vec::new(),
            plots: Vec::new(),
            matrices: Vec::new(),
            grids: Vec::new(),
            flowcharts: Vec::new(),
        }
    }
}

fn default_inkk_signature() -> String {
    "inkk_clipboard_v1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_color_hex_string() {
        let json = r##"{"id":"1","pen_type":"Ballpoint","color":"#000000","size":3.0,"opacity":1.0,"points":[]}"##;
        let stroke: Stroke = serde_json::from_str(json).unwrap();
        assert_eq!(stroke.color, [0, 0, 0, 255]);

        let json_red = r##"{"id":"2","pen_type":"Ballpoint","color":"#FF0000","size":3.0,"opacity":1.0,"points":[]}"##;
        let stroke_red: Stroke = serde_json::from_str(json_red).unwrap();
        assert_eq!(stroke_red.color, [255, 0, 0, 255]);

        let json_rgba = r##"{"id":"3","pen_type":"Ballpoint","color":"#00FF0080","size":3.0,"opacity":1.0,"points":[]}"##;
        let stroke_rgba: Stroke = serde_json::from_str(json_rgba).unwrap();
        assert_eq!(stroke_rgba.color, [0, 255, 0, 128]);
    }

    #[test]
    fn test_deserialize_color_array() {
        let json_arr4 = r#"{"id":"1","pen_type":"Ballpoint","color":[10,20,30,240],"size":3.0,"opacity":1.0,"points":[]}"#;
        let stroke4: Stroke = serde_json::from_str(json_arr4).unwrap();
        assert_eq!(stroke4.color, [10, 20, 30, 240]);

        let json_arr3 = r#"{"id":"2","pen_type":"Ballpoint","color":[10,20,30],"size":3.0,"opacity":1.0,"points":[]}"#;
        let stroke3: Stroke = serde_json::from_str(json_arr3).unwrap();
        assert_eq!(stroke3.color, [10, 20, 30, 255]);

        let json_floats = r#"{"id":"3","pen_type":"Ballpoint","color":[1.0, 0.0, 0.5, 1.0],"size":3.0,"opacity":1.0,"points":[]}"#;
        let stroke_f: Stroke = serde_json::from_str(json_floats).unwrap();
        assert_eq!(stroke_f.color, [255, 0, 128, 255]);
    }

    #[test]
    fn test_deserialize_color_object() {
        let json = r#"{"id":"1","pen_type":"Ballpoint","color":{"r":255,"g":128,"b":0,"a":200},"size":3.0,"opacity":1.0,"points":[]}"#;
        let stroke: Stroke = serde_json::from_str(json).unwrap();
        assert_eq!(stroke.color, [255, 128, 0, 200]);
    }

    #[test]
    fn test_backward_compatibility_old_notebook() {
        let old_json = r##"{
            "title": "Old Notebook",
            "default_paper": "Ruled",
            "pages": [{
                "id": "p1",
                "paper_type": "Ruled",
                "width": 1000.0,
                "height": 1400.0,
                "strokes": [{
                    "id": "s1",
                    "pen_type": "Ballpoint",
                    "color": "#000000",
                    "size": 3.0,
                    "opacity": 1.0,
                    "points": [{"x": 100.0, "y": 200.0, "pressure": 0.5}]
                }]
            }]
        }"##;

        let nb: Notebook = serde_json::from_str(old_json).unwrap();
        assert_eq!(nb.title, "Old Notebook");
        assert_eq!(nb.pages.len(), 1);
        assert_eq!(nb.pages[0].strokes.len(), 1);
        assert_eq!(nb.pages[0].strokes[0].color, [0, 0, 0, 255]);
        assert_eq!(nb.pages[0].strokes[0].points[0].thickness, 0.0);
        assert!(nb.pages[0].text_boxes.is_empty());
    }

    #[test]
    fn test_text_box_serialization_and_defaults() {
        let tb = TextBox::new(100.0, 200.0, [15, 23, 42, 255], 18.0);
        let serialized = serde_json::to_string(&tb).unwrap();
        let deserialized: TextBox = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.x, 100.0);
        assert_eq!(deserialized.y, 200.0);
        assert_eq!(deserialized.color, [15, 23, 42, 255]);
        assert_eq!(deserialized.font_size, 18.0);
        assert!(!deserialized.is_bold);
        assert!(!deserialized.is_italic);
    }

    #[test]
    fn test_load_user_old_handnotes_if_present() {
        use std::path::Path;
        let p1 = Path::new(r"C:\Users\Lenovo\Downloads\Quick Scratchpad.handnote");
        if p1.exists() {
            let res = crate::storage::NotebookStorage::load(p1);
            assert!(
                res.is_ok(),
                "Failed to load Quick Scratchpad: {:?}",
                res.err()
            );
        }
        let p2 = Path::new(r"C:\Users\Lenovo\Downloads\My Notebook 1.handnote");
        if p2.exists() {
            let res = crate::storage::NotebookStorage::load(p2);
            assert!(res.is_ok(), "Failed to load My Notebook 1: {:?}", res.err());
        }
    }

    #[test]
    fn test_image_entity_serialization_and_defaults() {
        let img = ImageEntity::new(50.0, 80.0, 300.0, 200.0, vec![1, 2, 3, 4]);
        let serialized = serde_json::to_string(&img).unwrap();
        let deserialized: ImageEntity = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.x, 50.0);
        assert_eq!(deserialized.y, 80.0);
        assert_eq!(deserialized.width, 300.0);
        assert_eq!(deserialized.height, 200.0);
        assert_eq!(deserialized.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_notebook_with_images_and_strokes_roundtrip() {
        let mut page = Page::new(PaperType::Grid);
        page.strokes.push(Stroke {
            id: "s1".into(),
            pen_type: PenType::Ballpoint,
            shape_type: ShapeType::None,
            is_filled: false,
            color: [0, 0, 0, 255],
            original_color: None,
            size: 3.0,
            opacity: 1.0,
            points: vec![Point::new(10.0, 20.0, 0.5), Point::new(30.0, 40.0, 0.6)],
        });
        page.images.push(ImageEntity::new(
            100.0,
            150.0,
            400.0,
            300.0,
            vec![10, 20, 30],
        ));

        let mut nb = Notebook::default();
        nb.pages = vec![page];

        let json = serde_json::to_string(&nb).unwrap();
        let loaded: Notebook = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.pages[0].strokes.len(), 1);
        assert_eq!(loaded.pages[0].images.len(), 1);
        assert_eq!(loaded.pages[0].images[0].width, 400.0);
        assert_eq!(loaded.pages[0].images[0].height, 300.0);
    }

    #[test]
    fn test_math_entities_serialization() {
        let mut page = Page::new(PaperType::Blank);
        page.plots.push(PlotEntity {
            id: "p1".into(),
            x: 50.0,
            y: 100.0,
            width: 500.0,
            height: 350.0,
            x_min: -3.14,
            x_max: 3.14,
            y_min: -1.0,
            y_max: 1.0,
            curves: vec![PlotCurveData {
                label: "sin(x)".into(),
                color: [37, 99, 235, 255],
                points: vec![[-3.14, 0.0], [0.0, 0.0], [3.14, 0.0]],
            }],
            show_grid: true,
            show_axes: true,
            title: "Trigonometric Plot".into(),
            dimension: None,
            x_step: None,
            y_step: None,
            z_min: None,
            z_max: None,
            z_step: None,
            source_code: None,
        });
        page.matrices.push(MatrixEntity {
            id: "m1".into(),
            x: 600.0,
            y: 100.0,
            width: 250.0,
            height: 120.0,
            label: "A =".into(),
            data: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            source_code: None,
        });
        page.grids.push(TableGridEntity {
            id: "g1".into(),
            x: 50.0,
            y: 500.0,
            rows: 3,
            cols: 4,
            col_width: 100.0,
            row_height: 35.0,
            cells: vec![vec!["Cell".into(); 4]; 3],
            source_code: None,
        });
        page.flowcharts.push(FlowchartEntity {
            id: "flow1".into(),
            x: 80.0,
            y: 80.0,
            width: 450.0,
            height: 350.0,
            dsl_source: "start\nprint \"Hello Flowchart\"\nend".into(),
            graph: None,
            title: "Test Flowchart".into(),
        });

        let serialized = serde_json::to_string(&page).unwrap();
        let deserialized: Page = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.plots.len(), 1);
        assert_eq!(deserialized.matrices.len(), 1);
        assert_eq!(deserialized.grids.len(), 1);
        assert_eq!(deserialized.flowcharts.len(), 1);
        assert_eq!(deserialized.plots[0].curves[0].label, "sin(x)");
        assert_eq!(deserialized.matrices[0].data[1][0], 3.0);
        assert_eq!(deserialized.grids[0].rows, 3);
        assert_eq!(deserialized.flowcharts[0].title, "Test Flowchart");
        assert!(deserialized.flowcharts[0]
            .dsl_source
            .contains("Hello Flowchart"));
    }

    #[test]
    fn test_notebook_page_reordering() {
        let mut notebook = Notebook::new("Test Notebook");
        notebook.pages[0].id = "page_0".to_string();
        notebook.add_page();
        notebook.pages[1].id = "page_1".to_string();
        notebook.add_page();
        notebook.pages[2].id = "page_2".to_string();

        assert_eq!(notebook.pages.len(), 3);
        assert_eq!(notebook.pages[0].id, "page_0");
        assert_eq!(notebook.pages[1].id, "page_1");
        assert_eq!(notebook.pages[2].id, "page_2");

        // Move page 2 up -> should swap with page 1
        assert!(notebook.move_page_up(2));
        assert_eq!(notebook.pages[0].id, "page_0");
        assert_eq!(notebook.pages[1].id, "page_2");
        assert_eq!(notebook.pages[2].id, "page_1");

        // Move page 0 down -> should swap with page 1
        assert!(notebook.move_page_down(0));
        assert_eq!(notebook.pages[0].id, "page_2");
        assert_eq!(notebook.pages[1].id, "page_0");
        assert_eq!(notebook.pages[2].id, "page_1");
    }

    #[test]
    fn test_clipboard_bundle_serde() {
        let mut bundle = InkkClipboardBundle::default();
        bundle.strokes.push(Stroke {
            id: "s1".to_string(),
            pen_type: PenType::Ballpoint,
            shape_type: ShapeType::None,
            is_filled: false,
            color: [255, 0, 0, 255],
            original_color: None,
            size: 4.0,
            opacity: 1.0,
            points: vec![Point { x: 10.0, y: 20.0, pressure: 0.8, thickness: 2.0 }],
        });

        let json = serde_json::to_string(&bundle).unwrap();
        assert!(json.contains("inkk_clipboard_v1"));
        let parsed: InkkClipboardBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.strokes.len(), 1);
        assert_eq!(parsed.strokes[0].id, "s1");
    }
}
