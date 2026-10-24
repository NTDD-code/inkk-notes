//! Configurable Keyboard Shortcuts System for Inkk
//! Provides custom keybinding mapping, JSON persistence, and a large, beautiful management dialog.

use eframe::egui::{self, Color32, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Categories for organizing shortcuts in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutCategory {
    All,
    Tools,
    Editing,
    File,
    Studio,
    Navigation,
}

/// Representation of a key combination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCombo {
    pub key: String, // e.g. "1", "S", "PageDown", "Delete"
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyCombo {
    pub fn new(key: &str, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key: key.to_string(),
            ctrl,
            shift,
            alt,
        }
    }

    /// Formats the combo for user display (e.g. "Ctrl + S" or "Shift + R")
    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        parts.push(&self.key);
        parts.join(" + ")
    }

    /// Returns list of individual badge labels (e.g. ["Ctrl", "S"])
    pub fn key_badges(&self) -> Vec<String> {
        let mut badges = Vec::new();
        if self.ctrl {
            badges.push("Ctrl".to_string());
        }
        if self.alt {
            badges.push("Alt".to_string());
        }
        if self.shift {
            badges.push("Shift".to_string());
        }
        badges.push(self.key.clone());
        badges
    }

    /// Converts the stored key string to `egui::Key` if valid
    pub fn to_egui_key(&self) -> Option<egui::Key> {
        string_to_key(&self.key)
    }

    /// Checks if this key combination is pressed right now in the egui context
    pub fn is_triggered(&self, ctx: &egui::Context) -> bool {
        let Some(egui_key) = self.to_egui_key() else {
            return false;
        };

        let (ctrl_down, shift_down, alt_down) = ctx.input(|i| {
            (
                i.modifiers.ctrl || i.modifiers.command,
                i.modifiers.shift,
                i.modifiers.alt,
            )
        });

        if self.ctrl == ctrl_down && self.shift == shift_down && self.alt == alt_down {
            ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers {
                        alt: self.alt,
                        ctrl: self.ctrl,
                        shift: self.shift,
                        mac_cmd: false,
                        command: self.ctrl,
                    },
                    egui_key,
                )
            })
        } else {
            false
        }
    }
}

/// Metadata description for an action in the shortcuts table
#[derive(Debug, Clone)]
pub struct ShortcutDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_combo: KeyCombo,
    pub category: ShortcutCategory,
}

