//! Model definitions for Inkk Geometry Sketchpad (GSP) & 3D Spatial Studio.

use eframe::egui::{Color32, Pos2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointConstraint {
    Free,
    Midpoint(String, String),
    Intersection(String, String),
    CircleCenter(String),
    PointOnCircle { circle_id: String, angle: f32 },
    PointOnSegment { line_id: String, t: f32 },
    Projected3D,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub id: String,
    pub label: String,
    pub pos: Pos2,
    pub pos3d: Option<[f32; 3]>,
    pub color: Color32,
    pub is_free: bool,
    pub is_selected: bool,
    pub is_hidden: bool,
    pub trace_enabled: bool,
    pub constraint: PointConstraint,
}

impl GeoPoint {
    pub fn new_2d(id: impl Into<String>, label: impl Into<String>, pos: Pos2) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            pos,
            pos3d: None,
            color: Color32::from_rgb(220, 40, 40),
            is_free: true,
            is_selected: false,
            is_hidden: false,
            trace_enabled: false,
            constraint: PointConstraint::Free,
        }
    }

    pub fn new_3d(id: impl Into<String>, label: impl Into<String>, p3d: [f32; 3]) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            pos: Pos2::ZERO,
            pos3d: Some(p3d),
            color: Color32::from_rgb(220, 40, 40),
            is_free: true,
            is_selected: false,
            is_hidden: false,
            trace_enabled: false,
            constraint: PointConstraint::Projected3D,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineKind {
    Segment,
    Ray,
    Line,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoLine {
    pub id: String,
    pub label: Option<String>,
    pub p1: String,
    pub p2: String,
    pub kind: LineKind,
    pub is_dashed: bool,
    pub color: Color32,
    pub width: f32,
    pub is_selected: bool,
    pub is_hidden: bool,
}

impl GeoLine {
    pub fn new(id: impl Into<String>, p1: impl Into<String>, p2: impl Into<String>, is_dashed: bool) -> Self {
        Self {
            id: id.into(),
            label: None,
            p1: p1.into(),
            p2: p2.into(),
            kind: LineKind::Segment,
            is_dashed,
            color: Color32::from_rgb(25, 60, 180),
            width: 2.0,
            is_selected: false,
            is_hidden: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoCircle {
    pub id: String,
    pub center: String,
    pub radius_pt: Option<String>,
    pub fixed_radius: Option<f32>,
    pub is_dashed: bool,
    pub color: Color32,
    pub is_selected: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoArc {
    pub id: String,
    pub center: String,
    pub start_pt: String,
    pub end_pt: String,
    pub is_dashed: bool,
    pub color: Color32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoPolygon {
    pub id: String,
    pub vertices: Vec<String>,
    pub fill_color: Option<Color32>,
    pub stroke_color: Color32,
    pub is_selected: bool,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureKind {
    Length,
    Angle,
    Perimeter,
    Area,
    Coordinates,
    Slope,
    Distance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoMeasurement {
    pub id: String,
    pub kind: MeasureKind,
    pub target_ids: Vec<String>,
    pub cached_value: f64,
    pub display_text: String,
    pub pos: Pos2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Toggle3DRotate,
    ViewPlaneOxy,
    ViewPlaneOxz,
    ViewPlaneOyz,
    ToggleAnimation,
    Reset3D,
    ToggleHideSelected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoActionButton {
    pub id: String,
    pub label: String,
    pub kind: ActionKind,
    pub pos: Pos2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoFunctionGraph {
    pub id: String,
    pub expr: String,
    pub color: Color32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Geo3DCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub zoom: f32,
    pub center: Pos2,
    pub is_auto_rotating: bool,
    pub rotation_speed: f32,
}

impl Default for Geo3DCamera {
    fn default() -> Self {
        Self {
            pitch: 0.35,
            yaw: -0.55,
            roll: 0.0,
            zoom: 1.0,
            center: Pos2::new(350.0, 270.0),
            is_auto_rotating: false,
            rotation_speed: 0.015,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TracePoint {
    pub pos: Pos2,
    pub color: Color32,
    pub point_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SketchpadTool {
    Select,
    Point,
    Compass,
    Segment,
    DashedSegment,
    Ray,
    Line,
    Polygon,
    TextLabel,
    MeasureDistance,
    MeasureAngle,
    MeasureArea,
    Rotate3D,
}

#[derive(Debug, Clone, Default)]
pub struct SketchpadModel {
    pub points: Vec<GeoPoint>,
    pub lines: Vec<GeoLine>,
    pub circles: Vec<GeoCircle>,
    pub arcs: Vec<GeoArc>,
    pub polygons: Vec<GeoPolygon>,
    pub measurements: Vec<GeoMeasurement>,
    pub action_buttons: Vec<GeoActionButton>,
    pub functions: Vec<GeoFunctionGraph>,
    pub camera3d: Geo3DCamera,
    pub traces: Vec<TracePoint>,
    pub show_axes: bool,
    pub show_grid: bool,
    pub next_id: usize,
    pub is_animating_points: bool,
    pub animation_t: f32,
}

impl SketchpadModel {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            lines: Vec::new(),
            circles: Vec::new(),
            arcs: Vec::new(),
            polygons: Vec::new(),
            measurements: Vec::new(),
            action_buttons: Vec::new(),
            functions: Vec::new(),
            camera3d: Geo3DCamera::default(),
            traces: Vec::new(),
            show_axes: true,
            show_grid: true,
            next_id: 1,
            is_animating_points: false,
            animation_t: 0.0,
        }
    }

    pub fn gen_id(&mut self, prefix: &str) -> String {
        let id = format!("{}_{}", prefix, self.next_id);
        self.next_id += 1;
        id
    }

    pub fn get_point(&self, id: &str) -> Option<&GeoPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    pub fn get_point_mut(&mut self, id: &str) -> Option<&mut GeoPoint> {
        self.points.iter_mut().find(|p| p.id == id)
    }

    pub fn get_line(&self, id: &str) -> Option<&GeoLine> {
        self.lines.iter().find(|l| l.id == id)
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.lines.clear();
        self.circles.clear();
        self.arcs.clear();
        self.polygons.clear();
        self.measurements.clear();
        self.action_buttons.clear();
        self.functions.clear();
        self.traces.clear();
        self.next_id = 1;
        self.camera3d = Geo3DCamera::default();
    }

    pub fn clear_traces(&mut self) {
        self.traces.clear();
    }
}
