use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::PI;

use crate::theme::Palette;

const SNAP_THRESHOLD: f32 = 28.0;
const CORNER_RADIUS: f32 = 14.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerKind {
    None,
    Ruler,       // Straight glassmorphic ruler
    Protractor,  // Circular glassmorphic protractor / compass
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerHit {
    None,
    Body,
    Dial,
    EdgeTop,
    EdgeBottom,
    ProtractorEdge,
}

#[derive(Debug, Clone)]
pub struct RulerState {
    pub kind: RulerKind,
    pub center: Pos2,
    pub angle_rad: f32,
    pub length: f32,
    pub width: f32,
    pub radius: f32,

    // Gesture tracking
    pub is_dragging: bool,
    pub is_rotating: bool,
    pub drag_start_pointer: Pos2,
    pub drag_start_center: Pos2,
    pub rotate_start_angle: f32,
    pub rotate_initial_angle: f32,
    pub last_click_time: std::time::Instant,
}

impl Default for RulerState {
    fn default() -> Self {
        Self {
            kind: RulerKind::None,
            center: Pos2::new(560.0, 360.0),
            angle_rad: 0.0,
            length: 560.0,
            width: 88.0,
            radius: 160.0,

            is_dragging: false,
            is_rotating: false,
            drag_start_pointer: Pos2::ZERO,
            drag_start_center: Pos2::ZERO,
            rotate_start_angle: 0.0,
            rotate_initial_angle: 0.0,
            last_click_time: std::time::Instant::now(),
        }
    }
}

impl RulerState {
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.kind != RulerKind::None
    }

    pub fn toggle_ruler(&mut self) {
        self.kind = match self.kind {
            RulerKind::Ruler => RulerKind::None,
            _ => RulerKind::Ruler,
        };
    }

    pub fn toggle_protractor(&mut self) {
        self.kind = match self.kind {
            RulerKind::Protractor => RulerKind::None,
            _ => RulerKind::Protractor,
        };
    }

    /// Unit direction vector along ruler length
    pub fn dir(&self) -> Vec2 {
        Vec2::new(self.angle_rad.cos(), self.angle_rad.sin())
    }

    /// Unit normal vector perpendicular to ruler (towards bottom edge)
    pub fn normal(&self) -> Vec2 {
        Vec2::new(-self.angle_rad.sin(), self.angle_rad.cos())
    }

    /// Check if cursor hits ruler body, dial or background
    pub fn hit_test(&self, p: Pos2) -> RulerHit {
        if self.kind == RulerKind::None {
            return RulerHit::None;
        }

        match self.kind {
            RulerKind::Ruler => {
                let rel = p - self.center;
                let dir = self.dir();
                let normal = self.normal();
                let u = rel.dot(dir);
                let v = rel.dot(normal);

                let half_l = self.length * 0.5;
                let half_w = self.width * 0.5;

                // Center circular dial (radius 36px)
                if rel.length() <= 36.0 {
                    return RulerHit::Dial;
                }

                // Check distance to straight measuring edges for drawing snap hint
                let straight_limit = half_l - CORNER_RADIUS;
                if u >= -straight_limit - 8.0 && u <= straight_limit + 8.0 {
                    let d_top = (v + half_w).abs();
                    let d_bottom = (v - half_w).abs();
                    if d_top <= SNAP_THRESHOLD {
                        return RulerHit::EdgeTop;
                    }
                    if d_bottom <= SNAP_THRESHOLD {
                        return RulerHit::EdgeBottom;
                    }
                }

                // Inside ruler body (including rounded squircle ends)
                if u >= -half_l && u <= half_l && v >= -half_w && v <= half_w {
                    return RulerHit::Body;
                }
                RulerHit::None
            }
            RulerKind::Protractor => {
                let rel = p - self.center;
                let d = rel.length();

                // Center dial (radius 36px)
                if d <= 36.0 {
                    return RulerHit::Dial;
                }

                // Circumference edge snap
                if (d - self.radius).abs() <= SNAP_THRESHOLD {
                    return RulerHit::ProtractorEdge;
                }

                // Inside protractor disk
                if d <= self.radius {
                    return RulerHit::Body;
                }

                RulerHit::None
            }
            RulerKind::None => RulerHit::None,
        }
    }

    /// Snaps pointer position P to ruler edge or protractor circle.
    /// Returns Some(snapped_screen_pos) if within SNAP_THRESHOLD, else None.
    pub fn snap_point(&self, p: Pos2) -> Option<Pos2> {
        if self.kind == RulerKind::None {
            return None;
        }

        match self.kind {
            RulerKind::Ruler => {
                let rel = p - self.center;
                let dir = self.dir();
                let normal = self.normal();
                let u = rel.dot(dir);
                let v = rel.dot(normal);

                let half_l = self.length * 0.5;
                let half_w = self.width * 0.5;
                let straight_limit = half_l - CORNER_RADIUS;

                // Within longitudinal bounds of the straight measuring edge (+ snap margin)
                if u >= -straight_limit - 12.0 && u <= straight_limit + 12.0 {
                    let d_top = (v + half_w).abs();
                    let d_bottom = (v - half_w).abs();

                    if d_top.min(d_bottom) <= SNAP_THRESHOLD {
                        let snapped_v = if d_top < d_bottom { -half_w } else { half_w };
                        let snapped_u = u.clamp(-straight_limit, straight_limit);
                        let snapped_p = self.center + dir * snapped_u + normal * snapped_v;
                        return Some(snapped_p);
                    }
                }
                None
            }
            RulerKind::Protractor => {
                let rel = p - self.center;
                let d = rel.length();

                if (d - self.radius).abs() <= SNAP_THRESHOLD {
                    let dir = if d > 1e-4 { rel / d } else { Vec2::X };
                    let snapped_p = self.center + dir * self.radius;
                    return Some(snapped_p);
                }
                None
            }
            RulerKind::None => None,
        }
    }

    /// Handle pointer events and mouse wheel rotation
    pub fn handle_events(&mut self, ui: &egui::Ui, canvas_rect: Rect) -> bool {
        if self.kind == RulerKind::None {
            return false;
        }

        let pointer = ui.input(|i| i.pointer.clone());
        let pointer_pos = pointer.interact_pos().or(pointer.latest_pos());

        // 1. Mouse wheel over ruler body rotates angle by 1 degree (or 5 degrees with Shift)
        if let Some(pos) = pointer_pos {
            let hit = self.hit_test(pos);
            if hit != RulerHit::None {
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll.abs() > 0.1 {
                    let step = if ui.input(|i| i.modifiers.shift) {
                        5.0_f32.to_radians()
                    } else {
                        1.0_f32.to_radians()
                    };
                    self.angle_rad += scroll.signum() * step;
                    self.normalize_angle();
                    return true;
                }
            }
        }

        // 2. Primary button click & drag
        if pointer.primary_pressed() {
            if let Some(pos) = pointer_pos {
                let hit = self.hit_test(pos);
                if hit == RulerHit::Dial {
                    // Double click on dial resets angle to horizontal 0 deg (or nearest 90 deg)
                    let now = std::time::Instant::now();
                    if now.duration_since(self.last_click_time).as_millis() < 300 {
                        self.snap_angle_to_cardinal();
                        self.last_click_time = now;
                        return true;
                    }
                    self.last_click_time = now;

                    // Start rotating from dial
                    self.is_rotating = true;
                    self.rotate_start_angle = (pos.y - self.center.y).atan2(pos.x - self.center.x);
                    self.rotate_initial_angle = self.angle_rad;
                    return true;
                } else if hit == RulerHit::Body {
                    // Start dragging ruler body
                    self.is_dragging = true;
                    self.drag_start_pointer = pos;
                    self.drag_start_center = self.center;
                    return true;
                }
            }
        }

        if pointer.primary_down() {
            if self.is_rotating {
                if let Some(pos) = pointer_pos {
                    let cur_angle = (pos.y - self.center.y).atan2(pos.x - self.center.x);
                    let delta = cur_angle - self.rotate_start_angle;
                    self.angle_rad = self.rotate_initial_angle + delta;
                    self.normalize_angle();
                    return true;
                }
            } else if self.is_dragging {
                if let Some(pos) = pointer_pos {
                    let delta = pos - self.drag_start_pointer;
                    self.center = self.drag_start_center + delta;
                    // Clamp within canvas rect margin
                    self.center.x = self.center.x.clamp(canvas_rect.left() + 40.0, canvas_rect.right() - 40.0);
                    self.center.y = self.center.y.clamp(canvas_rect.top() + 40.0, canvas_rect.bottom() - 40.0);
                    return true;
                }
            }
        } else {
            self.is_dragging = false;
            self.is_rotating = false;
        }

        self.is_dragging || self.is_rotating
    }

    fn normalize_angle(&mut self) {
        while self.angle_rad > PI {
            self.angle_rad -= 2.0 * PI;
        }
        while self.angle_rad < -PI {
            self.angle_rad += 2.0 * PI;
        }
    }

    pub fn snap_angle_to_cardinal(&mut self) {
        // Snap to nearest 45 degree angle (0, 45, 90, 135, 180...)
        let deg = self.angle_rad.to_degrees();
        let step = 45.0;
        let rounded = (deg / step).round() * step;
        self.angle_rad = rounded.to_radians();
        self.normalize_angle();
    }

    pub fn angle_degrees(&self) -> f32 {
        let mut deg = self.angle_rad.to_degrees() % 360.0;
        if deg < 0.0 {
            deg += 360.0;
        }
        deg
    }

    /// Render ruler or protractor overlay onto painter
    pub fn render_overlay(&self, painter: &Painter, p: &Palette, is_dark: bool) {
        if self.kind == RulerKind::None {
            return;
        }

        match self.kind {
            RulerKind::Ruler => self.render_straight_ruler(painter, p, is_dark),
            RulerKind::Protractor => self.render_protractor(painter, p, is_dark),
            RulerKind::None => {}
        }
    }

    /// Generate vertices for an oriented rounded rectangle (Apple iOS Widget style squircle)
    pub fn oriented_rounded_rect_points(
        center: Pos2,
        dir: Vec2,
        normal: Vec2,
        length: f32,
        width: f32,
        radius: f32,
        pts_per_corner: usize,
    ) -> Vec<Pos2> {
        let half_l = length * 0.5;
        let half_w = width * 0.5;
        let r = radius.min(half_l * 0.45).min(half_w * 0.45);

        let inner_l = half_l - r;
        let inner_w = half_w - r;

        let mut pts = Vec::with_capacity((pts_per_corner + 1) * 4);

        // 4 corner arcs in clockwise order:
        // Top-Right: (+inner_l, -inner_w), angles -π/2 to 0
        // Bottom-Right: (+inner_l, +inner_w), angles 0 to +π/2
        // Bottom-Left: (-inner_l, +inner_w), angles +π/2 to +π
        // Top-Left: (-inner_l, -inner_w), angles +π to +3π/2
        let corners = [
            (inner_l, -inner_w, -std::f32::consts::FRAC_PI_2, 0.0_f32),
            (inner_l, inner_w, 0.0_f32, std::f32::consts::FRAC_PI_2),
            (-inner_l, inner_w, std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
            (-inner_l, -inner_w, std::f32::consts::PI, std::f32::consts::PI * 1.5_f32),
        ];

        for (cu, cv, start_a, end_a) in corners {
            for step in 0..=pts_per_corner {
                let t = step as f32 / pts_per_corner as f32;
                let a = start_a + (end_a - start_a) * t;
                let u = cu + r * a.cos();
                let v = cv + r * a.sin();
                pts.push(center + dir * u + normal * v);
            }
        }

        pts
    }

    fn render_straight_ruler(&self, painter: &Painter, p: &Palette, is_dark: bool) {
        let dir = self.dir();
        let normal = self.normal();
        let half_l = self.length * 0.5;
        let half_w = self.width * 0.5;
        let straight_limit = half_l - CORNER_RADIUS;

        // 1. Apple-style Multi-Layered Frosted Glass Shadows
        // Layer A: Ambient soft contact shadow
        let shadow_ambient = Self::oriented_rounded_rect_points(
            self.center + Vec2::new(0.0, 3.0),
            dir,
            normal,
            self.length + 2.0,
            self.width + 2.0,
            CORNER_RADIUS + 1.0,
            5,
        );
        painter.add(egui::Shape::convex_polygon(
            shadow_ambient,
            if is_dark { Color32::from_black_alpha(55) } else { Color32::from_black_alpha(20) },
            Stroke::NONE,
        ));

        // Layer B: Deep elevation drop shadow (creates the floating iOS widget depth)
        let shadow_depth = Self::oriented_rounded_rect_points(
            self.center + Vec2::new(0.0, 10.0),
            dir,
            normal,
            self.length,
            self.width,
            CORNER_RADIUS,
            5,
        );
        painter.add(egui::Shape::convex_polygon(
            shadow_depth,
            if is_dark { Color32::from_black_alpha(90) } else { Color32::from_black_alpha(32) },
            Stroke::NONE,
        ));

        // 2. Apple iOS Widget Translucent Frosted Glass Body
        let body_pts = Self::oriented_rounded_rect_points(
            self.center,
            dir,
            normal,
            self.length,
            self.width,
            CORNER_RADIUS,
            6,
        );
        // Translucent frosted glass tint: very clear so you can read text & grid underneath!
        let body_fill = if is_dark {
            Color32::from_rgba_unmultiplied(22, 28, 38, 155) // 60% opacity dark acrylic
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 165) // 64% opacity luminous glass
        };
        // Apple Signature Specular Highlight Rim (Viền kính bắt sáng kiểu iPhone widget)
        let rim_stroke = Stroke::new(
            1.2_f32,
            if is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 55)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 230)
            },
        );
        painter.add(egui::Shape::convex_polygon(body_pts, body_fill, rim_stroke));

        // 3. Apple Subsurface Bevel Reflection (Vệt sáng phản xạ nhẹ mép trên)
        let sheen_l = straight_limit - 4.0;
        let sheen_start = self.center - dir * sheen_l - normal * (half_w - 2.0);
        let sheen_end = self.center + dir * sheen_l - normal * (half_w - 2.0);
        painter.line_segment(
            [sheen_start, sheen_end],
            Stroke::new(
                1.0_f32,
                if is_dark {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 38)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 180)
                },
            ),
        );

        // 4. Subtle Snap Edge Accent Guidelines (Đường chỉ dẫn mép thước sắc nét)
        let p_edge_top_l = self.center - dir * straight_limit - normal * half_w;
        let p_edge_top_r = self.center + dir * straight_limit - normal * half_w;
        painter.line_segment(
            [p_edge_top_l, p_edge_top_r],
            Stroke::new(1.0_f32, p.accent.gamma_multiply(0.40)),
        );
        let p_edge_bot_l = self.center - dir * straight_limit + normal * half_w;
        let p_edge_bot_r = self.center + dir * straight_limit + normal * half_w;
        painter.line_segment(
            [p_edge_bot_l, p_edge_bot_r],
            Stroke::new(1.0_f32, p.accent.gamma_multiply(0.30)),
        );

        // 5. Ruler Ticks & Numeric Labels along straight edges
        let tick_color = p.text_muted.gamma_multiply(0.85);
        let major_tick_color = p.text;
        let num_ticks = (self.length / 8.0) as i32;

        for i in -num_ticks / 2..=num_ticks / 2 {
            let offset_x = i as f32 * 8.0;
            // Only place ticks along the 100% straight measuring edge
            if offset_x.abs() > straight_limit - 4.0 {
                continue;
            }

            let is_cm = i % 10 == 0;
            let is_half_cm = i % 5 == 0 && !is_cm;

            let tick_len = if is_cm {
                15.0_f32
            } else if is_half_cm {
                9.5_f32
            } else {
                5.5_f32
            };

            // Top edge tick
            let p_edge_top = self.center + dir * offset_x - normal * half_w;
            let p_tick_top = p_edge_top + normal * tick_len;
            painter.line_segment(
                [p_edge_top, p_tick_top],
                Stroke::new(
                    if is_cm { 1.2_f32 } else { 0.8_f32 },
                    if is_cm { major_tick_color } else { tick_color },
                ),
            );

            // Bottom edge tick
            let p_edge_bottom = self.center + dir * offset_x + normal * half_w;
            let p_tick_bottom = p_edge_bottom - normal * tick_len;
            painter.line_segment(
                [p_edge_bottom, p_tick_bottom],
                Stroke::new(
                    if is_cm { 1.2_f32 } else { 0.8_f32 },
                    if is_cm { major_tick_color } else { tick_color },
                ),
            );

            // Number label for cm
            if is_cm && offset_x.abs() > 36.0 {
                let cm_num = (offset_x / 80.0).abs().round() as i32;
                let text_pos = p_tick_top + normal * 7.0;
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_CENTER,
                    format!("{}", cm_num),
                    egui::FontId::proportional(10.5),
                    p.text_muted,
                );
            }
        }

        // 6. Center Dial (Apple Style)
        self.render_center_dial(painter, p, is_dark);
    }

    fn render_protractor(&self, painter: &Painter, p: &Palette, is_dark: bool) {
        let center = self.center;
        let radius = self.radius;

        // 1. Apple-style Multi-Layered Frosted Glass Shadows
        // Ambient soft shadow
        painter.circle_filled(
            center + Vec2::new(0.0, 3.0),
            radius + 2.0,
            if is_dark { Color32::from_black_alpha(55) } else { Color32::from_black_alpha(20) },
        );
        // Deep depth shadow
        painter.circle_filled(
            center + Vec2::new(0.0, 10.0),
            radius,
            if is_dark { Color32::from_black_alpha(90) } else { Color32::from_black_alpha(32) },
        );

        // 2. Apple iOS Widget Translucent Frosted Glass Disk
        let fill = if is_dark {
            Color32::from_rgba_unmultiplied(22, 28, 38, 155) // 60% opacity dark acrylic
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 165) // 64% opacity luminous glass
        };
        // Apple Signature Specular Rim
        let rim_stroke = Stroke::new(
            1.2_f32,
            if is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 55)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 230)
            },
        );
        painter.circle(center, radius, fill, rim_stroke);

        // Outer circumference accent guideline
        painter.circle_stroke(center, radius, Stroke::new(1.0_f32, p.accent.gamma_multiply(0.40)));

        // 3. Inner guideline rings
        painter.circle_stroke(center, radius * 0.70, Stroke::new(0.8_f32, p.border.gamma_multiply(0.6)));
        painter.circle_stroke(center, radius * 0.35, Stroke::new(0.8_f32, p.border.gamma_multiply(0.5)));

        // 4. Degree ticks and labels around the circle
        for deg in (0..360).step_by(5) {
            let rad = (deg as f32).to_radians();
            let d_vec = Vec2::new(rad.cos(), rad.sin());

            let is_major = deg % 30 == 0;
            let is_cardinal = deg % 90 == 0;
            let is_medium = deg % 10 == 0 && !is_major;

            let tick_len = if is_cardinal {
                18.0_f32
            } else if is_major {
                14.0_f32
            } else if is_medium {
                9.0_f32
            } else {
                5.0_f32
            };

            let p_outer = center + d_vec * radius;
            let p_inner = center + d_vec * (radius - tick_len);

            painter.line_segment(
                [p_outer, p_inner],
                Stroke::new(
                    if is_major { 1.2_f32 } else { 0.8_f32 },
                    if is_major { p.text } else { p.text_muted },
                ),
            );

            // Major degree labels
            if is_major {
                let p_label = center + d_vec * (radius - 24.0);
                painter.text(
                    p_label,
                    egui::Align2::CENTER_CENTER,
                    format!("{}°", deg),
                    egui::FontId::proportional(9.5),
                    if is_cardinal { p.accent } else { p.text_muted },
                );
            }
        }

        // Center crosshair
        painter.line_segment(
            [center - Vec2::new(12.0, 0.0), center + Vec2::new(12.0, 0.0)],
            Stroke::new(1.2_f32, p.accent),
        );
        painter.line_segment(
            [center - Vec2::new(0.0, 12.0), center + Vec2::new(0.0, 12.0)],
            Stroke::new(1.2_f32, p.accent),
        );

        // Center Dial Readout
        self.render_center_dial(painter, p, is_dark);
    }

    fn render_center_dial(&self, painter: &Painter, p: &Palette, is_dark: bool) {
        let center = self.center;
        let dial_r = 34.0_f32;

        // Dial shadow
        painter.circle_filled(
            center + Vec2::new(0.0, 2.5),
            dial_r + 1.0,
            if is_dark { Color32::from_black_alpha(60) } else { Color32::from_black_alpha(25) },
        );

        // Dial base - matching frosted glass
        let dial_fill = if is_dark {
            Color32::from_rgba_unmultiplied(20, 26, 36, 215)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 225)
        };
        let dial_rim = Stroke::new(
            1.2_f32,
            if is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 75)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 240)
            },
        );
        painter.circle(center, dial_r, dial_fill, dial_rim);

        // Inner circular guideline
        painter.circle_stroke(
            center,
            dial_r * 0.76,
            Stroke::new(0.8_f32, p.accent.gamma_multiply(0.28)),
        );

        // Center pivot dot
        painter.circle_filled(center, 3.5_f32, p.accent);

        // Angle indicator arrow
        let dir = self.dir();
        painter.line_segment([center, center + dir * 21.0], Stroke::new(2.0_f32, p.accent));

        // Numeric angle label
        let deg_text = format!("{:.0}°", self.angle_degrees());
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            deg_text,
            egui::FontId::proportional(12.5),
            p.text,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruler_snap_straight() {
        let ruler = RulerState {
            kind: RulerKind::Ruler,
            center: Pos2::new(200.0, 200.0),
            angle_rad: 0.0, // Horizontal
            length: 400.0,
            width: 80.0,    // Top edge at y = 160, bottom edge at y = 240
            ..Default::default()
        };

        // Pointer near top edge (y = 165, should snap to 160)
        let p = Pos2::new(220.0, 165.0);
        let snapped = ruler.snap_point(p);
        assert!(snapped.is_some());
        let sp = snapped.unwrap();
        assert_eq!(sp.x, 220.0);
        assert_eq!(sp.y, 160.0);

        // Pointer far away (y = 100), no snap
        let far_p = Pos2::new(220.0, 100.0);
        assert!(ruler.snap_point(far_p).is_none());
    }

    #[test]
    fn test_protractor_snap_circular() {
        let protractor = RulerState {
            kind: RulerKind::Protractor,
            center: Pos2::new(100.0, 100.0),
            radius: 50.0,
            ..Default::default()
        };

        // Pointer near radius (d = 52.0)
        let p = Pos2::new(152.0, 100.0);
        let snapped = protractor.snap_point(p);
        assert!(snapped.is_some());
        let sp = snapped.unwrap();
        assert!((sp.x - 150.0).abs() < 1e-3);
        assert!((sp.y - 100.0).abs() < 1e-3);
    }
}
