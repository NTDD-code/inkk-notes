//! High-performance interactive rendering for mathematical plots, matrices, and tables.
use crate::model::{MatrixEntity, PlotEntity, TableGridEntity};
use crate::theme::Palette;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

pub struct MathRenderer;

impl MathRenderer {
    /// Renders an interactive 2D function plot card with coordinate axes, ticks, and curves.
    pub fn render_plot(
        painter: &egui::Painter,
        plot: &PlotEntity,
        scale: f32,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        palette: &Palette,
        is_dark: bool,
    ) {
        let card_min = to_screen(Pos2::new(plot.x, plot.y));
        let card_size = Vec2::new(plot.width * scale, plot.height * scale);
        let card_rect = Rect::from_min_size(card_min, card_size);

        // 1. Plot Card Background
        let card_bg = if is_dark {
            Color32::from_rgba_unmultiplied(22, 27, 34, 230)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 240)
        };
        painter.rect_filled(card_rect, 8.0, card_bg);
        painter.rect_stroke(card_rect, 8.0, Stroke::new(1.0_f32, palette.border));

        // 2. Title & Header
        let header_h = 28.0 * scale.min(1.2).max(0.7);
        if !plot.title.is_empty() {
            painter.text(
                card_rect.left_top() + Vec2::new(12.0 * scale, 8.0 * scale),
                egui::Align2::LEFT_TOP,
                &plot.title,
                egui::FontId::proportional((13.0 * scale).max(10.0)),
                palette.text,
            );
        }

        // Close / Delete '×' button at top-right
        let btn_size = (20.0 * scale).clamp(16.0, 24.0);
        let close_center = card_rect.right_top() + Vec2::new(-btn_size * 0.75, btn_size * 0.75);
        painter.circle_filled(
            close_center,
            btn_size * 0.42,
            palette.danger.gamma_multiply(0.18),
        );
        painter.text(
            close_center,
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(btn_size * 0.60),
            palette.danger,
        );

        // Edit '✏' button next to close button
        let edit_center = close_center - Vec2::new(btn_size * 1.05, 0.0);
        painter.circle_filled(
            edit_center,
            btn_size * 0.42,
            palette.accent.gamma_multiply(0.18),
        );
        painter.text(
            edit_center,
            egui::Align2::CENTER_CENTER,
            "✏",
            egui::FontId::proportional(btn_size * 0.48),
            palette.accent,
        );

        // Check for dedicated 1D number line or 3D coordinate space
        if plot.dimension.as_deref() == Some("1D") {
            Self::render_1d_number_line(painter, card_rect, plot, scale, palette, header_h);
            return;
        }
        if plot.dimension.as_deref() == Some("3D") {
            Self::render_3d_axes(painter, card_rect, plot, scale, palette, is_dark, header_h);
            return;
        }

        // 3. Inner graph bounds (with padding for tick labels)
        let pad_l = 42.0 * scale;
        let pad_r = 16.0 * scale;
        let pad_t = header_h + 10.0 * scale;
        let pad_b = 26.0 * scale;
        let graph_rect = Rect::from_min_max(
            card_rect.min + Vec2::new(pad_l, pad_t),
            card_rect.max - Vec2::new(pad_r, pad_b),
        );

        if graph_rect.width() <= 10.0 || graph_rect.height() <= 10.0 {
            return;
        }

        let inner_bg = if is_dark {
            Color32::from_rgb(13, 17, 23)
        } else {
            Color32::from_rgb(248, 250, 252)
        };
        painter.rect_filled(graph_rect, 4.0, inner_bg);
        painter.rect_stroke(graph_rect, 4.0, Stroke::new(1.0_f32, palette.border));

        let x_min = plot.x_min;
        let x_max = plot.x_max;
        let y_min = plot.y_min;
        let y_max = plot.y_max;
        let x_span = (x_max - x_min).max(1e-5);
        let y_span = (y_max - y_min).max(1e-5);

        let to_graph_px = |x_data: f64, y_data: f64| -> Pos2 {
            let u = ((x_data - x_min) / x_span) as f32;
            let v = ((y_data - y_min) / y_span) as f32;
            Pos2::new(
                graph_rect.min.x + u * graph_rect.width(),
                graph_rect.max.y - v * graph_rect.height(),
            )
        };

