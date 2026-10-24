//! Small reusable widgets for the app chrome, styled from `theme`.
use eframe::egui::{
    self, Color32, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2, WidgetInfo,
    WidgetType,
};

use crate::theme::{self, Palette};
use crate::vector_icon::VectorIcon;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToolIcon<'a> {
    Glyph(&'a str),
    Vector(VectorIcon),
}

impl<'a> From<&'a str> for ToolIcon<'a> {
    fn from(s: &'a str) -> Self {
        match s {
            "save" | "Save" | "💾" => ToolIcon::Vector(VectorIcon::Save),
            "pages" | "Pages" | "📑" => ToolIcon::Vector(VectorIcon::Pages),
            "open" | "Open" | "📂" => ToolIcon::Vector(VectorIcon::Open),
            "math_studio" | "MathStudio" | "🔬" => ToolIcon::Vector(VectorIcon::MathStudio),
            "matrix" | "Matrix" | "📊" => ToolIcon::Vector(VectorIcon::Matrix),
            "axes" | "Axes" | "📐" => ToolIcon::Vector(VectorIcon::Axes),
            "table" | "Table" | "▦" => ToolIcon::Vector(VectorIcon::Table),
            "pdf" | "Pdf" | "export_pdf" => ToolIcon::Vector(VectorIcon::Pdf),
            glyph => ToolIcon::Glyph(glyph),
        }
    }
}

impl<'a> From<VectorIcon> for ToolIcon<'a> {
    fn from(v: VectorIcon) -> Self {
        ToolIcon::Vector(v)
    }
}

/// Square icon button with an active state, used for tools and quick actions.
pub fn tool_button<'a>(
    ui: &mut Ui,
    p: &Palette,
    icon: impl Into<ToolIcon<'a>>,
    active: bool,
    tooltip: &str,
) -> Response {
    let icon = icon.into();
    let size = Vec2::splat(theme::TOOL_BUTTON_SIZE);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            ui.is_enabled(),
            active,
            tooltip,
        )
    });

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke, fg) = if active {
            (p.accent_soft, Stroke::new(1.0_f32, p.accent), p.accent)
        } else if resp.hovered() {
            (p.hover, Stroke::NONE, p.text)
        } else {
            (Color32::TRANSPARENT, Stroke::NONE, p.text_muted)
        };
        let fg = if ui.is_enabled() {
            fg
        } else {
            p.text_muted.gamma_multiply(0.45)
        };
        painter.rect(rect, theme::RADIUS, fill, stroke);

        match icon {
            ToolIcon::Glyph(s) => {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    s,
                    egui::FontId::proportional(17.0),
                    fg,
                );
            }
            ToolIcon::Vector(v) => {
                let icon_size = 19.0;
                let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
                v.paint(painter, icon_rect, fg, 1.8);
            }
        }
    }
    resp.on_hover_text(tooltip)
}

/// Primary button with vector icon and English text.
pub fn primary_icon_button(ui: &mut Ui, p: &Palette, icon: VectorIcon, label: &str) -> Response {
    let padding = Vec2::new(10.0, 5.0);
    let icon_size = 16.0;
    let font_id = egui::FontId::proportional(14.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font_id, p.on_accent);
    let desired_size = Vec2::new(
        padding.x * 2.0 + icon_size + 6.0 + galley.rect.width(),
        (padding.y * 2.0 + icon_size.max(galley.rect.height())).max(28.0),
    );
    let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if resp.is_pointer_button_down_on() {
            p.accent_soft
        } else if resp.hovered() {
            p.accent.gamma_multiply(0.9)
        } else {
            p.accent
        };
        painter.rect_filled(rect, Rounding::same(theme::RADIUS_SMALL), bg);
        let content_start_x = rect.min.x + padding.x;
        let icon_rect = Rect::from_min_size(
            Pos2::new(content_start_x, rect.center().y - icon_size * 0.5),
            Vec2::splat(icon_size),
        );
        icon.paint(painter, icon_rect, p.on_accent, 1.8);
        let text_pos = Pos2::new(
            content_start_x + icon_size + 6.0,
            rect.center().y - galley.rect.height() * 0.5,
        );
        painter.galley(text_pos, galley, p.on_accent);
    }
    resp
}

/// Secondary button with vector icon and English text.
#[allow(dead_code)]
pub fn secondary_icon_button(ui: &mut Ui, p: &Palette, icon: VectorIcon, label: &str) -> Response {
    let padding = Vec2::new(10.0, 5.0);
    let icon_size = 16.0;
    let font_id = egui::FontId::proportional(14.0);
    let fg = if ui.is_enabled() {
        p.text
    } else {
        p.text_muted
    };
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, fg);
    let desired_size = Vec2::new(
        padding.x * 2.0 + icon_size + 6.0 + galley.rect.width(),
        (padding.y * 2.0 + icon_size.max(galley.rect.height())).max(28.0),
    );
    let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke) = if resp.hovered() {
            (p.hover, Stroke::new(1.0_f32, p.border))
        } else {
            (p.surface_alt, Stroke::new(1.0_f32, p.border))
        };
        painter.rect(rect, Rounding::same(theme::RADIUS_SMALL), fill, stroke);
        let content_start_x = rect.min.x + padding.x;
        let icon_rect = Rect::from_min_size(
            Pos2::new(content_start_x, rect.center().y - icon_size * 0.5),
            Vec2::splat(icon_size),
        );
        icon.paint(painter, icon_rect, fg, 1.8);
        let text_pos = Pos2::new(
            content_start_x + icon_size + 6.0,
            rect.center().y - galley.rect.height() * 0.5,
        );
        painter.galley(text_pos, galley, fg);
    }
    resp
}

