//! Interactive Step-by-Step Onboarding Walkthrough Tour for Inkk
//! Highlights live UI regions with clean spotlight cutouts and punchy callout cards.

use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistence data to track whether the user has seen the tour
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourStatus {
    pub has_completed: bool,
}

impl Default for TourStatus {
    fn default() -> Self {
        Self {
            has_completed: false,
        }
    }
}

/// A single step in the interactive onboarding tour
#[derive(Debug, Clone)]
pub struct TourStep {
    pub tag: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub tip: &'static str,
}

/// State for the onboarding tour
pub struct OnboardingTourState {
    pub is_active: bool,
    pub current_step: usize,
    pub has_completed: bool,

    // Captured live screen rects of UI regions
    pub rect_canvas: Rect,
    pub rect_docs: Rect,
    pub rect_tools: Rect,
    pub rect_stencils: Rect,
    pub rect_math: Rect,
}

impl Default for OnboardingTourState {
    fn default() -> Self {
        Self::new()
    }
}

impl OnboardingTourState {
    pub fn new() -> Self {
        let status = Self::load_status();
        Self {
            is_active: false,
            current_step: 0,
            has_completed: status.has_completed,
            rect_canvas: Rect::NOTHING,
            rect_docs: Rect::NOTHING,
            rect_tools: Rect::NOTHING,
            rect_stencils: Rect::NOTHING,
            rect_math: Rect::NOTHING,
        }
    }

