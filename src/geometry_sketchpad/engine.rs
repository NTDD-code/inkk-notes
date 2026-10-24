//! Geometry & Math Engine for Sketchpad.
//! Implements Euclidean constructions, constraint solving, 3D spatial projections,
//! measurements, transformations, and algebraic curve plotting.

use eframe::egui::{Pos2, Vec2};
use std::f32::consts::PI;

use super::model::{
    Geo3DCamera, MeasureKind, PointConstraint, SketchpadModel,
};

pub struct GeometryEngine;

impl GeometryEngine {
    /// Solves all point constraints and updates their positions.
    pub fn update_geometry(model: &mut SketchpadModel) {
        // 1. If 3D camera auto-rotates, step the yaw
        if model.camera3d.is_auto_rotating {
            model.camera3d.yaw += model.camera3d.rotation_speed;
            if model.camera3d.yaw > PI * 2.0 {
                model.camera3d.yaw -= PI * 2.0;
            }
        }

        // 2. Point animation along hosts
        if model.is_animating_points {
            model.animation_t += 0.02;
            if model.animation_t > 1.0 {
                model.animation_t = 0.0;
            }
        }

        // 3. Update 3D projected points
        let cam = model.camera3d.clone();
        for pt in &mut model.points {
            if let Some(p3d) = pt.pos3d {
                let (screen_pos, _depth) = Self::project_3d(p3d, &cam);
                pt.pos = screen_pos;
            }
        }

        // 4. Update 2D constraints iteratively (up to 3 passes to resolve dependencies)
        for _ in 0..3 {
            let pts_snapshot: Vec<(String, Pos2)> = model.points.iter().map(|p| (p.id.clone(), p.pos)).collect();
            let get_pos = |id: &str| -> Option<Pos2> {
                pts_snapshot.iter().find(|(pid, _)| pid == id).map(|(_, pos)| *pos)
            };

            for pt in &mut model.points {
                match &pt.constraint {
                    PointConstraint::Midpoint(p1, p2) => {
                        if let (Some(pos1), Some(pos2)) = (get_pos(p1), get_pos(p2)) {
                            pt.pos = Pos2::new((pos1.x + pos2.x) * 0.5, (pos1.y + pos2.y) * 0.5);
                        }
                    }
                    PointConstraint::Intersection(l1_id, l2_id) => {
                        if let (Some(l1), Some(l2)) = (
                            model.lines.iter().find(|l| l.id == *l1_id),
                            model.lines.iter().find(|l| l.id == *l2_id),
                        ) {
                            if let (Some(p1), Some(p2), Some(p3), Some(p4)) = (
                                get_pos(&l1.p1),
                                get_pos(&l1.p2),
                                get_pos(&l2.p1),
                                get_pos(&l2.p2),
                            ) {
                                if let Some(inter) = Self::intersect_lines(p1, p2, p3, p4) {
                                    pt.pos = inter;
                                }
                            }
                        }
                    }
                    PointConstraint::CircleCenter(c_id) => {
                        if let Some(c) = model.circles.iter().find(|c| c.id == *c_id) {
                            if let Some(c_pos) = get_pos(&c.center) {
                                pt.pos = c_pos;
                            }
                        }
                    }
                    PointConstraint::PointOnCircle { circle_id, angle } => {
                        if let Some(c) = model.circles.iter().find(|c| c.id == *circle_id) {
                            if let Some(center_pos) = get_pos(&c.center) {
                                let r = if let Some(r_id) = &c.radius_pt {
                                    get_pos(r_id).map(|rp| (rp - center_pos).length()).unwrap_or(50.0)
                                } else {
                                    c.fixed_radius.unwrap_or(50.0)
                                };
                                let cur_angle = if model.is_animating_points {
                                    angle + model.animation_t * std::f32::consts::TAU
                                } else {
                                    *angle
                                };
                                pt.pos = Pos2::new(
                                    center_pos.x + r * cur_angle.cos(),
                                    center_pos.y + r * cur_angle.sin(),
                                );
                            }
                        }
                    }
                    PointConstraint::PointOnSegment { line_id, t } => {
                        if let Some(line) = model.lines.iter().find(|l| l.id == *line_id) {
                            if let (Some(p1), Some(p2)) = (get_pos(&line.p1), get_pos(&line.p2)) {
                                let cur_t = if model.is_animating_points {
                                    (model.animation_t * 2.0).clamp(0.0, 1.0)
                                } else {
                                    *t
                                };
                                pt.pos = p1 + (p2 - p1) * cur_t;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // 5. Update measurements
        Self::update_measurements(model);

        // 6. Record traces for points that have trace_enabled
        for pt in &model.points {
            if pt.trace_enabled {
                let should_add = match model.traces.last() {
                    Some(last) => last.point_id != pt.id || (last.pos - pt.pos).length_sq() > 4.0,
                    None => true,
                };
                if should_add {
                    model.traces.push(super::model::TracePoint {
                        pos: pt.pos,
                        color: pt.color,
                        point_id: pt.id.clone(),
                    });
                    if model.traces.len() > 2000 {
                        model.traces.remove(0);
                    }
                }
            }
        }
    }

    /// Projects a 3D coordinate [X, Y, Z] onto 2D screen pos with yaw/pitch Euler rotation.
    pub fn project_3d(p: [f32; 3], cam: &Geo3DCamera) -> (Pos2, f32) {
        let (sin_y, cos_y) = cam.yaw.sin_cos();
        let (sin_p, cos_p) = cam.pitch.sin_cos();

        // 1. Rotate around Y axis (azimuth / yaw)
        let x1 = p[0] * cos_y + p[2] * sin_y;
        let y1 = p[1];
        let z1 = -p[0] * sin_y + p[2] * cos_y;

        // 2. Rotate around X axis (elevation / pitch)
        let x2 = x1;
        let y2 = y1 * cos_p - z1 * sin_p;
        let z2 = y1 * sin_p + z1 * cos_p;

        // 3. Isometric / Cavalier camera projection
        let screen_x = cam.center.x + x2 * cam.zoom;
        let screen_y = cam.center.y - y2 * cam.zoom; // Invert Y for screen

        (Pos2::new(screen_x, screen_y), z2)
    }

    /// Computes intersection of two 2D lines (p1-p2 and p3-p4).
    pub fn intersect_lines(p1: Pos2, p2: Pos2, p3: Pos2, p4: Pos2) -> Option<Pos2> {
        let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = ((p1.x - p3.x) * (p3.y - p4.y) - (p1.y - p3.y) * (p3.x - p4.x)) / denom;
        Some(Pos2::new(
            p1.x + t * (p2.x - p1.x),
            p1.y + t * (p2.y - p1.y),
        ))
    }

    /// Updates cached measurement values.
    fn update_measurements(model: &mut SketchpadModel) {
        for m in &mut model.measurements {
            match m.kind {
                MeasureKind::Length | MeasureKind::Distance => {
                    if m.target_ids.len() >= 2 {
                        if let (Some(p1), Some(p2)) = (
                            model.points.iter().find(|p| p.id == m.target_ids[0]),
                            model.points.iter().find(|p| p.id == m.target_ids[1]),
                        ) {
                            let dist = ((p1.pos.x - p2.pos.x).powi(2) + (p1.pos.y - p2.pos.y).powi(2)).sqrt();
                            // Assume 1 unit = 40 px
                            let val_cm = (dist / 40.0) as f64;
                            m.cached_value = val_cm;
                            m.display_text = format!("{} {}{} = {:.2} cm", p1.label, p2.label, if m.kind == MeasureKind::Distance { " Dist" } else { "" }, val_cm);
                        }
                    }
                }
                MeasureKind::Angle => {
                    if m.target_ids.len() >= 3 {
                        if let (Some(pa), Some(pb), Some(pc)) = (
                            model.points.iter().find(|p| p.id == m.target_ids[0]),
                            model.points.iter().find(|p| p.id == m.target_ids[1]),
                            model.points.iter().find(|p| p.id == m.target_ids[2]),
                        ) {
                            // Angle B-A-C (vertex at pa)
                            let v1 = Vec2::new(pb.pos.x - pa.pos.x, pb.pos.y - pa.pos.y);
                            let v2 = Vec2::new(pc.pos.x - pa.pos.x, pc.pos.y - pa.pos.y);
                            let dot = v1.x * v2.x + v1.y * v2.y;
                            let mag = v1.length() * v2.length();
                            if mag > 1e-4 {
                                let cos_theta = (dot / mag).clamp(-1.0, 1.0);
                                let angle_deg = cos_theta.acos() * 180.0 / PI;
                                m.cached_value = angle_deg as f64;
                                m.display_text = format!("∠{}{}{} = {:.1}°", pb.label, pa.label, pc.label, angle_deg);
                            }
                        }
                    }
                }
                MeasureKind::Area => {
                    if let Some(poly) = model.polygons.iter().find(|poly| poly.id == m.target_ids.first().cloned().unwrap_or_default()) {
                        let pts: Vec<Pos2> = poly
                            .vertices
                            .iter()
                            .filter_map(|vid| model.points.iter().find(|p| p.id == *vid).map(|p| p.pos))
                            .collect();
                        if pts.len() >= 3 {
                            let mut area = 0.0;
                            let n = pts.len();
                            for i in 0..n {
                                let j = (i + 1) % n;
                                area += (pts[i].x * pts[j].y) - (pts[j].x * pts[i].y);
                            }
                            let area_cm2 = (area.abs() * 0.5 / (40.0 * 40.0)) as f64;
                            m.cached_value = area_cm2;
                            m.display_text = format!("Area(Polygon) = {:.2} cm²", area_cm2);
                        }
                    }
                }
                MeasureKind::Perimeter => {
                    if let Some(poly) = model.polygons.iter().find(|poly| poly.id == m.target_ids.first().cloned().unwrap_or_default()) {
                        let pts: Vec<Pos2> = poly
                            .vertices
                            .iter()
                            .filter_map(|vid| model.points.iter().find(|p| p.id == *vid).map(|p| p.pos))
                            .collect();
                        if pts.len() >= 2 {
                            let mut perim = 0.0;
                            let n = pts.len();
                            for i in 0..n {
                                let j = (i + 1) % n;
                                perim += (pts[i] - pts[j]).length();
                            }
                            let perim_cm = (perim / 40.0) as f64;
                            m.cached_value = perim_cm;
                            m.display_text = format!("Perimeter = {:.2} cm", perim_cm);
                        }
                    }
                }
                MeasureKind::Coordinates => {
                    if let Some(p) = model.points.iter().find(|p| p.id == m.target_ids.first().cloned().unwrap_or_default()) {
                        let cx = (p.pos.x - model.camera3d.center.x) / 40.0;
                        let cy = (model.camera3d.center.y - p.pos.y) / 40.0;
                        m.display_text = format!("{}({:.2}, {:.2})", p.label, cx, cy);
                    }
                }
                MeasureKind::Slope => {
                    if m.target_ids.len() >= 2 {
                        if let (Some(p1), Some(p2)) = (
                            model.points.iter().find(|p| p.id == m.target_ids[0]),
                            model.points.iter().find(|p| p.id == m.target_ids[1]),
                        ) {
                            let dx = p2.pos.x - p1.pos.x;
                            let dy = -(p2.pos.y - p1.pos.y);
                            if dx.abs() > 1e-4 {
                                let slope = (dy / dx) as f64;
                                m.cached_value = slope;
                                m.display_text = format!("Slope({}{}) = {:.2}", p1.label, p2.label, slope);
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Geometric Transformations ---

    /// Translation: translates selected points by delta
    pub fn translate_selected(model: &mut SketchpadModel, delta: Vec2) {
        for pt in &mut model.points {
            if pt.is_selected {
                if let Some(p3d) = &mut pt.pos3d {
                    p3d[0] += delta.x;
                    p3d[1] -= delta.y;
                } else {
                    pt.pos += delta;
                }
            }
        }
    }

    /// Rotation: rotates selected points around center by angle_deg
    pub fn rotate_selected(model: &mut SketchpadModel, center: Pos2, angle_deg: f32) {
        let rad = angle_deg * PI / 180.0;
        let (sin_a, cos_a) = rad.sin_cos();
        for pt in &mut model.points {
            if pt.is_selected {
                let dx = pt.pos.x - center.x;
                let dy = pt.pos.y - center.y;
                let nx = center.x + dx * cos_a - dy * sin_a;
                let ny = center.y + dx * sin_a + dy * cos_a;
                pt.pos = Pos2::new(nx, ny);
            }
        }
    }

    /// Dilation (Scaling): dilates selected points from center by factor k
    pub fn dilate_selected(model: &mut SketchpadModel, center: Pos2, factor: f32) {
        for pt in &mut model.points {
            if pt.is_selected {
                pt.pos = center + (pt.pos - center) * factor;
            }
        }
    }

    /// Reflection across a line (p1 - p2)
    pub fn reflect_selected_across_line(model: &mut SketchpadModel, p1: Pos2, p2: Pos2) {
        let line_dir = (p2 - p1).normalized();
        for pt in &mut model.points {
            if pt.is_selected {
                let v = pt.pos - p1;
                let proj_len = v.dot(line_dir);
                let proj_pt = p1 + line_dir * proj_len;
                // Reflection = 2 * Proj - pt
                pt.pos = proj_pt * 2.0 - pt.pos.to_vec2();
            }
        }
    }

    /// Reflection across a point
    pub fn reflect_selected_across_point(model: &mut SketchpadModel, center: Pos2) {
        for pt in &mut model.points {
            if pt.is_selected {
                pt.pos = center * 2.0 - pt.pos.to_vec2();
            }
        }
    }

    /// Evaluate mathematical function y = f(x)
    pub fn eval_func(expr: &str, x: f32) -> Option<f32> {
        let clean = expr.trim().to_lowercase();
        if clean == "sin(x)" {
            Some(x.sin())
        } else if clean == "cos(x)" {
            Some(x.cos())
        } else if clean == "tan(x)" {
            Some(x.tan())
        } else if clean == "x^2" || clean == "x*x" {
            Some(x * x)
        } else if clean == "x^3" {
            Some(x * x * x)
        } else if clean == "sqrt(x)" {
            if x >= 0.0 { Some(x.sqrt()) } else { None }
        } else if clean == "exp(x)" {
            Some(x.exp())
        } else if clean == "ln(x)" || clean == "log(x)" {
            if x > 0.0 { Some(x.ln()) } else { None }
        } else if clean == "1/x" {
            if x.abs() > 1e-4 { Some(1.0 / x) } else { None }
        } else {
            // Default sinusoidal harmonic
            Some(x.sin() * 0.8 + (x * 2.0).cos() * 0.3)
        }
    }
}