/// Pill-shaped chip with vector icon.
pub fn vector_chip(
    ui: &mut Ui,
    p: &Palette,
    icon: VectorIcon,
    label: &str,
    active: bool,
) -> Response {
    let padding = Vec2::new(10.0, 4.0);
    let icon_size = 15.0;
    let font_id = egui::FontId::proportional(13.0);
    let fg = if active { p.accent } else { p.text };
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_id, fg);
    let desired_size = Vec2::new(
        padding.x * 2.0 + icon_size + 6.0 + galley.rect.width(),
        28.0,
    );
    let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke) = if active {
            (p.accent_soft, Stroke::new(1.0_f32, p.accent))
        } else if resp.hovered() {
            (p.hover, Stroke::NONE)
        } else {
            (p.surface_alt, Stroke::NONE)
        };
        painter.rect(rect, Rounding::same(14.0), fill, stroke);
        let content_start_x = rect.min.x + padding.x;
        let icon_rect = Rect::from_min_size(
            Pos2::new(content_start_x, rect.center().y - icon_size * 0.5),
            Vec2::splat(icon_size),
        );
        icon.paint(painter, icon_rect, fg, 1.8);
        let text_pos = Pos2::new(
            content_start_x + icon_size + 6.0,
            rect.center().y - galley.rect.height() * 0.5,
        );
        painter.galley(text_pos, galley, fg);
    }
    resp
}

/// Vertical card widget with vector icon on top and label below, matching the reference images.
#[allow(dead_code)]
pub fn icon_column_card(
    ui: &mut Ui,
    p: &Palette,
    icon: VectorIcon,
    label: &str,
    active: bool,
    tooltip: &str,
) -> Response {
    let btn_size = Vec2::new(54.0, 36.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(72.0, 64.0), Sense::click());
    resp.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            ui.is_enabled(),
            active,
            tooltip,
        )
    });

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let btn_rect =
            Rect::from_center_size(Pos2::new(rect.center().x, rect.min.y + 20.0), btn_size);
        let (fill, stroke, fg) = if active {
            (p.accent_soft, Stroke::new(1.0_f32, p.accent), p.accent)
        } else if resp.hovered() {
            (p.hover, Stroke::NONE, p.text)
        } else {
            (Color32::TRANSPARENT, Stroke::NONE, p.text)
        };
        painter.rect(btn_rect, Rounding::same(10.0), fill, stroke);
        let icon_rect = Rect::from_center_size(btn_rect.center(), Vec2::splat(20.0));
        icon.paint(painter, icon_rect, fg, 1.8);

        let label_color = if active { p.accent } else { p.text };
        let galley = ui.painter().layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(12.5),
            label_color,
        );
        let label_pos = Pos2::new(
            rect.center().x - galley.rect.width() * 0.5,
            rect.min.y + 44.0,
        );
        painter.galley(label_pos, galley, label_color);
    }
    resp.on_hover_text(tooltip)
}

/// Pill-shaped text toggle used for sub-options (pen type, filled shapes, ...).
pub fn chip(ui: &mut Ui, p: &Palette, label: &str, active: bool) -> Response {
    let text = if active {
        RichText::new(label).color(p.accent).strong()
    } else {
        RichText::new(label).color(p.text)
    };
    let btn = egui::Button::new(text)
        .fill(if active { p.accent_soft } else { p.surface_alt })
        .stroke(if active {
            Stroke::new(1.0_f32, p.accent)
        } else {
            Stroke::NONE
        })
        .rounding(Rounding::same(theme::RADIUS_MD));
    let resp = ui.add(btn);
    // Draw the signature brick-red margin motif along the active chip edge
    if active && ui.is_rect_visible(resp.rect) {
        let bottom_y = resp.rect.max.y - 1.0;
        let p1 = Pos2::new(resp.rect.min.x + 6.0, bottom_y);
        let p2 = Pos2::new(resp.rect.max.x - 6.0, bottom_y);
        ui.painter()
            .line_segment([p1, p2], Stroke::new(2.0_f32, p.accent));
    }
    resp
}

