//! Inkk Autosave & Crash Recovery System.
//!
//! Provides automatic background snapshots of active notebooks to a dedicated
//! recovery directory (`Documents/Inkk/Autosave/`), configurable save intervals,
//! dirty-state tracking, and instant crash recovery on application startup.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

use crate::model::Notebook;
use crate::storage::NotebookStorage;
use crate::theme::{self, Palette};

/// Configurable autosave interval options.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AutosaveInterval {
    Sec15 = 15,
    Sec30 = 30,
    Min1 = 60,
    Min2 = 120,
    Min5 = 300,
}

impl AutosaveInterval {
    pub const ALL: [AutosaveInterval; 5] = [
        AutosaveInterval::Sec15,
        AutosaveInterval::Sec30,
        AutosaveInterval::Min1,
        AutosaveInterval::Min2,
        AutosaveInterval::Min5,
    ];

    pub fn as_secs(&self) -> u64 {
        *self as u64
    }

    pub fn label(&self) -> &'static str {
        match self {
            AutosaveInterval::Sec15 => "15 seconds",
            AutosaveInterval::Sec30 => "30 seconds (Recommended)",
            AutosaveInterval::Min1 => "1 minute",
            AutosaveInterval::Min2 => "2 minutes",
            AutosaveInterval::Min5 => "5 minutes",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            AutosaveInterval::Sec15 => "15s",
            AutosaveInterval::Sec30 => "30s",
            AutosaveInterval::Min1 => "1m",
            AutosaveInterval::Min2 => "2m",
            AutosaveInterval::Min5 => "5m",
        }
    }
}

/// Metadata saved alongside the recovery snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoveryMetadata {
    pub timestamp_unix: u64,
    pub time_str: String,
    pub notebook_title: String,
    pub original_file_path: Option<PathBuf>,
    pub page_count: usize,
    pub stroke_count: usize,
    pub is_clean_exit: bool,
}

/// Autosave and crash recovery state manager.
#[derive(Clone, Debug)]
pub struct AutosaveManager {
    /// Whether background autosave is active.
    pub enabled: bool,
    /// Autosave frequency interval.
    pub interval: AutosaveInterval,
    /// Whether there are unsaved strokes or edits.
    pub is_dirty: bool,
    /// Last time an autosave snapshot or explicit save occurred.
    pub last_autosave_time: Instant,
    /// Human-readable time of last snapshot.
    pub last_autosave_str: Option<String>,
    /// Dedicated directory for autosave recovery files.
    pub autosave_dir: PathBuf,
    /// Whether the Autosave Settings modal is currently open.
    pub show_settings_dialog: bool,
    /// Active recovery notice banner if restored after an unexpected exit.
    pub recovery_restored_banner: Option<RecoveryMetadata>,
    /// Thread-safe flag indicating a background autosave worker is active.
    pub is_saving: Arc<AtomicBool>,
}

