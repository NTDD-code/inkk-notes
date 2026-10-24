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
        // 1. Dual-elevation soft floating shadow (Elevation 2)
        let s1_offset = Vec2::new(0.0, 3.0_f32 * scale.min(1.2_f32));
        let s2_offset = Vec2::new(0.0, 8.0_f32 * scale.min(1.2_f32));
        let (s1_color, s2_color) = if is_dark {
            (
                Color32::from_black_alpha(80),
                Color32::from_black_alpha(120),
            )
        } else {
            (Color32::from_black_alpha(15), Color32::from_black_alpha(25))
        };
        painter.rect_filled(page_rect_screen.translate(s1_offset), 4.0_f32, s1_color);
        painter.rect_filled(page_rect_screen.translate(s2_offset), 8.0_f32, s2_color);

        // 2. Warm paper sheet base
        let page_bg = if is_dark {
            Color32::from_rgb(36, 33, 29) // Warm brownish-black #24211D
        } else {
            Color32::from_rgb(255, 253, 249) // Warm ivory paper #FFFDF9
        };
        painter.rect_filled(page_rect_screen, 3.0_f32, page_bg);

        // 3. Micro warm paper border
        let border_color = if is_dark {
            Color32::from_rgb(56, 51, 45) // Warm dark border #38332D
        } else {
            Color32::from_rgb(226, 218, 207) // Warm subtle border #E2DACF
        };
        painter.rect_stroke(
            page_rect_screen,
            3.0_f32,
            Stroke::new(1.0_f32, border_color),
        );

        // 4. Clip lines inside page
        let page_painter = painter.with_clip_rect(page_rect_screen);

        let line_color = if is_dark {
            Color32::from_rgb(50, 46, 40) // Warm dark line #322E28
        } else {
            Color32::from_rgb(230, 222, 212) // Warm soft line #E6DFD4
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

                // Vertical red margin line (Inkk signature Brick Red)
                let margin_color = if is_dark {
                    Color32::from_rgb(140, 59, 48) // Warm brick red #8C3B30
                } else {
                    Color32::from_rgb(200, 90, 75) // Signature brick red #C85A4B
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
                    Color32::from_rgb(70, 64, 56)
                } else {
                    Color32::from_rgb(215, 206, 194)
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

    /// Mathematical curve smoothing - BALANCED for Native + Smooth
    /// Äá»§ mÆ°á»£t Ä‘á»ƒ loáº¡i jitter nhÆ°ng GIá»® Ä‘á»™ CRISP, SNAPPY nhÆ° Scribble Ink
    fn generate_smooth_polyline<F>(pts: &[Point], to_screen: &F) -> Vec<(Pos2, f32, f32)>
    where
        F: Fn(Pos2) -> Pos2,
    {
        if pts.is_empty() {
            return Vec::new();
        }

        // BÆ¯á»šC 1: Lá»c Ä‘iá»ƒm trÃ¹ng (< 0.9 screen pixel - khÃ´ng quÃ¡ sÃ¡t)
        let mut screen_pts: Vec<(Pos2, f32, f32)> = Vec::with_capacity(pts.len());
        for p in pts {
            let sp = to_screen(Pos2::new(p.x, p.y));
            let pres = p.pressure.clamp(0.05, 1.0);
            let th = p.thickness;
            if let Some(&(last_pos, _, _)) = screen_pts.last() {
                if (sp.x - last_pos.x).powi(2) + (sp.y - last_pos.y).powi(2) >= 0.81 {
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
            let steps = ((dist / 1.5).ceil() as usize).clamp(2, 12);
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

        // BÆ¯á»šC 2: Midpoint Quadratic Bezier - ÃT STEPS hÆ¡n Ä‘á»ƒ CRISP
        let mut smooth = Vec::with_capacity(n * 6);
        smooth.push(screen_pts[0]);

        // Äoáº¡n Ä‘áº§u
        let mid0 = Pos2::new(
            (screen_pts[0].0.x + screen_pts[1].0.x) * 0.5,
            (screen_pts[0].0.y + screen_pts[1].0.y) * 0.5,
        );
        let mid_pres0 = (screen_pts[0].1 + screen_pts[1].1) * 0.5;
        let mid_th0 = (screen_pts[0].2 + screen_pts[1].2) * 0.5;

        let dist0 = screen_pts[0].0.distance(mid0);
        let steps0 = ((dist0 / 1.8).ceil() as usize).clamp(2, 6);
        for s in 1..=steps0 {
            let t = s as f32 / steps0 as f32;
            let pt = Pos2::new(
                screen_pts[0].0.x + (mid0.x - screen_pts[0].0.x) * t,
                screen_pts[0].0.y + (mid0.y - screen_pts[0].0.y) * t,
            );
            let pres = screen_pts[0].1 + (mid_pres0 - screen_pts[0].1) * t;
            let th = screen_pts[0].2 + (mid_th0 - screen_pts[0].2) * t;
            smooth.push((pt, pres, th));
        }

        // CÃ¡c Ä‘oáº¡n thÃ¢n - ÃT STEPS (1.2px spacing) Ä‘á»ƒ nÃ©t CRISP
        for i in 1..n - 1 {
            let p_prev = screen_pts[i - 1];
            let p_curr = screen_pts[i];
            let p_next = screen_pts[i + 1];

            let start_mid = Pos2::new(
                (p_prev.0.x + p_curr.0.x) * 0.5,
                (p_prev.0.y + p_curr.0.y) * 0.5,
            );
            let end_mid = Pos2::new(
                (p_curr.0.x + p_next.0.x) * 0.5,
                (p_curr.0.y + p_next.0.y) * 0.5,
            );

            let start_pres = (p_prev.1 + p_curr.1) * 0.5;
            let end_pres = (p_curr.1 + p_next.1) * 0.5;

            let start_th = (p_prev.2 + p_curr.2) * 0.5;
            let end_th = (p_curr.2 + p_next.2) * 0.5;

            let chord = start_mid.distance(end_mid);
            // ÃT STEPS hÆ¡n - spacing 1.2px thay vÃ¬ 0.8px
            let steps = ((chord / 1.2).ceil() as usize).clamp(3, 20);

            for s in 1..=steps {
                let t = s as f32 / steps as f32;
                let omt = 1.0 - t;

                let x = omt * omt * start_mid.x + 2.0 * omt * t * p_curr.0.x + t * t * end_mid.x;
                let y = omt * omt * start_mid.y + 2.0 * omt * t * p_curr.0.y + t * t * end_mid.y;

                let pres = omt * start_pres + t * end_pres;
                let th = omt * start_th + t * end_th;

                let pt = Pos2::new(x, y);
                if let Some(&(last_pos, _, _)) = smooth.last() {
                    if (pt.x - last_pos.x).powi(2) + (pt.y - last_pos.y).powi(2) > 0.16 {
                        smooth.push((pt, pres, th));
                    }
                } else {
                    smooth.push((pt, pres, th));
                }
            }
        }

        // Äoáº¡n cuá»‘i
        let p_last = screen_pts[n - 1];
        let p_penult = screen_pts[n - 2];
        let last_mid = Pos2::new(
            (p_penult.0.x + p_last.0.x) * 0.5,
            (p_penult.0.y + p_last.0.y) * 0.5,
        );
        let dist_end = last_mid.distance(p_last.0);
        let steps_end = ((dist_end / 1.8).ceil() as usize).clamp(2, 6);
        for s in 1..=steps_end {
            let t = s as f32 / steps_end as f32;
            let pt = Pos2::new(
                last_mid.x + (p_last.0.x - last_mid.x) * t,
                last_mid.y + (p_last.0.y - last_mid.y) * t,
            );
            let pres = p_penult.1 + (p_last.1 - p_penult.1) * t;
            let th = p_penult.2 + (p_last.2 - p_penult.2) * t;
            smooth.push((pt, pres, th));
        }

        smooth
    }

    pub fn render_strokes<F>(painter: &Painter, strokes: &[NoteStroke], scale: f32, to_screen: &F)
    where
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

                painter.add(egui::Shape::line(
                    poly_pts.clone(),
                    Stroke::new(w, hl_color),
                ));
                if let Some(&first) = poly_pts.first() {
                    painter.circle_filled(first, w * 0.5, hl_color);
                }
                if let Some(&last) = poly_pts.last() {
                    painter.circle_filled(last, w * 0.5, hl_color);
                }
            }
            PenType::Ballpoint | PenType::Pencil | PenType::Fountain => {
                // ToÃ n bá»™ bÃºt má»±c/bÃºt bi/bÃºt chÃ¬ Ä‘á»u hiá»ƒn thá»‹ dáº£i ribbon cÃ³ bá» dÃ y biáº¿n thiÃªn theo tá»«ng vi Ä‘iá»ƒm
                // (Variable-width Ribbon Mesh), loáº¡i bá» hoÃ n toÃ n tÃ¬nh tráº¡ng nÃ©t váº½ bá»‹ cá»©ng Ä‘Æ¡ nhÆ° á»‘ng sáº¯t!
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
                Self::render_variable_width_ribbon(
                    painter,
                    &smooth_pts,
                    stroke.size,
                    scale,
                    draw_color,
                );
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

    /// === ELASTIC TAIL - NATIVE CRISP FEEL ===
    /// ÄuÃ´i Cá»°C NGáº®N, THáº²NG, BÃM SÃT nhÆ° DirectInk - ZERO LAG!
    pub fn render_elastic_tail<F>(
        painter: &Painter,
        stroke: &NoteStroke,
        raw_cursor_screen: Pos2,
        scale: f32,
        to_screen: &F,
    ) where
        F: Fn(Pos2) -> Pos2,
    {
        if stroke.shape_type != ShapeType::None || stroke.points.is_empty() {
            return;
        }

        let pts = &stroke.points;
        let n = pts.len();
        let last_pt = pts[n - 1];
        let end = to_screen(Pos2::new(last_pt.x, last_pt.y));
        let raw = raw_cursor_screen;

        let dx = raw.x - end.x;
        let dy = raw.y - end.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // NgÆ°á»¡ng Cá»°C THáº¤P (0.3px) - váº½ ngay láº­p tá»©c
        if dist > 0.3 {
            // ÄuÃ´i Cá»°C NGáº®N (max 5px) - gáº§n nhÆ° tháº³ng tá»›i con trá»
            let max_tail = 5.0 * scale.clamp(0.8, 1.1);
            let target_raw = if dist > max_tail {
                Pos2::new(
                    end.x + (dx / dist) * max_tail,
                    end.y + (dy / dist) * max_tail,
                )
            } else {
                raw
            };

            // Control point: Cá»°C NHáº¸ (5% strength) - gáº§n nhÆ° Ä‘Æ°á»ng tháº³ng
            let ctrl = if n >= 2 {
                let p_penult = to_screen(Pos2::new(pts[n - 2].x, pts[n - 2].y));
                let vx = end.x - p_penult.x;
                let vy = end.y - p_penult.y;
                Pos2::new(end.x + vx * 0.05, end.y + vy * 0.05)
            } else {
                end
            };

            // ÄÆ°á»ng ná»‘i THáº²NG, Ã­t steps (4) Ä‘á»ƒ crisp
            let steps = 4;
            let mut tail_pts = Vec::with_capacity(steps + 1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let omt = 1.0 - t;
                let x = omt * omt * end.x + 2.0 * omt * t * ctrl.x + t * t * target_raw.x;
                let y = omt * omt * end.y + 2.0 * omt * t * ctrl.y + t * t * target_raw.y;
                tail_pts.push(Pos2::new(x, y));
            }

            let base_alpha = (stroke.opacity * 255.0) as u8;
            let color = Color32::from_rgba_unmultiplied(
                stroke.color[0],
                stroke.color[1],
                stroke.color[2],
                base_alpha,
            );

            let th = if last_pt.thickness > 0.01 {
                (last_pt.thickness * scale * 0.88).max(0.8)
            } else {
                (stroke.size * scale * 0.85).max(0.8)
            };

            // Ná»‘i nÃ©t CRISP
            painter.add(egui::Shape::line(tail_pts, Stroke::new(th, color)));
        }
    }
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
            let r = if th > 0.01 {
                (th * scale * 0.5).max(0.6)
            } else {
                (base_size * scale * 0.4).max(0.6)
            };
            painter.circle_filled(p_start, r, color);
        }
        if let Some(&(p_end, _, th)) = smooth_pts.last() {
            let r = if th > 0.01 {
                (th * scale * 0.5).max(0.6)
            } else {
                (base_size * scale * 0.4).max(0.6)
            };
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
                    poly.push(Pos2::new(
                        center.x + theta.cos() * rx,
                        center.y + theta.sin() * ry,
                    ));
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
                    let angle =
                        -std::f32::consts::FRAC_PI_2 + (i as f32) * (std::f32::consts::TAU / 10.0);
                    let r = if i % 2 == 0 { 1.0 } else { 0.4 };
                    poly.push(Pos2::new(
                        cx + angle.cos() * rx * r,
                        cy + angle.sin() * ry * r,
                    ));
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
            ShapeType::GraphQuadrant1 => {
                let origin = Pos2::new(rect.left(), rect.bottom());
                let y_top = Pos2::new(rect.left(), rect.top());
                let x_right = Pos2::new(rect.right(), rect.bottom());
                painter.line_segment([origin, y_top], stroke_spec);
                painter.line_segment([origin, x_right], stroke_spec);
            }
            ShapeType::Graph2D => {
                let c = rect.center();
                let x_left = Pos2::new(rect.left(), c.y);
                let x_right = Pos2::new(rect.right(), c.y);
                let y_bottom = Pos2::new(c.x, rect.bottom());
                let y_top = Pos2::new(c.x, rect.top());
                painter.line_segment([x_left, x_right], stroke_spec);
                painter.line_segment([y_bottom, y_top], stroke_spec);
            }
            ShapeType::Graph3D => {
                let c = rect.center();
                let x_left = Pos2::new(rect.left(), c.y);
                let x_right = Pos2::new(rect.right(), c.y);
                let y_bottom = Pos2::new(c.x, rect.bottom());
                let y_top = Pos2::new(c.x, rect.top());
                painter.line_segment([x_left, x_right], stroke_spec);
                painter.line_segment([y_bottom, y_top], stroke_spec);

                // Diagonal Z axis: solid from bottom-left to center, dashed from center to top-right!
                let diag_start = Pos2::new(rect.left(), rect.bottom());
                let diag_end = Pos2::new(rect.right(), rect.top());
                painter.line_segment([diag_start, c], stroke_spec);

                let dir = diag_end - c;
                let total_len = dir.length();
                if total_len > 1.0 {
                    let unit = dir / total_len;
                    let dash_len = (7.0 * scale).max(3.0);
                    let gap_len = (5.0 * scale).max(2.0);
                    let mut dist = 0.0;
                    while dist < total_len {
                        let seg_start = c + unit * dist;
                        let seg_end = c + unit * (dist + dash_len).min(total_len);
                        painter.line_segment([seg_start, seg_end], stroke_spec);
                        dist += dash_len + gap_len;
                    }
                }
            }
            ShapeType::Cube3D => {
                let w = rect.width();
                let h = rect.height();
                let dx = w * 0.28;
                let dy = h * 0.28;
                let f_tl = Pos2::new(rect.left(), rect.top() + dy);
                let f_tr = Pos2::new(rect.right() - dx, rect.top() + dy);
                let f_br = Pos2::new(rect.right() - dx, rect.bottom());
                let f_bl = Pos2::new(rect.left(), rect.bottom());

                let b_tl = Pos2::new(rect.left() + dx, rect.top());
                let b_tr = Pos2::new(rect.right(), rect.top());
                let b_br = Pos2::new(rect.right(), rect.bottom() - dy);
                let b_bl = Pos2::new(rect.left() + dx, rect.bottom() - dy);

                if is_filled {
                    painter.rect_filled(Rect::from_min_max(f_tl, f_br), 0.0, color);
                }

                painter.line_segment([f_tl, f_tr], stroke_spec);
                painter.line_segment([f_tr, f_br], stroke_spec);
                painter.line_segment([f_br, f_bl], stroke_spec);
                painter.line_segment([f_bl, f_tl], stroke_spec);

                painter.line_segment([b_tl, b_tr], stroke_spec);
                painter.line_segment([b_tr, b_br], stroke_spec);
                painter.line_segment([f_tl, b_tl], stroke_spec);
                painter.line_segment([f_tr, b_tr], stroke_spec);
                painter.line_segment([f_br, b_br], stroke_spec);

                Self::draw_dashed_segment(painter, b_bl, b_tl, stroke_spec, scale);
                Self::draw_dashed_segment(painter, b_bl, b_br, stroke_spec, scale);
                Self::draw_dashed_segment(painter, b_bl, f_bl, stroke_spec, scale);
            }
            ShapeType::Pyramid3D => {
                let w = rect.width();
                let h = rect.height();
                let apex = Pos2::new(rect.left() + w * 0.5, rect.top());
                let a = Pos2::new(rect.left(), rect.bottom());
                let b = Pos2::new(rect.left() + w * 0.65, rect.bottom());
                let c = Pos2::new(rect.right(), rect.bottom() - h * 0.28);
                let d = Pos2::new(rect.left() + w * 0.35, rect.bottom() - h * 0.28);

                if is_filled {
                    let mut mesh = egui::Mesh::default();
                    mesh.colored_vertex(a, color);
                    mesh.colored_vertex(b, color);
                    mesh.colored_vertex(apex, color);
                    mesh.add_triangle(0, 1, 2);
                    painter.add(egui::Shape::Mesh(mesh));
                }

                painter.line_segment([apex, a], stroke_spec);
                painter.line_segment([apex, b], stroke_spec);
                painter.line_segment([apex, c], stroke_spec);
                painter.line_segment([a, b], stroke_spec);
                painter.line_segment([b, c], stroke_spec);

                Self::draw_dashed_segment(painter, a, d, stroke_spec, scale);
                Self::draw_dashed_segment(painter, d, c, stroke_spec, scale);
                Self::draw_dashed_segment(painter, apex, d, stroke_spec, scale);
            }
            ShapeType::Sphere3D => {
                let c = rect.center();
                let r = (rect.width().min(rect.height())) * 0.5;
                if r > 1.0 {
                    if is_filled {
                        painter.circle_filled(c, r, color);
                    }
                    painter.circle_stroke(c, r, stroke_spec);

                    let rx = r;
                    let ry = r * 0.32;
                    let steps = 36;
                    let mut front_pts = Vec::with_capacity(steps / 2 + 1);
                    for i in 0..=(steps / 2) {
                        let theta = std::f32::consts::PI * (i as f32) / (steps as f32 * 0.5);
                        front_pts.push(Pos2::new(c.x + rx * theta.cos(), c.y + ry * theta.sin()));
                    }
                    for w in front_pts.windows(2) {
                        painter.line_segment([w[0], w[1]], stroke_spec);
                    }

                    let mut back_pts = Vec::with_capacity(steps / 2 + 1);
                    for i in (steps / 2)..=steps {
                        let theta = std::f32::consts::PI * (i as f32) / (steps as f32 * 0.5);
                        back_pts.push(Pos2::new(c.x + rx * theta.cos(), c.y + ry * theta.sin()));
                    }
                    for (idx, w) in back_pts.windows(2).enumerate() {
                        if idx % 2 == 0 {
                            painter.line_segment([w[0], w[1]], stroke_spec);
                        }
                    }

                    let mrx = r * 0.28;
                    let mry = r;
                    let mut meridian_front = Vec::new();
                    for i in -9..=9 {
                        let theta = std::f32::consts::FRAC_PI_2 * (i as f32 / 9.0);
                        meridian_front.push(Pos2::new(c.x + mrx * theta.cos(), c.y + mry * theta.sin()));
                    }
                    for w in meridian_front.windows(2) {
                        painter.line_segment([w[0], w[1]], stroke_spec);
                    }
                }
            }
        }
    }

    fn draw_dashed_segment(painter: &Painter, p1: Pos2, p2: Pos2, stroke: Stroke, scale: f32) {
        let dir = p2 - p1;
        let total_len = dir.length();
        if total_len <= 1.0 {
            return;
        }
        let unit = dir / total_len;
        let dash_len = (6.0 * scale).max(3.0);
        let gap_len = (4.0 * scale).max(2.0);
        let mut dist = 0.0;
        while dist < total_len {
            let seg_start = p1 + unit * dist;
            let seg_end = p1 + unit * (dist + dash_len).min(total_len);
            painter.line_segment([seg_start, seg_end], stroke);
            dist += dash_len + gap_len;
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
            let angle =
                -std::f32::consts::FRAC_PI_2 + (k as f32) * (std::f32::consts::TAU / sides as f32);
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