/// Bespoke Ink Color Popover widget replacing egui's default debug color picker.
pub fn ink_palette_button(
    ui: &mut Ui,
    p: &Palette,
    current_color: &mut Color32,
    custom_swatches: &mut Vec<Color32>,
    opacity: &mut f32,
    is_dark: bool,
) -> Response {
    let button_id = egui::Id::new("ink_palette_popover");
    let is_open = ui.data(|d| d.get_temp::<bool>(button_id).unwrap_or(false));

    // Allocate space for button: circle swatch + small chevron
    let (rect, mut resp) =
        ui.allocate_exact_size(Vec2::new(34.0, theme::TOOL_BUTTON_SIZE), Sense::click());
    if resp.clicked() {
        ui.data_mut(|d| d.insert_temp(button_id, !is_open));
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if is_open {
            p.accent_soft
        } else if resp.hovered() {
            p.hover
        } else {
            Color32::TRANSPARENT
        };
        let border_stroke = if is_open {
            Stroke::new(1.0_f32, p.accent)
        } else {
            Stroke::NONE
        };
        painter.rect(rect, Rounding::same(theme::RADIUS_MD), bg, border_stroke);

        let c = Pos2::new(rect.min.x + 13.0, rect.center().y);
        let alpha_u8 = ((*opacity).clamp(0.1, 1.0) * 255.0) as u8;
        let displayed_color = Color32::from_rgba_unmultiplied(
            current_color.r(),
            current_color.g(),
            current_color.b(),
            alpha_u8,
        );
        painter.circle_filled(c, 8.5, displayed_color);
        painter.circle_stroke(c, 8.5, Stroke::new(1.0_f32, p.border));

        // Small chevron dot indicator
        let arrow_pos = Pos2::new(rect.min.x + 25.0, rect.center().y);
        let arrow_color = if is_open { p.accent } else { p.text_muted };
        painter.circle_filled(arrow_pos, 2.0, arrow_color);
    }
    resp = resp.on_hover_text("Ink Palette & Density (Click to pick ink)");

    if is_open {
        let mut close_popup = false;
        let popup_pos = rect.left_bottom() + Vec2::new(0.0, 4.0);

        let custom_sel_id = ui.make_persistent_id("ink_palette_selected_custom");
        let mut selected_custom_idx =
            ui.data(|d| d.get_temp::<Option<usize>>(custom_sel_id).flatten());

        let hsv_id = ui.make_persistent_id("ink_palette_hsv");
        let mut hsv = ui
            .data(|d| d.get_temp::<(f32, f32, f32)>(hsv_id))
            .unwrap_or_else(|| theme::rgb_to_hsv(*current_color));

        let area_resp = egui::Area::new(button_id)
            .fixed_pos(popup_pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(p.surface)
                    .stroke(Stroke::new(1.0_f32, p.border))
                    .rounding(Rounding::same(theme::RADIUS_MD))
                    .shadow(egui::Shadow {
                        offset: Vec2::new(0.0, 4.0),
                        blur: 16.0,
                        spread: 0.0,
                        color: p.shadow,
                    })
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 3.0;
                        ui.set_min_width(330.0);

                        // Header
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Ink Studio")
                                    .font(theme::font_serif(13.0))
                                    .color(p.text)
                                    .strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let close_btn = egui::Button::new(
                                    RichText::new("×")
                                        .size(14.0)
                                        .color(p.text_muted),
                                )
                                .frame(false);
                                if ui.add(close_btn).on_hover_text("Close").clicked() {
                                    close_popup = true;
                                }
                            });
                        });

                        ui.add_space(2.0);

                        // Two Columns: Left (Swatches) + Right (2D Spectrum Picker)
                        ui.horizontal(|ui| {
                            // --- LEFT COLUMN: Archival & Custom Swatches ---
                            ui.vertical(|ui| {
                                ui.set_width(148.0);

                                ui.label(RichText::new("Archival Inks").font(theme::font_serif(11.0)).color(p.text_muted));

                                let swatches = theme::ink_swatches(is_dark);
                                egui::Grid::new("archival_swatches_grid")
                                    .spacing([5.0, 4.0])
                                    .show(ui, |ui| {
                                        for (idx, (swatch_col, name)) in swatches.iter().enumerate() {
                                            let active = *current_color == *swatch_col;
                                            let (s_rect, s_resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                                            if s_resp.clicked() {
                                                *current_color = *swatch_col;
                                                hsv = theme::rgb_to_hsv(*swatch_col);
                                                selected_custom_idx = None;
                                            }
                                            if s_resp.double_clicked() {
                                                close_popup = true;
                                            }
                                            if ui.is_rect_visible(s_rect) {
                                                let painter = ui.painter();
                                                let c = s_rect.center();
                                                if active {
                                                    painter.circle_stroke(c, 11.0, Stroke::new(2.0_f32, p.accent));
                                                    painter.circle_filled(c, 7.5, *swatch_col);
                                                } else {
                                                    let r = if s_resp.hovered() { 9.0 } else { 8.0 };
                                                    painter.circle_filled(c, r, *swatch_col);
                                                }
                                                painter.circle_stroke(c, if active { 7.5 } else { 8.0 }, Stroke::new(1.0_f32, p.border));
                                            }
                                            s_resp.on_hover_text(*name);

                                            if (idx + 1) % 3 == 0 {
                                                ui.end_row();
                                            }
                                        }
                                    });

                                ui.add_space(3.0);
                                ui.separator();
                                ui.add_space(1.0);

                                // Custom Swatches Header & Add Button
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Custom Inks").font(theme::font_serif(11.0)).color(p.text_muted));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let can_add = custom_swatches.len() < 12;
                                        if ui.add_enabled(can_add, egui::Button::new(RichText::new("+ Add").size(9.5)))
                                            .on_hover_text("Save current color to custom swatches")
                                            .clicked()
                                        {
                                            custom_swatches.push(*current_color);
                                            selected_custom_idx = Some(custom_swatches.len() - 1);
                                        }
                                    });
                                });

                                let mut delete_custom_idx = None;
                                let mut replace_custom_idx = None;

                                egui::Grid::new("custom_swatches_grid")
                                    .spacing([5.0, 4.0])
                                    .show(ui, |ui| {
                                        for (idx, swatch_col) in custom_swatches.iter().enumerate() {
                                            let is_selected_slot = selected_custom_idx == Some(idx);
                                            let is_current = *current_color == *swatch_col;
                                            let (s_rect, s_resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                                            if s_resp.clicked() {
                                                *current_color = *swatch_col;
                                                hsv = theme::rgb_to_hsv(*swatch_col);
                                                selected_custom_idx = Some(idx);
                                            }

                                            s_resp.context_menu(|ui| {
                                                if ui.button("Replace with Current Color").clicked() {
                                                    replace_custom_idx = Some(idx);
                                                    ui.close_menu();
                                                }
                                                if ui.button("Delete Swatch").clicked() {
                                                    delete_custom_idx = Some(idx);
                                                    ui.close_menu();
                                                }
                                            });

                                            if ui.is_rect_visible(s_rect) {
                                                let painter = ui.painter();
                                                let c = s_rect.center();
                                                if is_selected_slot || is_current {
                                                    painter.circle_stroke(c, 11.0, Stroke::new(2.0_f32, p.accent));
                                                    painter.circle_filled(c, 7.5, *swatch_col);
                                                } else {
                                                    let r = if s_resp.hovered() { 9.0 } else { 8.0 };
                                                    painter.circle_filled(c, r, *swatch_col);
                                                }
                                                painter.circle_stroke(c, if is_selected_slot || is_current { 7.5 } else { 8.0 }, Stroke::new(1.0_f32, p.border));
                                            }
                                            s_resp.on_hover_text(format!("Custom #{}: #{:02X}{:02X}{:02X} (Right-click to edit)", idx + 1, swatch_col.r(), swatch_col.g(), swatch_col.b()));

                                            if (idx + 1) % 4 == 0 {
                                                ui.end_row();
                                            }
                                        }
                                    });

                                if let Some(del_idx) = delete_custom_idx {
                                    if del_idx < custom_swatches.len() {
                                        custom_swatches.remove(del_idx);
                                        selected_custom_idx = None;
                                    }
                                }
                                if let Some(rep_idx) = replace_custom_idx {
                                    if rep_idx < custom_swatches.len() {
                                        custom_swatches[rep_idx] = *current_color;
                                    }
                                }
                            });

                            // Thin Vertical Divider
                            ui.add_space(2.0);
                            let (v_rect, _) = ui.allocate_exact_size(Vec2::new(7.0, 150.0), Sense::hover());
                            ui.painter().vline(v_rect.center().x, v_rect.y_range(), Stroke::new(1.0_f32, p.border));
                            ui.add_space(2.0);

                            // --- RIGHT COLUMN: 2D Spectrum Picker + Rainbow Hue Bar + Hex ---
                            ui.vertical(|ui| {
                                ui.set_width(165.0);

                                ui.label(RichText::new("Color Spectrum").font(theme::font_serif(11.0)).color(p.text_muted));

                                // 2D Saturation x Value canvas
                                let spectrum_w = 162.0;
                                let spectrum_h = 90.0;
                                let (sv_rect, sv_resp) = ui.allocate_exact_size(Vec2::new(spectrum_w, spectrum_h), Sense::click_and_drag());
                                if sv_resp.clicked() || sv_resp.dragged() {
                                    if let Some(mpos) = sv_resp.interact_pointer_pos() {
                                        let s = ((mpos.x - sv_rect.min.x) / sv_rect.width()).clamp(0.0, 1.0);
                                        let v = ((sv_rect.max.y - mpos.y) / sv_rect.height()).clamp(0.0, 1.0);
                                        hsv.1 = s;
                                        hsv.2 = v;
                                        *current_color = theme::hsv_to_rgb(hsv.0, hsv.1, hsv.2);
                                    }
                                }

                                if ui.is_rect_visible(sv_rect) {
                                    let painter = ui.painter();
                                    let mut mesh = egui::Mesh::default();
                                    const N: usize = 10;
                                    for xi in 0..=N {
                                        for yi in 0..=N {
                                            let xt = xi as f32 / N as f32;
                                            let yt = yi as f32 / N as f32;
                                            let col = theme::hsv_to_rgb(hsv.0, xt, yt);
                                            let x = sv_rect.min.x + xt * sv_rect.width();
                                            let y = sv_rect.max.y - yt * sv_rect.height();
                                            mesh.colored_vertex(Pos2::new(x, y), col);

                                            if xi < N && yi < N {
                                                let x_off = 1;
                                                let y_off = (N + 1) as u32;
                                                let tl = (yi * (N + 1) + xi) as u32;
                                                mesh.add_triangle(tl, tl + x_off, tl + y_off);
                                                mesh.add_triangle(tl + x_off, tl + y_off, tl + y_off + x_off);
                                            }
                                        }
                                    }
                                    painter.add(egui::Shape::mesh(mesh));
                                    painter.rect_stroke(sv_rect, Rounding::same(theme::RADIUS_SM), Stroke::new(1.0_f32, p.border));

                                    // Cursor circle on the 2D plane
                                    let cur_x = sv_rect.min.x + hsv.1 * sv_rect.width();
                                    let cur_y = sv_rect.max.y - hsv.2 * sv_rect.height();
                                    let cursor_p = Pos2::new(cur_x, cur_y);
                                    let contrast_col = if hsv.2 > 0.5 && hsv.1 < 0.6 { Color32::BLACK } else { Color32::WHITE };
                                    painter.circle_stroke(cursor_p, 4.0, Stroke::new(1.5_f32, contrast_col));
                                }

                                ui.add_space(2.0);

                                // 1D Rainbow Hue Bar
                                let (hue_rect, hue_resp) = ui.allocate_exact_size(Vec2::new(spectrum_w, 12.0), Sense::click_and_drag());
                                if hue_resp.clicked() || hue_resp.dragged() {
                                    if let Some(mpos) = hue_resp.interact_pointer_pos() {
                                        let h = ((mpos.x - hue_rect.min.x) / hue_rect.width()).clamp(0.0, 1.0);
                                        hsv.0 = h;
                                        *current_color = theme::hsv_to_rgb(hsv.0, hsv.1, hsv.2);
                                    }
                                }

                                if ui.is_rect_visible(hue_rect) {
                                    let painter = ui.painter();
                                    let mut mesh = egui::Mesh::default();
                                    const HUE_STEPS: usize = 24;
                                    for i in 0..=HUE_STEPS {
                                        let t = i as f32 / HUE_STEPS as f32;
                                        let col = theme::hsv_to_rgb(t, 1.0, 1.0);
                                        let x = hue_rect.min.x + t * hue_rect.width();
                                        mesh.colored_vertex(Pos2::new(x, hue_rect.min.y), col);
                                        mesh.colored_vertex(Pos2::new(x, hue_rect.max.y), col);

                                        if i < HUE_STEPS {
                                            let idx = (i * 2) as u32;
                                            mesh.add_triangle(idx, idx + 1, idx + 2);
                                            mesh.add_triangle(idx + 1, idx + 2, idx + 3);
                                        }
                                    }
                                    painter.add(egui::Shape::mesh(mesh));
                                    painter.rect_stroke(hue_rect, Rounding::same(theme::RADIUS_SM), Stroke::new(1.0_f32, p.border));

                                    // Indicator thumb line
                                    let thumb_x = hue_rect.min.x + hsv.0 * hue_rect.width();
                                    painter.line_segment([Pos2::new(thumb_x, hue_rect.min.y - 1.0), Pos2::new(thumb_x, hue_rect.max.y + 1.0)], Stroke::new(2.5_f32, Color32::WHITE));
                                    painter.line_segment([Pos2::new(thumb_x, hue_rect.min.y - 1.0), Pos2::new(thumb_x, hue_rect.max.y + 1.0)], Stroke::new(1.0_f32, Color32::BLACK));
                                }

                                ui.add_space(3.0);

                                // Live Swatch & Hex input + Update Button
                                ui.horizontal(|ui| {
                                    let (p_rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
                                    ui.painter().circle_filled(p_rect.center(), 8.5, *current_color);
                                    ui.painter().circle_stroke(p_rect.center(), 8.5, Stroke::new(1.0_f32, p.border));

                                    ui.label(RichText::new("Hex:").size(10.5).color(p.text_muted));
                                    let mut hex = format!("#{:02X}{:02X}{:02X}", current_color.r(), current_color.g(), current_color.b());
                                    let hex_resp = ui.add(
                                        egui::TextEdit::singleline(&mut hex)
                                            .desired_width(58.0)
                                            .font(egui::TextStyle::Monospace),
                                    );
                                    if hex_resp.lost_focus() || hex_resp.changed() {
                                        let trimmed = hex.trim().trim_start_matches('#');
                                        if let Ok(val) = u32::from_str_radix(trimmed, 16) {
                                            if trimmed.len() == 6 {
                                                let r = ((val >> 16) & 0xFF) as u8;
                                                let g = ((val >> 8) & 0xFF) as u8;
                                                let b = (val & 0xFF) as u8;
                                                *current_color = Color32::from_rgba_unmultiplied(r, g, b, current_color.a());
                                                hsv = theme::rgb_to_hsv(*current_color);
                                            }
                                        }
                                    }

                                    // Quick Update Swatch action if a custom slot is selected
                                    if let Some(sel_idx) = selected_custom_idx {
                                        if sel_idx < custom_swatches.len() {
                                            if ui.small_button("Update")
                                                .on_hover_text("Update selected custom swatch to current color")
                                                .clicked()
                                            {
                                                custom_swatches[sel_idx] = *current_color;
                                            }
                                        }
                                    }
                                });

                                ui.add_space(6.0);

                                // Dynamic Inkk mascot showcase card - reflects active ink color in real-time!
                                let card_size = Vec2::new(spectrum_w, 82.0);
                                let (card_rect, card_resp) = ui.allocate_exact_size(card_size, Sense::hover());
                                if ui.is_rect_visible(card_rect) {
                                    let painter = ui.painter();
                                    let bg = if card_resp.hovered() {
                                        p.hover
                                    } else {
                                        p.surface_alt
                                    };
                                    painter.rect_filled(card_rect, Rounding::same(theme::RADIUS_MD), bg);
                                    painter.rect_stroke(card_rect, Rounding::same(theme::RADIUS_MD), Stroke::new(1.0_f32, p.border));

                                    // Inkk mascot centered on the left, rendered big (50.4 x 58.8 px)
                                    let mascot_center = Pos2::new(card_rect.min.x + 36.0, card_rect.center().y);

                                    // Radiant ink ambient glow behind the mascot
                                    painter.circle_filled(
                                        mascot_center,
                                        30.0,
                                        current_color.gamma_multiply(if card_resp.hovered() { 0.22 } else { 0.12 }),
                                    );

                                    // Draw large Inkk mascot (pixel_size = 4.2 -> 50.4w x 58.8h)
                                    draw_squid_mascot_dynamic(
                                        painter,
                                        mascot_center,
                                        4.2,
                                        *current_color,
                                        1.0,
                                        p,
                                    );

                                    // English typography labels & dynamic Hex chip
                                    painter.text(
                                        Pos2::new(card_rect.min.x + 68.0, card_rect.center().y - 20.0),
                                        egui::Align2::LEFT_CENTER,
                                        "Inkk Mascot",
                                        theme::font_serif(12.5),
                                        p.text,
                                    );

                                    let hex_str = format!("#{:02X}{:02X}{:02X}", current_color.r(), current_color.g(), current_color.b());
                                    let chip_rect = Rect::from_min_size(
                                        Pos2::new(card_rect.min.x + 68.0, card_rect.center().y - 9.0),
                                        Vec2::new(84.0, 19.0),
                                    );
                                    painter.rect_filled(chip_rect, Rounding::same(theme::RADIUS_SM), p.surface);
                                    painter.rect_stroke(chip_rect, Rounding::same(theme::RADIUS_SM), Stroke::new(1.0_f32, p.border));

                                    painter.circle_filled(
                                        Pos2::new(chip_rect.min.x + 9.5, chip_rect.center().y),
                                        4.0,
                                        *current_color,
                                    );
                                    painter.circle_stroke(
                                        Pos2::new(chip_rect.min.x + 9.5, chip_rect.center().y),
                                        4.0,
                                        Stroke::new(0.5_f32, p.border),
                                    );
                                    painter.text(
                                        Pos2::new(chip_rect.min.x + 19.0, chip_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        &hex_str,
                                        egui::FontId::monospace(10.5),
                                        p.text,
                                    );

                                    painter.text(
                                        Pos2::new(card_rect.min.x + 68.0, card_rect.center().y + 19.0),
                                        egui::Align2::LEFT_CENTER,
                                        "Live Ink Tint",
                                        egui::FontId::proportional(10.0),
                                        p.text_muted,
                                    );
                                }
                                card_resp.on_hover_text("Inkk mascot — dynamically dips in your selected ink color!");
                            });
                        });
                    });
            });

        let popup_rect = area_resp.response.rect;

        // Auto-close if pointer interaction occurs outside the popup:
        // When the user starts writing on the canvas or clicks outside, the popup closes instantly.
        if ui.input(|i| i.pointer.any_pressed() || i.pointer.button_down(egui::PointerButton::Primary)) {
            if let Some(pos) = ui.input(|i| i.pointer.press_origin().or(i.pointer.interact_pos())) {
                if !popup_rect.contains(pos) && !rect.contains(pos) {
                    close_popup = true;
                }
            }
        }

        ui.data_mut(|d| {
            d.insert_temp(custom_sel_id, selected_custom_idx);
            d.insert_temp(hsv_id, hsv);
        });

        if close_popup || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.data_mut(|d| d.insert_temp(button_id, false));
        }
    }

    resp
}