impl Default for AutosaveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AutosaveManager {
    /// Create a new AutosaveManager pointing to the user's Documents/Inkk/Autosave directory.
    pub fn new() -> Self {
        let autosave_dir = Self::default_autosave_dir();
        Self {
            enabled: true,
            interval: AutosaveInterval::Sec30,
            is_dirty: false,
            last_autosave_time: Instant::now(),
            last_autosave_str: None,
            autosave_dir,
            show_settings_dialog: false,
            recovery_restored_banner: None,
            is_saving: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Resolves the dedicated autosave directory in User Documents (e.g. `Documents/Inkk/Autosave`).
    pub fn default_autosave_dir() -> PathBuf {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let dir = PathBuf::from(user_profile)
                .join("Documents")
                .join("Inkk")
                .join("Autosave");
            let _ = fs::create_dir_all(&dir);
            dir
        } else {
            let dir = PathBuf::from("autosave");
            let _ = fs::create_dir_all(&dir);
            dir
        }
    }

    /// Path to the recovery notebook compressed file.
    pub fn recovery_notebook_path(&self) -> PathBuf {
        self.autosave_dir.join("recovery_session.handnote")
    }

    /// Path to the recovery session metadata JSON.
    pub fn recovery_meta_path(&self) -> PathBuf {
        self.autosave_dir.join("recovery_info.json")
    }

    /// Helper to get current formatted time string.
    pub fn current_time_str() -> String {
        let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let total_secs = dur.as_secs();
        let s = total_secs % 60;
        let m = (total_secs / 60) % 60;
        let h = ((total_secs / 3600) + 7) % 24; // Local timezone offset (UTC+7 / ICT)
        format!("{:02}:{:02}:{:02}", h, m, s)
    }

    /// Checks if a valid crash recovery session exists on disk.
    /// Returns `Some((Notebook, Option<original_path>, RecoveryMetadata))` if an unfinalized
    /// recovery file is found with meaningful content.
    pub fn check_recovery_available() -> Option<(Notebook, Option<PathBuf>, RecoveryMetadata)> {
        let dir = Self::default_autosave_dir();
        let handnote_path = dir.join("recovery_session.handnote");
        let meta_path = dir.join("recovery_info.json");

        if !handnote_path.exists() || !meta_path.exists() {
            return None;
        }

        let meta_str = fs::read_to_string(&meta_path).ok()?;
        let meta: RecoveryMetadata = serde_json::from_str(&meta_str).ok()?;

        // If the session was marked as clean exit (user explicitly saved), skip recovery
        if meta.is_clean_exit {
            return None;
        }

        // Attempt to load the recovered notebook
        let notebook = NotebookStorage::load(&handnote_path).ok()?;
        let total_strokes: usize = notebook.pages.iter().map(|p| p.strokes.len()).sum();
        let has_content = total_strokes > 0
            || (!notebook.pages.is_empty()
                && (!notebook.pages[0].images.is_empty()
                    || !notebook.pages[0].text_boxes.is_empty()
                    || !notebook.pages[0].plots.is_empty()
                    || !notebook.pages[0].matrices.is_empty()
                    || !notebook.pages[0].flowcharts.is_empty()));

        if !has_content {
            return None;
        }

        Some((notebook, meta.original_file_path.clone(), meta))
    }

    /// Save an autosave recovery snapshot immediately.
    pub fn save_recovery(
        &mut self,
        notebook: &Notebook,
        original_path: Option<&Path>,
    ) -> Result<(), String> {
        let _ = fs::create_dir_all(&self.autosave_dir);
        let handnote_path = self.recovery_notebook_path();
        let meta_path = self.recovery_meta_path();

        // 1. Write compressed notebook snapshot
        NotebookStorage::save(&handnote_path, notebook)?;

        // 2. Write metadata
        let total_strokes: usize = notebook.pages.iter().map(|p| p.strokes.len()).sum();
        let time_str = Self::current_time_str();
        let unix_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let meta = RecoveryMetadata {
            timestamp_unix: unix_time,
            time_str: time_str.clone(),
            notebook_title: notebook.title.clone(),
            original_file_path: original_path.map(|p| p.to_path_buf()),
            page_count: notebook.pages.len(),
            stroke_count: total_strokes,
            is_clean_exit: false,
        };

        let meta_json = serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize recovery metadata: {}", e))?;
        fs::write(&meta_path, meta_json)
            .map_err(|e| format!("Failed to write recovery metadata: {}", e))?;

        self.last_autosave_time = Instant::now();
        self.last_autosave_str = Some(time_str);
        self.is_dirty = false;

        Ok(())
    }

    /// Mark the current state as clean (e.g. after user explicitly saved or saved-as).
    /// Updates the metadata file to `is_clean_exit = true`.
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
        let meta_path = self.recovery_meta_path();
        if meta_path.exists() {
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(mut meta) = serde_json::from_str::<RecoveryMetadata>(&content) {
                    meta.is_clean_exit = true;
                    if let Ok(updated) = serde_json::to_string_pretty(&meta) {
                        let _ = fs::write(&meta_path, updated);
                    }
                }
            }
        }
    }

    /// Flag unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Check if background autosave timer has elapsed and dirty flag is true.
    pub fn should_autosave(&self) -> bool {
        self.enabled
            && self.is_dirty
            && !self.is_saving.load(Ordering::Relaxed)
            && self.last_autosave_time.elapsed().as_secs() >= self.interval.as_secs()
    }

    /// Asynchronously save recovery snapshot in a background thread to prevent UI stutter.
    pub fn trigger_autosave_background(
        &mut self,
        notebook: &Notebook,
        original_path: Option<&Path>,
    ) {
        if self.is_saving.load(Ordering::Relaxed) {
            return;
        }

        self.is_saving.store(true, Ordering::Relaxed);
        let is_saving_flag = self.is_saving.clone();
        let autosave_dir = self.autosave_dir.clone();
        let handnote_path = self.recovery_notebook_path();
        let meta_path = self.recovery_meta_path();
        let notebook_clone = notebook.clone();
        let original_path_buf = original_path.map(|p| p.to_path_buf());
        let time_str = Self::current_time_str();

        self.last_autosave_time = Instant::now();
        self.last_autosave_str = Some(time_str.clone());
        self.is_dirty = false;

        std::thread::spawn(move || {
            let _ = fs::create_dir_all(&autosave_dir);
            // 1. Write compressed notebook snapshot
            let _ = NotebookStorage::save(&handnote_path, &notebook_clone);

            // 2. Write metadata
            let total_strokes: usize = notebook_clone.pages.iter().map(|p| p.strokes.len()).sum();
            let unix_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let meta = RecoveryMetadata {
                timestamp_unix: unix_time,
                time_str,
                notebook_title: notebook_clone.title.clone(),
                original_file_path: original_path_buf,
                page_count: notebook_clone.pages.len(),
                stroke_count: total_strokes,
                is_clean_exit: false,
            };

            if let Ok(meta_json) = serde_json::to_string_pretty(&meta) {
                let _ = fs::write(&meta_path, meta_json);
            }

            is_saving_flag.store(false, Ordering::Relaxed);
        });
    }

    /// Discard and delete the recovery snapshot files.
    pub fn discard_recovery(&mut self) {
        let _ = fs::remove_file(self.recovery_notebook_path());
        let _ = fs::remove_file(self.recovery_meta_path());
        self.recovery_restored_banner = None;
    }

    /// Open the autosave folder in Windows File Explorer.
    pub fn open_autosave_folder(&self) {
        let _ = fs::create_dir_all(&self.autosave_dir);
        let _ = std::process::Command::new("explorer")
            .arg(&self.autosave_dir)
            .spawn();
    }

    /// Render the interactive Autosave & Crash Recovery Settings modal.
    pub fn render_dialog(
        &mut self,
        ctx: &egui::Context,
        p: &Palette,
        notebook: &Notebook,
        current_path: Option<&Path>,
    ) {
        if !self.show_settings_dialog {
            return;
        }

        let mut is_open = self.show_settings_dialog;
        let mut save_now = false;
        let mut open_folder = false;
        let mut close_clicked = false;

        egui::Window::new("Autosave & Crash Recovery Settings")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_size([440.0, 360.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                // Header info
                ui.label(
                    egui::RichText::new(
                        "Inkk automatically protects your work against crashes, power cuts, or sudden exits by saving background snapshots.",
                    )
                    .size(11.5)
                    .color(p.text_muted),
                );

                ui.add_space(2.0);

                // Enable / Disable Toggle Switch Card
                egui::Frame::none()
                    .fill(p.surface_alt)
                    .stroke(Stroke::new(1.0_f32, p.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Automatic Background Saving")
                                        .font(theme::font_serif(13.5))
                                        .strong()
                                        .color(p.text),
                                );
                                ui.label(
                                    egui::RichText::new(if self.enabled {
                                        "Active • Saves recovery snapshots periodically when edits are made."
                                    } else {
                                        "Disabled • Recovery snapshots will NOT be automatically taken."
                                    })
                                    .size(11.0)
                                    .color(if self.enabled {
                                        Color32::from_rgb(16, 185, 129)
                                    } else {
                                        p.text_muted
                                    }),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let mut toggle_val = self.enabled;
                                if ui.checkbox(&mut toggle_val, "").changed() {
                                    self.enabled = toggle_val;
                                    if self.enabled {
                                        self.mark_dirty();
                                    }
                                }
                            });
                        });
                    });

                // Autosave Frequency Intervals
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("Autosave Frequency Interval")
                        .font(theme::font_serif(13.0))
                        .strong()
                        .color(p.text),
                );

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                    for opt in AutosaveInterval::ALL {
                        let is_selected = self.interval == opt;
                        let bg = if is_selected {
                            p.accent.gamma_multiply(0.2)
                        } else {
                            p.surface_alt
                        };
                        let border_color = if is_selected {
                            p.accent
                        } else {
                            p.border
                        };
                        let text_color = if is_selected {
                            p.accent
                        } else {
                            p.text
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(opt.label())
                                .size(11.5)
                                .color(text_color)
                                .strong(),
                        )
                        .fill(bg)
                        .stroke(Stroke::new(1.0_f32, border_color))
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(0.0, 28.0));

                        if ui.add_enabled(self.enabled, btn).clicked() {
                            self.interval = opt;
                        }
                    }
                });

                // Status & Storage Location Card
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(p.surface_alt)
                    .stroke(Stroke::new(1.0_f32, p.border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (dot_color, status_text) = if !self.enabled {
                                (Color32::from_rgb(156, 163, 175), "Autosave is OFF")
                            } else if self.is_dirty {
                                (Color32::from_rgb(245, 158, 11), "Unsaved changes pending autosave...")
                            } else {
                                (Color32::from_rgb(16, 185, 129), "Up to date • Notebook protected")
                            };

                            let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 4.0, dot_color);
                            ui.label(
                                egui::RichText::new(status_text)
                                    .size(11.5)
                                    .strong()
                                    .color(dot_color),
                            );

                            if let Some(ref last_time) = self.last_autosave_str {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("Last snapshot: {}", last_time))
                                            .size(10.5)
                                            .color(p.text_muted),
                                    );
                                });
                            }
                        });

                        ui.add_space(4.0);
                        let dir_display = self.autosave_dir.display().to_string();
                        ui.label(
                            egui::RichText::new(format!("Folder: {}", dir_display))
                                .size(10.5)
                                .color(p.text_muted)
                                .monospace(),
                        );
                    });

                // Bottom actions: Save Snapshot Now, Open Folder, Close
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            egui::RichText::new("Open Autosave Folder")
                                .size(11.5)
                                .color(p.text),
                        )
                        .on_hover_text("Open recovery snapshot folder in Windows Explorer")
                        .clicked()
                    {
                        open_folder = true;
                    }

                    if ui
                        .add_enabled(
                            self.enabled,
                            egui::Button::new(
                                egui::RichText::new("Snapshot Now")
                                    .size(11.5)
                                    .color(p.accent)
                                    .strong(),
                            ),
                        )
                        .on_hover_text("Immediately write a recovery snapshot to disk")
                        .clicked()
                    {
                        save_now = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new("Done").size(12.0).strong())
                            .clicked()
                        {
                            close_clicked = true;
                        }
                    });
                });
            });

        if close_clicked {
            is_open = false;
        }
        self.show_settings_dialog = is_open;

        if open_folder {
            self.open_autosave_folder();
        }
        if save_now {
            let _ = self.save_recovery(notebook, current_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Notebook, Point, Stroke};

    fn make_test_notebook() -> Notebook {
        let mut nb = Notebook::new("Test Autosave Notebook");
        let stroke = Stroke {
            id: "stroke_1".to_string(),
            pen_type: crate::model::PenType::Ballpoint,
            shape_type: crate::model::ShapeType::Rectangle,
            is_filled: false,
            color: [0, 0, 0, 255],
            original_color: None,
            size: 2.0,
            opacity: 1.0,
            points: vec![
                Point { x: 10.0, y: 10.0, pressure: 0.5, thickness: 2.0 },
                Point { x: 20.0, y: 20.0, pressure: 0.7, thickness: 2.5 },
            ],
        };
        nb.pages[0].strokes.push(stroke);
        nb
    }

    #[test]
    fn test_autosave_interval_attributes() {
        assert_eq!(AutosaveInterval::ALL.len(), 5);
        assert_eq!(AutosaveInterval::Sec15.as_secs(), 15);
        assert_eq!(AutosaveInterval::Sec30.as_secs(), 30);
        assert_eq!(AutosaveInterval::Min1.as_secs(), 60);
        assert_eq!(AutosaveInterval::Min2.as_secs(), 120);
        assert_eq!(AutosaveInterval::Min5.as_secs(), 300);

        assert_eq!(AutosaveInterval::Sec30.short_label(), "30s");
        assert!(AutosaveInterval::Sec30.label().contains("30 seconds"));
    }

    #[test]
    fn test_autosave_manager_defaults() {
        let mgr = AutosaveManager::new();
        assert!(mgr.enabled);
        assert_eq!(mgr.interval, AutosaveInterval::Sec30);
        assert!(!mgr.is_dirty);
        assert!(mgr.autosave_dir.exists());
    }

    #[test]
    fn test_save_and_recovery_flow() {
        let temp_dir = std::env::temp_dir().join(format!("inkk_test_autosave_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let _ = fs::create_dir_all(&temp_dir);

        let mut mgr = AutosaveManager::new();
        mgr.autosave_dir = temp_dir.clone();

        let nb = make_test_notebook();
        let orig_path = PathBuf::from("C:\\fake_path\\my_note.handnote");

        // Should not autosave initially (not dirty)
        assert!(!mgr.should_autosave());

        mgr.mark_dirty();
        assert!(mgr.is_dirty);

        // Perform save recovery
        let res = mgr.save_recovery(&nb, Some(&orig_path));
        assert!(res.is_ok());
        assert!(!mgr.is_dirty);
        assert!(mgr.recovery_notebook_path().exists());
        assert!(mgr.recovery_meta_path().exists());

        // Check recovery metadata
        let meta_str = fs::read_to_string(mgr.recovery_meta_path()).unwrap();
        let meta: RecoveryMetadata = serde_json::from_str(&meta_str).unwrap();
        assert_eq!(meta.notebook_title, "Test Autosave Notebook");
        assert_eq!(meta.stroke_count, 1);
        assert_eq!(meta.original_file_path, Some(orig_path.clone()));
        assert!(!meta.is_clean_exit);

        // Mark clean
        mgr.mark_clean();
        let clean_meta_str = fs::read_to_string(mgr.recovery_meta_path()).unwrap();
        let clean_meta: RecoveryMetadata = serde_json::from_str(&clean_meta_str).unwrap();
        assert!(clean_meta.is_clean_exit);

        // Discard recovery
        mgr.discard_recovery();
        assert!(!mgr.recovery_notebook_path().exists());
        assert!(!mgr.recovery_meta_path().exists());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
