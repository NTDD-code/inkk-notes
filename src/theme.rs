//! Centralized UI theme: every color, radius and spacing used by the app chrome lives here.
use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};

/// Semantic color palette for one theme variant.
#[derive(Clone, Copy)]
pub struct Palette {
    pub accent: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub desk: Color32,
    pub surface: Color32,
    pub surface_alt: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub hover: Color32,
    pub success: Color32,
    pub danger: Color32,
    pub shadow: Color32,
}

impl Palette {
    pub fn get(dark: bool) -> Self {
        if dark {
            Self {
                accent: Color32::from_rgb(96, 165, 250),
                accent_soft: Color32::from_rgba_unmultiplied(96, 165, 250, 40),
                on_accent: Color32::from_rgb(15, 23, 42),
                desk: Color32::from_rgb(17, 20, 25),
                surface: Color32::from_rgb(28, 32, 39),
                surface_alt: Color32::from_rgb(38, 43, 52),
                border: Color32::from_rgb(52, 58, 68),
                text: Color32::from_rgb(226, 232, 240),
                text_muted: Color32::from_rgb(148, 158, 172),
                hover: Color32::from_rgb(45, 51, 61),
                success: Color32::from_rgb(74, 222, 128),
                danger: Color32::from_rgb(248, 113, 113),
                shadow: Color32::from_black_alpha(110),
            }
        } else {
            Self {
                accent: Color32::from_rgb(37, 99, 235),
                accent_soft: Color32::from_rgba_unmultiplied(37, 99, 235, 28),
                on_accent: Color32::WHITE,
                desk: Color32::from_rgb(236, 240, 245),
                surface: Color32::from_rgb(255, 255, 255),
                surface_alt: Color32::from_rgb(244, 246, 249),
                border: Color32::from_rgb(222, 227, 234),
                text: Color32::from_rgb(30, 41, 59),
                text_muted: Color32::from_rgb(100, 116, 139),
                hover: Color32::from_rgb(237, 241, 246),
                success: Color32::from_rgb(22, 163, 74),
                danger: Color32::from_rgb(220, 38, 38),
                shadow: Color32::from_black_alpha(30),
            }
        }
    }
}

pub const RADIUS: f32 = 8.0;
pub const RADIUS_SMALL: f32 = 6.0;
pub const TOOL_BUTTON_SIZE: f32 = 34.0;
pub const SWATCH_SIZE: f32 = 24.0;
pub const TOAST_SECS: f32 = 3.0;

/// Default ink colors offered as quick swatches.
pub const INK_SWATCHES: [(Color32, &str); 7] = [
    (Color32::from_rgb(15, 23, 42), "Black"),
    (Color32::from_rgb(220, 38, 38), "Red"),
    (Color32::from_rgb(37, 99, 235), "Blue"),
    (Color32::from_rgb(22, 163, 74), "Green"),
    (Color32::from_rgb(234, 179, 8), "Yellow"),
    (Color32::from_rgb(147, 51, 234), "Purple"),
    (Color32::from_rgb(248, 250, 252), "White"),
];
pub const DEFAULT_INK_LIGHT: Color32 = Color32::from_rgb(15, 23, 42);
pub const DEFAULT_INK_DARK: Color32 = Color32::from_rgb(248, 250, 252);