/// Smart Ink Density / Opacity Control:
/// Replaces clunky raw sliders with tactile presets (100% Solid, 75% Wash, 50% Medium, 25% Faint)
/// each featuring an authentic ink alpha indicator dot, plus a sleek dropdown with a live gradient track.
pub fn smart_opacity_control(ui: &mut Ui, p: &Palette, opacity: &mut f32, ink_color: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;

        let presets: [(f32, &str, &str); 4] = [
            (1.0, "100%", "Solid Ink (100%)"),
            (0.75, "75%", "Wash / Shading (75%)"),
            (0.50, "50%", "Medium Translucent (50%)"),
            (0.25, "25%", "Faint Watermark (25%)"),
        ];

        let mut matched_preset = false;

        for (val, label, tip) in presets {
            let is_active = (*opacity - val).abs() < 0.04;
            if is_active {
                matched_preset = true;
            }

            let chip_w = if val == 1.0 { 40.0 } else { 36.0 };
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(chip_w, 22.0), Sense::click());
            if resp.clicked() {
                *opacity = val;
            }

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let bg = if is_active {
                    p.accent_soft
                } else if resp.hovered() {
                    p.hover
                } else {
                    p.surface
                };
                let stroke = if is_active {
                    Stroke::new(1.0_f32, p.accent)
                } else if resp.hovered() {
                    Stroke::new(1.0_f32, p.border)
                } else {
                    Stroke::new(0.5_f32, p.border)
                };

                painter.rect(rect, Rounding::same(theme::RADIUS_SM), bg, stroke);

                // Preview dot displaying the ink color at this preset's alpha
                let dot_center = Pos2::new(rect.min.x + 7.5, rect.center().y);
                let alpha_u8 = (val * 255.0) as u8;
                let dot_color = Color32::from_rgba_unmultiplied(
                    ink_color.r(),
                    ink_color.g(),
                    ink_color.b(),
                    alpha_u8,
                );
                painter.circle_filled(dot_center, 3.5, dot_color);
                painter.circle_stroke(dot_center, 3.5, Stroke::new(0.5_f32, p.border));

                // Label
                let text_color = if is_active { p.accent } else { p.text };
                painter.text(
                    Pos2::new(rect.min.x + 22.0, rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    text_color,
                );
            }
            resp.on_hover_text(tip);
        }

        // Custom fine-tune popover button (▾ or Custom %)
        let popup_id = egui::Id::new("smart_opacity_custom_popup");
        let is_popup_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));

        let custom_w = if matched_preset { 20.0 } else { 48.0 };
        let (c_rect, c_resp) = ui.allocate_exact_size(Vec2::new(custom_w, 22.0), Sense::click());
        if c_resp.clicked() {
            ui.data_mut(|d| d.insert_temp(popup_id, !is_popup_open));
        }

        let is_custom_active = !matched_preset || is_popup_open;
        if ui.is_rect_visible(c_rect) {
            let painter = ui.painter();
            let bg = if is_custom_active {
                p.accent_soft
            } else if c_resp.hovered() {
                p.hover
            } else {
                p.surface
            };
            let stroke = if is_custom_active {
                Stroke::new(1.0_f32, p.accent)
            } else {
                Stroke::new(0.5_f32, p.border)
            };
            painter.rect(c_rect, Rounding::same(theme::RADIUS_SM), bg, stroke);

            let txt_color = if is_custom_active {
                p.accent
            } else {
                p.text_muted
            };

            if matched_preset {
                // Vector down chevron - avoids font tofu / missing glyph box
                let c = c_rect.center();
                let triangle = vec![
                    Pos2::new(c.x - 3.5, c.y - 1.5),
                    Pos2::new(c.x + 3.5, c.y - 1.5),
                    Pos2::new(c.x, c.y + 2.5),
                ];
                painter.add(egui::Shape::convex_polygon(triangle, txt_color, Stroke::NONE));
            } else {
                let text = format!("{:.0}%", *opacity * 100.0);
                painter.text(
                    Pos2::new(c_rect.center().x - 4.0, c_rect.center().y),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    egui::FontId::proportional(11.0),
                    txt_color,
                );
                let c = Pos2::new(c_rect.max.x - 7.5, c_rect.center().y);
                let triangle = vec![
                    Pos2::new(c.x - 3.0, c.y - 1.5),
                    Pos2::new(c.x + 3.0, c.y - 1.5),
                    Pos2::new(c.x, c.y + 2.0),
                ];
                painter.add(egui::Shape::convex_polygon(triangle, txt_color, Stroke::NONE));
            }
        }
        c_resp.on_hover_text("Fine-tune custom ink opacity");

        // Custom popover with gradient ink slider & decorative Inkk mascot
        if is_popup_open {
            let mut close_popup = false;
            let popup_x = (c_rect.left_bottom().x - 110.0).max(12.0);
            let popup_pos = Pos2::new(popup_x, c_rect.left_bottom().y + 4.0);
            egui::Area::new(popup_id)
                .fixed_pos(popup_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(p.surface)
                        .stroke(Stroke::new(1.0_f32, p.border))
                        .rounding(Rounding::same(theme::RADIUS_MD))
                        .shadow(egui::Shadow {
                            offset: Vec2::new(0.0, 4.0),
                            blur: 16.0,
                            spread: 0.0,
                            color: p.shadow,
                        })
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.set_width(264.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Ink Dilution / Opacity")
                                        .font(theme::font_serif(12.5))
                                        .color(p.text)
                                        .strong(),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let close_btn = ui.add(
                                            egui::Button::new(
                                                RichText::new("×")
                                                    .size(14.0)
                                                    .color(p.text_muted),
                                            )
                                            .frame(false),
                                        );
                                        if close_btn.clicked() {
                                            close_popup = true;
                                        }
                                        close_btn.on_hover_text("Close");
                                    },
                                );
                            });

                            ui.add_space(8.0);

                            // Side-by-side content: Controls on left, Inkk mascot card on right
                            ui.horizontal(|ui| {
                                // Left column: slider + live percentage + presets
                                ui.vertical(|ui| {
                                    ui.set_width(172.0);

                                    // Live gradient track showing ink fade from 10% to 100%
                                    let (track_rect, track_resp) = ui.allocate_exact_size(
                                        Vec2::new(172.0, 16.0),
                                        Sense::click_and_drag(),
                                    );
                                    if ui.is_rect_visible(track_rect) {
                                        let painter = ui.painter();
                                        painter.rect_filled(track_rect, Rounding::same(8.0), p.surface_alt);

                                        let num_steps = 20;
                                        let step_w = track_rect.width() / num_steps as f32;
                                        for i in 0..num_steps {
                                            let t = i as f32 / (num_steps - 1) as f32;
                                            let step_alpha = (0.10 + 0.90 * t).clamp(0.0, 1.0);
                                            let step_color = Color32::from_rgba_unmultiplied(
                                                ink_color.r(),
                                                ink_color.g(),
                                                ink_color.b(),
                                                (step_alpha * 255.0) as u8,
                                            );
                                            let x0 = track_rect.min.x + i as f32 * step_w;
                                            let x1 = (x0 + step_w).min(track_rect.max.x);
                                            let r = Rect::from_min_max(
                                                Pos2::new(x0, track_rect.min.y),
                                                Pos2::new(x1, track_rect.max.y),
                                            );
                                            painter.rect_filled(r, Rounding::ZERO, step_color);
                                        }
                                        painter.rect_stroke(
                                            track_rect,
                                            Rounding::same(8.0),
                                            Stroke::new(1.0_f32, p.border),
                                        );

                                        // Thumb knob
                                        let t_val = ((*opacity - 0.10) / 0.90).clamp(0.0, 1.0);
                                        let thumb_x = track_rect.min.x + t_val * track_rect.width();
                                        let thumb_pos = Pos2::new(thumb_x, track_rect.center().y);
                                        painter.circle_filled(thumb_pos, 7.5, p.surface);
                                        painter.circle_stroke(
                                            thumb_pos,
                                            7.5,
                                            Stroke::new(2.0_f32, p.accent),
                                        );
                                        painter.circle_filled(thumb_pos, 4.0, ink_color);
                                    }

                                    if track_resp.clicked() || track_resp.dragged() {
                                        if let Some(mpos) = track_resp.interact_pointer_pos() {
                                            let t = ((mpos.x - track_rect.min.x) / track_rect.width())
                                                .clamp(0.0, 1.0);
                                            *opacity = ((0.10 + 0.90 * t) * 100.0).round() / 100.0;
                                        }
                                    }

                                    ui.add_space(5.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("{:.0}%", *opacity * 100.0))
                                                .font(egui::FontId::proportional(12.0))
                                                .strong()
                                                .color(p.text),
                                        );

                                        let label_desc = if *opacity >= 0.88 {
                                            "Solid"
                                        } else if *opacity >= 0.65 {
                                            "Medium"
                                        } else if *opacity >= 0.40 {
                                            "Diluted"
                                        } else {
                                            "Wash"
                                        };
                                        ui.label(
                                            RichText::new(label_desc)
                                                .font(egui::FontId::proportional(10.5))
                                                .color(p.text_muted),
                                        );
                                    });

                                    ui.add_space(4.0);
                                    // Quick preset chips
                                    ui.horizontal(|ui| {
                                        for &(val, lbl) in &[(0.25, "25%"), (0.50, "50%"), (0.75, "75%"), (1.00, "100%")] {
                                            let is_sel = (*opacity - val).abs() < 0.01;
                                            let bg = if is_sel { p.accent_soft } else { p.surface_alt };
                                            let txt_col = if is_sel { p.accent } else { p.text_muted };
                                            let btn = egui::Button::new(
                                                RichText::new(lbl).size(10.0).color(txt_col),
                                            )
                                            .fill(bg)
                                            .stroke(Stroke::new(0.5_f32, if is_sel { p.accent } else { p.border }))
                                            .rounding(Rounding::same(theme::RADIUS_SM));

                                            if ui.add(btn).clicked() {
                                                *opacity = val;
                                            }
                                        }
                                    });
                                });

                                ui.add_space(8.0);

                                // Right column: Inkk mascot showcase card + Done button
                                ui.vertical(|ui| {
                                    let badge_w = 64.0;
                                    let badge_h = 72.0;
                                    let (badge_rect, badge_resp) = ui.allocate_exact_size(
                                        Vec2::new(badge_w, badge_h),
                                        Sense::hover(),
                                    );
                                    if ui.is_rect_visible(badge_rect) {
                                        let painter = ui.painter();
                                        let bg_card = if badge_resp.hovered() {
                                            p.hover
                                        } else {
                                            p.surface_alt
                                        };
                                        painter.rect_filled(badge_rect, Rounding::same(theme::RADIUS_MD), bg_card);
                                        painter.rect_stroke(
                                            badge_rect,
                                            Rounding::same(theme::RADIUS_MD),
                                            Stroke::new(1.0_f32, p.border),
                                        );

                                        // Draw the Inkk squid mascot reacting dynamically to opacity!
                                        let mascot_center = Pos2::new(
                                            badge_rect.center().x,
                                            badge_rect.min.y + 28.0,
                                        );
                                        draw_squid_mascot_dynamic(
                                            painter,
                                            mascot_center,
                                            2.4,
                                            ink_color,
                                            *opacity,
                                            p,
                                        );

                                        // Mascot branding title
                                        painter.text(
                                            Pos2::new(badge_rect.center().x, badge_rect.max.y - 11.0),
                                            egui::Align2::CENTER_CENTER,
                                            "Inkk",
                                            theme::font_serif(11.0),
                                            p.text_muted,
                                        );
                                    }
                                    badge_resp.on_hover_text("Inkk mascot — dynamically shifts with ink dilution!");

                                    ui.add_space(4.0);
                                    let done_btn = egui::Button::new(
                                        RichText::new("Done").size(11.0).color(p.text),
                                    )
                                    .fill(p.surface_alt)
                                    .stroke(Stroke::new(0.5_f32, p.border))
                                    .rounding(Rounding::same(theme::RADIUS_SM));

                                    if ui.add_sized(Vec2::new(badge_w, 20.0), done_btn).clicked() {
                                        close_popup = true;
                                    }
                                });
                            });
                        });
                });

            if close_popup || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.data_mut(|d| d.insert_temp(popup_id, false));
            }
        }
    });
}

