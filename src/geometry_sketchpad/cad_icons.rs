//! Pure Vector CAD Icon Painter for Geometry Sketchpad.
//! Draws crisp AutoCAD-style technical icons directly with egui Painter.
//! ZERO Unicode glyphs, ZERO external fonts, ZERO tofu square boxes [□].

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::PI;

pub struct CadIcon;

impl CadIcon {
    /// AutoCAD Pickbox / Selection cursor icon
    pub fn draw_select(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let box_s = 4.0;
        let arm_len = 8.0;
        // Central pickbox
        painter.rect_stroke(Rect::from_center_size(c, Vec2::splat(box_s * 2.0)), 0.0, stroke);
        // Crosshair arms extending out
        painter.line_segment([Pos2::new(c.x - arm_len, c.y), Pos2::new(c.x - box_s, c.y)], stroke);
        painter.line_segment([Pos2::new(c.x + box_s, c.y), Pos2::new(c.x + arm_len, c.y)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y - arm_len), Pos2::new(c.x, c.y - box_s)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y + box_s), Pos2::new(c.x, c.y + arm_len)], stroke);
    }

    /// AutoCAD Point / Node icon: Crosshair with central dot
    pub fn draw_point(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let len = 7.0;
        painter.line_segment([Pos2::new(c.x - len, c.y), Pos2::new(c.x + len, c.y)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y - len), Pos2::new(c.x, c.y + len)], stroke);
        painter.circle_filled(c, 2.5, stroke.color);
    }

    /// AutoCAD Line icon: Diagonal segment with square endpoints
    pub fn draw_line(painter: &Painter, rect: Rect, stroke: Stroke) {
        let p1 = Pos2::new(rect.left() + 5.0, rect.bottom() - 5.0);
        let p2 = Pos2::new(rect.right() - 5.0, rect.top() + 5.0);
        painter.line_segment([p1, p2], stroke);
        painter.rect_filled(Rect::from_center_size(p1, Vec2::splat(3.5)), 0.0, stroke.color);
        painter.rect_filled(Rect::from_center_size(p2, Vec2::splat(3.5)), 0.0, stroke.color);
    }

    /// AutoCAD Dashed Line icon
    pub fn draw_dashed_line(painter: &Painter, rect: Rect, stroke: Stroke) {
        let p1 = Pos2::new(rect.left() + 5.0, rect.bottom() - 5.0);
        let p2 = Pos2::new(rect.right() - 5.0, rect.top() + 5.0);
        let dir = p2 - p1;
        let steps = 4;
        for i in 0..steps {
            if i % 2 == 0 {
                let s = p1 + dir * (i as f32 / steps as f32);
                let e = p1 + dir * ((i + 1) as f32 / steps as f32);
                painter.line_segment([s, e], stroke);
            }
        }
        painter.rect_filled(Rect::from_center_size(p1, Vec2::splat(3.0)), 0.0, stroke.color);
        painter.rect_filled(Rect::from_center_size(p2, Vec2::splat(3.0)), 0.0, stroke.color);
    }

    /// AutoCAD Ray icon: Point with directional arrow
    pub fn draw_ray(painter: &Painter, rect: Rect, stroke: Stroke) {
        let p1 = Pos2::new(rect.left() + 5.0, rect.bottom() - 5.0);
        let p2 = Pos2::new(rect.right() - 4.0, rect.top() + 4.0);
        painter.line_segment([p1, p2], stroke);
        painter.circle_filled(p1, 2.5, stroke.color);
        // Arrowhead
        let dir = (p2 - p1).normalized();
        let normal = Vec2::new(-dir.y, dir.x);
        let a1 = p2 - dir * 6.0 + normal * 3.5;
        let a2 = p2 - dir * 6.0 - normal * 3.5;
        painter.line_segment([p2, a1], stroke);
        painter.line_segment([p2, a2], stroke);
    }

    /// AutoCAD XLine (Infinite Line) icon: Double arrows
    pub fn draw_xline(painter: &Painter, rect: Rect, stroke: Stroke) {
        let p1 = Pos2::new(rect.left() + 3.0, rect.bottom() - 3.0);
        let p2 = Pos2::new(rect.right() - 3.0, rect.top() + 3.0);
        painter.line_segment([p1, p2], stroke);
        let dir = (p2 - p1).normalized();
        let normal = Vec2::new(-dir.y, dir.x);
        // Arrow at p2
        painter.line_segment([p2, p2 - dir * 5.0 + normal * 3.0], stroke);
        painter.line_segment([p2, p2 - dir * 5.0 - normal * 3.0], stroke);
        // Arrow at p1
        painter.line_segment([p1, p1 + dir * 5.0 + normal * 3.0], stroke);
        painter.line_segment([p1, p1 + dir * 5.0 - normal * 3.0], stroke);
    }

    /// AutoCAD Circle icon: Circle with center crosshair
    pub fn draw_circle(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let r = 8.0;
        painter.circle_stroke(c, r, stroke);
        let ch = 3.0;
        painter.line_segment([Pos2::new(c.x - ch, c.y), Pos2::new(c.x + ch, c.y)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y - ch), Pos2::new(c.x, c.y + ch)], stroke);
    }

    /// AutoCAD Arc icon: 3-point curved arc
    pub fn draw_arc(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center() + Vec2::new(0.0, 4.0);
        let r = 9.0;
        let steps = 16;
        let mut prev = None;
        for i in 0..=steps {
            let theta = PI * 0.15 + (PI * 0.70) * (i as f32 / steps as f32);
            let p = Pos2::new(c.x - r * theta.cos(), c.y - r * theta.sin());
            if let Some(prv) = prev {
                painter.line_segment([prv, p], stroke);
            }
            prev = Some(p);
        }
    }

    /// AutoCAD Polygon icon: Pentagon with vertex grips
    pub fn draw_polygon(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let r = 8.0;
        let sides = 5;
        let mut pts = Vec::with_capacity(sides);
        for i in 0..sides {
            let angle = -PI * 0.5 + (std::f32::consts::TAU * i as f32 / sides as f32);
            pts.push(Pos2::new(c.x + r * angle.cos(), c.y + r * angle.sin()));
        }
        for i in 0..sides {
            let next = (i + 1) % sides;
            painter.line_segment([pts[i], pts[next]], stroke);
            painter.rect_filled(Rect::from_center_size(pts[i], Vec2::splat(2.5)), 0.0, stroke.color);
        }
    }

    /// AutoCAD Dimension / Measure icon: Dimension line with arrows and witness lines
    pub fn draw_dimension(painter: &Painter, rect: Rect, stroke: Stroke) {
        let y = rect.center().y;
        let x1 = rect.left() + 5.0;
        let x2 = rect.right() - 5.0;
        // Dimension line
        painter.line_segment([Pos2::new(x1, y), Pos2::new(x2, y)], stroke);
        // Extension lines
        painter.line_segment([Pos2::new(x1, y - 6.0), Pos2::new(x1, y + 6.0)], stroke);
        painter.line_segment([Pos2::new(x2, y - 6.0), Pos2::new(x2, y + 6.0)], stroke);
        // Arrowheads pointing outward
        painter.line_segment([Pos2::new(x1, y), Pos2::new(x1 + 4.0, y - 2.5)], stroke);
        painter.line_segment([Pos2::new(x1, y), Pos2::new(x1 + 4.0, y + 2.5)], stroke);
        painter.line_segment([Pos2::new(x2, y), Pos2::new(x2 - 4.0, y - 2.5)], stroke);
        painter.line_segment([Pos2::new(x2, y), Pos2::new(x2 - 4.0, y + 2.5)], stroke);
    }

    /// AutoCAD Angle Dimension icon
    pub fn draw_angle_dim(painter: &Painter, rect: Rect, stroke: Stroke) {
        let vertex = Pos2::new(rect.left() + 4.0, rect.bottom() - 4.0);
        let p_top = Pos2::new(rect.left() + 10.0, rect.top() + 4.0);
        let p_right = Pos2::new(rect.right() - 4.0, rect.bottom() - 4.0);
        painter.line_segment([vertex, p_top], stroke);
        painter.line_segment([vertex, p_right], stroke);
        // Angular arc
        let steps = 8;
        let r = 8.0;
        let mut prev = None;
        for i in 0..=steps {
            let theta = (PI * 0.38) * (i as f32 / steps as f32);
            let pt = Pos2::new(vertex.x + r * theta.cos(), vertex.y - r * theta.sin());
            if let Some(prv) = prev {
                painter.line_segment([prv, pt], stroke);
            }
            prev = Some(pt);
        }
    }

    /// AutoCAD Area icon: Polygon with diagonal hatch lines
    pub fn draw_area(painter: &Painter, rect: Rect, stroke: Stroke) {
        let p1 = Pos2::new(rect.left() + 5.0, rect.bottom() - 5.0);
        let p2 = Pos2::new(rect.right() - 5.0, rect.bottom() - 5.0);
        let p3 = Pos2::new(rect.right() - 7.0, rect.top() + 6.0);
        let p4 = Pos2::new(rect.left() + 8.0, rect.top() + 4.0);
        painter.line_segment([p1, p2], stroke);
        painter.line_segment([p2, p3], stroke);
        painter.line_segment([p3, p4], stroke);
        painter.line_segment([p4, p1], stroke);
        // Hatch lines
        let h_stroke = Stroke::new(1.0_f32, stroke.color.gamma_multiply(0.6));
        painter.line_segment([Pos2::new(p1.x + 3.0, p1.y), Pos2::new(p1.x + 8.0, p1.y - 8.0)], h_stroke);
        painter.line_segment([Pos2::new(p1.x + 7.0, p1.y), Pos2::new(p1.x + 13.0, p1.y - 9.0)], h_stroke);
    }

    /// AutoCAD 3D Cube icon: Isometric cube with dashed rear edge
    pub fn draw_cube3d(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let w = 8.0;
        let h = 8.0;
        let d = 4.0;
        // Front face
        let f1 = Pos2::new(c.x - w * 0.5, c.y);
        let f2 = Pos2::new(c.x + w * 0.5 - d, c.y);
        let f3 = Pos2::new(c.x + w * 0.5 - d, c.y + h * 0.7);
        let f4 = Pos2::new(c.x - w * 0.5, c.y + h * 0.7);
        painter.line_segment([f1, f2], stroke);
        painter.line_segment([f2, f3], stroke);
        painter.line_segment([f3, f4], stroke);
        painter.line_segment([f4, f1], stroke);
        // Back top edges
        let b1 = Pos2::new(f1.x + d, f1.y - d);
        let b2 = Pos2::new(f2.x + d, f2.y - d);
        let b3 = Pos2::new(f3.x + d, f3.y - d);
        painter.line_segment([f1, b1], stroke);
        painter.line_segment([f2, b2], stroke);
        painter.line_segment([b1, b2], stroke);
        painter.line_segment([b2, b3], stroke);
        painter.line_segment([f3, b3], stroke);
        // Dashed rear edge
        let d_stroke = Stroke::new(1.0_f32, stroke.color.gamma_multiply(0.6));
        let b4 = Pos2::new(f4.x + d, f4.y - d);
        painter.line_segment([b1, b4], d_stroke);
        painter.line_segment([f4, b4], d_stroke);
    }

    /// AutoCAD 3D Pyramid icon
    pub fn draw_pyramid3d(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let apex = Pos2::new(c.x, rect.top() + 4.0);
        let a = Pos2::new(rect.left() + 4.0, rect.bottom() - 4.0);
        let b = Pos2::new(rect.left() + 15.0, rect.bottom() - 4.0);
        let d = Pos2::new(rect.right() - 4.0, rect.bottom() - 8.0);
        painter.line_segment([apex, a], stroke);
        painter.line_segment([apex, b], stroke);
        painter.line_segment([apex, d], stroke);
        painter.line_segment([a, b], stroke);
        painter.line_segment([b, d], stroke);
        // Hidden rear edge dashed
        let hidden = Pos2::new(rect.left() + 9.0, rect.bottom() - 9.0);
        let d_stroke = Stroke::new(1.0_f32, stroke.color.gamma_multiply(0.6));
        painter.line_segment([a, hidden], d_stroke);
        painter.line_segment([hidden, d], d_stroke);
        painter.line_segment([apex, hidden], d_stroke);
    }

    /// AutoCAD 3D Sphere icon: Circle with equator & meridian
    pub fn draw_sphere3d(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let r = 8.5;
        painter.circle_stroke(c, r, stroke);
        // Equator ellipse front
        let steps = 12;
        let mut prev = None;
        for i in 0..=steps {
            let theta = PI * (i as f32 / steps as f32);
            let pt = Pos2::new(c.x + r * theta.cos(), c.y + (r * 0.35) * theta.sin());
            if let Some(prv) = prev {
                painter.line_segment([prv, pt], stroke);
            }
            prev = Some(pt);
        }
        // Equator back dashed
        let d_stroke = Stroke::new(1.0_f32, stroke.color.gamma_multiply(0.5));
        let mut prev_b = None;
        for i in 0..=steps {
            let theta = PI + PI * (i as f32 / steps as f32);
            let pt = Pos2::new(c.x + r * theta.cos(), c.y + (r * 0.35) * theta.sin());
            if let Some(prv) = prev_b {
                if i % 2 == 0 {
                    painter.line_segment([prv, pt], d_stroke);
                }
            }
            prev_b = Some(pt);
        }
    }

    /// AutoCAD 3D Orbit icon: 3D orbit ring with cardinal arrows
    pub fn draw_orbit3d(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let r = 8.0;
        painter.circle_stroke(c, r, stroke);
        let arm = 4.0;
        // Top, Bottom, Left, Right tiny arrows
        painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x - arm, c.y - r - 2.5)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x + arm, c.y - r - 2.5)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y + r), Pos2::new(c.x - arm, c.y + r + 2.5)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y + r), Pos2::new(c.x + arm, c.y + r + 2.5)], stroke);
    }

    /// AutoCAD Move / Translate icon: 4-way cross arrows
    pub fn draw_move(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let len = 7.0;
        painter.line_segment([Pos2::new(c.x - len, c.y), Pos2::new(c.x + len, c.y)], stroke);
        painter.line_segment([Pos2::new(c.x, c.y - len), Pos2::new(c.x, c.y + len)], stroke);
        // Arrowheads
        let a = 2.5;
        painter.line_segment([Pos2::new(c.x + len, c.y), Pos2::new(c.x + len - a, c.y - a)], stroke);
        painter.line_segment([Pos2::new(c.x + len, c.y), Pos2::new(c.x + len - a, c.y + a)], stroke);
        painter.line_segment([Pos2::new(c.x - len, c.y), Pos2::new(c.x - len + a, c.y - a)], stroke);
        painter.line_segment([Pos2::new(c.x - len, c.y), Pos2::new(c.x - len + a, c.y + a)], stroke);
    }

    /// AutoCAD Rotate icon: Curved arrow
    pub fn draw_rotate(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let r = 7.0;
        let steps = 14;
        let mut prev = None;
        for i in 0..=steps {
            let theta = PI * 0.2 + (PI * 1.5) * (i as f32 / steps as f32);
            let pt = Pos2::new(c.x + r * theta.cos(), c.y + r * theta.sin());
            if let Some(prv) = prev {
                painter.line_segment([prv, pt], stroke);
            }
            prev = Some(pt);
        }
        if let Some(tip) = prev {
            painter.line_segment([tip, tip + Vec2::new(3.0, -3.0)], stroke);
            painter.line_segment([tip, tip + Vec2::new(-1.0, -4.0)], stroke);
        }
        painter.circle_filled(c, 2.0, stroke.color);
    }

    /// AutoCAD Mirror / Reflect icon: Symmetrical triangles across dashed line
    pub fn draw_mirror(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        // Dashed mirror line
        let d_stroke = Stroke::new(1.0_f32, stroke.color.gamma_multiply(0.7));
        painter.line_segment([Pos2::new(c.x, rect.top() + 3.0), Pos2::new(c.x, rect.bottom() - 3.0)], d_stroke);
        // Left triangle
        let l_top = Pos2::new(c.x - 3.0, c.y - 6.0);
        let l_bot = Pos2::new(c.x - 3.0, c.y + 6.0);
        let l_tip = Pos2::new(c.x - 8.0, c.y);
        painter.line_segment([l_top, l_bot], stroke);
        painter.line_segment([l_bot, l_tip], stroke);
        painter.line_segment([l_tip, l_top], stroke);
        // Right triangle (mirrored)
        let r_top = Pos2::new(c.x + 3.0, c.y - 6.0);
        let r_bot = Pos2::new(c.x + 3.0, c.y + 6.0);
        let r_tip = Pos2::new(c.x + 8.0, c.y);
        painter.line_segment([r_top, r_bot], stroke);
        painter.line_segment([r_bot, r_tip], stroke);
        painter.line_segment([r_tip, r_top], stroke);
    }

    /// AutoCAD Midpoint Osnap icon: Triangle marker on segment
    pub fn draw_midpoint_snap(painter: &Painter, rect: Rect, stroke: Stroke) {
        let y = rect.center().y;
        painter.line_segment([Pos2::new(rect.left() + 3.0, y), Pos2::new(rect.right() - 3.0, y)], stroke);
        let c = rect.center();
        let t_top = Pos2::new(c.x, y - 6.0);
        let t_left = Pos2::new(c.x - 4.5, y + 2.0);
        let t_right = Pos2::new(c.x + 4.5, y + 2.0);
        painter.line_segment([t_top, t_left], stroke);
        painter.line_segment([t_left, t_right], stroke);
        painter.line_segment([t_right, t_top], stroke);
    }

    /// AutoCAD Intersection Osnap icon: Cross mark on intersection
    pub fn draw_intersect_snap(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        let len = 8.0;
        // Two intersecting lines
        painter.line_segment([Pos2::new(c.x - len, c.y - len * 0.5), Pos2::new(c.x + len, c.y + len * 0.5)], stroke);
        painter.line_segment([Pos2::new(c.x - len, c.y + len * 0.5), Pos2::new(c.x + len, c.y - len * 0.5)], stroke);
        // Cross marker at intersection
        let ch = 3.5;
        let c_stroke = Stroke::new(1.8_f32, Color32::from_rgb(0, 230, 118)); // AutoCAD Green OSNAP!
        painter.line_segment([Pos2::new(c.x - ch, c.y - ch), Pos2::new(c.x + ch, c.y + ch)], c_stroke);
        painter.line_segment([Pos2::new(c.x - ch, c.y + ch), Pos2::new(c.x + ch, c.y - ch)], c_stroke);
    }

    /// AutoCAD Export to Whiteboard icon
    pub fn draw_insert_whiteboard(painter: &Painter, rect: Rect, stroke: Stroke) {
        let c = rect.center();
        // Whiteboard rectangle
        painter.rect_stroke(Rect::from_center_size(c + Vec2::new(0.0, 2.0), Vec2::new(16.0, 12.0)), 1.5, stroke);
        // Arrow pointing in from top
        let arr_col = Color32::from_rgb(0, 230, 118);
        let a_stroke = Stroke::new(2.0_f32, arr_col);
        let top = Pos2::new(c.x, rect.top() + 2.0);
        let bot = Pos2::new(c.x, c.y + 2.0);
        painter.line_segment([top, bot], a_stroke);
        painter.line_segment([bot, bot + Vec2::new(-3.0, -3.5)], a_stroke);
        painter.line_segment([bot, bot + Vec2::new(3.0, -3.5)], a_stroke);
    }
}