pub fn ink_swatches(dark: bool) -> [(Color32, &'static str); 7] {
    if dark {
        [
            (DEFAULT_INK_DARK, "White"),
            (Color32::from_rgb(248, 113, 113), "Red"),
            (Color32::from_rgb(96, 165, 250), "Blue"),
            (Color32::from_rgb(74, 222, 128), "Green"),
            (Color32::from_rgb(250, 204, 21), "Yellow"),
            (Color32::from_rgb(192, 132, 252), "Purple"),
            (Color32::from_rgb(15, 23, 42), "Black"),
        ]
    } else {
        INK_SWATCHES
    }
}

/// Convert RGB [0..255] to HSL (H: 0..360, S: 0..1, L: 0..1)
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;

    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let delta = max - min;
    let l = (max + min) * 0.5;

    if delta < 1e-5 {
        return (0.0, 0.0, l);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let mut h = if (max - rf).abs() < 1e-5 {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if (max - gf).abs() < 1e-5 {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    if h < 0.0 {
        h += 360.0;
    }

    (h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
}

/// Convert HSL (H: 0..360, S: 0..1, L: 0..1) to RGB [0..255]
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);
    let h = (h % 360.0 + 360.0) % 360.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = l - c * 0.5;

    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;

    (r, g, b)
}

/// Adapt a stroke color when entering Dark Mode:
/// - Dark/black monochromes -> crisp bright off-white / light silver.
/// - Saturated dark colors (Navy, Dark Red, Dark Green, Purple) -> vibrant pastel tones with identical Hue.
/// - Already bright colors (Yellow, Cyan, White) -> stay bright and high-contrast against dark paper.
pub fn adapt_color_to_dark(c: [u8; 4]) -> [u8; 4] {
    let [r, g, b, a] = c;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let (h, s, l) = rgb_to_hsl(r, g, b);

    // 1. Monochromes & near-neutrals (Black, dark slate, charcoal, dark gray)
    if delta <= 28 || s <= 0.14 {
        if l <= 0.45 {
            // Dark neutral -> Crisp off-white (248..252) down to clean silver (200)
            let v = (250.0 - (l / 0.45) * 45.0).round().clamp(200.0, 252.0) as u8;
            return [v, v, v, a];
        }
        return c;
    }

    // 2. Colored inks (Red, Blue, Green, Purple, Brown...)
    if l < 0.60 {
        // Lift lightness into [0.66, 0.80] range for optimal readability on dark paper
        let target_l = (0.75 - l * 0.15).clamp(0.66, 0.80);
        // Tune saturation slightly to avoid OLED/LCD chromatic glare while keeping vibrant
        let target_s = (s * 0.85 + 0.10).clamp(0.50, 0.95);
        let (nr, ng, nb) = hsl_to_rgb(h, target_s, target_l);
        return [nr, ng, nb, a];
    }

    // Colors that are already bright (L >= 0.60, like highlighter yellow) are kept intact
    c
}

/// Adapt a stroke color when returning to Light Mode (used when strokes were drawn in Dark Mode):
/// - Bright/white monochromes -> dark slate / black.
/// - Light pastels -> richer, deeper tones for high contrast against white paper.
pub fn adapt_color_to_light(c: [u8; 4]) -> [u8; 4] {
    let [r, g, b, a] = c;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let (h, s, l) = rgb_to_hsl(r, g, b);

    // 1. Monochromes & near-neutrals
    if delta <= 28 || s <= 0.14 {
        if l >= 0.65 {
            return [15, 23, 42, a]; // DEFAULT_INK_LIGHT
        }
        return c;
    }

    // 2. Colored inks drawn in dark mode (bright pastels)
    if l > 0.55 {
        let target_l = (0.35 + (l - 0.55) * 0.25).clamp(0.35, 0.48);
        let target_s = (s * 0.95 + 0.05).clamp(0.65, 0.98);
        let (nr, ng, nb) = hsl_to_rgb(h, target_s, target_l);
        return [nr, ng, nb, a];
    }

    c
}

pub fn adapt_color32_to_dark(c: Color32) -> Color32 {
    let [r, g, b, a] = adapt_color_to_dark([c.r(), c.g(), c.b(), c.a()]);
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn adapt_color32_to_light(c: Color32) -> Color32 {
    let [r, g, b, a] = adapt_color_to_light([c.r(), c.g(), c.b(), c.a()]);
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Toolbar icons. All must exist in egui's bundled fonts (checked by `tests::icons_have_glyphs`).
pub mod icon {
    pub const PEN: &str = "✏";
    pub const HIGHLIGHTER: &str = "🖊";
    pub const ERASER_STROKE: &str = "🗑";
    pub const ERASER_POINT: &str = "✂";
    pub const LASSO: &str = "⛶";
    pub const PAN: &str = "✋";
    pub const UNDO: &str = "⟲";
    pub const REDO: &str = "⟳";
    pub const SUN: &str = "☀";
    pub const MOON: &str = "🌙";
    pub const PAGES: &str = "📑";
    pub const OPEN: &str = "📂";
    pub const SAVE: &str = "💾";
    pub const ADD: &str = "➕";
    pub const PREV: &str = "◀";
    pub const NEXT: &str = "▶";
    pub const EXTEND: &str = "⬇";
    pub const FIT: &str = "⛶";
    pub const DUPLICATE: &str = "🗐";
    pub const DELETE: &str = "🗑";
    pub const CLOSE: &str = "✖";
    pub const CHECK: &str = "✔";
    pub const WARN: &str = "⚠";
    pub const NOTEBOOK: &str = "📓";
    pub const RULER: &str = "📏";
    pub const PROTRACTOR: &str = "📐";
    pub const PICTURE: &str = "🖼";

    #[cfg_attr(not(test), allow(dead_code))]
    pub const ALL: &[&str] = &[
        PEN, HIGHLIGHTER, ERASER_STROKE, ERASER_POINT, LASSO, PAN, UNDO, REDO, SUN, MOON, PAGES,
        OPEN, SAVE, ADD, PREV, NEXT, EXTEND, FIT, DUPLICATE, DELETE, CLOSE, CHECK, WARN,
        NOTEBOOK, RULER, PROTRACTOR, PICTURE, "↗", "➡", "↔", "☐", "○", "⏶", "⬟", "⬣", "☆", "−", "+",
    ];
}

/// Builds egui visuals on top of the stock light/dark theme using the palette.
pub fn visuals(dark: bool) -> egui::Visuals {
    let p = Palette::get(dark);
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };

    v.panel_fill = p.surface;
    v.window_fill = p.surface;
    v.extreme_bg_color = p.surface_alt;
    v.faint_bg_color = p.surface_alt;
    v.window_stroke = Stroke::new(1.0_f32, p.border);
    v.window_rounding = Rounding::same(RADIUS + 2.0);
    v.menu_rounding = Rounding::same(RADIUS);
    v.window_shadow = egui::Shadow { offset: Vec2::new(0.0, 6.0), blur: 18.0, spread: 0.0, color: p.shadow };
    v.popup_shadow = egui::Shadow { offset: Vec2::new(0.0, 4.0), blur: 12.0, spread: 0.0, color: p.shadow };
    v.selection.bg_fill = p.accent;
    v.selection.stroke = Stroke::new(1.0_f32, p.on_accent);
    v.hyperlink_color = p.accent;
    v.override_text_color = Some(p.text);

    let w = &mut v.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text);
    for (state, fill) in [
        (&mut w.inactive, p.surface_alt),
        (&mut w.hovered, p.hover),
        (&mut w.active, p.accent_soft),
        (&mut w.open, p.hover),
    ] {
        state.rounding = Rounding::same(RADIUS_SMALL);
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.expansion = 0.0;
    }
    w.inactive.bg_stroke = Stroke::NONE;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, p.border);
    w.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    w.open.bg_stroke = Stroke::new(1.0_f32, p.border);
    v
}

/// Applies visuals plus spacing tweaks to the context.
pub fn apply(ctx: &egui::Context, dark: bool) {
    ctx.set_visuals(visuals(dark));
    ctx.style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(6.0, 6.0);
        s.spacing.button_padding = Vec2::new(10.0, 5.0);
        s.spacing.interact_size.y = 28.0;
        s.spacing.slider_width = 110.0;
    });
}