    /// Path to user configuration file: `Documents/Inkk/tour_status.json`
    pub fn config_path() -> PathBuf {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let dir = PathBuf::from(user_profile).join("Documents").join("Inkk");
            let _ = fs::create_dir_all(&dir);
            dir.join("tour_status.json")
        } else {
            PathBuf::from("tour_status.json")
        }
    }

    pub fn load_status() -> TourStatus {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(status) = serde_json::from_str::<TourStatus>(&content) {
                    return status;
                }
            }
        }
        TourStatus::default()
    }

    pub fn save_status(&self) {
        let path = Self::config_path();
        let status = TourStatus {
            has_completed: self.has_completed,
        };
        if let Ok(json) = serde_json::to_string_pretty(&status) {
            let _ = fs::write(path, json);
        }
    }

    /// Start or re-start the onboarding tour
    pub fn start(&mut self) {
        self.is_active = true;
        self.current_step = 0;
    }

    /// Move to next step or complete if at end
    pub fn next(&mut self) {
        let steps = Self::steps();
        if self.current_step + 1 < steps.len() {
            self.current_step += 1;
        } else {
            self.complete();
        }
    }

    /// Move to previous step
    pub fn prev(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
        }
    }

    /// Dismiss and mark as completed so it runs once
    pub fn complete(&mut self) {
        self.is_active = false;
        self.has_completed = true;
        self.save_status();
    }

    /// Skip the tour
    pub fn skip(&mut self) {
        self.complete();
    }

    /// Curated steps: short, punchy explanations of purpose, how it works, and tool types
    pub fn steps() -> Vec<TourStep> {
        vec![
            TourStep {
                tag: "STEP 1 OF 5 • OVERVIEW",
                title: "Infinite Digital Stationery",
                description: "Inkk is designed for natural, responsive handwriting, technical sketching, and deep lecture notes.",
                tip: "Write freely with pen or mouse. Right-click anywhere for quick context shortcuts.",
            },
            TourStep {
                tag: "STEP 2 OF 5 • TOOLS",
                title: "Core Drawing & Inking Tools",
                description: "Pick natural ballpoint, fountain, or pencil pens. Use Stroke or Point erasers to clean up, and Lasso (7) to select and move strokes.",
                tip: "Switch tools quickly using number keys 1 through 8, or press [ / ] to resize brushes.",
            },
            TourStep {
                tag: "STEP 3 OF 5 • STENCILS",
                title: "Precision Ruler & Protractor",
                description: "Snap straight lines along the Straight Ruler (R), or measure and draw exact angles with the 360° Circular Protractor (Shift+R).",
                tip: "Drag the ruler body to move, and drag the circular dial to rotate to any angle.",
            },
            TourStep {
                tag: "STEP 4 OF 5 • COMPUTING",
                title: "Math & Flowchart Studios",
                description: "Built-in MATLAB computing engine for plotting and matrix math, plus Flowchart Studio for algorithm logic and mind maps.",
                tip: "Press M to open Math Studio, or Ctrl+F to open the visual Flowchart Studio.",
            },
            TourStep {
                tag: "STEP 5 OF 5 • NOTEBOOKS",
                title: "Autosave & Vector PDF Export",
                description: "Your work is autosaved continuously with crash recovery. Organize files in the Library (Ctrl+L) or export crisp vector PDFs (Ctrl+Shift+E).",
                tip: "Press F1 anytime to view and customize keyboard shortcuts to your liking.",
            },
        ]
    }

    /// Renders the spotlight cutout and interactive callout card
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.is_active {
            return;
        }

        // Allow keyboard navigation: Left (prev), Right/Space/Enter (next), Esc (skip)
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.skip();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)) {
            self.next();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.prev();
            return;
        }

        let steps = Self::steps();
        let current_step_info = &steps[self.current_step.min(steps.len() - 1)];

        let screen_rect = ctx.screen_rect();

        // Determine target spotlight rect based on current step
        let target_rect = match self.current_step {
            0 => {
                // Step 1: Canvas overview (center region)
                if self.rect_canvas.is_positive() {
                    let c = self.rect_canvas.center();
                    Rect::from_center_size(c, Vec2::new(520.0, 280.0))
                } else {
                    Rect::from_center_size(screen_rect.center(), Vec2::new(520.0, 280.0))
                }
            }
            1 => {
                // Step 2: Drawing Tools in top bar
                if self.rect_tools.is_positive() {
                    self.rect_tools.expand2(Vec2::new(6.0, 4.0))
                } else {
                    Rect::from_min_size(Pos2::new(screen_rect.center().x - 160.0, 8.0), Vec2::new(320.0, 42.0))
                }
            }
            2 => {
                // Step 3: Stencils (Ruler & Protractor) in top bar
                if self.rect_stencils.is_positive() {
                    self.rect_stencils.expand2(Vec2::new(6.0, 4.0))
                } else {
                    Rect::from_min_size(Pos2::new(screen_rect.center().x + 120.0, 8.0), Vec2::new(90.0, 42.0))
                }
            }
            3 => {
                // Step 4: Math Studio & Flowchart in top bar
                if self.rect_math.is_positive() {
                    self.rect_math.expand2(Vec2::new(6.0, 4.0))
                } else {
                    Rect::from_min_size(Pos2::new(screen_rect.center().x + 220.0, 8.0), Vec2::new(200.0, 42.0))
                }
            }
            4 => {
                // Step 5: Document Actions (Library, Save, PDF) in top bar
                if self.rect_docs.is_positive() {
                    self.rect_docs.expand2(Vec2::new(6.0, 4.0))
                } else {
                    Rect::from_min_size(Pos2::new(80.0, 8.0), Vec2::new(190.0, 42.0))
                }
            }
            _ => Rect::from_center_size(screen_rect.center(), Vec2::new(400.0, 200.0)),
        };

        // Colors matching Inkk dark warm aesthetic
        let mask_color = Color32::from_rgba_unmultiplied(15, 14, 12, 185);
        let brick_accent = Color32::from_rgb(0xB5, 0x41, 0x2E);
        let card_bg = Color32::from_rgb(0x1F, 0x1E, 0x1B);
        let card_border = Color32::from_rgb(0x3A, 0x37, 0x32);
        let text_cream = Color32::from_rgb(0xF5, 0xF0, 0xE6);
        let text_muted = Color32::from_rgb(0xA8, 0xA0, 0x92);

        // Render Spotlight Overlay
        egui::Area::new(egui::Id::new("onboarding_spotlight_overlay"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                let painter = ui.painter();

                // Draw 4-quadrant mask cutout around target_rect
                let t_min = target_rect.min;
                let t_max = target_rect.max;

                // Top mask
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(screen_rect.max.x, t_min.y)),
                    Rounding::ZERO,
                    mask_color,
                );
                // Bottom mask
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(0.0, t_max.y), Pos2::new(screen_rect.max.x, screen_rect.max.y)),
                    Rounding::ZERO,
                    mask_color,
                );
                // Left mask
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(0.0, t_min.y), Pos2::new(t_min.x, t_max.y)),
                    Rounding::ZERO,
                    mask_color,
                );
                // Right mask
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(t_max.x, t_min.y), Pos2::new(screen_rect.max.x, t_max.y)),
                    Rounding::ZERO,
                    mask_color,
                );

                // Glowing highlight border around target region
                painter.rect_stroke(
                    target_rect.expand(2.0),
                    Rounding::same(6.0),
                    Stroke::new(2.0_f32, brick_accent),
                );
                painter.rect_stroke(
                    target_rect.expand(5.0),
                    Rounding::same(8.0),
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0xB5, 0x41, 0x2E, 90)),
                );

                // Callout Card Dimensions & Position
                let card_w = 400.0f32.min(screen_rect.width() - 32.0);
                let card_h = 220.0f32;

                // Position card safely below or above target
                let card_x = (target_rect.center().x - card_w * 0.5).clamp(16.0, screen_rect.width() - card_w - 16.0);
                let card_y = if target_rect.bottom() + card_h + 20.0 < screen_rect.bottom() {
                    target_rect.bottom() + 14.0
                } else if target_rect.top() - card_h - 20.0 > screen_rect.top() {
                    target_rect.top() - card_h - 14.0
                } else {
                    (screen_rect.center().y - card_h * 0.5).max(40.0)
                };

                let card_rect = Rect::from_min_size(Pos2::new(card_x, card_y), Vec2::new(card_w, card_h));

                // Ambient shadow behind card
                for s in 1..=4 {
                    let grow = s as f32 * 4.0;
                    let shadow_rect = card_rect.expand(grow);
                    let shadow_col = Color32::from_rgba_unmultiplied(10, 9, 8, (32 / s) as u8);
                    painter.rect_filled(shadow_rect, Rounding::same(8.0 + grow * 0.5), shadow_col);
                }

                // Card surface & border
                painter.rect_filled(card_rect, Rounding::same(8.0), card_bg);
                painter.rect_stroke(card_rect, Rounding::same(8.0), Stroke::new(1.0_f32, card_border));

                // Connector indicator line pointing from card toward target
                if card_y > target_rect.bottom() {
                    let anchor_x = target_rect.center().x.clamp(card_rect.left() + 20.0, card_rect.right() - 20.0);
                    painter.line_segment(
                        [Pos2::new(anchor_x, card_rect.top()), Pos2::new(anchor_x, target_rect.bottom() + 3.0)],
                        Stroke::new(1.5_f32, brick_accent),
                    );
                }

                // Render Content inside card
                let content_rect = card_rect.shrink(18.0);
                let mut card_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(content_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );

                // Header row: Step tag + Skip '✕' button
                card_ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(current_step_info.tag)
                            .size(10.5)
                            .strong()
                            .color(brick_accent),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(
                                egui::RichText::new("✕ Skip")
                                    .size(11.0)
                                    .color(text_muted),
                            )
                            .on_hover_text("Skip guide and start using Inkk")
                            .clicked()
                        {
                            self.skip();
                        }
                    });
                });

                card_ui.add_space(6.0);

                // Title
                card_ui.label(
                    egui::RichText::new(current_step_info.title)
                        .font(crate::theme::font_serif(17.0))
                        .color(text_cream),
                );

                card_ui.add_space(6.0);

                // Description
                card_ui.label(
                    egui::RichText::new(current_step_info.description)
                        .size(12.5)
                        .color(text_cream),
                );

                card_ui.add_space(6.0);

                // Tip note
                card_ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("💡").size(12.0));
                    ui.label(
                        egui::RichText::new(current_step_info.tip)
                            .size(11.0)
                            .color(text_muted),
                    );
                });

                card_ui.add_space(12.0);

                // Bottom row: Dots progress + Back & Next buttons
                card_ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        // Progress dots
                        for i in 0..steps.len() {
                            let is_current = i == self.current_step;
                            let dot_color = if is_current { brick_accent } else { Color32::from_rgb(0x4A, 0x45, 0x3C) };
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                            ui.painter().circle_filled(rect.center(), if is_current { 3.5 } else { 2.5 }, dot_color);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Next / Finish button
                            let is_last = self.current_step + 1 >= steps.len();
                            let next_label = if is_last { "Start Writing ➔" } else { "Next ➔" };
                            let next_btn = egui::Button::new(
                                egui::RichText::new(next_label)
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(brick_accent)
                            .rounding(Rounding::same(4.0));

                            if ui.add(next_btn).clicked() {
                                self.next();
                            }

                            ui.add_space(6.0);

                            // Back button
                            if self.current_step > 0 {
                                let back_btn = egui::Button::new(
                                    egui::RichText::new("⮜ Back")
                                        .size(12.0)
                                        .color(text_muted),
                                )
                                .fill(Color32::from_rgb(0x28, 0x25, 0x20))
                                .rounding(Rounding::same(4.0));

                                if ui.add(back_btn).clicked() {
                                    self.prev();
                                }
                            }
                        });
                    });
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tour_steps_count_and_content() {
        let steps = OnboardingTourState::steps();
        assert_eq!(steps.len(), 5);
        for s in steps {
            assert!(!s.title.is_empty());
            assert!(!s.description.is_empty());
            assert!(!s.tip.is_empty());
        }
    }

    #[test]
    fn test_tour_step_progression() {
        let mut tour = OnboardingTourState::new();
        tour.start();
        assert!(tour.is_active);
        assert_eq!(tour.current_step, 0);

        tour.next();
        assert_eq!(tour.current_step, 1);

        tour.prev();
        assert_eq!(tour.current_step, 0);

        // Advance to end
        for _ in 0..10 {
            tour.next();
        }
        assert!(!tour.is_active);
        assert!(tour.has_completed);
    }

    #[test]
    fn test_tour_skip() {
        let mut tour = OnboardingTourState::new();
        tour.start();
        assert!(tour.is_active);
        tour.skip();
        assert!(!tour.is_active);
        assert!(tour.has_completed);
    }
}
