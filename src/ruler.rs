use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, Vec2};
use std::f32::consts::PI;

use crate::theme::Palette;

const SNAP_THRESHOLD: f32 = 32.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RulerKind {
    None,
    Ruler,      // Straight glassmorphic ruler (full-screen length)
    Protractor, // Circular glassmorphic protractor / compass
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveRulerEdge {
    None,
    RulerTop,
    RulerBottom,
    ProtractorCircumference,
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

    // Active locked edge during inking stroke
    pub active_stroke_edge: ActiveRulerEdge,

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
            length: 6000.0, // Full-screen spanning length
            width: 84.0,
            radius: 160.0,

            active_stroke_edge: ActiveRulerEdge::None,

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
                let normal = self.normal();
                let v = rel.dot(normal);
                let half_w = self.width * 0.5;

                // 1. Center circular dial (radius 36px)
                if rel.length() <= 36.0 {
                    return RulerHit::Dial;
                }

                // 2. Straight measuring edges:
                // Top edge is at v = -half_w. Snap zone: 32px outside, 12px inside.
                if v >= -half_w - SNAP_THRESHOLD && v <= -half_w + 12.0 {
                    return RulerHit::EdgeTop;
                }
                // Bottom edge is at v = +half_w. Snap zone: 12px inside, 32px outside.
                if v >= half_w - 12.0 && v <= half_w + SNAP_THRESHOLD {
                    return RulerHit::EdgeBottom;
                }

                // 3. Inside ruler body: click drags ruler, strictly prevents accidental inking inside!
                if v.abs() < half_w - 12.0 {
                    return RulerHit::Body;
                }

                RulerHit::None
            }
            RulerKind::Protractor => {
                let rel = p - self.center;
                let d = rel.length();

                // 1. Center dial (radius 36px)
                if d <= 36.0 {
                    return RulerHit::Dial;
                }

                // 2. Circumference edge snap (outer measuring edge):
                // Snap zone: 12px inside to 32px outside
                if d >= self.radius - 12.0 && d <= self.radius + SNAP_THRESHOLD {
                    return RulerHit::ProtractorEdge;
                }

                // 3. Inside protractor disk: click drags protractor, strictly prevents accidental inking inside!
                if d < self.radius - 12.0 {
                    return RulerHit::Body;
                }

                RulerHit::None
            }
            RulerKind::None => RulerHit::None,
        }
    }

    /// Snaps pointer position P to ruler edge or protractor circle.
    /// Returns Some(snapped_screen_pos) if along edge or clamped from interior, else None.
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
                let half_w = self.width * 0.5;

                // 1. Active locked edge during inking stroke:
                // Locks 100% of the stroke to the straight line across the entire screen!
                match self.active_stroke_edge {
                    ActiveRulerEdge::RulerTop => {
                        return Some(self.center + dir * u - normal * half_w);
                    }
                    ActiveRulerEdge::RulerBottom => {
                        return Some(self.center + dir * u + normal * half_w);
                    }
                    ActiveRulerEdge::ProtractorCircumference => {}
                    ActiveRulerEdge::None => {}
                }

                // 2. Freehand snap or stencil barrier:
                // Top edge snap
                if v >= -half_w - SNAP_THRESHOLD && v <= -half_w + 12.0 {
                    return Some(self.center + dir * u - normal * half_w);
                }
                // Bottom edge snap
                if v >= half_w - 12.0 && v <= half_w + SNAP_THRESHOLD {
                    return Some(self.center + dir * u + normal * half_w);
                }
                // Stencil barrier: ink inside ruler body is blocked and clamped to the nearest edge
                if v.abs() < half_w {
                    let snapped_v = if v < 0.0 { -half_w } else { half_w };
                    return Some(self.center + dir * u + normal * snapped_v);
                }

                None
            }
            RulerKind::Protractor => {
                let rel = p - self.center;
                let d = rel.length();
                let dir = if d > 1e-4 { rel / d } else { Vec2::X };

                // 1. Active locked circular stroke during inking:
                // Locks 100% of the stroke to the perfect circle circumference!
                if self.active_stroke_edge == ActiveRulerEdge::ProtractorCircumference {
                    return Some(self.center + dir * self.radius);
                }

                // 2. Freehand snap or stencil barrier:
                // Circumference snap
                if d >= self.radius - 12.0 && d <= self.radius + SNAP_THRESHOLD {
                    return Some(self.center + dir * self.radius);
                }
                // Stencil barrier: ink inside protractor disk is blocked and clamped to the circumference
                if d < self.radius {
                    return Some(self.center + dir * self.radius);
                }

                None
            }
            RulerKind::None => None,
        }
    }

    /// Handle pointer events and mouse wheel rotation
    pub fn handle_events(&mut self, ui: &egui::Ui, canvas_rect: Rect) -> bool {
        if self.kind == RulerKind::None {
            self.active_stroke_edge = ActiveRulerEdge::None;
            return false;
        }

        let pointer = ui.input(|i| i.pointer.clone());
        let pointer_pos = pointer.interact_pos().or(pointer.latest_pos());

        // 1. Mouse wheel: rotates angle by 1 deg (or 5 deg with Shift) when near dial/center
        if let Some(pos) = pointer_pos {
            let rel = pos - self.center;
            // Only consume wheel rotation if near center (<= 140px) and not zooming (no Ctrl)
            if rel.length() <= 140.0 && !ui.input(|i| i.modifiers.ctrl) {
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
        }

        // 2. Primary button click & drag
        if pointer.primary_pressed() {
            if let Some(pos) = pointer_pos {
                let hit = self.hit_test(pos);
                match hit {
                    RulerHit::Dial => {
                        // Double click on dial resets angle to horizontal 0 deg (or nearest 45 deg)
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
                        self.active_stroke_edge = ActiveRulerEdge::None;
                        return true;
                    }
                    RulerHit::Body => {
                        // Start dragging ruler body - consumes event so NO ink is drawn inside!
                        self.is_dragging = true;
                        self.drag_start_pointer = pos;
                        self.drag_start_center = self.center;
                        self.active_stroke_edge = ActiveRulerEdge::None;
                        return true;
                    }
                    RulerHit::EdgeTop => {
                        // Lock stroke to Top Edge for guaranteed straight line
                        self.active_stroke_edge = ActiveRulerEdge::RulerTop;
                        return false; // Let app handle inking
                    }
                    RulerHit::EdgeBottom => {
                        // Lock stroke to Bottom Edge for guaranteed straight line
                        self.active_stroke_edge = ActiveRulerEdge::RulerBottom;
                        return false; // Let app handle inking
                    }
                    RulerHit::ProtractorEdge => {
                        // Lock stroke to Protractor Circumference for guaranteed circular line
                        self.active_stroke_edge = ActiveRulerEdge::ProtractorCircumference;
                        return false; // Let app handle inking
                    }
                    RulerHit::None => {
                        self.active_stroke_edge = ActiveRulerEdge::None;
                    }
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
                    self.center.x = self
                        .center
                        .x
                        .clamp(canvas_rect.left() + 40.0, canvas_rect.right() - 40.0);
                    self.center.y = self
                        .center
                        .y
                        .clamp(canvas_rect.top() + 40.0, canvas_rect.bottom() - 40.0);
                    return true;
                }
            }
        } else {
            self.is_dragging = false;
            self.is_rotating = false;
            self.active_stroke_edge = ActiveRulerEdge::None;
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
    #[allow(dead_code)]
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
            (
                -inner_l,
                inner_w,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
            ),
            (
                -inner_l,
                -inner_w,
                std::f32::consts::PI,
                std::f32::consts::PI * 1.5_f32,
            ),
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
        let half_l = (self.length * 0.5).max(3200.0);
        let half_w = self.width * 0.5;

        // 1. Full-screen length translucent frosted glass body (no rounded end caps)
        let p_top_l = self.center - dir * half_l - normal * half_w;
        let p_top_r = self.center + dir * half_l - normal * half_w;
        let p_bot_r = self.center + dir * half_l + normal * half_w;
        let p_bot_l = self.center - dir * half_l + normal * half_w;

        // Ambient contact shadow
        let s_top_l = p_top_l + Vec2::new(0.0, 2.0);
        let s_top_r = p_top_r + Vec2::new(0.0, 2.0);
        let s_bot_r = p_bot_r + Vec2::new(0.0, 2.0);
        let s_bot_l = p_bot_l + Vec2::new(0.0, 2.0);
        painter.add(egui::Shape::convex_polygon(
            vec![s_top_l, s_top_r, s_bot_r, s_bot_l],
            if is_dark {
                Color32::from_black_alpha(45)
            } else {
                Color32::from_black_alpha(15)
            },
            Stroke::NONE,
        ));

        // Elevation drop shadow
        let d_top_l = p_top_l + Vec2::new(0.0, 8.0);
        let d_top_r = p_top_r + Vec2::new(0.0, 8.0);
        let d_bot_r = p_bot_r + Vec2::new(0.0, 8.0);
        let d_bot_l = p_bot_l + Vec2::new(0.0, 8.0);
        painter.add(egui::Shape::convex_polygon(
            vec![d_top_l, d_top_r, d_bot_r, d_bot_l],
            if is_dark {
                Color32::from_black_alpha(75)
            } else {
                Color32::from_black_alpha(25)
            },
            Stroke::NONE,
        ));

        // Translucent frosted glass tint: very clear so you can read text & grid underneath!
        let body_fill = if is_dark {
            Color32::from_rgba_unmultiplied(22, 28, 38, 155) // 60% opacity dark acrylic
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 165) // 64% opacity luminous glass
        };
        let rim_stroke = Stroke::new(
            1.2_f32,
            if is_dark {
                Color32::from_rgba_unmultiplied(255, 255, 255, 55)
            } else {
                Color32::from_rgba_unmultiplied(255, 255, 255, 230)
            },
        );
        painter.add(egui::Shape::convex_polygon(
            vec![p_top_l, p_top_r, p_bot_r, p_bot_l],
            body_fill,
            rim_stroke,
        ));

        // Subsurface Bevel Reflection (Vệt sáng phản xạ nhẹ mép trên)
        let sheen_top_l = self.center - dir * half_l - normal * (half_w - 2.0);
        let sheen_top_r = self.center + dir * half_l - normal * (half_w - 2.0);
        painter.line_segment(
            [sheen_top_l, sheen_top_r],
            Stroke::new(
                1.0_f32,
                if is_dark {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 38)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 180)
                },
            ),
        );

        // Snap Edge Accent Guidelines (Đường chỉ dẫn mép thước sắc nét)
        painter.line_segment(
            [p_top_l, p_top_r],
            Stroke::new(1.0_f32, p.accent.gamma_multiply(0.40)),
        );
        painter.line_segment(
            [p_bot_l, p_bot_r],
            Stroke::new(1.0_f32, p.accent.gamma_multiply(0.30)),
        );

        // Ticks & Numeric Labels along visible portion of the straight edges
        let tick_color = p.text_muted.gamma_multiply(0.85);
        let major_tick_color = p.text;

        let clip = painter.clip_rect();
        let corners = [
            clip.left_top(),
            clip.right_top(),
            clip.left_bottom(),
            clip.right_bottom(),
        ];
        let mut min_u = f32::MAX;
        let mut max_u = f32::MIN;
        for c in corners {
            let u = (c - self.center).dot(dir);
            if u < min_u {
                min_u = u;
            }
            if u > max_u {
                max_u = u;
            }
        }
        min_u = (min_u - 50.0).max(-half_l);
        max_u = (max_u + 50.0).min(half_l);

        let start_tick = (min_u / 8.0).floor() as i32;
        let end_tick = (max_u / 8.0).ceil() as i32;

        for i in start_tick..=end_tick {
            let offset_x = i as f32 * 8.0;

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

            // Number label for cm (skip near center dial so it doesn't overlap dial)
            if is_cm && offset_x.abs() > 42.0 {
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

        // Center Dial (Apple Style)
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
            if is_dark {
                Color32::from_black_alpha(55)
            } else {
                Color32::from_black_alpha(20)
            },
        );
        // Deep depth shadow
        painter.circle_filled(
            center + Vec2::new(0.0, 10.0),
            radius,
            if is_dark {
                Color32::from_black_alpha(90)
            } else {
                Color32::from_black_alpha(32)
            },
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
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0_f32, p.accent.gamma_multiply(0.40)),
        );

        // 3. Inner guideline rings
        painter.circle_stroke(
            center,
            radius * 0.70,
            Stroke::new(0.8_f32, p.border.gamma_multiply(0.6)),
        );
        painter.circle_stroke(
            center,
            radius * 0.35,
            Stroke::new(0.8_f32, p.border.gamma_multiply(0.5)),
        );

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
            if is_dark {
                Color32::from_black_alpha(60)
            } else {
                Color32::from_black_alpha(25)
            },
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
        painter.line_segment(
            [center, center + dir * 21.0],
            Stroke::new(2.0_f32, p.accent),
        );

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
            width: 80.0, // Top edge at y = 160, bottom edge at y = 240
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

    #[test]
    fn test_ruler_full_screen_length_and_body_barrier() {
        let mut ruler = RulerState {
            kind: RulerKind::Ruler,
            center: Pos2::new(200.0, 200.0),
            angle_rad: 0.0, // Horizontal
            length: 6000.0,
            width: 80.0, // Top edge at y = 160, bottom edge at y = 240
            ..Default::default()
        };

        // 1. Far away along x-axis (e.g. x = 2500) still snaps smoothly (full-screen length)
        let far_x = Pos2::new(2500.0, 162.0);
        let snapped_far = ruler.snap_point(far_x);
        assert!(snapped_far.is_some());
        assert_eq!(snapped_far.unwrap().y, 160.0);

        // 2. Pointer deep inside body (y = 200) is clamped to nearest edge, NEVER inside body!
        let inside_p = Pos2::new(300.0, 200.0);
        let clamped = ruler.snap_point(inside_p);
        assert!(clamped.is_some());
        assert!(clamped.unwrap().y == 160.0 || clamped.unwrap().y == 240.0);

        // 3. Active locked edge: when drawing top edge, points stay 100% on y = 160
        ruler.active_stroke_edge = ActiveRulerEdge::RulerTop;
        let wandering_p = Pos2::new(450.0, 195.0); // Hand slipped into body
        let locked_sp = ruler.snap_point(wandering_p);
        assert!(locked_sp.is_some());
        assert_eq!(locked_sp.unwrap().x, 450.0);
        assert_eq!(locked_sp.unwrap().y, 160.0);
    }

    #[test]
    fn test_protractor_body_barrier_and_lock() {
        let mut protractor = RulerState {
            kind: RulerKind::Protractor,
            center: Pos2::new(200.0, 200.0),
            radius: 100.0,
            ..Default::default()
        };

        // 1. Pointer inside disk (d = 40.0) is clamped to outer circumference
        let inside_p = Pos2::new(240.0, 200.0);
        let clamped = protractor.snap_point(inside_p);
        assert!(clamped.is_some());
        let sp = clamped.unwrap();
        assert_eq!(sp.x, 300.0); // 200 + 100
        assert_eq!(sp.y, 200.0);

        // 2. Active locked circumference: stroke stays 100% on circle
        protractor.active_stroke_edge = ActiveRulerEdge::ProtractorCircumference;
        let wandering_p = Pos2::new(200.0, 150.0); // Slipped towards center (d = 50)
        let locked_sp = protractor.snap_point(wandering_p);
        assert!(locked_sp.is_some());
        let sp = locked_sp.unwrap();
        assert_eq!(sp.x, 200.0);
        assert_eq!(sp.y, 100.0); // 200 - 100 radius
    }
}