#[cfg(test)]
mod tests {
    use super::icon;
    use eframe::egui;

    #[test]
    fn icons_have_glyphs() {
        let ctx = egui::Context::default();
        // Fonts are only loaded after the first frame.
        let _ = ctx.run(Default::default(), |_| {});
        let font = egui::FontId::proportional(16.0);
        let missing: Vec<&str> = ctx.fonts(|f| {
            icon::ALL.iter().copied().filter(|s| !f.has_glyphs(&font, s)).collect()
        });
        assert!(missing.is_empty(), "missing glyphs: {missing:?}");
    }

    #[test]
    fn test_dark_mode_black_to_white() {
        use super::{adapt_color_to_dark, adapt_color_to_light};
        // Pure black -> crisp bright off-white
        let black = [0, 0, 0, 255];
        let dark_adapted = adapt_color_to_dark(black);
        assert!(dark_adapted[0] >= 240 && dark_adapted[1] >= 240 && dark_adapted[2] >= 240);

        // Slate black (theme default ink) -> crisp bright off-white
        let slate_black = [15, 23, 42, 255];
        let slate_adapted = adapt_color_to_dark(slate_black);
        assert!(slate_adapted[0] >= 235 && slate_adapted[1] >= 235 && slate_adapted[2] >= 235);

        // White drawn in dark mode -> converts to dark/black in light mode
        let white = [248, 250, 252, 255];
        let light_adapted = adapt_color_to_light(white);
        assert!(light_adapted[0] <= 30 && light_adapted[1] <= 30 && light_adapted[2] <= 45);
    }

    #[test]
    fn test_dark_mode_colored_inks_preserve_hue_and_brighten() {
        use super::{adapt_color_to_dark, rgb_to_hsl};
        // Blue ink
        let blue = [37, 99, 235, 255];
        let (h_orig, _, _) = rgb_to_hsl(blue[0], blue[1], blue[2]);
        let blue_dark = adapt_color_to_dark(blue);
        let (h_adapted, _, l_adapted) = rgb_to_hsl(blue_dark[0], blue_dark[1], blue_dark[2]);
        assert!((h_orig - h_adapted).abs() < 2.0, "Hue must be preserved for blue");
        assert!(l_adapted >= 0.65, "Lightness must be lifted for readability on dark paper");

        // Red ink
        let red = [220, 38, 38, 255];
        let (h_red_orig, _, _) = rgb_to_hsl(red[0], red[1], red[2]);
        let red_dark = adapt_color_to_dark(red);
        let (h_red_adapted, _, l_red_adapted) = rgb_to_hsl(red_dark[0], red_dark[1], red_dark[2]);
        assert!((h_red_orig - h_red_adapted).abs() < 2.0, "Hue must be preserved for red");
        assert!(l_red_adapted >= 0.65, "Red must be lifted into vibrant coral/pink");

        // Green ink
        let green = [22, 163, 74, 255];
        let (h_green_orig, _, _) = rgb_to_hsl(green[0], green[1], green[2]);
        let green_dark = adapt_color_to_dark(green);
        let (h_green_adapted, _, l_green_adapted) = rgb_to_hsl(green_dark[0], green_dark[1], green_dark[2]);
        assert!((h_green_orig - h_green_adapted).abs() < 2.0, "Hue must be preserved for green");
        assert!(l_green_adapted >= 0.65, "Green must be lifted into vibrant mint/emerald");
    }
}