        let graph_painter = painter.with_clip_rect(graph_rect);

        // 4. Grid lines & Tick Labels
        let num_ticks = 5;
        let grid_stroke = Stroke::new(
            1.0_f32,
            if is_dark {
                Color32::from_rgb(33, 38, 45)
            } else {
                Color32::from_rgb(226, 232, 240)
            },
        );
        let font_id = egui::FontId::proportional((10.0 * scale).clamp(8.0, 12.0));

        // X Ticks
        for i in 0..=num_ticks {
            let frac = i as f64 / num_ticks as f64;
            let val = x_min + frac * x_span;
            let px_x = graph_rect.min.x + (frac as f32) * graph_rect.width();

            if plot.show_grid {
                graph_painter.line_segment(
                    [
                        Pos2::new(px_x, graph_rect.min.y),
                        Pos2::new(px_x, graph_rect.max.y),
                    ],
                    grid_stroke,
                );
            }

            // Label below graph
            let lbl = format!("{:.1}", val);
            painter.text(
                Pos2::new(px_x, graph_rect.max.y + 4.0 * scale),
                egui::Align2::CENTER_TOP,
                lbl,
                font_id.clone(),
                palette.text_muted,
            );
        }

        // Y Ticks
        for i in 0..=num_ticks {
            let frac = i as f64 / num_ticks as f64;
            let val = y_min + frac * y_span;
            let px_y = graph_rect.max.y - (frac as f32) * graph_rect.height();

            if plot.show_grid {
                graph_painter.line_segment(
                    [
                        Pos2::new(graph_rect.min.x, px_y),
                        Pos2::new(graph_rect.max.x, px_y),
                    ],
                    grid_stroke,
                );
            }

            // Label to left of graph
            let lbl = format!("{:.1}", val);
            painter.text(
                Pos2::new(graph_rect.min.x - 6.0 * scale, px_y),
                egui::Align2::RIGHT_CENTER,
                lbl,
                font_id.clone(),
                palette.text_muted,
            );
        }

        // 5. Zero Coordinate Axes (Ox, Oy)
        if plot.show_axes {
            let axis_stroke = Stroke::new(1.5_f32, palette.text_muted);
            // X Axis (y = 0)
            if y_min <= 0.0 && y_max >= 0.0 {
                let y_zero_px = to_graph_px(0.0, 0.0).y;
                graph_painter.line_segment(
                    [
                        Pos2::new(graph_rect.min.x, y_zero_px),
                        Pos2::new(graph_rect.max.x, y_zero_px),
                    ],
                    axis_stroke,
                );
            }
            // Y Axis (x = 0)
            if x_min <= 0.0 && x_max >= 0.0 {
                let x_zero_px = to_graph_px(0.0, 0.0).x;
                graph_painter.line_segment(
                    [
                        Pos2::new(x_zero_px, graph_rect.min.y),
                        Pos2::new(x_zero_px, graph_rect.max.y),
                    ],
                    axis_stroke,
                );
            }
        }

        // 6. Draw Curves (MATLAB default color cycle)
        let default_colors = [
            Color32::from_rgb(0, 114, 189),  // #0072BD
            Color32::from_rgb(217, 83, 25),  // #D95319
            Color32::from_rgb(237, 177, 32), // #EDB120
            Color32::from_rgb(126, 47, 142), // #7E2F8E
            Color32::from_rgb(119, 172, 48), // #77AC30
            Color32::from_rgb(77, 190, 238), // #4DBEEE
            Color32::from_rgb(162, 20, 47),  // #A2142F
        ];

        let mut legend_items: Vec<(String, Color32)> = Vec::new();

        for (idx, curve) in plot.curves.iter().enumerate() {
            if curve.points.len() < 2 {
                continue;
            }
            let curve_color = if curve.color[3] > 0 {
                Color32::from_rgba_unmultiplied(
                    curve.color[0],
                    curve.color[1],
                    curve.color[2],
                    curve.color[3],
                )
            } else {
                default_colors[idx % default_colors.len()]
            };

            let screen_points: Vec<Pos2> = curve
                .points
                .iter()
                .map(|pt| to_graph_px(pt[0], pt[1]))
                .collect();

            let stroke_width = (2.0 * scale).clamp(1.5, 4.0);
            for i in 0..screen_points.len() - 1 {
                graph_painter.line_segment(
                    [screen_points[i], screen_points[i + 1]],
                    Stroke::new(stroke_width, curve_color),
                );
            }

            if !curve.label.is_empty() {
                legend_items.push((curve.label.clone(), curve_color));
            }
        }

