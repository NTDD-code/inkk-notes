//! SVG and Raster Export for Geometry Sketchpad.
//! Generates vector SVG for whiteboard insertion and saves PNG/SVG files.

use eframe::egui::Pos2;
use super::engine::GeometryEngine;
use super::model::SketchpadModel;

pub struct SketchpadExporter;

impl SketchpadExporter {
    /// Exports the current sketch to an SVG vector string.
    pub fn to_svg(sketch: &SketchpadModel, width: f32, height: f32, is_dark: bool) -> String {
        let bg_color = if is_dark { "#1e1e1e" } else { "#ffffff" };
        let text_color = if is_dark { "#e0e0e0" } else { "#1a1a1a" };
        let axis_color = if is_dark { "#444444" } else { "#d0d0d0" };

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
<rect width="100%" height="100%" fill="{bg}" />
"#,
            w = width,
            h = height,
            bg = bg_color
        );

        let get_pos = |id: &str| -> Option<Pos2> {
            sketch.points.iter().find(|p| p.id == id).map(|p| p.pos)
        };

        // 1. Grid & Axes
        if sketch.show_axes {
            let cx = sketch.camera3d.center.x;
            let cy = sketch.camera3d.center.y;
            svg.push_str(&format!(
                r#"<line x1="0" y1="{cy}" x2="{w}" y2="{cy}" stroke="{axis}" stroke-width="1.5" />
<line x1="{cx}" y1="0" x2="{cx}" y2="{h}" stroke="{axis}" stroke-width="1.5" />
<polygon points="{w},{cy} {x1},{y1} {x1},{y2}" fill="{axis}" />
<polygon points="{cx},0 {x3},{y3} {x4},{y3}" fill="{axis}" />
"#,
                w = width,
                h = height,
                cx = cx,
                cy = cy,
                axis = axis_color,
                x1 = width - 10.0,
                y1 = cy - 4.0,
                y2 = cy + 4.0,
                x3 = cx - 4.0,
                x4 = cx + 4.0,
                y3 = 10.0,
            ));
        }

