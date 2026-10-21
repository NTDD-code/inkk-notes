//! Small reusable widgets for the app chrome, styled from `theme`.
use eframe::egui::{self, Color32, Response, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

use crate::theme::{self, Palette};

/// Square icon button with an active state, used for tools and quick actions.
pub fn tool_button(ui: &mut Ui, p: &Palette, icon: &str, active: bool, tooltip: &str) -> Response {
    let size = Vec2::splat(theme::TOOL_BUTTON_SIZE);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    resp.widget_info(|| WidgetInfo::selected(WidgetType::SelectableLabel, ui.is_enabled(), active, tooltip));

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke, fg) = if active {
            (p.accent_soft, Stroke::new(1.0_f32, p.accent), p.accent)
        } else if resp.hovered() {
            (p.hover, Stroke::NONE, p.text)
        } else {
            (Color32::TRANSPARENT, Stroke::NONE, p.text_muted)
        };
        let fg = if ui.is_enabled() { fg } else { p.text_muted.gamma_multiply(0.45) };
        painter.rect(rect, theme::RADIUS, fill, stroke);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(17.0),
            fg,
        );
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
        .stroke(if active { Stroke::new(1.0_f32, p.accent) } else { Stroke::NONE })
        .rounding(egui::Rounding::same(14.0));
    ui.add(btn)
}

/// Filled call-to-action button.
pub fn primary_button(ui: &mut Ui, p: &Palette, label: &str) -> Response {
    let btn = egui::Button::new(RichText::new(label).color(p.on_accent).strong())
        .fill(p.accent)
        .rounding(egui::Rounding::same(theme::RADIUS_SMALL));
    ui.add(btn)
}

/// Round color swatch with a selection ring.
pub fn color_swatch(ui: &mut Ui, p: &Palette, color: Color32, active: bool, name: &str) -> Response {
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
        painter.circle_stroke(c, if active { 7.5 } else { 8.0 }, Stroke::new(1.0_f32, p.border));
    }
    resp.on_hover_text(name)
}

/// Thin vertical divider between toolbar groups.
pub fn divider(ui: &mut Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(9.0, 22.0), Sense::hover());
    ui.painter().vline(rect.center().x, rect.y_range(), Stroke::new(1.0_f32, p.border));
}

/// Small muted caption text.
pub fn caption(ui: &mut Ui, p: &Palette, text: impl Into<String>) -> Response {
    ui.label(RichText::new(text.into()).size(12.0).color(p.text_muted))
}
