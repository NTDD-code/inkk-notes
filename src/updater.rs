//! Inkk Auto-Updater & Market Distribution System
//! Provides asynchronous update checking, manifest parsing, semantic versioning,
//! and an in-app update notification modal dialog for commercial distribution.

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Current compiled application version from Cargo.toml
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default update manifest URL for Inkk official releases
pub const DEFAULT_UPDATE_URL: &str =
    "https://raw.githubusercontent.com/NTDD-code/inkk-notes/main/packaging/update_manifest.json";

/// Parsed remote update manifest structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub release_date: String,
    pub title: String,
    pub release_notes: String,
    pub download_url: String,
    #[serde(default)]
    pub mandatory: bool,
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Status of the background update check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpdateAvailable(UpdateManifest),
    UpToDate(String),
    Error(String),
}

/// User configuration for updater persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdaterConfig {
    pub check_on_startup: bool,
    pub update_url: String,
    pub last_checked_epoch_sec: u64,
}

impl Default for UpdaterConfig {
    fn default() -> Self {
        Self {
            check_on_startup: true,
            update_url: DEFAULT_UPDATE_URL.to_string(),
            last_checked_epoch_sec: 0,
        }
    }
}

/// State managing background update checks and the update modal dialog
pub struct UpdaterState {
    pub status: Arc<Mutex<UpdateStatus>>,
    pub is_dialog_open: bool,
    pub has_prompted_auto: bool,
    pub config: UpdaterConfig,
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdaterState {
    pub fn new() -> Self {
        let config = Self::load_config();
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
            is_dialog_open: false,
            has_prompted_auto: false,
            config,
        }
    }

    /// Path to user configuration file: `Documents/Inkk/update_config.json`
    pub fn config_path() -> PathBuf {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let dir = PathBuf::from(user_profile).join("Documents").join("Inkk");
            let _ = fs::create_dir_all(&dir);
            dir.join("update_config.json")
        } else {
            PathBuf::from("update_config.json")
        }
    }

    pub fn load_config() -> UpdaterConfig {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<UpdaterConfig>(&content) {
                    return cfg;
                }
            }
        }
        UpdaterConfig::default()
    }

    pub fn save_config(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(&self.config) {
            let _ = fs::write(path, json);
        }
    }

    /// Trigger update check (can be called on startup or manually from menu)
    pub fn check_for_updates(&mut self, show_dialog: bool) {
        if show_dialog {
            self.is_dialog_open = true;
        }

        let status_arc = Arc::clone(&self.status);
        let url = self.config.update_url.clone();

        // Update current status to Checking
        if let Ok(mut st) = status_arc.lock() {
            *st = UpdateStatus::Checking;
        }

        // Record last check time
        self.config.last_checked_epoch_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.save_config();

        // Spawn background thread to fetch manifest
        std::thread::spawn(move || {
            let result = fetch_update_manifest(&url);
            if let Ok(mut st) = status_arc.lock() {
                match result {
                    Ok(manifest) => {
                        if is_newer(&manifest.version, CURRENT_VERSION) {
                            *st = UpdateStatus::UpdateAvailable(manifest);
                        } else {
                            *st = UpdateStatus::UpToDate(CURRENT_VERSION.to_string());
                        }
                    }
                    Err(e) => {
                        *st = UpdateStatus::Error(e);
                    }
                }
            }
        });
    }

    /// Open update dialog manually
    pub fn open_dialog(&mut self) {
        self.is_dialog_open = true;
        let need_check = if let Ok(st) = self.status.lock() {
            matches!(*st, UpdateStatus::Idle)
        } else {
            false
        };
        if need_check {
            self.check_for_updates(true);
        }
    }

    /// Render update modal dialog if open or if an update was found in the background
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.is_dialog_open && !self.has_prompted_auto {
            if let Ok(st) = self.status.lock() {
                if matches!(*st, UpdateStatus::UpdateAvailable(_)) {
                    self.is_dialog_open = true;
                    self.has_prompted_auto = true;
                }
            }
        }

        if !self.is_dialog_open {
            return;
        }

        let mut is_open = self.is_dialog_open;
        let mut close_dialog = false;
        let card_bg = Color32::from_rgb(0x1F, 0x1E, 0x1B);
        let card_border = Color32::from_rgb(0x3A, 0x37, 0x32);
        let brick_accent = Color32::from_rgb(0xB5, 0x41, 0x2E);
        let text_cream = Color32::from_rgb(0xF5, 0xF0, 0xE6);
        let text_muted = Color32::from_rgb(0xA8, 0xA0, 0x92);

        let status_clone = if let Ok(st) = self.status.lock() {
            st.clone()
        } else {
            UpdateStatus::Idle
        };

        egui::Window::new("Inkk Software Update")
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(card_bg)
                    .stroke(Stroke::new(1.0_f32, card_border))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(20.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);

                match status_clone {
                    UpdateStatus::Idle | UpdateStatus::Checking => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                egui::RichText::new("Checking for updates from Inkk server...")
                                    .size(13.5)
                                    .color(text_cream),
                            );
                        });
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(format!("Current Version: v{}", CURRENT_VERSION))
                                .size(11.5)
                                .color(text_muted),
                        );
                    }
                    UpdateStatus::UpdateAvailable(ref manifest) => {
                        // Title header
                        ui.label(
                            egui::RichText::new("A New Version of Inkk is Available!")
                                .font(crate::theme::font_serif(18.0))
                                .strong()
                                .color(brick_accent),
                        );

                        // Version badge comparison
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Current: v{}", CURRENT_VERSION))
                                    .size(12.0)
                                    .color(text_muted),
                            );
                            ui.label(egui::RichText::new("->").size(12.0).color(brick_accent));
                            ui.label(
                                egui::RichText::new(format!("New: v{}", manifest.version))
                                    .size(12.5)
                                    .strong()
                                    .color(Color32::from_rgb(0x4E, 0xC9, 0xB0)),
                            );
                            if !manifest.release_date.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!("({})", manifest.release_date))
                                        .size(11.0)
                                        .color(text_muted),
                                );
                            }
                        });

                        ui.separator();

                        if !manifest.title.is_empty() {
                            ui.label(
                                egui::RichText::new(&manifest.title)
                                    .size(13.0)
                                    .strong()
                                    .color(text_cream),
                            );
                        }

                        // Release notes scroll box
                        ui.label(
                            egui::RichText::new("What's New:")
                                .size(11.5)
                                .strong()
                                .color(text_muted),
                        );
                        egui::ScrollArea::vertical()
                            .max_height(140.0)
                            .show(ui, |ui| {
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(0x16, 0x15, 0x13))
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.0_f32, card_border))
                                    .inner_margin(egui::Margin::same(10.0))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.label(
                                            egui::RichText::new(&manifest.release_notes)
                                                .size(11.5)
                                                .color(text_cream),
                                        );
                                    });
                            });

                        ui.add_space(8.0);

                        // Action Buttons
                        ui.horizontal(|ui| {
                            let download_btn = egui::Button::new(
                                egui::RichText::new("Download & Install Update")
                                    .size(12.5)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(brick_accent)
                            .rounding(Rounding::same(4.0));

                            if ui.add(download_btn).clicked() {
                                open_browser(&manifest.download_url);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("Remind Me Later")
                                            .size(12.0)
                                            .color(text_muted),
                                    )
                                    .clicked()
                                {
                                    close_dialog = true;
                                }
                            });
                        });
                    }
                    UpdateStatus::UpToDate(ref v) => {
                        ui.label(
                            egui::RichText::new("You're Up to Date!")
                                .font(crate::theme::font_serif(17.0))
                                .strong()
                                .color(Color32::from_rgb(0x4E, 0xC9, 0xB0)),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Inkk v{} is currently the newest version available.",
                                v
                            ))
                            .size(12.5)
                            .color(text_cream),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("Close")
                                            .size(12.0)
                                            .color(text_cream),
                                    )
                                    .clicked()
                                {
                                    close_dialog = true;
                                }
                            });
                        });
                    }
                    UpdateStatus::Error(ref err) => {
                        ui.label(
                            egui::RichText::new("Check Failed")
                                .font(crate::theme::font_serif(16.0))
                                .strong()
                                .color(brick_accent),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "Could not connect to update server: {}\nPlease check your internet connection.",
                                err
                            ))
                            .size(12.0)
                            .color(text_muted),
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui
                                .button(
                                    egui::RichText::new("Retry")
                                        .size(12.0)
                                        .color(text_cream),
                                )
                                .clicked()
                            {
                                self.check_for_updates(true);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(
                                        egui::RichText::new("Close")
                                            .size(12.0)
                                            .color(text_muted),
                                    )
                                    .clicked()
                                {
                                    close_dialog = true;
                                }
                            });
                        });
                    }
                }
            });

        self.is_dialog_open = is_open && !close_dialog;
    }
}

