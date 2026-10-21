use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    #[serde(default = "default_point_thickness")]
    pub thickness: f32,
}

fn default_point_thickness() -> f32 {
    0.0
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
    Ballpoint,
    Fountain,
    Calligraphy,
    Pencil,
    Highlighter,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ToolMode {
    Pen,
    Shape(ShapeType),
    Highlighter,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub id: String,
    pub pen_type: PenType,
    #[serde(default = "default_shape_type")]
    pub shape_type: ShapeType,
    #[serde(default)]
    pub is_filled: bool,
    pub color: [u8; 4],
    pub size: f32,
    pub opacity: f32,
    pub points: Vec<Point>,
}

fn default_shape_type() -> ShapeType {
    ShapeType::None
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
            if p.x < min_x { min_x = p.x; }
            if p.x > max_x { max_x = p.x; }
            if p.y < min_y { min_y = p.y; }
            if p.y > max_y { max_y = p.y; }
        }
        Some((min_x, min_y, max_x, max_y))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub paper_type: PaperType,
    pub width: f32,
    pub height: f32,
    pub strokes: Vec<Stroke>,
}

impl Page {
    pub fn new(paper_type: PaperType) -> Self {
        Self {
            id: format!("page_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
            paper_type,
            width: 1000.0,
            height: 1400.0,
            strokes: Vec::new(),
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
    pub title: String,
    pub default_paper: PaperType,
    pub pages: Vec<Page>,
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
    pub fn current_page(&self) -> &Page {
        &self.pages[self.current_page_index]
    }

    pub fn current_page_mut(&mut self) -> &mut Page {
        &mut self.pages[self.current_page_index]
    }

    pub fn add_page(&mut self) {
        let paper = self.current_page().paper_type;
        self.pages.insert(self.current_page_index + 1, Page::new(paper));
        self.current_page_index += 1;
    }

    pub fn duplicate_current_page(&mut self) {
        let mut clone = self.current_page().clone();
        clone.id = format!("page_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
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
}