        // 2. Polygons
        for poly in &sketch.polygons {
            if poly.is_hidden {
                continue;
            }
            let pts: Vec<Pos2> = poly.vertices.iter().filter_map(|vid| get_pos(vid)).collect();
            if pts.len() >= 3 {
                let pts_str = pts
                    .iter()
                    .map(|p| format!("{:.1},{:.1}", p.x, p.y))
                    .collect::<Vec<_>>()
                    .join(" ");

                let fill_attr = if let Some(fc) = poly.fill_color {
                    let fill_hex = format!("#{:02x}{:02x}{:02x}", fc.r(), fc.g(), fc.b());
                    let fill_opacity = fc.a() as f32 / 255.0;
                    format!(r#"fill="{}" fill-opacity="{:.2}""#, fill_hex, fill_opacity)
                } else {
                    r#"fill="none""#.to_string()
                };

                let stroke_hex = format!(
                    "#{:02x}{:02x}{:02x}",
                    poly.stroke_color.r(),
                    poly.stroke_color.g(),
                    poly.stroke_color.b()
                );
                svg.push_str(&format!(
                    r#"<polygon points="{}" {} stroke="{}" stroke-width="2" />
"#,
                    pts_str, fill_attr, stroke_hex
                ));
            }
        }

        // 3. Circles
        for circle in &sketch.circles {
            if circle.is_hidden {
                continue;
            }
            if let Some(cp) = get_pos(&circle.center) {
                let r = if let Some(rid) = &circle.radius_pt {
                    get_pos(rid).map(|rp| (rp - cp).length()).unwrap_or(50.0)
                } else {
                    circle.fixed_radius.unwrap_or(50.0)
                };
                let col_hex = format!("#{:02x}{:02x}{:02x}", circle.color.r(), circle.color.g(), circle.color.b());
                let dash_attr = if circle.is_dashed { r#"stroke-dasharray="6,4""# } else { "" };
                svg.push_str(&format!(
                    r#"<circle cx="{:.1}" cy="{:.1}" r="{:.1}" fill="none" stroke="{}" stroke-width="2" {} />
"#,
                    cp.x, cp.y, r, col_hex, dash_attr
                ));
            }
        }

        // 4. Lines (Segments, Rays, Lines)
        for line in &sketch.lines {
            if line.is_hidden {
                continue;
            }
            if let (Some(p1), Some(p2)) = (get_pos(&line.p1), get_pos(&line.p2)) {
                let col_hex = format!("#{:02x}{:02x}{:02x}", line.color.r(), line.color.g(), line.color.b());
                let dash_attr = if line.is_dashed { r#"stroke-dasharray="6,4""# } else { "" };
                svg.push_str(&format!(
                    r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{:.1}" {} />
"#,
                    p1.x, p1.y, p2.x, p2.y, col_hex, line.width, dash_attr
                ));
            }
        }

        // 5. Function Curves
        for f in &sketch.functions {
            let col_hex = format!("#{:02x}{:02x}{:02x}", f.color.r(), f.color.g(), f.color.b());
            let mut path_data = String::new();
            let mut first = true;
            let cx = sketch.camera3d.center.x;
            let cy = sketch.camera3d.center.y;
            let scale = 40.0;

            let steps = 120;
            for i in 0..=steps {
                let px = (i as f32 / steps as f32) * width;
                let math_x = (px - cx) / scale;
                if let Some(math_y) = GeometryEngine::eval_func(&f.expr, math_x) {
                    let py = cy - math_y * scale;
                    if py >= -50.0 && py <= height + 50.0 {
                        if first {
                            path_data.push_str(&format!("M{:.1},{:.1}", px, py));
                            first = false;
                        } else {
                            path_data.push_str(&format!(" L{:.1},{:.1}", px, py));
                        }
                    } else {
                        first = true;
                    }
                }
            }
            if !path_data.is_empty() {
                svg.push_str(&format!(
                    r#"<path d="{}" fill="none" stroke="{}" stroke-width="2.5" />
"#,
                    path_data, col_hex
                ));
            }
        }

        // 6. Trace Points (Locus)
        if !sketch.traces.is_empty() {
            let mut trace_path = String::new();
            let mut first = true;
            for tr in &sketch.traces {
                if first {
                    trace_path.push_str(&format!("M{:.1},{:.1}", tr.pos.x, tr.pos.y));
                    first = false;
                } else {
                    trace_path.push_str(&format!(" L{:.1},{:.1}", tr.pos.x, tr.pos.y));
                }
            }
            svg.push_str(&format!(
                r##"<path d="{}" fill="none" stroke="#e02020" stroke-width="1.8" stroke-dasharray="2,2" />
"##,
                trace_path
            ));
        }

        // 7. Points & Labels
        for pt in &sketch.points {
            if pt.is_hidden {
                continue;
            }
            let col_hex = format!("#{:02x}{:02x}{:02x}", pt.color.r(), pt.color.g(), pt.color.b());
            svg.push_str(&format!(
                r##"<circle cx="{:.1}" cy="{:.1}" r="4" fill="{}" stroke="#ffffff" stroke-width="1.5" />
"##,
                pt.pos.x, pt.pos.y, col_hex
            ));
            if !pt.label.is_empty() {
                svg.push_str(&format!(
                    r#"<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="14" font-weight="bold" fill="{}">{}</text>
"#,
                    pt.pos.x + 6.0,
                    pt.pos.y - 6.0,
                    text_color,
                    pt.label
                ));
            }
        }

        // 8. Measurements overlay
        for m in &sketch.measurements {
            svg.push_str(&format!(
                r#"<text x="{:.1}" y="{:.1}" font-family="sans-serif" font-size="13" font-weight="600" fill="{}">{}</text>
"#,
                m.pos.x, m.pos.y, text_color, m.display_text
            ));
        }

        svg.push_str("</svg>");
        svg
    }

    /// Renders the sketch directly to PNG bytes for whiteboard insertion.
    pub fn to_raster_png(sketch: &SketchpadModel, width: f32, height: f32, is_dark: bool) -> Option<(Vec<u8>, f32, f32)> {
        let svg_str = Self::to_svg(sketch, width, height, is_dark);
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_str(&svg_str, &opt).ok()?;

        let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32)?;
        let render_ts = tiny_skia::Transform::identity();
        resvg::render(&tree, render_ts, &mut pixmap.as_mut());

        let png_bytes = pixmap.encode_png().ok()?;
        Some((png_bytes, width, height))
    }
}