/// Fetch update manifest JSON via built-in system curl (handles HTTPS with Windows system certs)
fn fetch_update_manifest(url: &str) -> Result<UpdateManifest, String> {
    // If local test file path
    if url.starts_with("file://") || url.ends_with(".json") && PathBuf::from(url).exists() {
        let path = url.trim_start_matches("file://");
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        return serde_json::from_str::<UpdateManifest>(&content).map_err(|e| e.to_string());
    }

    // Use Windows built-in curl.exe with timeout
    let mut cmd = std::process::Command::new("curl");
    cmd.args(["-s", "-L", "--max-time", "8", url]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output = cmd.output().map_err(|e| format!("curl execution error: {}", e))?;

    if !output.status.success() {
        return Err(format!("Server returned error code {:?}", output.status.code()));
    }

    let body = String::from_utf8(output.stdout).map_err(|e| format!("UTF-8 error: {}", e))?;
    if body.trim().is_empty() {
        return Err("Empty response from server".to_string());
    }

    serde_json::from_str::<UpdateManifest>(&body)
        .map_err(|e| format!("Invalid JSON manifest: {}", e))
}

/// Compare semantic versions (e.g. "0.2.0" > "0.1.0")
pub fn parse_version(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().trim_start_matches('v').trim_start_matches('V');
    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.get(0).and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_newer(remote: &str, current: &str) -> bool {
    parse_version(remote) > parse_version(current)
}

/// Open default browser to given URL
pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/c", "start", "", url]);
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing_and_comparison() {
        assert_eq!(parse_version("0.1.0"), (0, 1, 0));
        assert_eq!(parse_version("v0.2.5"), (0, 2, 5));
        assert_eq!(parse_version("1.0"), (1, 0, 0));

        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
    }

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "version": "0.2.0",
            "release_date": "2026-11-01",
            "title": "Inkk Major Autumn Update",
            "release_notes": "- Fullscreen mode (F11)\n- Enhanced stencil rulers\n- Auto-updater",
            "download_url": "https://github.com/inkk-notes/inkk/releases/tag/v0.2.0"
        }"#;

        let manifest: Result<UpdateManifest, _> = serde_json::from_str(json);
        assert!(manifest.is_ok());
        let m = manifest.unwrap();
        assert_eq!(m.version, "0.2.0");
        assert_eq!(m.title, "Inkk Major Autumn Update");
        assert!(is_newer(&m.version, "0.1.0"));
    }
}
