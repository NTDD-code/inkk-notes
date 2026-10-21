use egui::{Color32, Mesh, Painter, Pos2, Rect, Stroke, Vec2};

use crate::model::{PaperType, PenType, Point, ShapeType, Stroke as NoteStroke};

pub struct NoteRenderer;

impl NoteRenderer {
    pub fn render_background<F>(
        painter: &Painter,
        page_rect_screen: Rect,
        paper_type: PaperType,
        page_width: f32,
        page_height: f32,
        scale: f32,
        to_screen: &F,
        is_dark: bool,
    ) where
        F: Fn(Pos2) -> Pos2,
    {
        // 1. Page shadow
        let shadow_rect = page_rect_screen.translate(Vec2::new(0.0, 6.0_f32 * scale.min(1.2_f32)));
        let shadow_color = if is_dark {
            Color32::from_black_alpha(100)
        } else {
            Color32::from_black_alpha(35)
        };
        painter.rect_filled(shadow_rect, 4.0_f32, shadow_color);

        // 2. Page sheet base
        let page_bg = if is_dark {
            Color32::from_rgb(34, 38, 44)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        painter.rect_filled(page_rect_screen, 2.0_f32, page_bg);

        // 3. Page border
        let border_color = if is_dark {
            Color32::from_rgb(55, 62, 72)
        } else {
            Color32::from_rgb(215, 225, 235)
        };
        painter.rect_stroke(page_rect_screen, 2.0_f32, Stroke::new(1.0_f32, border_color));

        // 4. Clip lines inside page
        let page_painter = painter.with_clip_rect(page_rect_screen);

        let line_color = if is_dark {
            Color32::from_rgb(52, 58, 68)
        } else {
            Color32::from_rgb(215, 230, 245)
        };

        match paper_type {
            PaperType::Blank => {}
            PaperType::Ruled => {
                let spacing = 34.0_f32;
                let margin_x = 85.0_f32;

                // Horizontal ruled lines
                let mut y = spacing * 2.0_f32;
                while y < page_height - 30.0_f32 {
                    let p1 = to_screen(Pos2::new(0.0_f32, y));
                    let p2 = to_screen(Pos2::new(page_width, y));
                    page_painter.line_segment([p1, p2], Stroke::new(1.0_f32, line_color));
                    y += spacing;
                }

                // Vertical red margin line
                let margin_color = if is_dark {
                    Color32::from_rgb(110, 50, 50)
                } else {
                    Color32::from_rgb(250, 160, 160)
                };
                let m1 = to_screen(Pos2::new(margin_x, 0.0_f32));
                let m2 = to_screen(Pos2::new(margin_x, page_height));
                page_painter.line_segment([m1, m2], Stroke::new(1.5_f32, margin_color));
            }
            PaperType::Grid => {
                let size = 26.0_f32;
                let mut x = size;
                while x < page_width {
                    let p1 = to_screen(Pos2::new(x, 0.0_f32));
                    let p2 = to_screen(Pos2::new(x, page_height));
                    page_painter.line_segment([p1, p2], Stroke::new(0.8_f32, line_color));
                    x += size;
                }
                let mut y = size;
                while y < page_height - 30.0_f32 {
                    let p1 = to_screen(Pos2::new(0.0_f32, y));
                    let p2 = to_screen(Pos2::new(page_width, y));
                    page_painter.line_segment([p1, p2], Stroke::new(0.8_f32, line_color));
                    y += size;
                }
            }
            PaperType::DotGrid => {
                let spacing = 26.0_f32;
                let dot_color = if is_dark {
                    Color32::from_rgb(68, 76, 88)
                } else {
                    Color32::from_rgb(200, 210, 222)
                };
                let mut x = spacing;
                while x < page_width {
                    let mut y = spacing;
                    while y < page_height - 30.0_f32 {
                        let pt = to_screen(Pos2::new(x, y));
                        page_painter.circle_filled(pt, 1.3_f32 * scale.min(1.2_f32), dot_color);
                        y += spacing;
                    }
                    x += spacing;
                }
            }
            PaperType::Cornell => {
                let cue_w = (page_width * 0.28_f32).max(180.0_f32);
                let header_h = 80.0_f32;
                let summary_h = (page_height * 0.18_f32).max(140.0_f32);
                let notes_bottom = page_height - summary_h;
                let divider_color = if is_dark {
                    Color32::from_rgb(85, 95, 108)
                } else {
                    Color32::from_rgb(148, 163, 184)
                };

                // Ruled lines inside notes area
                let spacing = 34.0_f32;
                let mut y = header_h + spacing;
                while y < notes_bottom {
                    let p1 = to_screen(Pos2::new(cue_w, y));
                    let p2 = to_screen(Pos2::new(page_width, y));
                    page_painter.line_segment([p1, p2], Stroke::new(1.0_f32, line_color));
                    y += spacing;
                }

                // Header line
                page_painter.line_segment(
                    [
                        to_screen(Pos2::new(0.0_f32, header_h)),
                        to_screen(Pos2::new(page_width, header_h)),
                    ],
                    Stroke::new(1.8_f32, divider_color),
                );
                // Cue line
                page_painter.line_segment(
                    [
                        to_screen(Pos2::new(cue_w, header_h)),
                        to_screen(Pos2::new(cue_w, notes_bottom)),
                    ],
                    Stroke::new(1.8_f32, divider_color),
                );
                // Summary line
                page_painter.line_segment(
                    [
                        to_screen(Pos2::new(0.0_f32, notes_bottom)),
                        to_screen(Pos2::new(page_width, notes_bottom)),
                    ],
                    Stroke::new(1.8_f32, divider_color),
                );
            }
        }

        // Bottom infinite indicator
        let bottom_p1 = to_screen(Pos2::new(30.0_f32, page_height - 20.0_f32));
        let bottom_p2 = to_screen(Pos2::new(page_width - 30.0_f32, page_height - 20.0_f32));
        let ind_color = if is_dark {
            Color32::from_rgb(70, 80, 95)
        } else {
            Color32::from_rgb(180, 195, 210)
        };
        page_painter.line_segment([bottom_p1, bottom_p2], Stroke::new(1.0_f32, ind_color));
    }

    /// Mathematical Centripetal Catmull-Rom to Cubic Bezier curve interpolator
    /// (Faithful translation of C++ StrokeAlgorithm::Geometry::BuildCatmullRomSpline)
    fn generate_smooth_polyline<F>(pts: &[Point], to_screen: &F) -> Vec<(Pos2, f32, f32)>
    where
        F: Fn(Pos2) -> Pos2,
    {
        if pts.is_empty() {
            return Vec::new();
        }

        let mut screen_pts: Vec<(Pos2, f32, f32)> = Vec::with_capacity(pts.len());
        for p in pts {
            let sp = to_screen(Pos2::new(p.x, p.y));
            let pres = p.pressure.clamp(0.05, 1.0);
            let th = p.thickness;
            if let Some(&(last_pos, _, _)) = screen_pts.last() {
                if (sp.x - last_pos.x).powi(2) + (sp.y - last_pos.y).powi(2) > 0.36 {
                    screen_pts.push((sp, pres, th));
                }
            } else {
                screen_pts.push((sp, pres, th));
            }
        }

        if screen_pts.len() < 2 {
            return screen_pts;
        }

        let n = screen_pts.len();
        if n == 2 {
            let (p1, pres1, th1) = screen_pts[0];
            let (p2, pres2, th2) = screen_pts[1];
            let dist = p1.distance(p2);
            let steps = ((dist / 1.5).ceil() as usize).clamp(2, 16);
            let mut smooth = Vec::with_capacity(steps + 1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let pt = Pos2::new(p1.x + (p2.x - p1.x) * t, p1.y + (p2.y - p1.y) * t);
                let pres = pres1 + (pres2 - pres1) * t;
                let th = th1 + (th2 - th1) * t;
                smooth.push((pt, pres, th));
            }
            return smooth;
        }

        let mut smooth = Vec::with_capacity(n * 6);
        smooth.push(screen_pts[0]);

        // Tension divisor = 6.0 (Centripetal Catmull-Rom C1 continuity from C++ engine)
        const TENSION: f32 = 6.0;

        for i in 0..n - 1 {
            let (p1, pres1, th1) = screen_pts[i];
            let (p2, pres2, th2) = screen_pts[i + 1];
            let p0 = if i == 0 {
                Pos2::new(2.0 * p1.x - p2.x, 2.0 * p1.y - p2.y)
            } else {
                screen_pts[i - 1].0
            };
            let p3 = if i + 2 < n {
                screen_pts[i + 2].0
            } else {
                Pos2::new(2.0 * p2.x - p1.x, 2.0 * p2.y - p1.y)
            };

            // Cubic Bezier control points from Catmull-Rom tangents
            let c1 = Pos2::new(p1.x + (p2.x - p0.x) / TENSION, p1.y + (p2.y - p0.y) / TENSION);
            let c2 = Pos2::new(p2.x - (p3.x - p1.x) / TENSION, p2.y - (p3.y - p1.y) / TENSION);

            let dist = p1.distance(p2);
            let steps = ((dist / 1.5).ceil() as usize).clamp(2, 32);

            for s in 1..=steps {
                let t = s as f32 / steps as f32;
                let omt = 1.0 - t;
                let omt2 = omt * omt;
                let omt3 = omt2 * omt;
                let t2 = t * t;
                let t3 = t2 * t;

                // Standard Cubic Bezier formula: (1-t)^3*P1 + 3*(1-t)^2*t*C1 + 3*(1-t)*t^2*C2 + t^3*P2
                let x = omt3 * p1.x + 3.0 * omt2 * t * c1.x + 3.0 * omt * t2 * c2.x + t3 * p2.x;
                let y = omt3 * p1.y + 3.0 * omt2 * t * c1.y + 3.0 * omt * t2 * c2.y + t3 * p2.y;

                let interp_pres = pres1 * (1.0 - t) + pres2 * t;
                let interp_th = th1 * (1.0 - t) + th2 * t;

                let pt = Pos2::new(x, y);
                if let Some(&(last_pos, _, _)) = smooth.last() {
                    if (pt.x - last_pos.x).powi(2) + (pt.y - last_pos.y).powi(2) > 0.25 {
                        smooth.push((pt, interp_pres, interp_th));
                    }
                } else {
                    smooth.push((pt, interp_pres, interp_th));
                }
            }
        }

        smooth
    }

    pub fn render_strokes<F>(
        painter: &Painter,
        strokes: &[NoteStroke],
        scale: f32,
        to_screen: &F,
    ) where
        F: Fn(Pos2) -> Pos2,
    {
        // Pass 1: Highlighters always rendered underneath regular ink
        for stroke in strokes {
            if stroke.pen_type == PenType::Highlighter {
                Self::render_single_stroke(painter, stroke, scale, to_screen);
            }
        }
        // Pass 2: Regular ink strokes on top
        for stroke in strokes {
            if stroke.pen_type != PenType::Highlighter {
                Self::render_single_stroke(painter, stroke, scale, to_screen);
            }
        }
    }

    pub fn render_single_stroke<F>(
        painter: &Painter,
        stroke: &NoteStroke,
        scale: f32,
        to_screen: &F,
    ) where
        F: Fn(Pos2) -> Pos2,
    {
        let pts = &stroke.points;
        if pts.is_empty() {
            return;
        }

        let base_alpha = (stroke.opacity * 255.0) as u8;
        let color = Color32::from_rgba_unmultiplied(
            stroke.color[0],
            stroke.color[1],
            stroke.color[2],
            base_alpha,
        );

        // === SHAPE RENDERING (from C++ StrokeAlgorithm::Geometry::BuildShape) ===
        if stroke.shape_type != ShapeType::None {
            Self::render_shape(painter, stroke, scale, to_screen, color);
            return;
        }

        // Single isolated point (dot)
        if pts.len() == 1 {
            let p0 = to_screen(Pos2::new(pts[0].x, pts[0].y));
            let pres = pts[0].pressure.clamp(0.05, 1.0);
            if stroke.pen_type == PenType::Highlighter {
                let hl_color = Color32::from_rgba_unmultiplied(
                    stroke.color[0],
                    stroke.color[1],
                    stroke.color[2],
                    (base_alpha as f32 * 0.40) as u8,
                );
                let r = (stroke.size * 3.6 * scale * 0.5).max(1.5);
                painter.circle_filled(p0, r, hl_color);
            } else {
                let r = (stroke.size * (0.35 + 0.85 * pres) * scale * 0.5).max(0.8);
                painter.circle_filled(p0, r, color);
            }
            return;
        }

        let smooth_pts = Self::generate_smooth_polyline(pts, to_screen);
        if smooth_pts.len() < 2 {
            return;
        }

        match stroke.pen_type {
            PenType::Highlighter => {
                let hl_color = Color32::from_rgba_unmultiplied(
                    stroke.color[0],
                    stroke.color[1],
                    stroke.color[2],
                    (base_alpha as f32 * 0.40) as u8,
                );
                let w = (stroke.size * 3.6 * scale).max(2.0);
                let poly_pts: Vec<Pos2> = smooth_pts.iter().map(|(p, _, _)| *p).collect();

                painter.add(egui::Shape::line(poly_pts.clone(), Stroke::new(w, hl_color)));
                if let Some(&first) = poly_pts.first() {
                    painter.circle_filled(first, w * 0.5, hl_color);
                }
                if let Some(&last) = poly_pts.last() {
                    painter.circle_filled(last, w * 0.5, hl_color);
                }
            }
            PenType::Ballpoint | PenType::Pencil | PenType::Fountain => {
                // Toàn bộ bút mực/bút bi/bút chì đều hiển thị dải ribbon có bề dày biến thiên theo từng vi điểm
                // (Variable-width Ribbon Mesh), loại bỏ hoàn toàn tình trạng nét vẽ bị cứng đơ như ống sắt!
                let draw_color = if stroke.pen_type == PenType::Pencil {
                    Color32::from_rgba_unmultiplied(
                        stroke.color[0],
                        stroke.color[1],
                        stroke.color[2],
                        (base_alpha as f32 * 0.88) as u8,
                    )
                } else {
                    color
                };
                Self::render_variable_width_ribbon(painter, &smooth_pts, stroke.size, scale, draw_color);
            }
            PenType::Calligraphy => {
                let angle = std::f32::consts::FRAC_PI_4; // 45 degree chisel
                let n = smooth_pts.len();

                let mut left_pts = Vec::with_capacity(n);
                let mut right_pts = Vec::with_capacity(n);

                for i in 0..n {
                    let p = smooth_pts[i].0;
                    let pres = smooth_pts[i].1.clamp(0.05, 1.0);
                    let base_w = stroke.size * (1.1 + 1.4 * pres) * scale;
                    let dx = angle.cos() * (base_w * 0.5);
                    let dy = angle.sin() * (base_w * 0.5);

                    left_pts.push(Pos2::new(p.x - dx, p.y - dy));
                    right_pts.push(Pos2::new(p.x + dx, p.y + dy));
                }

                let mut mesh = Mesh::default();
                for i in 0..n {
                    mesh.colored_vertex(left_pts[i], color);
                    mesh.colored_vertex(right_pts[i], color);
                }

                for i in 0..n - 1 {
                    let l0 = (2 * i) as u32;
                    let r0 = (2 * i + 1) as u32;
                    let l1 = (2 * (i + 1)) as u32;
                    let r1 = (2 * (i + 1) + 1) as u32;

                    mesh.add_triangle(l0, r0, l1);
                    mesh.add_triangle(r0, r1, l1);
                }

                if !mesh.is_empty() {
                    painter.add(egui::Shape::Mesh(mesh));
                }

                let edge_stroke = Stroke::new(1.0_f32, color);
                painter.add(egui::Shape::line(left_pts, edge_stroke));
                painter.add(egui::Shape::line(right_pts, edge_stroke));
            }
        }
    }

    /// Renders variable width strokes (Fountain pen) with normal expansion
    fn render_variable_width_ribbon(
        painter: &Painter,
        smooth_pts: &[(Pos2, f32, f32)],
        base_size: f32,
        scale: f32,
        color: Color32,
    ) {
        let n = smooth_pts.len();
        if n < 2 {
            return;
        }

        let mut left_pts = Vec::with_capacity(n);
        let mut right_pts = Vec::with_capacity(n);

        for i in 0..n {
            let p = smooth_pts[i].0;
            let pres = smooth_pts[i].1.clamp(0.05, 1.0);
            let dyn_th = smooth_pts[i].2;
            let width = if dyn_th > 0.01 {
                (dyn_th * scale).max(1.0)
            } else {
                (base_size * (0.6 + 0.8 * pres) * scale).max(1.0)
            };
            let half_w = width * 0.5;

            // Compute normal direction perpendicular to curve tangent
            let dir = if i == 0 {
                let next = smooth_pts[1].0;
                (next - p).normalized()
            } else if i == n - 1 {
                let prev = smooth_pts[n - 2].0;
                (p - prev).normalized()
            } else {
                let prev = smooth_pts[i - 1].0;
                let next = smooth_pts[i + 1].0;
                (next - prev).normalized()
            };

            let normal = Vec2::new(-dir.y, dir.x);
            left_pts.push(p + normal * half_w);
            right_pts.push(p - normal * half_w);
        }

        let mut mesh = Mesh::default();
        for i in 0..n {
            mesh.colored_vertex(left_pts[i], color);
            mesh.colored_vertex(right_pts[i], color);
        }

        for i in 0..n - 1 {
            let l0 = (2 * i) as u32;
            let r0 = (2 * i + 1) as u32;
            let l1 = (2 * (i + 1)) as u32;
            let r1 = (2 * (i + 1) + 1) as u32;

            mesh.add_triangle(l0, r0, l1);
            mesh.add_triangle(r0, r1, l1);
        }

        if !mesh.is_empty() {
            painter.add(egui::Shape::Mesh(mesh));
        }

        // Antialiased boundary line for crisp edges
        let edge_stroke = Stroke::new(0.75_f32, color);
        painter.add(egui::Shape::line(left_pts, edge_stroke));
        painter.add(egui::Shape::line(right_pts, edge_stroke));

        // Smooth round end caps
        if let Some(&(p_start, _, th)) = smooth_pts.first() {
            let r = if th > 0.01 { (th * scale * 0.5).max(0.6) } else { (base_size * scale * 0.4).max(0.6) };
            painter.circle_filled(p_start, r, color);
        }
        if let Some(&(p_end, _, th)) = smooth_pts.last() {
            let r = if th > 0.01 { (th * scale * 0.5).max(0.6) } else { (base_size * scale * 0.4).max(0.6) };
            painter.circle_filled(p_end, r, color);
        }
    }

    /// Renders shapes matching C++ StrokeAlgorithm::Geometry::BuildShape
    fn render_shape<F>(
        painter: &Painter,
        stroke: &NoteStroke,
        scale: f32,
        to_screen: &F,
        color: Color32,
    ) where
        F: Fn(Pos2) -> Pos2,
    {
        let pts = &stroke.points;
        if pts.len() < 2 {
            return;
        }

        let p1 = to_screen(Pos2::new(pts[0].x, pts[0].y));
        let p2 = to_screen(Pos2::new(pts.last().unwrap().x, pts.last().unwrap().y));
        let thickness = (stroke.size * scale).max(1.0);
        let stroke_spec = Stroke::new(thickness, color);
        let is_filled = stroke.is_filled;

        let rect = Rect::from_min_max(
            Pos2::new(p1.x.min(p2.x), p1.y.min(p2.y)),
            Pos2::new(p1.x.max(p2.x), p1.y.max(p2.y)),
        );

        match stroke.shape_type {
            ShapeType::None => {}
            ShapeType::Line => {
                painter.line_segment([p1, p2], stroke_spec);
            }
            ShapeType::Rectangle => {
                if is_filled {
                    painter.rect_filled(rect, 0.0, color);
                } else {
                    painter.rect_stroke(rect, 0.0, stroke_spec);
                }
            }
            ShapeType::Circle => {
                let rx = rect.width() / 2.0;
                let ry = rect.height() / 2.0;
                let center = rect.center();

                // Generate smooth ellipse vertices
                let segments = 64;
                let mut poly = Vec::with_capacity(segments);
                for k in 0..segments {
                    let theta = k as f32 * std::f32::consts::TAU / segments as f32;
                    poly.push(Pos2::new(center.x + theta.cos() * rx, center.y + theta.sin() * ry));
                }

                if is_filled {
                    let mut mesh = Mesh::default();
                    let center_idx = mesh.vertices.len() as u32;
                    mesh.colored_vertex(center, color);
                    let first_ring_idx = mesh.vertices.len() as u32;
                    for &pt in &poly {
                        mesh.colored_vertex(pt, color);
                    }
                    for k in 0..segments {
                        let i1 = first_ring_idx + k as u32;
                        let i2 = if k + 1 == segments {
                            first_ring_idx
                        } else {
                            first_ring_idx + (k + 1) as u32
                        };
                        mesh.add_triangle(center_idx, i1, i2);
                    }
                    painter.add(egui::Shape::Mesh(mesh));
                } else {
                    poly.push(poly[0]);
                    painter.add(egui::Shape::line(poly, stroke_spec));
                }
            }
            ShapeType::Triangle => {
                let top = Pos2::new(rect.center().x, rect.top());
                let right = Pos2::new(rect.right(), rect.bottom());
                let left = Pos2::new(rect.left(), rect.bottom());
                let poly = vec![top, right, left, top];

                if is_filled {
                    let mut mesh = Mesh::default();
                    mesh.colored_vertex(top, color);
                    mesh.colored_vertex(right, color);
                    mesh.colored_vertex(left, color);
                    mesh.add_triangle(0, 1, 2);
                    painter.add(egui::Shape::Mesh(mesh));
                } else {
                    painter.add(egui::Shape::line(poly, stroke_spec));
                }
            }
            ShapeType::Arrow | ShapeType::DoubleArrow => {
                painter.line_segment([p1, p2], stroke_spec);
                let angle = (p2.y - p1.y).atan2(p2.x - p1.x);
                let head_len = 15.0 * scale + thickness;

                let p_end1 = Pos2::new(
                    p2.x - head_len * (angle - 0.5).cos(),
                    p2.y - head_len * (angle - 0.5).sin(),
                );
                let p_end2 = Pos2::new(
                    p2.x - head_len * (angle + 0.5).cos(),
                    p2.y - head_len * (angle + 0.5).sin(),
                );
                painter.line_segment([p2, p_end1], stroke_spec);
                painter.line_segment([p2, p_end2], stroke_spec);

                if stroke.shape_type == ShapeType::DoubleArrow {
                    let p_start1 = Pos2::new(
                        p1.x + head_len * (angle - 0.5).cos(),
                        p1.y + head_len * (angle - 0.5).sin(),
                    );
                    let p_start2 = Pos2::new(
                        p1.x + head_len * (angle + 0.5).cos(),
                        p1.y + head_len * (angle + 0.5).sin(),
                    );
                    painter.line_segment([p1, p_start1], stroke_spec);
                    painter.line_segment([p1, p_start2], stroke_spec);
                }
            }
            ShapeType::Pentagon => {
                Self::render_regular_polygon(painter, rect, 5, is_filled, color, stroke_spec);
            }
            ShapeType::Hexagon => {
                Self::render_regular_polygon(painter, rect, 6, is_filled, color, stroke_spec);
            }
            ShapeType::Star => {
                let cx = rect.center().x;
                let cy = rect.center().y;
                let rx = rect.width() / 2.0;
                let ry = rect.height() / 2.0;

                let mut poly = Vec::with_capacity(11);
                for i in 0..10 {
                    let angle = -std::f32::consts::FRAC_PI_2 + (i as f32) * (std::f32::consts::TAU / 10.0);
                    let r = if i % 2 == 0 { 1.0 } else { 0.4 };
                    poly.push(Pos2::new(cx + angle.cos() * rx * r, cy + angle.sin() * ry * r));
                }
                poly.push(poly[0]);

                if is_filled {
                    let mut mesh = Mesh::default();
                    let center_idx = mesh.vertices.len() as u32;
                    mesh.colored_vertex(rect.center(), color);
                    let first_ring_idx = mesh.vertices.len() as u32;
                    for i in 0..10 {
                        mesh.colored_vertex(poly[i], color);
                    }
                    for i in 0..10 {
                        let i1 = first_ring_idx + i as u32;
                        let i2 = if i + 1 == 10 {
                            first_ring_idx
                        } else {
                            first_ring_idx + (i + 1) as u32
                        };
                        mesh.add_triangle(center_idx, i1, i2);
                    }
                    painter.add(egui::Shape::Mesh(mesh));
                } else {
                    painter.add(egui::Shape::line(poly, stroke_spec));
                }
            }
        }
    }

    fn render_regular_polygon(
        painter: &Painter,
        rect: Rect,
        sides: usize,
        is_filled: bool,
        color: Color32,
        stroke_spec: Stroke,
    ) {
        let cx = rect.center().x;
        let cy = rect.center().y;
        let rx = rect.width() / 2.0;
        let ry = rect.height() / 2.0;

        let mut poly = Vec::with_capacity(sides + 1);
        for k in 0..sides {
            let angle = -std::f32::consts::FRAC_PI_2 + (k as f32) * (std::f32::consts::TAU / sides as f32);
            poly.push(Pos2::new(cx + angle.cos() * rx, cy + angle.sin() * ry));
        }
        poly.push(poly[0]);

        if is_filled {
            let mut mesh = Mesh::default();
            let center_idx = mesh.vertices.len() as u32;
            mesh.colored_vertex(rect.center(), color);
            let first_ring_idx = mesh.vertices.len() as u32;
            for k in 0..sides {
                mesh.colored_vertex(poly[k], color);
            }
            for k in 0..sides {
                let i1 = first_ring_idx + k as u32;
                let i2 = if k + 1 == sides {
                    first_ring_idx
                } else {
                    first_ring_idx + (k + 1) as u32
                };
                mesh.add_triangle(center_idx, i1, i2);
            }
            painter.add(egui::Shape::Mesh(mesh));
        } else {
            painter.add(egui::Shape::line(poly, stroke_spec));
        }
    }
}
