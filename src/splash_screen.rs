use eframe::egui::{self, Color32, FontId, Pos2, Rect, Rounding, Stroke, Vec2};
use std::path::PathBuf;

/// Color tokens strictly adhering to Inkk's minimal, warm design palette:
/// Dark bg: #1F1E1B | Cream text: #F5F0E6 | Brick Red accent: #B5412E | Border: #3A3732
/// Absolutely NO blue colors.
pub mod splash_colors {
    use eframe::egui::Color32;

    pub const BG_DARK: Color32 = Color32::from_rgb(0x1F, 0x1E, 0x1B);
    pub const CARD_SURFACE: Color32 = Color32::from_rgb(0x23, 0x21, 0x1D);
    pub const HOVER_SURFACE: Color32 = Color32::from_rgb(0x32, 0x2E, 0x27);
    pub const TEXT_CREAM: Color32 = Color32::from_rgb(0xF5, 0xF0, 0xE6);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0xA8, 0xA0, 0x92);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x75, 0x70, 0x66);
    pub const ACCENT_BRICK: Color32 = Color32::from_rgb(0xB5, 0x41, 0x2E);
    pub const BORDER: Color32 = Color32::from_rgb(0x3A, 0x37, 0x32);
    pub const BORDER_BRIGHT: Color32 = Color32::from_rgb(0x54, 0x4F, 0x46);
}

/// Actions emitted by the minimalist Inkk splash dialog
#[derive(Debug, Clone)]
pub enum SplashAction {
    NewBlank,
    NewRuled,
    NewGrid,
    NewCornell,
    OpenMathStudio,
    OpenFlowchart,
    OpenRecent(PathBuf),
    OpenLibrary,
    RecoverLastSession,
    OpenFilePicker,
    OpenKeyboardShortcuts,
    StartOnboardingTour,
}

/// Metadata summary passed from the app's notebook library
#[derive(Debug, Clone)]
pub struct SplashRecentItem {
    pub path: PathBuf,
    pub title: String,
    pub modified_str: String,
}

/// Minimalist Splash Dialog State
pub struct SplashScreenState {
    pub is_active: bool,
    pub texture: Option<egui::TextureHandle>,
    pub show_more_templates: bool,
}

impl Default for SplashScreenState {
    fn default() -> Self {
        Self {
            is_active: true,
            texture: None,
            show_more_templates: false,
        }
    }
}

