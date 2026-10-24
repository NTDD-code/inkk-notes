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
                // Inkk signature warm Brick Red accent
                accent: Color32::from_rgb(212, 98, 82), // #D46252
                accent_soft: Color32::from_rgba_unmultiplied(212, 98, 82, 45),
                on_accent: Color32::from_rgb(255, 252, 248),
                desk: Color32::from_rgb(22, 20, 18), // Deep warm dark desk #161412
                surface: Color32::from_rgb(31, 28, 25), // Warm dark surface #1F1C19
                surface_alt: Color32::from_rgb(41, 38, 34), // Warm dark card/panel #292622
                border: Color32::from_rgb(59, 54, 47), // Warm dark border #3B362F
                text: Color32::from_rgb(237, 230, 220), // Warm ivory text #EDE6DC
                text_muted: Color32::from_rgb(158, 150, 137), // Warm muted text #9E9689
                hover: Color32::from_rgb(53, 49, 43), // Warm hover #35312B
                success: Color32::from_rgb(86, 175, 102),
                danger: Color32::from_rgb(225, 85, 75),
                shadow: Color32::from_black_alpha(140),
            }
        } else {
            Self {
                // Inkk signature warm Brick Red accent
                accent: Color32::from_rgb(185, 75, 60), // #B94B3C
                accent_soft: Color32::from_rgba_unmultiplied(185, 75, 60, 32),
                on_accent: Color32::WHITE,
                desk: Color32::from_rgb(235, 228, 216), // Warm parchment desk #EBE4D8
                surface: Color32::from_rgb(247, 244, 238), // Warm cream surface #F7F4EE
                surface_alt: Color32::from_rgb(238, 232, 220), // Warm cream-alt #EEE8DC
                border: Color32::from_rgb(223, 215, 203), // Warm subtle border #DFD7CB
                text: Color32::from_rgb(32, 30, 27),    // Ink black #201E1B
                text_muted: Color32::from_rgb(117, 110, 99), // Warm muted text #756E63
                hover: Color32::from_rgb(228, 221, 208), // Warm hover #E4DDD0
                success: Color32::from_rgb(46, 133, 64),
                danger: Color32::from_rgb(195, 45, 45),
                shadow: Color32::from_black_alpha(35),
            }
        }
    }
}

// Unified Design Tokens
// 1. Three corner rounding tiers
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 8.0;
#[allow(dead_code)]
pub const RADIUS_LG: f32 = 14.0;
pub const RADIUS: f32 = RADIUS_MD;
pub const RADIUS_SMALL: f32 = RADIUS_SM;

// 2. Spacing scale (4, 8, 12, 16, 24)
#[allow(dead_code)]
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
#[allow(dead_code)]
pub const SPACE_LG: f32 = 16.0;
#[allow(dead_code)]
pub const SPACE_XL: f32 = 24.0;

// Dimensions
pub const TOOL_BUTTON_SIZE: f32 = 32.0;
#[allow(dead_code)]
pub const SWATCH_SIZE: f32 = 22.0;
pub const TOAST_SECS: f32 = 3.0;

/// Curated Archival Inks for Inkk stationery.
pub const INK_SWATCHES: [(Color32, &str); 9] = [
    (Color32::from_rgb(31, 30, 27), "Sumi Ink"),   // #1F1E1B
    (Color32::from_rgb(185, 75, 60), "Brick Red"), // #B94B3C
    (Color32::from_rgb(43, 76, 111), "Prussian Blue"), // #2B4C6F
    (Color32::from_rgb(74, 59, 50), "Sepia"),      // #4A3B32
    (Color32::from_rgb(62, 92, 61), "Forest Moss"), // #3E5C3D
    (Color32::from_rgb(200, 138, 44), "Warm Amber"), // #C88A2C
    (Color32::from_rgb(94, 60, 88), "Plum"),       // #5E3C58
    (Color32::from_rgb(110, 115, 120), "Graphite"), // #6E7378
    (Color32::from_rgb(250, 247, 242), "Chalk White"), // #FAF7F2
];