        // 7. Legend in top-right corner
        if !legend_items.is_empty() {
            let mut leg_pos = graph_rect.right_top() + Vec2::new(-8.0 * scale, 8.0 * scale);
            for (lbl, col) in legend_items {
                let galley = painter.layout_no_wrap(
                    lbl,
                    egui::FontId::proportional((10.0 * scale).max(8.0)),
                    palette.text,
                );
                let chip_w = 12.0 * scale;
                let total_w = chip_w + 6.0 + galley.size().x;
                let leg_item_min = Pos2::new(leg_pos.x - total_w, leg_pos.y);

                painter.line_segment(
                    [
                        leg_item_min + Vec2::new(0.0, galley.size().y * 0.5),
                        leg_item_min + Vec2::new(chip_w, galley.size().y * 0.5),
                    ],
                    Stroke::new(2.5_f32, col),
                );
                painter.galley(
                    leg_item_min + Vec2::new(chip_w + 6.0, 0.0),
                    galley,
                    palette.text,
                );

                leg_pos.y += 16.0 * scale;
            }
        }
    }

    /// Renders an algebraic matrix entity in bracketed notation.
    pub fn render_matrix(
        painter: &egui::Painter,
        matrix: &MatrixEntity,
        scale: f32,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        palette: &Palette,
        is_dark: bool,
    ) {
        let card_min = to_screen(Pos2::new(matrix.x, matrix.y));
        let card_size = Vec2::new(matrix.width * scale, matrix.height * scale);
        let card_rect = Rect::from_min_size(card_min, card_size);

        let bg = if is_dark {
            Color32::from_rgba_unmultiplied(22, 27, 34, 210)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 230)
        };
        painter.rect_filled(card_rect, 6.0, bg);
        painter.rect_stroke(card_rect, 6.0, Stroke::new(1.0_f32, palette.border));

        let rows = matrix.data.len();
        if rows == 0 {
            return;
        }
        let cols = matrix.data[0].len();
        if cols == 0 {
            return;
        }

        // Label on left: e.g. "A ="
        let mut x_offset = card_rect.min.x + 12.0 * scale;
        if !matrix.label.is_empty() {
            let lbl_galley = painter.layout_no_wrap(
                matrix.label.clone(),
                egui::FontId::proportional((15.0 * scale).max(11.0)),
                palette.text,
            );
            painter.galley(
                Pos2::new(x_offset, card_rect.center().y - lbl_galley.size().y * 0.5),
                lbl_galley.clone(),
                palette.text,
            );
            x_offset += lbl_galley.size().x + 8.0 * scale;
        }

        // Bracket box
        let bracket_rect = Rect::from_min_max(
            Pos2::new(x_offset, card_rect.min.y + 10.0 * scale),
            Pos2::new(
                card_rect.max.x - 12.0 * scale,
                card_rect.max.y - 10.0 * scale,
            ),
        );

        // Close / Delete '×' button at top-right
        let btn_size = (18.0 * scale).clamp(14.0, 22.0);
        let close_center = card_rect.right_top() + Vec2::new(-btn_size * 0.65, btn_size * 0.65);
        painter.circle_filled(
            close_center,
            btn_size * 0.4,
            palette.danger.gamma_multiply(0.18),
        );
        painter.text(
            close_center,
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(btn_size * 0.6),
            palette.danger,
        );

        // Edit '✏' button next to close button
        let edit_center = close_center - Vec2::new(btn_size * 1.05, 0.0);
        painter.circle_filled(
            edit_center,
            btn_size * 0.4,
            palette.accent.gamma_multiply(0.18),
        );
        painter.text(
            edit_center,
            egui::Align2::CENTER_CENTER,
            "✏",
            egui::FontId::proportional(btn_size * 0.48),
            palette.accent,
        );

        let bracket_color = palette.accent;
        let bracket_stroke = Stroke::new(2.0 * scale.min(1.5), bracket_color);
        let bw = 6.0 * scale;

        // Left bracket: [
        painter.line_segment(
            [
                bracket_rect.left_top(),
                bracket_rect.left_top() + Vec2::new(bw, 0.0),
            ],
            bracket_stroke,
        );
        painter.line_segment(
            [bracket_rect.left_top(), bracket_rect.left_bottom()],
            bracket_stroke,
        );
        painter.line_segment(
            [
                bracket_rect.left_bottom(),
                bracket_rect.left_bottom() + Vec2::new(bw, 0.0),
            ],
            bracket_stroke,
        );

        // Right bracket: ]
        painter.line_segment(
            [
                bracket_rect.right_top(),
                bracket_rect.right_top() - Vec2::new(bw, 0.0),
            ],
            bracket_stroke,
        );
        painter.line_segment(
            [bracket_rect.right_top(), bracket_rect.right_bottom()],
            bracket_stroke,
        );
        painter.line_segment(
            [
                bracket_rect.right_bottom(),
                bracket_rect.right_bottom() - Vec2::new(bw, 0.0),
            ],
            bracket_stroke,
        );

        // Render matrix elements evenly inside
        let inner_rect = bracket_rect.shrink2(Vec2::new(bw + 4.0 * scale, 4.0 * scale));
        let cell_w = inner_rect.width() / cols as f32;
        let cell_h = inner_rect.height() / rows as f32;

        for r in 0..rows {
            for c in 0..cols {
                let val = matrix.data[r][c];
                let val_str = if val.fract() == 0.0 && val.abs() < 1e9 {
                    format!("{:.0}", val)
                } else {
                    format!("{:.2}", val)
                };

                let center = Pos2::new(
                    inner_rect.min.x + (c as f32 + 0.5) * cell_w,
                    inner_rect.min.y + (r as f32 + 0.5) * cell_h,
                );

                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    val_str,
                    egui::FontId::monospace((13.0 * scale).clamp(9.0, 18.0)),
                    palette.text,
                );
            }
        }
    }

    /// Renders a structured column-row table grid.
    pub fn render_table_grid(
        painter: &egui::Painter,
        grid: &TableGridEntity,
        scale: f32,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        palette: &Palette,
        is_dark: bool,
    ) {
        let origin = to_screen(Pos2::new(grid.x, grid.y));
        let col_w = grid.col_width * scale;
        let row_h = grid.row_height * scale;
        let total_w = col_w * grid.cols as f32;
        let total_h = row_h * grid.rows as f32;
        let grid_rect = Rect::from_min_size(origin, Vec2::new(total_w, total_h));

        let bg = if is_dark {
            Color32::from_rgba_unmultiplied(22, 27, 34, 210)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 230)
        };
        painter.rect_filled(grid_rect, 4.0, bg);
        painter.rect_stroke(grid_rect, 4.0, Stroke::new(1.5_f32, palette.border));

        // Close / Delete '×' button at top-right
        let btn_size = (18.0 * scale).clamp(14.0, 22.0);
        let close_center = grid_rect.right_top() + Vec2::new(-btn_size * 0.75, btn_size * 0.75);
        painter.circle_filled(
            close_center,
            btn_size * 0.4,
            palette.danger.gamma_multiply(0.2),
        );
        painter.text(
            close_center,
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(btn_size * 0.6),
            palette.danger,
        );

        // Edit '✏' button next to close button
        let edit_center = close_center - Vec2::new(btn_size * 1.05, 0.0);
        painter.circle_filled(
            edit_center,
            btn_size * 0.4,
            palette.accent.gamma_multiply(0.18),
        );
        painter.text(
            edit_center,
            egui::Align2::CENTER_CENTER,
            "✏",
            egui::FontId::proportional(btn_size * 0.48),
            palette.accent,
        );

        // Horizontal grid lines
        for r in 1..grid.rows {
            let y = origin.y + (r as f32) * row_h;
            painter.line_segment(
                [Pos2::new(origin.x, y), Pos2::new(origin.x + total_w, y)],
                Stroke::new(1.0_f32, palette.border),
            );
        }

        // Vertical grid lines
        for c in 1..grid.cols {
            let x = origin.x + (c as f32) * col_w;
            painter.line_segment(
                [Pos2::new(x, origin.y), Pos2::new(x, origin.y + total_h)],
                Stroke::new(1.0_f32, palette.border),
            );
        }

        // Cells text
        for r in 0..grid.rows {
            for c in 0..grid.cols {
                if let Some(row_data) = grid.cells.get(r) {
                    if let Some(text) = row_data.get(c) {
                        let cell_rect = Rect::from_min_size(
                            origin + Vec2::new((c as f32) * col_w, (r as f32) * row_h),
                            Vec2::new(col_w, row_h),
                        );
                        painter.text(
                            cell_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional((12.0 * scale).max(9.0)),
                            palette.text,
                        );
                    }
                }
            }
        }
    }

    fn render_1d_number_line(
        painter: &egui::Painter,
        card_rect: Rect,
        plot: &PlotEntity,
        scale: f32,
        palette: &Palette,
        header_h: f32,
    ) {
        let cy = card_rect.min.y + header_h + (card_rect.height() - header_h) * 0.42;
        let x_start = card_rect.min.x + 32.0 * scale;
        let x_end = card_rect.max.x - 36.0 * scale;

        if x_end <= x_start {
            return;
        }

        let axis_stroke = Stroke::new(2.0 * scale.min(2.0), palette.text);
        // Main horizontal axis
        painter.line_segment([Pos2::new(x_start, cy), Pos2::new(x_end, cy)], axis_stroke);

        // Arrowhead at positive end
        let arrow_size = 7.0 * scale;
        painter.line_segment(
            [
                Pos2::new(x_end, cy),
                Pos2::new(x_end - arrow_size * 1.3, cy - arrow_size),
            ],
            axis_stroke,
        );
        painter.line_segment(
            [
                Pos2::new(x_end, cy),
                Pos2::new(x_end - arrow_size * 1.3, cy + arrow_size),
            ],
            axis_stroke,
        );

        // Label 'x'
        painter.text(
            Pos2::new(x_end + 10.0 * scale, cy),
            egui::Align2::LEFT_CENTER,
            "x",
            egui::FontId::proportional((14.0 * scale).max(10.0)),
            palette.accent,
        );

        let x_min = plot.x_min;
        let x_max = plot.x_max;
        let span = (x_max - x_min).max(1e-4);
        let step = plot.x_step.unwrap_or(2.0).max(0.001);

        let font_id = egui::FontId::proportional((10.0 * scale).clamp(8.0, 12.0));
        let first_tick = (x_min / step).ceil() * step;
        let mut curr = first_tick;

        let max_ticks = 40;
        let mut count = 0;
        while curr <= x_max + 1e-6 && count < max_ticks {
            count += 1;
            let frac = ((curr - x_min) / span) as f32;
            let px_x = x_start + frac * (x_end - x_start);

            let is_zero = curr.abs() < 1e-6;
            let tick_h = if is_zero { 9.0 * scale } else { 5.5 * scale };
            let tick_stroke = if is_zero {
                Stroke::new(2.5 * scale, palette.accent)
            } else {
                Stroke::new(1.2 * scale, palette.text_muted)
            };

            painter.line_segment(
                [Pos2::new(px_x, cy - tick_h), Pos2::new(px_x, cy + tick_h)],
                tick_stroke,
            );

            let lbl_str = if curr.fract() == 0.0 {
                format!("{:.0}", curr)
            } else {
                format!("{:.1}", curr)
            };
            let text_color = if is_zero {
                palette.accent
            } else {
                palette.text_muted
            };
            painter.text(
                Pos2::new(px_x, cy + tick_h + 4.0 * scale),
                egui::Align2::CENTER_TOP,
                lbl_str,
                font_id.clone(),
                text_color,
            );

            curr += step;
        }
    }

    fn render_3d_axes(
        painter: &egui::Painter,
        card_rect: Rect,
        plot: &PlotEntity,
        scale: f32,
        palette: &Palette,
        is_dark: bool,
        header_h: f32,
    ) {
        let origin = Pos2::new(
            card_rect.center().x,
            card_rect.min.y + header_h + (card_rect.height() - header_h) * 0.60,
        );
        let axis_len = (120.0 * scale).min(card_rect.width() * 0.40);

        // 3D Isometric Directions:
        // Z: Up (0, -1)
        // X: Down-Left (-cos(30°), sin(30°)) = (-0.866, 0.5)
        // Y: Down-Right (cos(30°), sin(30°)) = (0.866, 0.5)
        let dir_z = Vec2::new(0.0, -1.0);
        let dir_x = Vec2::new(-0.866025, 0.5);
        let dir_y = Vec2::new(0.866025, 0.5);

        let end_z = origin + dir_z * axis_len;
        let end_x = origin + dir_x * (axis_len * 1.05);
        let end_y = origin + dir_y * (axis_len * 1.05);

        // Optional ground grid plane
        if plot.show_grid {
            let grid_col = if is_dark {
                Color32::from_rgb(33, 38, 45)
            } else {
                Color32::from_rgb(226, 232, 240)
            };
            let grid_stroke = Stroke::new(1.0 * scale, grid_col);
            let n_div = 4;
            for i in 1..=n_div {
                let frac = i as f32 / n_div as f32;
                let pt_x = origin + dir_x * (axis_len * frac);
                let pt_y = origin + dir_y * (axis_len * frac);
                painter.line_segment([pt_x, pt_x + dir_y * axis_len], grid_stroke);
                painter.line_segment([pt_y, pt_y + dir_x * axis_len], grid_stroke);
            }
        }

        // Draw axes lines
        let stroke_z = Stroke::new(2.2 * scale, Color32::from_rgb(0, 114, 189)); // Z blue
        let stroke_x = Stroke::new(2.2 * scale, Color32::from_rgb(217, 83, 25)); // X red
        let stroke_y = Stroke::new(2.2 * scale, Color32::from_rgb(119, 172, 48)); // Y green

        painter.line_segment([origin, end_z], stroke_z);
        painter.line_segment([origin, end_x], stroke_x);
        painter.line_segment([origin, end_y], stroke_y);

        // Arrowheads & Axis Labels
        let arrow_sz = 6.0 * scale;
        // Z arrow
        painter.line_segment(
            [end_z, end_z + Vec2::new(-arrow_sz * 0.7, arrow_sz * 1.2)],
            stroke_z,
        );
        painter.line_segment(
            [end_z, end_z + Vec2::new(arrow_sz * 0.7, arrow_sz * 1.2)],
            stroke_z,
        );
        painter.text(
            end_z + Vec2::new(0.0, -12.0 * scale),
            egui::Align2::CENTER_CENTER,
            "Z (+)",
            egui::FontId::proportional(12.0 * scale),
            stroke_z.color,
        );

        // X arrow
        painter.line_segment(
            [end_x, end_x + Vec2::new(arrow_sz * 1.2, -arrow_sz * 0.3)],
            stroke_x,
        );
        painter.line_segment(
            [end_x, end_x + Vec2::new(arrow_sz * 0.5, -arrow_sz * 1.3)],
            stroke_x,
        );
        painter.text(
            end_x + Vec2::new(-16.0 * scale, 6.0 * scale),
            egui::Align2::CENTER_CENTER,
            "X (+)",
            egui::FontId::proportional(12.0 * scale),
            stroke_x.color,
        );

        // Y arrow
        painter.line_segment(
            [end_y, end_y + Vec2::new(-arrow_sz * 1.2, -arrow_sz * 0.3)],
            stroke_y,
        );
        painter.line_segment(
            [end_y, end_y + Vec2::new(-arrow_sz * 0.5, -arrow_sz * 1.3)],
            stroke_y,
        );
        painter.text(
            end_y + Vec2::new(16.0 * scale, 6.0 * scale),
            egui::Align2::CENTER_CENTER,
            "Y (+)",
            egui::FontId::proportional(12.0 * scale),
            stroke_y.color,
        );

        // Origin mark O(0,0,0)
        painter.circle_filled(origin, 3.5 * scale, palette.text);
        painter.text(
            origin + Vec2::new(-8.0 * scale, -10.0 * scale),
            egui::Align2::RIGHT_BOTTOM,
            "O(0,0,0)",
            egui::FontId::proportional(10.0 * scale),
            palette.text_muted,
        );
    }
}