impl SplashScreenState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve raw image bytes for bia.png with robust fallbacks:
    /// 1. Project assets/bia.png
    /// 2. User Downloads folder ($USERPROFILE/Downloads/bia.png or $HOME/Downloads/bia.png)
    /// 3. Embedded fallback binary
    pub fn load_image_bytes() -> Vec<u8> {
        if let Ok(bytes) = std::fs::read("assets/bia.png") {
            return bytes;
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let path = std::path::PathBuf::from(profile).join("Downloads").join("bia.png");
            if let Ok(bytes) = std::fs::read(&path) {
                return bytes;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let path = std::path::PathBuf::from(home).join("Downloads").join("bia.png");
            if let Ok(bytes) = std::fs::read(&path) {
                return bytes;
            }
        }
        include_bytes!("../assets/bia.png").to_vec()
    }

    /// Ensure the texture handle is loaded into egui Context
    fn ensure_texture(&mut self, ctx: &egui::Context) {
        if self.texture.is_none() {
            let bytes = Self::load_image_bytes();
            if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                let rgba = dyn_img.to_rgba8();
                let color_img = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    &rgba.into_raw(),
                );
                self.texture = Some(ctx.load_texture("splash_cover_art", color_img, egui::TextureOptions::LINEAR));
            }
        }
    }

    /// Re-trigger the splash dialog
    pub fn trigger(&mut self) {
        self.is_active = true;
        self.show_more_templates = false;
    }

    /// Dismiss the splash dialog
    pub fn dismiss(&mut self) {
        self.is_active = false;
    }

    /// Renders the minimalist splash dialog floating in the center of the whiteboard.
    /// Closes automatically if the user clicks/touches outside on the canvas or presses Esc.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        recent_items: &[SplashRecentItem],
        has_recoverable_session: bool,
    ) -> Option<SplashAction> {
        if !self.is_active {
            return None;
        }

        self.ensure_texture(ctx);

        let screen_rect = ctx.screen_rect();
        let mut triggered_action = None;

        // Pressing Escape dismisses the splash dialog
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.dismiss();
            return None;
        }

        // Minimalist Splash Card Dimensions
        let card_w = 600.0f32.min(screen_rect.width() - 32.0);
        let img_h = (card_w * 0.44).clamp(190.0, 250.0);
        let content_h = 285.0f32;
        let total_card_h = img_h + content_h;

        let card_x = screen_rect.center().x - card_w * 0.5;
        let card_y = (screen_rect.center().y - total_card_h * 0.5).max(35.0);
        let card_rect = Rect::from_min_size(Pos2::new(card_x, card_y), Vec2::new(card_w, total_card_h));

        // Touch/click outside on the whiteboard canvas -> dismiss dialog automatically
        if ctx.input(|i| i.pointer.primary_clicked() || i.pointer.primary_down()) {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos())) {
                if !card_rect.contains(pos) {
                    self.dismiss();
                    return None;
                }
            }
        }

        // Render as Foreground overlay area
        egui::Area::new(egui::Id::new("inkk_minimal_splash_dialog"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let painter = ui.painter();

                // Subtle warm background darkening so canvas remains gently visible
                let dim_color = Color32::from_rgba_unmultiplied(15, 14, 12, 120);
                painter.rect_filled(screen_rect, Rounding::ZERO, dim_color);

                // Warm ambient card shadow
                for s in 1..=4 {
                    let grow = s as f32 * 4.0;
                    let shadow_rect = card_rect.expand(grow);
                    let shadow_col = Color32::from_rgba_unmultiplied(10, 9, 8, (28 / s) as u8);
                    painter.rect_filled(shadow_rect, Rounding::same(8.0 + grow * 0.5), shadow_col);
                }

                // Card background and border (strictly #1F1E1B and #3A3732)
                let card_bg = splash_colors::BG_DARK;
                let card_border = splash_colors::BORDER;
                let card_rounding = Rounding::same(8.0);
                painter.rect_filled(card_rect, card_rounding, card_bg);

                // ----------------------------------------------------
                // 1. TOP COVER IMAGE SECTION
                // ----------------------------------------------------
                let img_rect = Rect::from_min_size(card_rect.min, Vec2::new(card_w, img_h));
                if let Some(tex) = &self.texture {
                    painter.image(
                        tex.id(),
                        img_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    painter.rect_filled(
                        img_rect,
                        Rounding { nw: 8.0, ne: 8.0, sw: 0.0, se: 0.0 },
                        splash_colors::CARD_SURFACE,
                    );
                }

                // Version text in top-right: e.g. "0.1.2" (clean and minimal)
                let ver_x = card_rect.max.x - 18.0;
                let ver_y = card_rect.min.y + 14.0;
                painter.text(
                    Pos2::new(ver_x, ver_y),
                    egui::Align2::RIGHT_TOP,
                    env!("CARGO_PKG_VERSION"),
                    FontId::proportional(12.0),
                    splash_colors::TEXT_MUTED,
                );

                // Artist attribution in bottom-right over image: "Artwork: ntdd"
                painter.text(
                    Pos2::new(card_rect.max.x - 18.0, card_rect.min.y + img_h - 14.0),
                    egui::Align2::RIGHT_BOTTOM,
                    "Artwork: ntdd",
                    FontId::proportional(11.0),
                    splash_colors::TEXT_MUTED,
                );

                // ----------------------------------------------------
                // 2. BOTTOM CONTENT SECTION (Two-Column Layout)
                // ----------------------------------------------------
                let body_top = card_rect.min.y + img_h;
                let body_rect = Rect::from_min_size(
                    Pos2::new(card_rect.min.x, body_top),
                    Vec2::new(card_w, content_h),
                );
                painter.rect_filled(
                    body_rect,
                    Rounding { nw: 0.0, ne: 0.0, sw: 8.0, se: 8.0 },
                    splash_colors::BG_DARK,
                );

                let pad_x = 28.0;
                let col_w = (card_w - pad_x * 3.0) * 0.5;
                let col1_x = card_rect.min.x + pad_x;
                let col2_x = col1_x + col_w + pad_x;
                let col_top = body_top + 18.0;

                // Section Headers: Serif typography as requested (#F5F0E6)
                let header_font = crate::theme::font_serif(16.0);
                painter.text(
                    Pos2::new(col1_x, col_top),
                    egui::Align2::LEFT_TOP,
                    "New File",
                    header_font.clone(),
                    splash_colors::TEXT_CREAM,
                );
                painter.text(
                    Pos2::new(col2_x, col_top),
                    egui::Align2::LEFT_TOP,
                    "Recent Files",
                    header_font,
                    splash_colors::TEXT_CREAM,
                );

                let row_h = 27.0;
                let item_start_y = col_top + 28.0;

                // --- LEFT COLUMN: New File Templates ---
                // Only keep Blank, Ruled, Grid; others under "More templates..."
                if !self.show_more_templates {
                    let main_templates = [
                        ("Blank Canvas", 0),
                        ("Ruled Paper", 1),
                        ("Grid Paper", 2),
                        ("More templates...", 3),
                    ];

                    for (idx, (label, type_idx)) in main_templates.iter().enumerate() {
                        let y = item_start_y + idx as f32 * row_h;
                        let item_rect = Rect::from_min_size(Pos2::new(col1_x, y), Vec2::new(col_w, row_h));
                        let hovered = ctx.input(|i| {
                            i.pointer.hover_pos().map_or(false, |p| item_rect.contains(p))
                        });

                        if hovered {
                            painter.rect_filled(item_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                            painter.rect_stroke(item_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::BORDER_BRIGHT));
                        }

                        let icon_center = Pos2::new(col1_x + 12.0, y + row_h * 0.5);
                        let icon_col = if hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED };
                        draw_splash_vector_icon(&painter, *type_idx, icon_center, icon_col);

                        let text_col = if hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED };
                        painter.text(
                            Pos2::new(col1_x + 28.0, y + row_h * 0.5),
                            egui::Align2::LEFT_CENTER,
                            *label,
                            FontId::proportional(12.5),
                            text_col,
                        );

                        if hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                            match type_idx {
                                0 => {
                                    self.dismiss();
                                    triggered_action = Some(SplashAction::NewBlank);
                                }
                                1 => {
                                    self.dismiss();
                                    triggered_action = Some(SplashAction::NewRuled);
                                }
                                2 => {
                                    self.dismiss();
                                    triggered_action = Some(SplashAction::NewGrid);
                                }
                                3 => {
                                    // Expand more templates
                                    self.show_more_templates = true;
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    // Expanded "More templates..." view (Cornell, Math Studio, Flowchart)
                    let more_templates = [
                        ("Cornell Notes", 4),
                        ("Math Studio", 5),
                        ("Flowchart Studio", 6),
                        ("← Back to templates", 7),
                    ];

                    for (idx, (label, type_idx)) in more_templates.iter().enumerate() {
                        let y = item_start_y + idx as f32 * row_h;
                        let item_rect = Rect::from_min_size(Pos2::new(col1_x, y), Vec2::new(col_w, row_h));
                        let hovered = ctx.input(|i| {
                            i.pointer.hover_pos().map_or(false, |p| item_rect.contains(p))
                        });

                        if hovered {
                            painter.rect_filled(item_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                            painter.rect_stroke(item_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::BORDER_BRIGHT));
                        }

                        let icon_center = Pos2::new(col1_x + 12.0, y + row_h * 0.5);
                        let icon_col = if hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED };
                        draw_splash_vector_icon(&painter, *type_idx, icon_center, icon_col);

                        let text_col = if hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED };
                        painter.text(
                            Pos2::new(col1_x + 28.0, y + row_h * 0.5),
                            egui::Align2::LEFT_CENTER,
                            *label,
                            FontId::proportional(12.5),
                            text_col,
                        );

                        if hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                            match type_idx {
                                4 => {
                                    self.dismiss();
                                    triggered_action = Some(SplashAction::NewCornell);
                                }
                                5 => {
                                    self.dismiss();
                                    triggered_action = Some(SplashAction::OpenMathStudio);
                                }
                                6 => {
                                    self.dismiss();
                                    triggered_action = Some(SplashAction::OpenFlowchart);
                                }
                                7 => {
                                    self.show_more_templates = false;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // --- RIGHT COLUMN: Recent Files ---
                // Only show real files from disk. If empty, show faint "No recent files".
                if recent_items.is_empty() {
                    let y = item_start_y + 4.0;
                    painter.text(
                        Pos2::new(col2_x + 2.0, y + row_h * 0.5),
                        egui::Align2::LEFT_CENTER,
                        "No recent files",
                        FontId::proportional(12.0),
                        splash_colors::TEXT_FAINT,
                    );
                } else {
                    for (idx, item) in recent_items.iter().take(4).enumerate() {
                        let y = item_start_y + idx as f32 * row_h;
                        let item_rect = Rect::from_min_size(Pos2::new(col2_x, y), Vec2::new(col_w, row_h));
                        let hovered = ctx.input(|i| {
                            i.pointer.hover_pos().map_or(false, |p| item_rect.contains(p))
                        });

                        if hovered {
                            painter.rect_filled(item_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                            painter.rect_stroke(item_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::BORDER_BRIGHT));
                        }

                        let icon_center = Pos2::new(col2_x + 12.0, y + row_h * 0.5);
                        let icon_col = if hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED };
                        draw_splash_vector_icon(&painter, 8, icon_center, icon_col);

                        let text_col = if hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED };
                        painter.text(
                            Pos2::new(col2_x + 28.0, y + row_h * 0.5),
                            egui::Align2::LEFT_CENTER,
                            &item.title,
                            FontId::proportional(12.5),
                            text_col,
                        );

                        // Modified time string on right (e.g. "2h ago")
                        painter.text(
                            Pos2::new(item_rect.max.x - 6.0, y + row_h * 0.5),
                            egui::Align2::RIGHT_CENTER,
                            &item.modified_str,
                            FontId::proportional(11.0),
                            splash_colors::TEXT_FAINT,
                        );

                        if hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                            self.dismiss();
                            triggered_action = Some(SplashAction::OpenRecent(item.path.clone()));
                        }
                    }

                    // "More..." under recent files to browse all files
                    let more_y = item_start_y + 4.0 * row_h;
                    let more_rect = Rect::from_min_size(Pos2::new(col2_x, more_y), Vec2::new(col_w, row_h));
                    let more_hovered = ctx.input(|i| {
                        i.pointer.hover_pos().map_or(false, |p| more_rect.contains(p))
                    });
                    if more_hovered {
                        painter.rect_filled(more_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                    }
                    let more_icon_center = Pos2::new(col2_x + 12.0, more_y + row_h * 0.5);
                    draw_splash_vector_icon(&painter, 9, more_icon_center, splash_colors::TEXT_MUTED);
                    painter.text(
                        Pos2::new(col2_x + 28.0, more_y + row_h * 0.5),
                        egui::Align2::LEFT_CENTER,
                        "More...",
                        FontId::proportional(12.0),
                        if more_hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED },
                    );
                    if more_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                        self.dismiss();
                        triggered_action = Some(SplashAction::OpenLibrary);
                    }
                }

                // ----------------------------------------------------
                // 3. FOOTER ROW (Divider + Open... / Recover / Shortcuts)
                // ----------------------------------------------------
                let divider_y = card_rect.max.y - 58.0;
                painter.line_segment(
                    [Pos2::new(card_rect.min.x + pad_x, divider_y), Pos2::new(card_rect.max.x - pad_x, divider_y)],
                    Stroke::new(1.0_f32, splash_colors::BORDER),
                );

                let footer_y = divider_y + 14.0;
                let btn_h = 28.0;

                // Footer Left 1: "Open..." button
                let open_w = 95.0;
                let open_rect = Rect::from_min_size(Pos2::new(col1_x, footer_y), Vec2::new(open_w, btn_h));
                let open_hovered = ctx.input(|i| i.pointer.hover_pos().map_or(false, |p| open_rect.contains(p)));
                if open_hovered {
                    painter.rect_filled(open_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                    painter.rect_stroke(open_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::BORDER_BRIGHT));
                }
                draw_splash_vector_icon(&painter, 10, Pos2::new(col1_x + 12.0, footer_y + btn_h * 0.5), splash_colors::TEXT_MUTED);
                painter.text(
                    Pos2::new(col1_x + 28.0, footer_y + btn_h * 0.5),
                    egui::Align2::LEFT_CENTER,
                    "Open...",
                    FontId::proportional(12.5),
                    if open_hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED },
                );
                if open_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                    self.dismiss();
                    triggered_action = Some(SplashAction::OpenFilePicker);
                }

                // Footer Left 2: "Recover Last Session" (ONLY shown when there is genuinely an uncommitted session!)
                if has_recoverable_session {
                    let rec_x = col1_x + open_w + 12.0;
                    let rec_w = 160.0;
                    let rec_rect = Rect::from_min_size(Pos2::new(rec_x, footer_y), Vec2::new(rec_w, btn_h));
                    let rec_hovered = ctx.input(|i| i.pointer.hover_pos().map_or(false, |p| rec_rect.contains(p)));
                    if rec_hovered {
                        painter.rect_filled(rec_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                        painter.rect_stroke(rec_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::ACCENT_BRICK));
                    }
                    draw_splash_vector_icon(&painter, 11, Pos2::new(rec_x + 12.0, footer_y + btn_h * 0.5), splash_colors::ACCENT_BRICK);
                    painter.text(
                        Pos2::new(rec_x + 28.0, footer_y + btn_h * 0.5),
                        egui::Align2::LEFT_CENTER,
                        "Recover Last Session",
                        FontId::proportional(12.0),
                        if rec_hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED },
                    );
                    if rec_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                        self.dismiss();
                        triggered_action = Some(SplashAction::RecoverLastSession);
                    }
                }

                // Footer Right 1: "Quick Tour" (Interactive beginner's walkthrough)
                let help_w = 160.0;
                let help_x = card_rect.max.x - pad_x - help_w;
                let tour_w = 115.0;
                let tour_x = help_x - tour_w - 8.0;

                let tour_rect = Rect::from_min_size(Pos2::new(tour_x, footer_y), Vec2::new(tour_w, btn_h));
                let tour_hovered = ctx.input(|i| i.pointer.hover_pos().map_or(false, |p| tour_rect.contains(p)));

                if tour_hovered {
                    painter.rect_filled(tour_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                    painter.rect_stroke(tour_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::ACCENT_BRICK));
                } else {
                    painter.rect_filled(tour_rect, Rounding::same(5.0), splash_colors::CARD_SURFACE);
                    painter.rect_stroke(tour_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::BORDER));
                }

                draw_splash_vector_icon(
                    &painter,
                    13,
                    Pos2::new(tour_x + 13.0, footer_y + btn_h * 0.5),
                    if tour_hovered { splash_colors::ACCENT_BRICK } else { splash_colors::TEXT_MUTED },
                );
                painter.text(
                    Pos2::new(tour_x + 28.0, footer_y + btn_h * 0.5),
                    egui::Align2::LEFT_CENTER,
                    "Quick Tour",
                    FontId::proportional(12.5),
                    if tour_hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED },
                );
                if tour_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                    self.dismiss();
                    triggered_action = Some(SplashAction::StartOnboardingTour);
                }

                // Footer Right 2: Prominent "Keyboard Shortcuts" with key icon
                let help_rect = Rect::from_min_size(Pos2::new(help_x, footer_y), Vec2::new(help_w, btn_h));
                let help_hovered = ctx.input(|i| i.pointer.hover_pos().map_or(false, |p| help_rect.contains(p)));

                if help_hovered {
                    painter.rect_filled(help_rect, Rounding::same(5.0), splash_colors::HOVER_SURFACE);
                    painter.rect_stroke(help_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::ACCENT_BRICK));
                } else {
                    painter.rect_filled(help_rect, Rounding::same(5.0), splash_colors::CARD_SURFACE);
                    painter.rect_stroke(help_rect, Rounding::same(5.0), Stroke::new(1.0_f32, splash_colors::BORDER));
                }

                draw_splash_vector_icon(
                    &painter,
                    12,
                    Pos2::new(help_x + 14.0, footer_y + btn_h * 0.5),
                    if help_hovered { splash_colors::ACCENT_BRICK } else { splash_colors::TEXT_MUTED },
                );
                painter.text(
                    Pos2::new(help_x + 32.0, footer_y + btn_h * 0.5),
                    egui::Align2::LEFT_CENTER,
                    "Keyboard Shortcuts",
                    FontId::proportional(12.5),
                    if help_hovered { splash_colors::TEXT_CREAM } else { splash_colors::TEXT_MUTED },
                );
                if help_hovered && ctx.input(|i| i.pointer.primary_clicked()) {
                    self.dismiss();
                    triggered_action = Some(SplashAction::OpenKeyboardShortcuts);
                }

                // Outer border around the whole card strictly using #3A3732
                painter.rect_stroke(card_rect, card_rounding, Stroke::new(1.0_f32, card_border));
            });

        triggered_action
    }
}