/// Filled call-to-action button.
#[allow(dead_code)]
pub fn primary_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).color(p.on_accent).strong())
        .fill(p.accent)
        .rounding(egui::Rounding::same(theme::RADIUS_SMALL));
    ui.add(btn)
}

/// Round color swatch with a selection ring.
#[allow(dead_code)]
pub fn color_swatch(
    ui: &mut Ui,
    p: &Palette,
    color: Color32,
    active: bool,
    name: &str,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(theme::SWATCH_SIZE), Sense::click());
    resp.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, active, name));

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let c = rect.center();
        if active {
            painter.circle_stroke(c, 11.0, Stroke::new(2.0_f32, p.accent));
            painter.circle_filled(c, 7.5, color);
        } else {
            let r = if resp.hovered() { 9.0 } else { 8.0 };
            painter.circle_filled(c, r, color);
        }
        // Thin outline keeps light swatches visible on light surfaces (and dark on dark).
        painter.circle_stroke(
            c,
            if active { 7.5 } else { 8.0 },
            Stroke::new(1.0_f32, p.border),
        );
    }
    resp.on_hover_text(name)
}

/// Thin vertical divider between toolbar groups.
pub fn divider(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(9.0, 22.0), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0_f32, p.border),
    );
}

/// Small muted caption text.
pub fn caption(ui: &mut Ui, p: &Palette, text: impl Into<String>) -> Response {
    ui.label(RichText::new(text.into()).size(12.0).color(p.text_muted))
}