pub const DEFAULT_INK_LIGHT: Color32 = Color32::from_rgb(31, 30, 27);
#[allow(dead_code)]
pub const DEFAULT_INK_DARK: Color32 = Color32::from_rgb(250, 247, 242);

pub fn ink_swatches(dark: bool) -> [(Color32, &'static str); 9] {
    if dark {
        [
            (Color32::from_rgb(250, 247, 242), "Chalk White"),
            (Color32::from_rgb(212, 98, 82), "Brick Red"),
            (Color32::from_rgb(110, 155, 205), "Soft Prussian"),
            (Color32::from_rgb(180, 155, 135), "Warm Sepia"),
            (Color32::from_rgb(120, 175, 120), "Moss Green"),
            (Color32::from_rgb(225, 175, 80), "Warm Amber"),
            (Color32::from_rgb(180, 130, 175), "Plum Mist"),
            (Color32::from_rgb(150, 155, 160), "Silver Grey"),
            (Color32::from_rgb(31, 30, 27), "Sumi Ink"),
        ]
    } else {
        INK_SWATCHES
    }
}

/// Setup typography: loads Georgia serif font when available on Windows with robust fallbacks
pub fn setup_fonts(ctx: &egui::Context) {
    static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INIT.get_or_init(|| {
        let mut fonts = egui::FontDefinitions::default();
        let prop_fonts: Vec<String> = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .cloned()
            .unwrap_or_default();
        let serif_family = fonts
            .families
            .entry(egui::FontFamily::Name("serif".into()))
            .or_default();
        if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\georgia.ttf") {
            fonts
                .font_data
                .insert("georgia".to_owned(), egui::FontData::from_owned(bytes));
            serif_family.insert(0, "georgia".to_owned());
        } else if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\times.ttf") {
            fonts
                .font_data
                .insert("times".to_owned(), egui::FontData::from_owned(bytes));
            serif_family.insert(0, "times".to_owned());
        }
        for f in prop_fonts {
            if !serif_family.contains(&f) {
                serif_family.push(f);
            }
        }
        ctx.set_fonts(fonts);
    });
}

/// Helper to request Georgia serif font for soulful headings, titles, and brand wordmark.
pub fn font_serif(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name("serif".into()))
}

/// Convert sRGB Color32 to HSV components (H: 0..=1, S: 0..=1, V: 0..=1).
pub fn rgb_to_hsv(c: Color32) -> (f32, f32, f32) {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max > 1e-5 { delta / max } else { 0.0 };

    let h = if delta <= 1e-5 {
        0.0
    } else if (max - r).abs() < 1e-5 {
        let h_raw = (g - b) / delta;
        h_raw.rem_euclid(6.0) / 6.0
    } else if (max - g).abs() < 1e-5 {
        (((b - r) / delta) + 2.0) / 6.0
    } else {
        (((r - g) / delta) + 4.0) / 6.0
    };

    (h.clamp(0.0, 1.0), s.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// Convert HSV components (H, S, V in 0..=1) to sRGB Color32.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let h_norm = h.rem_euclid(1.0) * 6.0;
    let i = h_norm.floor() as i32;
    let f = h_norm - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color32::from_rgb(
        (r * 255.0).clamp(0.0, 255.0).round() as u8,
        (g * 255.0).clamp(0.0, 255.0).round() as u8,
        (b * 255.0).clamp(0.0, 255.0).round() as u8,
    )
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
        PEN,
        HIGHLIGHTER,
        ERASER_STROKE,
        ERASER_POINT,
        LASSO,
        PAN,
        UNDO,
        REDO,
        SUN,
        MOON,
        PAGES,
        OPEN,
        SAVE,
        ADD,
        PREV,
        NEXT,
        EXTEND,
        FIT,
        DUPLICATE,
        DELETE,
        CLOSE,
        CHECK,
        WARN,
        NOTEBOOK,
        RULER,
        PROTRACTOR,
        PICTURE,
        "↗",
        "➡",
        "↔",
        "☐",
        "○",
        "⏶",
        "⬟",
        "⬣",
        "☆",
        "−",
        "+",
    ];
}