/// Shortcuts manager maintaining current bindings, defaults, and persistence
pub struct ShortcutManager {
    pub bindings: HashMap<String, KeyCombo>,
    pub definitions: Vec<ShortcutDefinition>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutManager {
    pub fn new() -> Self {
        let defs = Self::default_definitions();
        let mut bindings = HashMap::new();
        for d in &defs {
            bindings.insert(d.id.to_string(), d.default_combo.clone());
        }

        let mut mgr = Self {
            bindings,
            definitions: defs,
        };
        mgr.load_from_disk();
        mgr
    }

    /// Default definitions list for all actions in Inkk
    pub fn default_definitions() -> Vec<ShortcutDefinition> {
        vec![
            // --- TOOLS ---
            ShortcutDefinition {
                id: "tool_pen",
                name: "Pen",
                description: "Digital ink pen tool",
                default_combo: KeyCombo::new("1", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_highlighter",
                name: "Highlighter",
                description: "Translucent highlighter pen",
                default_combo: KeyCombo::new("2", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_shape",
                name: "Shape Tool",
                description: "Draw lines, ellipses, rectangles, and polygons",
                default_combo: KeyCombo::new("3", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_text",
                name: "Text Box",
                description: "Insert and edit typed text boxes",
                default_combo: KeyCombo::new("4", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_eraser_stroke",
                name: "Stroke Eraser",
                description: "Erase entire stroke in one swipe",
                default_combo: KeyCombo::new("5", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_eraser_point",
                name: "Point Eraser",
                description: "Precise cursor-based point eraser",
                default_combo: KeyCombo::new("6", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_lasso",
                name: "Lasso Select",
                description: "Freeform lasso selection tool",
                default_combo: KeyCombo::new("7", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_pan",
                name: "Pan Canvas",
                description: "Scroll and pan around whiteboard without drawing",
                default_combo: KeyCombo::new("8", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_brush_bigger",
                name: "Increase Brush Size",
                description: "Increase pen or eraser thickness (+1pt)",
                default_combo: KeyCombo::new("]", false, false, false),
                category: ShortcutCategory::Tools,
            },
            ShortcutDefinition {
                id: "tool_brush_smaller",
                name: "Decrease Brush Size",
                description: "Decrease pen or eraser thickness (-1pt)",
                default_combo: KeyCombo::new("[", false, false, false),
                category: ShortcutCategory::Tools,
            },

            // --- FILE & EXPORT ---
            ShortcutDefinition {
                id: "file_save",
                name: "Save Notebook",
                description: "Save changes to current notebook file",
                default_combo: KeyCombo::new("S", true, false, false),
                category: ShortcutCategory::File,
            },
            ShortcutDefinition {
                id: "file_open",
                name: "Open Notebook",
                description: "Open an existing .inkk notebook file",
                default_combo: KeyCombo::new("O", true, false, false),
                category: ShortcutCategory::File,
            },
            ShortcutDefinition {
                id: "file_export_pdf",
                name: "Export PDF",
                description: "Export current page or entire notebook to vector PDF",
                default_combo: KeyCombo::new("E", true, true, false),
                category: ShortcutCategory::File,
            },
            ShortcutDefinition {
                id: "file_library",
                name: "Library & Bookshelf",
                description: "Open notebook library manager",
                default_combo: KeyCombo::new("L", true, false, false),
                category: ShortcutCategory::File,
            },

            // --- EDITING & CANVAS ---
            ShortcutDefinition {
                id: "edit_undo",
                name: "Undo",
                description: "Revert last drawing or editing action",
                default_combo: KeyCombo::new("Z", true, false, false),
                category: ShortcutCategory::Editing,
            },
            ShortcutDefinition {
                id: "edit_redo",
                name: "Redo",
                description: "Re-apply previously undone action",
                default_combo: KeyCombo::new("Y", true, false, false),
                category: ShortcutCategory::Editing,
            },
            ShortcutDefinition {
                id: "edit_paste",
                name: "Paste Content",
                description: "Paste image or text from clipboard",
                default_combo: KeyCombo::new("V", true, false, false),
                category: ShortcutCategory::Editing,
            },
            ShortcutDefinition {
                id: "edit_delete",
                name: "Delete Selected",
                description: "Delete selected strokes or text boxes",
                default_combo: KeyCombo::new("Delete", false, false, false),
                category: ShortcutCategory::Editing,
            },
            ShortcutDefinition {
                id: "canvas_reset_zoom",
                name: "Reset Zoom (100%)",
                description: "Reset canvas view scale to 100%",
                default_combo: KeyCombo::new("0", true, false, false),
                category: ShortcutCategory::Editing,
            },

            // --- STUDIOS & TOOLS ---
            ShortcutDefinition {
                id: "instrument_ruler",
                name: "Straight Ruler",
                description: "Toggle straight drawing ruler on canvas",
                default_combo: KeyCombo::new("R", false, false, false),
                category: ShortcutCategory::Studio,
            },
            ShortcutDefinition {
                id: "instrument_protractor",
                name: "Protractor",
                description: "Toggle 360-degree protractor on canvas",
                default_combo: KeyCombo::new("R", false, true, false),
                category: ShortcutCategory::Studio,
            },
            ShortcutDefinition {
                id: "studio_math",
                name: "Math Studio",
                description: "Open MATLAB-style computing & graphing studio",
                default_combo: KeyCombo::new("M", false, false, false),
                category: ShortcutCategory::Studio,
            },
            ShortcutDefinition {
                id: "studio_flowchart",
                name: "Flowchart Studio",
                description: "Open flowchart and mind map studio",
                default_combo: KeyCombo::new("F", true, false, false),
                category: ShortcutCategory::Studio,
            },

            // --- NAVIGATION ---
            ShortcutDefinition {
                id: "nav_next_page",
                name: "Next Page",
                description: "Flip to next page in notebook",
                default_combo: KeyCombo::new("PageDown", false, false, false),
                category: ShortcutCategory::Navigation,
            },
            ShortcutDefinition {
                id: "nav_prev_page",
                name: "Previous Page",
                description: "Flip to previous page in notebook",
                default_combo: KeyCombo::new("PageUp", false, false, false),
                category: ShortcutCategory::Navigation,
            },
            ShortcutDefinition {
                id: "view_fullscreen",
                name: "Toggle Fullscreen",
                description: "Toggle full screen borderless view",
                default_combo: KeyCombo::new("F11", false, false, false),
                category: ShortcutCategory::Navigation,
            },
        ]
    }

    /// Path to user configuration file: `Documents/Inkk/shortcuts.json`
    pub fn config_path() -> PathBuf {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let dir = PathBuf::from(user_profile).join("Documents").join("Inkk");
            let _ = fs::create_dir_all(&dir);
            dir.join("shortcuts.json")
        } else {
            PathBuf::from("shortcuts.json")
        }
    }

    /// Load user custom bindings from disk
    pub fn load_from_disk(&mut self) {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<HashMap<String, KeyCombo>>(&content) {
                    for (id, combo) in loaded {
                        self.bindings.insert(id, combo);
                    }
                }
            }
        }
    }

    /// Save current bindings to disk
    pub fn save_to_disk(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(&self.bindings)
            .map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("File write error: {}", e))?;
        Ok(())
    }

    /// Reset all bindings to default
    pub fn reset_all_to_default(&mut self) {
        self.bindings.clear();
        for d in &self.definitions {
            self.bindings.insert(d.id.to_string(), d.default_combo.clone());
        }
    }

    /// Get current combo for an action ID
    pub fn get(&self, id: &str) -> Option<&KeyCombo> {
        self.bindings.get(id)
    }

    /// Check if action ID is triggered
    pub fn is_triggered(&self, ctx: &egui::Context, id: &str) -> bool {
        if let Some(combo) = self.get(id) {
            combo.is_triggered(ctx)
        } else {
            false
        }
    }
}

/// Helper converting human string to `egui::Key`
pub fn string_to_key(s: &str) -> Option<egui::Key> {
    match s.to_uppercase().as_str() {
        "1" => Some(egui::Key::Num1),
        "2" => Some(egui::Key::Num2),
        "3" => Some(egui::Key::Num3),
        "4" => Some(egui::Key::Num4),
        "5" => Some(egui::Key::Num5),
        "6" => Some(egui::Key::Num6),
        "7" => Some(egui::Key::Num7),
        "8" => Some(egui::Key::Num8),
        "9" => Some(egui::Key::Num9),
        "0" => Some(egui::Key::Num0),
        "A" => Some(egui::Key::A),
        "B" => Some(egui::Key::B),
        "C" => Some(egui::Key::C),
        "D" => Some(egui::Key::D),
        "E" => Some(egui::Key::E),
        "F" => Some(egui::Key::F),
        "G" => Some(egui::Key::G),
        "H" => Some(egui::Key::H),
        "I" => Some(egui::Key::I),
        "J" => Some(egui::Key::J),
        "K" => Some(egui::Key::K),
        "L" => Some(egui::Key::L),
        "M" => Some(egui::Key::M),
        "N" => Some(egui::Key::N),
        "O" => Some(egui::Key::O),
        "P" => Some(egui::Key::P),
        "Q" => Some(egui::Key::Q),
        "R" => Some(egui::Key::R),
        "S" => Some(egui::Key::S),
        "T" => Some(egui::Key::T),
        "U" => Some(egui::Key::U),
        "V" => Some(egui::Key::V),
        "W" => Some(egui::Key::W),
        "X" => Some(egui::Key::X),
        "Y" => Some(egui::Key::Y),
        "Z" => Some(egui::Key::Z),
        "[" | "OPENBRACKET" => Some(egui::Key::OpenBracket),
        "]" | "CLOSEBRACKET" => Some(egui::Key::CloseBracket),
        "DELETE" | "DEL" => Some(egui::Key::Delete),
        "BACKSPACE" => Some(egui::Key::Backspace),
        "PAGEDOWN" | "PGDN" => Some(egui::Key::PageDown),
        "PAGEUP" | "PGUP" => Some(egui::Key::PageUp),
        "ESC" | "ESCAPE" => Some(egui::Key::Escape),
        "F1" => Some(egui::Key::F1),
        "F2" => Some(egui::Key::F2),
        "F3" => Some(egui::Key::F3),
        "F4" => Some(egui::Key::F4),
        "F5" => Some(egui::Key::F5),
        "F11" => Some(egui::Key::F11),
        "F12" => Some(egui::Key::F12),
        _ => None,
    }
}

/// Helper converting `egui::Key` to clean human label
pub fn key_to_string(k: egui::Key) -> &'static str {
    match k {
        egui::Key::Num1 => "1",
        egui::Key::Num2 => "2",
        egui::Key::Num3 => "3",
        egui::Key::Num4 => "4",
        egui::Key::Num5 => "5",
        egui::Key::Num6 => "6",
        egui::Key::Num7 => "7",
        egui::Key::Num8 => "8",
        egui::Key::Num9 => "9",
        egui::Key::Num0 => "0",
        egui::Key::A => "A",
        egui::Key::B => "B",
        egui::Key::C => "C",
        egui::Key::D => "D",
        egui::Key::E => "E",
        egui::Key::F => "F",
        egui::Key::G => "G",
        egui::Key::H => "H",
        egui::Key::I => "I",
        egui::Key::J => "J",
        egui::Key::K => "K",
        egui::Key::L => "L",
        egui::Key::M => "M",
        egui::Key::N => "N",
        egui::Key::O => "O",
        egui::Key::P => "P",
        egui::Key::Q => "Q",
        egui::Key::R => "R",
        egui::Key::S => "S",
        egui::Key::T => "T",
        egui::Key::U => "U",
        egui::Key::V => "V",
        egui::Key::W => "W",
        egui::Key::X => "X",
        egui::Key::Y => "Y",
        egui::Key::Z => "Z",
        egui::Key::OpenBracket => "[",
        egui::Key::CloseBracket => "]",
        egui::Key::Delete => "Delete",
        egui::Key::Backspace => "Backspace",
        egui::Key::PageDown => "PageDown",
        egui::Key::PageUp => "PageUp",
        egui::Key::Escape => "Escape",
        egui::Key::F1 => "F1",
        egui::Key::F2 => "F2",
        egui::Key::F3 => "F3",
        egui::Key::F4 => "F4",
        egui::Key::F5 => "F5",
        egui::Key::F11 => "F11",
        egui::Key::F12 => "F12",
        _ => "Key",
    }
}

/// Interactive Shortcuts Dialog Modal
pub struct ShortcutsDialogState {
    pub is_open: bool,
    pub selected_category: ShortcutCategory,
    pub search_query: String,
    pub recording_action_id: Option<String>,
    pub status_feedback: Option<(String, std::time::Instant)>,
}

impl Default for ShortcutsDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_category: ShortcutCategory::All,
            search_query: String::new(),
            recording_action_id: None,
            status_feedback: None,
        }
    }
}

impl ShortcutsDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.recording_action_id = None;
        self.status_feedback = None;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.recording_action_id = None;
    }

    /// Renders the spacious, elegant keyboard shortcuts configuration window
    pub fn render(&mut self, ctx: &egui::Context, manager: &mut ShortcutManager) {
        if !self.is_open {
            return;
        }

        // Palette matching Inkk design specs:
        // Dark bg #1F1E1B, Cream text #F5F0E6, Brick Red #B5412E, Border #3A3732
        let bg_color = Color32::from_rgb(0x1F, 0x1E, 0x1B);
        let card_bg = Color32::from_rgb(0x24, 0x23, 0x20);
        let text_cream = Color32::from_rgb(0xF5, 0xF0, 0xE6);
        let text_muted = Color32::from_rgb(0xA8, 0xA0, 0x92);
        let accent_brick = Color32::from_rgb(0xB5, 0x41, 0x2E);
        let border_color = Color32::from_rgb(0x3A, 0x37, 0x32);

        // Check if user is currently recording a new keybinding
        if let Some(ref action_id) = self.recording_action_id.clone() {
            // Cancel on Escape
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.recording_action_id = None;
            } else {
                // Detect pressed keys
                let modifiers = ctx.input(|i| i.modifiers);
                let ctrl = modifiers.ctrl || modifiers.command;
                let shift = modifiers.shift;
                let alt = modifiers.alt;

                let keys_to_check = [
                    egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4,
                    egui::Key::Num5, egui::Key::Num6, egui::Key::Num7, egui::Key::Num8,
                    egui::Key::Num9, egui::Key::Num0,
                    egui::Key::A, egui::Key::B, egui::Key::C, egui::Key::D, egui::Key::E,
                    egui::Key::F, egui::Key::G, egui::Key::H, egui::Key::I, egui::Key::J,
                    egui::Key::K, egui::Key::L, egui::Key::M, egui::Key::N, egui::Key::O,
                    egui::Key::P, egui::Key::Q, egui::Key::R, egui::Key::S, egui::Key::T,
                    egui::Key::U, egui::Key::V, egui::Key::W, egui::Key::X, egui::Key::Y,
                    egui::Key::Z,
                    egui::Key::OpenBracket, egui::Key::CloseBracket,
                    egui::Key::Delete, egui::Key::Backspace,
                    egui::Key::PageDown, egui::Key::PageUp,
                    egui::Key::F1, egui::Key::F2, egui::Key::F3, egui::Key::F4,
                    egui::Key::F5, egui::Key::F11, egui::Key::F12,
                ];

                for key in keys_to_check {
                    if ctx.input(|i| i.key_pressed(key)) {
                        let key_label = key_to_string(key);
                        let new_combo = KeyCombo::new(key_label, ctrl, shift, alt);
                        manager.bindings.insert(action_id.clone(), new_combo);
                        self.status_feedback = Some((
                            format!("Updated shortcut for {}", action_id),
                            std::time::Instant::now(),
                        ));
                        self.recording_action_id = None;
                        break;
                    }
                }
            }
        }

        let mut is_window_open = self.is_open;
        let screen_size = ctx.screen_rect().size();
        let win_w = 780.0f32.min(screen_size.x - 40.0);
        let win_h = 580.0f32.min(screen_size.y - 40.0);

        egui::Window::new("⌨ Keyboard Shortcuts & Keybindings")
            .open(&mut is_window_open)
            .collapsible(false)
            .resizable(true)
            .default_size([win_w, win_h])
            .min_size([550.0, 420.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(bg_color)
                    .stroke(Stroke::new(1.0_f32, border_color))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(24.0, 20.0)),
            )
            .show(ctx, |ui| {
                // Header section
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Keyboard Shortcuts & Keybindings")
                                .font(crate::theme::font_serif(22.0))
                                .color(text_cream),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(
                                "Click on any action to record a new key combination. Custom shortcuts are saved automatically.",
                            )
                            .size(12.5)
                            .color(text_muted),
                        );
                    });
                });

                ui.add_space(14.0);

                // Filter & Search bar
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔍 Search:").size(12.5).color(text_muted));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search by action, description, or key...")
                            .desired_width(260.0),
                    );

                    ui.add_space(12.0);

                    // Category filters
                    let categories = [
                        (ShortcutCategory::All, "All"),
                        (ShortcutCategory::Tools, "Drawing Tools"),
                        (ShortcutCategory::File, "File & Export"),
                        (ShortcutCategory::Editing, "Editing"),
                        (ShortcutCategory::Studio, "Studios & Tools"),
                        (ShortcutCategory::Navigation, "Navigation"),
                    ];

                    for (cat, label) in categories {
                        let selected = self.selected_category == cat;
                        let btn = egui::Button::new(
                            egui::RichText::new(label)
                                .size(11.5)
                                .color(if selected { Color32::WHITE } else { text_muted }),
                        )
                        .fill(if selected { accent_brick } else { card_bg })
                        .rounding(Rounding::same(4.0));

                        if ui.add(btn).clicked() {
                            self.selected_category = cat;
                        }
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Scrollable Table of shortcuts
                let query = self.search_query.trim().to_lowercase();
                let filtered_defs: Vec<ShortcutDefinition> = manager
                    .definitions
                    .iter()
                    .filter(|d| {
                        if self.selected_category != ShortcutCategory::All
                            && d.category != self.selected_category
                        {
                            return false;
                        }
                        if !query.is_empty() {
                            let match_name = d.name.to_lowercase().contains(&query);
                            let match_desc = d.description.to_lowercase().contains(&query);
                            let combo_str = manager
                                .get(d.id)
                                .map(|c| c.display_string().to_lowercase())
                                .unwrap_or_default();
                            let match_combo = combo_str.contains(&query);
                            match_name || match_desc || match_combo
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect();

                let scroll_height = ui.available_height() - 55.0;
                let mut to_reset: Option<(String, KeyCombo)> = None;

                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 6.0);

                        for def in &filtered_defs {
                            let combo = manager
                                .get(def.id)
                                .cloned()
                                .unwrap_or_else(|| def.default_combo.clone());

                            let is_recording_this = self
                                .recording_action_id
                                .as_ref()
                                .map_or(false, |id| id == def.id);

                            let row_bg = if is_recording_this {
                                Color32::from_rgb(0x38, 0x22, 0x20)
                            } else {
                                card_bg
                            };

                            egui::Frame::none()
                                .fill(row_bg)
                                .stroke(Stroke::new(
                                    1.0_f32,
                                    if is_recording_this { accent_brick } else { border_color },
                                ))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        // Left info
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(def.name)
                                                    .size(13.5)
                                                    .strong()
                                                    .color(text_cream),
                                            );
                                            ui.label(
                                                egui::RichText::new(def.description)
                                                    .size(11.0)
                                                    .color(text_muted),
                                            );
                                        });

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                // Reset individual button
                                                let is_customized = combo != def.default_combo;
                                                if is_customized {
                                                    let reset_btn = egui::Button::new(
                                                        egui::RichText::new("Reset")
                                                            .size(11.0)
                                                            .color(text_muted),
                                                    )
                                                    .fill(Color32::TRANSPARENT);
                                                    if ui.add(reset_btn).on_hover_text("Reset to default shortcut for this action").clicked() {
                                                        to_reset = Some((def.id.to_string(), def.default_combo.clone()));
                                                    }
                                                }

                                                // Record / Change Button
                                                if is_recording_this {
                                                    let rec_btn = egui::Button::new(
                                                        egui::RichText::new("⏳ Press key... (Esc to cancel)")
                                                            .size(12.0)
                                                            .strong()
                                                            .color(Color32::WHITE),
                                                    )
                                                    .fill(accent_brick)
                                                    .rounding(Rounding::same(4.0));
                                                    if ui.add(rec_btn).clicked() {
                                                        self.recording_action_id = None;
                                                    }
                                                } else {
                                                    let change_btn = egui::Button::new(
                                                        egui::RichText::new("Rebind")
                                                            .size(11.5)
                                                            .color(text_cream),
                                                    )
                                                    .fill(Color32::from_rgb(0x32, 0x2E, 0x28))
                                                    .rounding(Rounding::same(4.0));
                                                    if ui.add(change_btn).on_hover_text("Click to assign a new shortcut").clicked() {
                                                        self.recording_action_id = Some(def.id.to_string());
                                                    }
                                                }

                                                ui.add_space(8.0);

                                                // Key Badges (Physical key styling)
                                                for badge in combo.key_badges().iter().rev() {
                                                    egui::Frame::none()
                                                        .fill(Color32::from_rgb(0x2C, 0x2A, 0x25))
                                                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x4C, 0x47, 0x3F)))
                                                        .rounding(Rounding::same(4.0))
                                                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new(badge)
                                                                    .size(12.0)
                                                                    .monospace()
                                                                    .color(text_cream),
                                                            );
                                                        });
                                                }
                                            },
                                        );
                                    });
                                });
                        }
                    });

                if let Some((action_id, default_combo)) = to_reset {
                    manager.bindings.insert(action_id, default_combo);
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                // Footer section with Reset All, Save, and Close
                ui.horizontal(|ui| {
                    if let Some((ref msg, at)) = self.status_feedback {
                        if at.elapsed().as_secs_f32() < 4.0 {
                            ui.label(
                                egui::RichText::new(format!("✓ {}", msg))
                                    .size(12.0)
                                    .color(Color32::from_rgb(0x8A, 0xC4, 0x8A)),
                            );
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Close")
                                        .size(12.5)
                                        .color(text_cream),
                                )
                                .fill(card_bg)
                                .rounding(Rounding::same(4.0)),
                            )
                            .clicked()
                        {
                            self.close();
                        }

                        // Save button
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("💾 Save Shortcuts")
                                        .size(12.5)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(accent_brick)
                                .rounding(Rounding::same(4.0)),
                            )
                            .on_hover_text("Save customized shortcuts to Inkk configuration")
                            .clicked()
                        {
                            match manager.save_to_disk() {
                                Ok(_) => {
                                    self.status_feedback = Some((
                                        "Shortcuts saved successfully!".to_string(),
                                        std::time::Instant::now(),
                                    ));
                                }
                                Err(e) => {
                                    self.status_feedback = Some((
                                        format!("Failed to save: {}", e),
                                        std::time::Instant::now(),
                                    ));
                                }
                            }
                        }

                        // Reset All button
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Reset All to Defaults")
                                        .size(12.0)
                                        .color(text_muted),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .on_hover_text("Restore all shortcuts to default factory settings")
                            .clicked()
                        {
                            manager.reset_all_to_default();
                            let _ = manager.save_to_disk();
                            self.status_feedback = Some((
                                "Reset all shortcuts to defaults!".to_string(),
                                std::time::Instant::now(),
                            ));
                        }
                    });
                });
            });

        self.is_open = is_window_open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_defaults_loaded() {
        let mgr = ShortcutManager::new();
        assert!(!mgr.definitions.is_empty());
        assert!(mgr.get("file_save").is_some());
        let save_combo = mgr.get("file_save").unwrap();
        assert_eq!(save_combo.key, "S");
        assert!(save_combo.ctrl);
        assert_eq!(save_combo.display_string(), "Ctrl + S");
    }

    #[test]
    fn test_string_to_key_mapping() {
        assert_eq!(string_to_key("S"), Some(egui::Key::S));
        assert_eq!(string_to_key("1"), Some(egui::Key::Num1));
        assert_eq!(string_to_key("Delete"), Some(egui::Key::Delete));
        assert_eq!(string_to_key("PageDown"), Some(egui::Key::PageDown));
        assert_eq!(string_to_key("F1"), Some(egui::Key::F1));
    }

    #[test]
    fn test_key_badges() {
        let combo = KeyCombo::new("E", true, true, false);
        assert_eq!(combo.key_badges(), vec!["Ctrl", "Shift", "E"]);
        assert_eq!(combo.display_string(), "Ctrl + Shift + E");
    }

    #[test]
    fn test_reset_shortcuts() {
        let mut mgr = ShortcutManager::new();
        mgr.bindings.insert("file_save".to_string(), KeyCombo::new("K", true, false, false));
        assert_eq!(mgr.get("file_save").unwrap().key, "K");
        mgr.reset_all_to_default();
        assert_eq!(mgr.get("file_save").unwrap().key, "S");
    }

    #[test]
    fn test_fullscreen_shortcut() {
        let mgr = ShortcutManager::new();
        assert!(mgr.get("view_fullscreen").is_some());
        assert_eq!(mgr.get("view_fullscreen").unwrap().key, "F11");
    }
}