/// Renders the signature Inkk pixel-art squid mascot for special moments (empty canvas, about, celebrations).
/// Faithfully follows the official Inkk SVG brand identity (12x14 pixel grid).
pub fn draw_squid_mascot(
    painter: &egui::Painter,
    center: Pos2,
    pixel_size: f32,
    body_color: Color32,
    eye_color: Color32,
    blush_color: Color32,
) {
    // Official 12x14 pixel grid from Inkk Brand SVG
    // 0: empty, 1: body, 2: eye white / mouth, 3: pupil, 4: blush
    const GRID: [[u8; 12]; 14] = [
        [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0], // row 0: tip
        [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0], // row 1
        [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0], // row 2
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // row 3: fins flare out
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0], // row 4: body
        [0, 0, 1, 2, 2, 1, 1, 2, 2, 1, 0, 0], // row 5: top of eyes
        [0, 0, 1, 2, 3, 1, 1, 3, 2, 1, 0, 0], // row 6: eyes (white + pupil)
        [0, 0, 1, 2, 3, 1, 1, 3, 2, 1, 0, 0], // row 7: eyes (white + pupil)
        [0, 0, 4, 1, 1, 2, 2, 1, 1, 4, 0, 0], // row 8: blush cheeks + smiling mouth
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0], // row 9: body
        [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0], // row 10: body base
        [0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0], // row 11: tentacle roots
        [0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0], // row 12: tentacles extending
        [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0], // row 13: tentacle tips spread
    ];

    let w = 12.0 * pixel_size;
    let h = 14.0 * pixel_size;
    let top_left = Pos2::new(center.x - w * 0.5, center.y - h * 0.5);

    let eye_white = Color32::from_rgb(245, 240, 230); // #F5F0E6

    for (r, row) in GRID.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            let col = match cell {
                1 => body_color,
                2 => eye_white,
                3 => eye_color,
                4 => blush_color,
                _ => continue,
            };
            let p = Pos2::new(
                top_left.x + c as f32 * pixel_size,
                top_left.y + r as f32 * pixel_size,
            );
            painter.rect_filled(Rect::from_min_size(p, Vec2::splat(pixel_size)), 0.5, col);
        }
    }
}

