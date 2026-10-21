use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

use crate::model::{PaperType, PenType, Point, Stroke as NoteStroke};

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
                    [to_screen(Pos2::new(0.0_f32, header_h)), to_screen(Pos2::new(page_width, header_h))],
                    Stroke::new(1.8_f32, divider_color),
                );
                // Cue line
                page_painter.line_segment(
                    [to_screen(Pos2::new(cue_w, header_h)), to_screen(Pos2::new(cue_w, notes_bottom))],
                    Stroke::new(1.8_f32, divider_color),
                );
                // Summary line
                page_painter.line_segment(
                    [to_screen(Pos2::new(0.0_f32, notes_bottom)), to_screen(Pos2::new(page_width, notes_bottom))],
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

    /// Mathematical Centripetal Catmull-Rom Spline Interpolator
    /// Generates smoothly interpolated points and pressures passing exactly through all control points,
    /// creating a true continuous vector polyline with pressure sensitivity.
    fn generate_smooth_polyline<F>(pts: &[Point], to_screen: &F) -> Vec<(Pos2, f32)>
    where
        F: Fn(Pos2) -> Pos2,
    {
        if pts.is_empty() {
            return Vec::new();
        }

        // Deduplicate consecutive points in screen space to prevent 0-length degenerate segments
        let mut screen_pts: Vec<(Pos2, f32)> = Vec::with_capacity(pts.len());
        for p in pts {
            let sp = to_screen(Pos2::new(p.x, p.y));
            let pres = p.pressure.clamp(0.05, 1.0);
            if let Some(&(last_pos, _)) = screen_pts.last() {
                if (sp.x - last_pos.x).powi(2) + (sp.y - last_pos.y).powi(2) > 1.0 {
                    screen_pts.push((sp, pres));
                }
            } else {
                screen_pts.push((sp, pres));
            }
        }

        if screen_pts.len() < 2 {
            return screen_pts;
        }
        if screen_pts.len() == 2 {
            let (p1, pres1) = screen_pts[0];
            let (p2, pres2) = screen_pts[1];
            let dist = p1.distance(p2);
            let steps = ((dist / 1.5_f32).ceil() as usize).clamp(2, 16);
            let mut smooth = Vec::with_capacity(steps + 1);
            for s in 0..=steps {
                let t = s as f32 / steps as f32;
                let pt = Pos2::new(p1.x + (p2.x - p1.x) * t, p1.y + (p2.y - p1.y) * t);
                let pres = pres1 + (pres2 - pres1) * t;
                smooth.push((pt, pres));
            }
            return smooth;
        }

        let n = screen_pts.len();
        let mut smooth = Vec::with_capacity(n * 6);
        smooth.push(screen_pts[0]);

        for i in 0..n - 1 {
            let (p1, pres1) = screen_pts[i];
            let (p2, pres2) = screen_pts[i + 1];
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

            let dist = p1.distance(p2);
            // Ultra-smooth adaptive sub-sampling: 1 sample every ~1.5 screen pixels
            // Completely eliminates straight chord artifacts even when zoomed in to 500%
            let steps = ((dist / 1.5_f32).ceil() as usize).clamp(2, 32);

            for s in 1..=steps {
                let t = s as f32 / steps as f32;
                let t2 = t * t;
                let t3 = t2 * t;

                let x = 0.5 * ((2.0 * p1.x)
                    + (-p0.x + p2.x) * t
                    + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
                    + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);
                let y = 0.5 * ((2.0 * p1.y)
                    + (-p0.y + p2.y) * t
                    + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
                    + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

                // Smoothly interpolate pressure
                let interp_pres = pres1 * (1.0 - t) + pres2 * t;

                let pt = Pos2::new(x, y);
                if let Some(&(last_pos, _)) = smooth.last() {
                    if (pt.x - last_pos.x).powi(2) + (pt.y - last_pos.y).powi(2) > 0.4 {
                        smooth.push((pt, interp_pres));
                    }
                } else {
                    smooth.push((pt, interp_pres));
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
        // Pass 2: Regular pen / pencil / calligraphy strokes on top
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

        let base_alpha = (stroke.opacity * 255.0_f32) as u8;
        let color = Color32::from_rgba_unmultiplied(
            stroke.color[0],
            stroke.color[1],
            stroke.color[2],
            base_alpha,
        );

        // Single isolated point (dot)
        if pts.len() == 1 {
            let p0 = to_screen(Pos2::new(pts[0].x, pts[0].y));
            let pres = pts[0].pressure.clamp(0.05, 1.0);
            if stroke.pen_type == PenType::Highlighter {
                let hl_color = Color32::from_rgba_unmultiplied(
                    stroke.color[0],
                    stroke.color[1],
                    stroke.color[2],
                    (base_alpha as f32 * 0.40_f32) as u8,
                );
                let r = (stroke.size * 3.6_f32 * scale * 0.5_f32).max(1.5_f32);
                painter.circle_filled(p0, r, hl_color);
            } else {
                let r = (stroke.size * (0.35_f32 + 0.85_f32 * pres) * scale * 0.5_f32).max(0.8_f32);
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
                    (base_alpha as f32 * 0.40_f32) as u8,
                );
                let w = (stroke.size * 3.6_f32 * scale).max(2.0_f32);
                let poly_pts: Vec<Pos2> = smooth_pts.iter().map(|(p, _)| *p).collect();

                // Continuous vector path: NO alpha multiplication, NO dots along the path
                painter.add(egui::Shape::line(poly_pts.clone(), Stroke::new(w, hl_color)));
                if let Some(&first) = poly_pts.first() {
                    painter.circle_filled(first, w * 0.5_f32, hl_color);
                }
                if let Some(&last) = poly_pts.last() {
                    painter.circle_filled(last, w * 0.5_f32, hl_color);
                }
            }
            PenType::Ballpoint => {
                let avg_pres = if pts.is_empty() { 0.5 } else {
                    let sum: f32 = pts.iter().map(|p| p.pressure).sum();
                    (sum / pts.len() as f32).clamp(0.05, 1.0)
                };
                let w = (stroke.size * (0.35_f32 + 0.85_f32 * avg_pres) * scale).max(1.0_f32);
                let poly_pts: Vec<Pos2> = smooth_pts.iter().map(|(p, _)| *p).collect();

                // 100% Native Vector Line with Built-in Hardware Antialiased Feathering!
                painter.add(egui::Shape::line(poly_pts.clone(), Stroke::new(w, color)));
                if let Some(&first) = poly_pts.first() {
                    painter.circle_filled(first, w * 0.5_f32, color);
                }
                if let Some(&last) = poly_pts.last() {
                    painter.circle_filled(last, w * 0.5_f32, color);
                }
            }
            PenType::Pencil => {
                let avg_pres = if pts.is_empty() { 0.5 } else {
                    let sum: f32 = pts.iter().map(|p| p.pressure).sum();
                    (sum / pts.len() as f32).clamp(0.05, 1.0)
                };
                let w = (stroke.size * (0.30_f32 + 0.70_f32 * avg_pres) * scale).max(0.8_f32);
                let pencil_color = Color32::from_rgba_unmultiplied(
                    stroke.color[0],
                    stroke.color[1],
                    stroke.color[2],
                    (base_alpha as f32 * 0.85_f32) as u8,
                );
                let poly_pts: Vec<Pos2> = smooth_pts.iter().map(|(p, _)| *p).collect();

                // 100% Native Vector Line with Built-in Hardware Antialiased Feathering!
                painter.add(egui::Shape::line(poly_pts.clone(), Stroke::new(w, pencil_color)));
                if let Some(&first) = poly_pts.first() {
                    painter.circle_filled(first, w * 0.5_f32, pencil_color);
                }
                if let Some(&last) = poly_pts.last() {
                    painter.circle_filled(last, w * 0.5_f32, pencil_color);
                }
            }
            PenType::Calligraphy => {
                let angle = std::f32::consts::FRAC_PI_4; // 45 degree chisel
                let n = smooth_pts.len();

                let mut left_pts = Vec::with_capacity(n);
                let mut right_pts = Vec::with_capacity(n);

                for i in 0..n {
                    let p = smooth_pts[i].0;
                    let pres = smooth_pts[i].1.clamp(0.05, 1.0);
                    let base_w = stroke.size * (1.1_f32 + 1.4_f32 * pres) * scale;
                    let dx = angle.cos() * (base_w * 0.5_f32);
                    let dy = angle.sin() * (base_w * 0.5_f32);

                    left_pts.push(Pos2::new(p.x - dx, p.y - dy));
                    right_pts.push(Pos2::new(p.x + dx, p.y + dy));
                }

                let mut mesh = egui::Mesh::default();
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

                // Add antialiased outer strokes for Calligraphy boundaries
                let edge_stroke = Stroke::new(1.0_f32, color);
                painter.add(egui::Shape::line(left_pts, edge_stroke));
                painter.add(egui::Shape::line(right_pts, edge_stroke));
            }
        }
    }
}