/// Builds egui visuals on top of the stock light/dark theme using the palette.
pub fn visuals(dark: bool) -> egui::Visuals {
    let p = Palette::get(dark);
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = p.surface;
    v.window_fill = p.surface;
    v.extreme_bg_color = p.surface_alt;
    v.faint_bg_color = p.surface_alt;
    v.window_stroke = Stroke::new(1.0_f32, p.border);
    v.window_rounding = Rounding::same(RADIUS + 2.0);
    v.menu_rounding = Rounding::same(RADIUS);
    v.window_shadow = egui::Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 18.0,
        spread: 0.0,
        color: p.shadow,
    };
    v.popup_shadow = egui::Shadow {
        offset: Vec2::new(0.0, 4.0),
        blur: 12.0,
        spread: 0.0,
        color: p.shadow,
    };
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
    setup_fonts(ctx);
    ctx.set_visuals(visuals(dark));
    ctx.style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(SPACE_SM, SPACE_SM);
        s.spacing.button_padding = Vec2::new(SPACE_MD, SPACE_SM);
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
            icon::ALL
                .iter()
                .copied()
                .filter(|s| !f.has_glyphs(&font, s))
                .collect()
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
        assert!(
            (h_orig - h_adapted).abs() < 2.0,
            "Hue must be preserved for blue"
        );
        assert!(
            l_adapted >= 0.65,
            "Lightness must be lifted for readability on dark paper"
        );

        // Red ink
        let red = [220, 38, 38, 255];
        let (h_red_orig, _, _) = rgb_to_hsl(red[0], red[1], red[2]);
        let red_dark = adapt_color_to_dark(red);
        let (h_red_adapted, _, l_red_adapted) = rgb_to_hsl(red_dark[0], red_dark[1], red_dark[2]);
        assert!(
            (h_red_orig - h_red_adapted).abs() < 2.0,
            "Hue must be preserved for red"
        );
        assert!(
            l_red_adapted >= 0.65,
            "Red must be lifted into vibrant coral/pink"
        );

        // Green ink
        let green = [22, 163, 74, 255];
        let (h_green_orig, _, _) = rgb_to_hsl(green[0], green[1], green[2]);
        let green_dark = adapt_color_to_dark(green);
        let (h_green_adapted, _, l_green_adapted) =
            rgb_to_hsl(green_dark[0], green_dark[1], green_dark[2]);
        assert!(
            (h_green_orig - h_green_adapted).abs() < 2.0,
            "Hue must be preserved for green"
        );
        assert!(
            l_green_adapted >= 0.65,
            "Green must be lifted into vibrant mint/emerald"
        );
    }

    #[test]
    fn test_rgb_hsv_roundtrip() {
        use super::{hsv_to_rgb, rgb_to_hsv};
        use eframe::egui::Color32;

        let colors = [
            Color32::from_rgb(255, 0, 0),     // Red
            Color32::from_rgb(0, 255, 0),     // Green
            Color32::from_rgb(0, 0, 255),     // Blue
            Color32::from_rgb(255, 255, 255), // White
            Color32::from_rgb(0, 0, 0),       // Black
            Color32::from_rgb(163, 35, 141),  // Purple (#A3238D)
            Color32::from_rgb(185, 75, 60),   // Brick Red
            Color32::from_rgb(43, 76, 111),   // Prussian Blue
        ];

        for &c in &colors {
            let (h, s, v) = rgb_to_hsv(c);
            let recon = hsv_to_rgb(h, s, v);
            let diff_r = (c.r() as i16 - recon.r() as i16).abs();
            let diff_g = (c.g() as i16 - recon.g() as i16).abs();
            let diff_b = (c.b() as i16 - recon.b() as i16).abs();
            assert!(
                diff_r <= 1 && diff_g <= 1 && diff_b <= 1,
                "Failed for {c:?}: got {recon:?}"
            );
        }
    }
}