/// Renders the signature Inkk mascot squid dynamically responding to ink color & dilution opacity.
/// Directly implements the 12x14 pixel art from the official Inkk SVG specification.
pub fn draw_squid_mascot_dynamic(
    painter: &egui::Painter,
    center: Pos2,
    pixel_size: f32,
    ink_color: Color32,
    opacity: f32,
    _p: &Palette,
) {
    // Official 12x14 pixel grid directly mapped from Inkk Brand SVG:
    // 0: empty
    // 1: body (ink tinted with dilution opacity)
    // 2: eye white (cream #F5F0E6)
    // 3: pupil (dark ink #1F1E1B looking inward)
    // 4: blush (coral pink #E8826E)
    // 5: mouth (cream smile #F5F0E6)
    const GRID: [[u8; 12]; 14] = [
        [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0], // row 0: tip (x=5..6)
        [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0], // row 1: (x=4..7)
        [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0], // row 2: (x=3..8)
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0], // row 3: fins flare out (x=1..10)
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0], // row 4: body (x=2..9)
        [0, 0, 1, 2, 2, 1, 1, 2, 2, 1, 0, 0], // row 5: top of eyes (white)
        [0, 0, 1, 2, 3, 1, 1, 3, 2, 1, 0, 0], // row 6: eyes (white + pupil)
        [0, 0, 1, 2, 3, 1, 1, 3, 2, 1, 0, 0], // row 7: eyes (white + pupil)
        [0, 0, 4, 1, 1, 5, 5, 1, 1, 4, 0, 0], // row 8: blush cheeks + smiling mouth
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0], // row 9: lower body
        [0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0], // row 10: body base
        [0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0], // row 11: tentacle roots
        [0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0], // row 12: tentacles extending
        [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0], // row 13: tentacle tips spread
    ];

    let w = 12.0 * pixel_size;
    let h = 14.0 * pixel_size;
    let top_left = Pos2::new(center.x - w * 0.5, center.y - h * 0.5);

    // Official brand palette colors from SVG:
    let brand_dark = Color32::from_rgb(31, 30, 27);
    let brand_cream = Color32::from_rgb(245, 240, 230);
    let brand_blush = Color32::from_rgb(232, 130, 110);

    // Check if the current ink is light or dark
    let lum = 0.299 * ink_color.r() as f32 + 0.587 * ink_color.g() as f32 + 0.114 * ink_color.b() as f32;
    let is_light_ink = lum > 160.0;

    // Body color faithfully reflects ink_color, modulated by dilution opacity
    let alpha = (opacity.clamp(0.20, 1.0) * 255.0) as u8;
    let body_col = Color32::from_rgba_unmultiplied(
        ink_color.r(),
        ink_color.g(),
        ink_color.b(),
        alpha,
    );

    let (eye_white, pupil, mouth, blush) = if is_light_ink {
        // Light ink mode: shaded eye whites and dark smile so facial features remain crisp
        (Color32::from_rgb(220, 215, 205), brand_dark, brand_dark, brand_blush)
    } else {
        // Dark/rich ink mode: cream eye whites and smile with dark pupils
        (brand_cream, brand_dark, brand_cream, brand_blush)
    };

    for (r, row) in GRID.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            let col = match cell {
                1 => body_col,
                2 => eye_white,
                3 => pupil,
                4 => blush,
                5 => mouth,
                _ => continue,
            };
            let p_pos = Pos2::new(
                top_left.x + c as f32 * pixel_size,
                top_left.y + r as f32 * pixel_size,
            );
            let cell_rect = Rect::from_min_size(p_pos, Vec2::splat(pixel_size));
            painter.rect_filled(cell_rect, 0.5, col);

            // Subtle edge definition for very light ink bodies
            if is_light_ink && cell == 1 {
                painter.rect_stroke(
                    cell_rect,
                    0.5,
                    Stroke::new(0.5_f32, Color32::from_black_alpha(35)),
                );
            }
        }
    }
}