/// Helper function to draw crisp vector icons matching Inkk's warm styling
fn draw_splash_vector_icon(painter: &egui::Painter, icon_type: usize, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.2_f32, color);
    match icon_type {
        0 => {
            // Blank sheet with folded corner
            let r = Rect::from_center_size(c, Vec2::new(11.0, 13.0));
            painter.rect_stroke(r, Rounding::same(1.0), stroke);
            painter.line_segment([Pos2::new(r.right() - 3.5, r.top()), Pos2::new(r.right(), r.top() + 3.5)], stroke);
        }
        1 => {
            // Ruled paper: sheet with 3 horizontal lines
            let r = Rect::from_center_size(c, Vec2::new(11.0, 13.0));
            painter.rect_stroke(r, Rounding::same(1.0), stroke);
            painter.line_segment([Pos2::new(r.left() + 2.5, c.y - 2.5), Pos2::new(r.right() - 2.5, c.y - 2.5)], stroke);
            painter.line_segment([Pos2::new(r.left() + 2.5, c.y), Pos2::new(r.right() - 2.5, c.y)], stroke);
            painter.line_segment([Pos2::new(r.left() + 2.5, c.y + 2.5), Pos2::new(r.right() - 2.5, c.y + 2.5)], stroke);
        }
        2 => {
            // Grid paper: 2x2 grid lines
            let r = Rect::from_center_size(c, Vec2::new(12.0, 12.0));
            painter.rect_stroke(r, Rounding::same(1.0), stroke);
            painter.line_segment([Pos2::new(c.x, r.top()), Pos2::new(c.x, r.bottom())], stroke);
            painter.line_segment([Pos2::new(r.left(), c.y), Pos2::new(r.right(), c.y)], stroke);
        }
        3 => {
            // More templates: 4 tiny rounded dots in grid
            let d = 2.2;
            painter.circle_filled(c + Vec2::new(-d, -d), 1.2, color);
            painter.circle_filled(c + Vec2::new(d, -d), 1.2, color);
            painter.circle_filled(c + Vec2::new(-d, d), 1.2, color);
            painter.circle_filled(c + Vec2::new(d, d), 1.2, color);
        }
        4 => {
            // Cornell notes: vertical column divider line
            let r = Rect::from_center_size(c, Vec2::new(11.0, 13.0));
            painter.rect_stroke(r, Rounding::same(1.0), stroke);
            painter.line_segment([Pos2::new(r.left() + 4.0, r.top()), Pos2::new(r.left() + 4.0, r.bottom() - 3.0)], stroke);
            painter.line_segment([Pos2::new(r.left(), r.bottom() - 3.0), Pos2::new(r.right(), r.bottom() - 3.0)], stroke);
        }
        5 => {
            // Math studio: Sigma symbol
            let half = 5.0;
            let top_l = Pos2::new(c.x - half, c.y - half);
            let top_r = Pos2::new(c.x + half, c.y - half);
            let mid = Pos2::new(c.x - 1.0, c.y);
            let bot_l = Pos2::new(c.x - half, c.y + half);
            let bot_r = Pos2::new(c.x + half, c.y + half);
            painter.line_segment([top_r, top_l], stroke);
            painter.line_segment([top_l, mid], stroke);
            painter.line_segment([mid, bot_l], stroke);
            painter.line_segment([bot_l, bot_r], stroke);
        }
        6 => {
            // Flowchart: Diamond decision node
            let s = 5.5;
            let top = Pos2::new(c.x, c.y - s);
            let right = Pos2::new(c.x + s, c.y);
            let bot = Pos2::new(c.x, c.y + s);
            let left = Pos2::new(c.x - s, c.y);
            painter.line_segment([top, right], stroke);
            painter.line_segment([right, bot], stroke);
            painter.line_segment([bot, left], stroke);
            painter.line_segment([left, top], stroke);
        }
        7 => {
            // Back arrow
            let half = 4.5;
            painter.line_segment([Pos2::new(c.x + half, c.y), Pos2::new(c.x - half, c.y)], stroke);
            painter.line_segment([Pos2::new(c.x - half, c.y), Pos2::new(c.x - 1.5, c.y - 3.0)], stroke);
            painter.line_segment([Pos2::new(c.x - half, c.y), Pos2::new(c.x - 1.5, c.y + 3.0)], stroke);
        }
        8 => {
            // Recent file: Inkk notebook file icon
            let r = Rect::from_center_size(c, Vec2::new(10.0, 12.0));
            painter.rect_stroke(r, Rounding::same(1.5), stroke);
            painter.line_segment([Pos2::new(r.left() + 2.5, r.top()), Pos2::new(r.left() + 2.5, r.bottom())], stroke);
        }
        9 => {
            // Magnifier / search icon
            let r = 3.8;
            let mc = c + Vec2::new(-1.5, -1.5);
            painter.circle_stroke(mc, r, stroke);
            painter.line_segment([mc + Vec2::new(2.6, 2.6), mc + Vec2::new(5.5, 5.5)], Stroke::new(1.6_f32, color));
        }
        10 => {
            // Open folder icon
            let r = Rect::from_center_size(c + Vec2::new(0.0, 1.0), Vec2::new(12.0, 9.0));
            painter.rect_stroke(r, Rounding::same(1.5), stroke);
            let tab = Rect::from_min_size(Pos2::new(r.left(), r.top() - 3.0), Vec2::new(5.0, 3.0));
            painter.rect_stroke(tab, Rounding::same(1.0), stroke);
        }
        11 => {
            // Recover: Circular counter-clockwise restore arrow
            let r = 4.5;
            painter.circle_stroke(c, r, stroke);
            painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x - 2.5, c.y - r - 2.0)], stroke);
            painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x - 2.5, c.y - r + 2.0)], stroke);
        }
        12 => {
            // Keyboard Shortcuts: crisp keyboard key icon
            let r = Rect::from_center_size(c, Vec2::new(13.0, 10.0));
            painter.rect_stroke(r, Rounding::same(2.0), stroke);
            // Keycaps grid inside
            painter.line_segment([Pos2::new(c.x - 3.5, c.y - 1.5), Pos2::new(c.x - 1.5, c.y - 1.5)], stroke);
            painter.line_segment([Pos2::new(c.x + 1.5, c.y - 1.5), Pos2::new(c.x + 3.5, c.y - 1.5)], stroke);
            painter.line_segment([Pos2::new(c.x - 2.5, c.y + 2.0), Pos2::new(c.x + 2.5, c.y + 2.0)], stroke);
        }
        13 => {
            // Quick Tour / Guide: 4-pointed sparkle guide star
            let s = 5.0;
            painter.line_segment([Pos2::new(c.x, c.y - s), Pos2::new(c.x, c.y + s)], stroke);
            painter.line_segment([Pos2::new(c.x - s, c.y), Pos2::new(c.x + s, c.y)], stroke);
            let d = 2.2;
            painter.line_segment([Pos2::new(c.x - d, c.y - d), Pos2::new(c.x + d, c.y + d)], stroke);
            painter.line_segment([Pos2::new(c.x - d, c.y + d), Pos2::new(c.x + d, c.y - d)], stroke);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_color_palette_values() {
        assert_eq!(splash_colors::BG_DARK, Color32::from_rgb(0x1F, 0x1E, 0x1B));
        assert_eq!(splash_colors::TEXT_CREAM, Color32::from_rgb(0xF5, 0xF0, 0xE6));
        assert_eq!(splash_colors::ACCENT_BRICK, Color32::from_rgb(0xB5, 0x41, 0x2E));
        assert_eq!(splash_colors::BORDER, Color32::from_rgb(0x3A, 0x37, 0x32));
    }

    #[test]
    fn test_splash_state_defaults() {
        let state = SplashScreenState::default();
        assert!(state.is_active);
        assert!(!state.show_more_templates);
        assert!(state.texture.is_none());
    }

    #[test]
    fn test_splash_image_bytes_loaded() {
        let bytes = SplashScreenState::load_image_bytes();
        assert!(!bytes.is_empty(), "Splash cover image bytes must not be empty");
        let dyn_img = image::load_from_memory(&bytes);
        assert!(dyn_img.is_ok(), "Splash image must decode as valid image");
    }
}
