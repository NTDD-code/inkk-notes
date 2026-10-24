//! Inkk Professional Notebook Library & Bookshelf Manager.
//!
//! Provides a modern, library-style document hub:
//! 1. Multi-source scanning (Default Documents/Inkk/Notebooks + user Downloads)
//! 2. Realistic book cover visual cards with paper templates & binding styling
//! 3. Search, live filtering, and sorting (Modified, Name, Pages)
//! 4. Grid view and compact List view modes
//! 5. Direct notebook creation, duplication, renaming, deletion, and PDF export
//! 6. Auto-saves the active document when switching between notebooks.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

use crate::model::PaperType;
use crate::storage::NotebookStorage;
use crate::theme::Palette;

/// Metadata for a notebook file discovered in the library.
#[derive(Debug, Clone)]
pub struct NotebookMeta {
    pub path: PathBuf,
    pub filename: String,
    pub title: String,
    pub page_count: usize,
    pub paper_type: PaperType,
    pub file_size_bytes: u64,
    pub modified: SystemTime,
    pub is_active: bool,
}

impl NotebookMeta {
    /// Formats elapsed time since modification into human-readable text.
    pub fn format_modified(&self) -> String {
        if let Ok(elapsed) = SystemTime::now().duration_since(self.modified) {
            let secs = elapsed.as_secs();
            if secs < 60 {
                "Just now".to_string()
            } else if secs < 3600 {
                format!("{}m ago", secs / 60)
            } else if secs < 86400 {
                format!("{}h ago", secs / 3600)
            } else if secs < 7 * 86400 {
                format!("{}d ago", secs / 86400)
            } else {
                format!("{}w ago", secs / (7 * 86400))
            }
        } else {
            "Recently".to_string()
        }
    }

    /// Formats file size into KB / MB.
    pub fn format_size(&self) -> String {
        let bytes = self.file_size_bytes;
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    /// Cover palette for realistic book rendering based on title hash.
    pub fn cover_palette(&self, is_dark: bool) -> (Color32, Color32) {
        let hash: usize = self.title.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        let palettes_light = [
            (Color32::from_rgb(46, 52, 64), Color32::from_rgb(33, 37, 46)),    // Slate Navy
            (Color32::from_rgb(142, 68, 54), Color32::from_rgb(112, 50, 39)),   // Burgundy Leather
            (Color32::from_rgb(52, 73, 94), Color32::from_rgb(40, 56, 73)),    // Royal Blue
            (Color32::from_rgb(70, 85, 66), Color32::from_rgb(52, 64, 49)),    // Sage Green
            (Color32::from_rgb(120, 90, 60), Color32::from_rgb(95, 70, 45)),   // Warm Ochre Leather
            (Color32::from_rgb(50, 50, 50), Color32::from_rgb(35, 35, 35)),    // Charcoal Classic
        ];
        let palettes_dark = [
            (Color32::from_rgb(35, 40, 50), Color32::from_rgb(25, 28, 36)),
            (Color32::from_rgb(110, 50, 40), Color32::from_rgb(85, 38, 30)),
            (Color32::from_rgb(40, 55, 72), Color32::from_rgb(30, 42, 56)),
            (Color32::from_rgb(55, 68, 52), Color32::from_rgb(42, 52, 39)),
            (Color32::from_rgb(95, 72, 48), Color32::from_rgb(75, 56, 36)),
            (Color32::from_rgb(42, 42, 42), Color32::from_rgb(30, 30, 30)),
        ];

        let list = if is_dark { &palettes_dark } else { &palettes_light };
        list[hash % list.len()]
    }
}

/// Sorting mode for notebook library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibrarySortBy {
    ModifiedDesc,
    NameAsc,
    PageCountDesc,
}

/// View presentation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryViewMode {
    Grid,
    List,
}

/// Action emitted by the Library dialog to be handled by the main app.
#[derive(Debug)]
pub enum LibraryAction {
    Open(PathBuf),
    CreateNew { title: String, paper_type: PaperType },
    Duplicate(PathBuf),
    Rename { path: PathBuf, new_title: String },
    Delete(PathBuf),
    ExportPdf(PathBuf),
    OpenFolderInExplorer,
    ImportFile,
}

/// Library State Manager.
pub struct LibraryState {
    pub is_open: bool,
    pub library_dir: PathBuf,
    pub items: Vec<NotebookMeta>,
    pub search_query: String,
    pub sort_by: LibrarySortBy,
    pub view_mode: LibraryViewMode,

    // Sub-dialogs
    pub show_create_modal: bool,
    pub new_title: String,
    pub new_paper: PaperType,

    pub renaming_item: Option<(PathBuf, String)>,
    pub delete_confirm: Option<PathBuf>,
}

impl Default for LibraryState {
    fn default() -> Self {
        let library_dir = Self::default_library_dir();
        Self {
            is_open: false,
            library_dir,
            items: Vec::new(),
            search_query: String::new(),
            sort_by: LibrarySortBy::ModifiedDesc,
            view_mode: LibraryViewMode::Grid,
            show_create_modal: false,
            new_title: String::new(),
            new_paper: PaperType::Ruled,
            renaming_item: None,
            delete_confirm: None,
        }
    }
}

impl LibraryState {
    /// Default notebook directory in User Documents (e.g. Documents/Inkk/Notebooks).
    pub fn default_library_dir() -> PathBuf {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let docs = PathBuf::from(user_profile)
                .join("Documents")
                .join("Inkk")
                .join("Notebooks");
            let _ = fs::create_dir_all(&docs);
            docs
        } else {
            let local = PathBuf::from("notes");
            let _ = fs::create_dir_all(&local);
            local
        }
    }

    /// Opens the library dialog and refreshes the scan.
    pub fn open(&mut self, active_path: Option<&Path>) {
        self.is_open = true;
        self.show_create_modal = false;
        self.renaming_item = None;
        self.delete_confirm = None;
        self.scan(active_path);
    }

    /// Scans both the official Inkk Library directory and the user's Downloads folder for notebooks.
    pub fn scan(&mut self, active_path: Option<&Path>) {
        let mut metas = Vec::new();

        // 1. Scan default library folder
        Self::scan_folder(&self.library_dir, active_path, &mut metas);

        // 2. Also scan user's Downloads folder if it exists (automatically picks up user's existing handnotes!)
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let dl = PathBuf::from(user_profile).join("Downloads");
            if dl.exists() && dl != self.library_dir {
                Self::scan_folder(&dl, active_path, &mut metas);
            }
        }

        // Sort items
        match self.sort_by {
            LibrarySortBy::ModifiedDesc => {
                metas.sort_by(|a, b| b.modified.cmp(&a.modified));
            }
            LibrarySortBy::NameAsc => {
                metas.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            }
            LibrarySortBy::PageCountDesc => {
                metas.sort_by(|a, b| b.page_count.cmp(&a.page_count));
            }
        }

        self.items = metas;
    }

    fn scan_folder(folder: &Path, active_path: Option<&Path>, out: &mut Vec<NotebookMeta>) {
        let entries = match fs::read_dir(folder) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            if ext != "handnote" && ext != "inkk" {
                continue;
            }

            // Check if already in out (avoid duplicates)
            if out.iter().any(|m| m.path == path) {
                continue;
            }

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let file_size_bytes = metadata.len();
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();

            // Load notebook quickly to obtain page count and title
            let (title, page_count, paper_type) = if let Ok(nb) = NotebookStorage::load(&path) {
                let t = if nb.title.trim().is_empty() {
                    path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string()
                } else {
                    nb.title
                };
                (t, nb.pages.len().max(1), nb.default_paper)
            } else {
                let t = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled").to_string();
                (t, 1, PaperType::Ruled)
            };

            let is_active = active_path.map(|p| p == path).unwrap_or(false);

            out.push(NotebookMeta {
                path,
                filename,
                title,
                page_count,
                paper_type,
                file_size_bytes,
                modified,
                is_active,
            });
        }
    }

    /// Renders the Library modal window with Grid and List views.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        p: &Palette,
        is_dark: bool,
        active_path: Option<&Path>,
    ) -> Option<LibraryAction> {
        if !self.is_open {
            return None;
        }

        let mut action: Option<LibraryAction> = None;
        let mut is_open = self.is_open;

        egui::Window::new("Inkk Notebook Library")
            .open(&mut is_open)
            .resizable(true)
            .collapsible(false)
            .default_size([720.0, 520.0])
            .min_size([580.0, 400.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                // --- TOP ACTION BAR ---
                ui.horizontal(|ui| {
                    // + New Notebook Button
                    let new_btn = egui::Button::new(
                        egui::RichText::new("+ New Notebook").color(p.on_accent).strong(),
                    )
                    .fill(p.accent)
                    .rounding(Rounding::same(crate::theme::RADIUS_SM));

                    if ui.add(new_btn).on_hover_text("Create a new handwritten notebook").clicked() {
                        self.show_create_modal = true;
                        self.new_title = format!("Notebook {}", self.items.len() + 1);
                    }

                    // Import Note Button
                    if ui.button("Import Note...").on_hover_text("Open external .handnote or .inkk file").clicked() {
                        action = Some(LibraryAction::ImportFile);
                    }

                    // Open Folder in Explorer
                    if ui.button("Open Folder").on_hover_text("Open notebooks folder in Windows Explorer").clicked() {
                        action = Some(LibraryAction::OpenFolderInExplorer);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // View Toggle: Grid vs List
                        if ui.selectable_label(self.view_mode == LibraryViewMode::List, "List").clicked() {
                            self.view_mode = LibraryViewMode::List;
                        }
                        if ui.selectable_label(self.view_mode == LibraryViewMode::Grid, "Grid").clicked() {
                            self.view_mode = LibraryViewMode::Grid;
                        }

                        // Sort selector
                        ui.label(egui::RichText::new("Sort:").size(11.0).color(p.text_muted));
                        egui::ComboBox::from_id_salt("lib_sort_by")
                            .width(105.0)
                            .selected_text(match self.sort_by {
                                LibrarySortBy::ModifiedDesc => "Recent",
                                LibrarySortBy::NameAsc => "A-Z",
                                LibrarySortBy::PageCountDesc => "Pages",
                            })
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut self.sort_by, LibrarySortBy::ModifiedDesc, "Recent").clicked() {
                                    self.scan(active_path);
                                }
                                if ui.selectable_value(&mut self.sort_by, LibrarySortBy::NameAsc, "A-Z").clicked() {
                                    self.scan(active_path);
                                }
                                if ui.selectable_value(&mut self.sort_by, LibrarySortBy::PageCountDesc, "Pages").clicked() {
                                    self.scan(active_path);
                                }
                            });

                        // Search box
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .hint_text("Search notes...")
                                .desired_width(140.0),
                        );
                    });
                });

                ui.separator();

                // Filter items by search query
                let query = self.search_query.trim().to_lowercase();
                let filtered: Vec<&NotebookMeta> = self.items
                    .iter()
                    .filter(|item| query.is_empty() || item.title.to_lowercase().contains(&query) || item.filename.to_lowercase().contains(&query))
                    .collect();

                // --- NOTEBOOK SHELF VIEW ---
                if filtered.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(egui::RichText::new("No notebooks found").font(crate::theme::font_serif(16.0)).color(p.text_muted));
                        ui.add_space(8.0);
                        if ui.button("+ Create your first notebook").clicked() {
                            self.show_create_modal = true;
                            self.new_title = "My First Notebook".to_string();
                        }
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            match self.view_mode {
                                LibraryViewMode::Grid => {
                                    let card_w = 175.0;
                                    let card_h = 220.0;
                                    let available_w = ui.available_width();
                                    let cols = ((available_w / (card_w + 14.0)).floor() as usize).max(1);

                                    egui::Grid::new("library_grid_cards")
                                        .spacing([14.0, 16.0])
                                        .show(ui, |ui| {
                                            for (idx, meta) in filtered.iter().enumerate() {
                                                if let Some(act) = Self::render_notebook_card(ui, meta, card_w, card_h, p, is_dark) {
                                                    action = Some(act);
                                                }

                                                if (idx + 1) % cols == 0 {
                                                    ui.end_row();
                                                }
                                            }
                                        });
                                }
                                LibraryViewMode::List => {
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    for meta in filtered {
                                        if let Some(act) = Self::render_notebook_list_row(ui, meta, p) {
                                            action = Some(act);
                                        }
                                    }
                                }
                            }
                        });
                }

                // --- FOOTER STATS ---
                ui.separator();
                ui.horizontal(|ui| {
                    let total_notes = self.items.len();
                    let total_pages: usize = self.items.iter().map(|m| m.page_count).sum();
                    ui.label(
                        egui::RichText::new(format!("{} Notebooks - {} Total Pages - Storage: {}", total_notes, total_pages, self.library_dir.display()))
                            .size(10.5)
                            .color(p.text_muted),
                    );
                });
            });

        // --- SUB-DIALOG: CREATE NEW NOTEBOOK ---
        if self.show_create_modal {
            let mut create_open = true;
            egui::Window::new("Create New Notebook")
                .open(&mut create_open)
                .resizable(false)
                .collapsible(false)
                .default_size([340.0, 240.0])
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center())
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    ui.label("Notebook Title:");
                    ui.text_edit_singleline(&mut self.new_title);

                    ui.label("Default Paper Template:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.new_paper, PaperType::Ruled, "Ruled");
                        ui.selectable_value(&mut self.new_paper, PaperType::Grid, "Grid");
                        ui.selectable_value(&mut self.new_paper, PaperType::DotGrid, "Dot Grid");
                        ui.selectable_value(&mut self.new_paper, PaperType::Cornell, "Cornell");
                        ui.selectable_value(&mut self.new_paper, PaperType::Blank, "Blank");
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("Create & Open").color(p.on_accent).strong()).clicked() {
                            action = Some(LibraryAction::CreateNew {
                                title: self.new_title.clone(),
                                paper_type: self.new_paper,
                            });
                            self.show_create_modal = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_create_modal = false;
                        }
                    });
                });

            if !create_open {
                self.show_create_modal = false;
            }
        }

        // --- SUB-DIALOG: RENAME NOTEBOOK ---
        if let Some((path, mut name)) = self.renaming_item.take() {
            let mut rename_open = true;
            let mut do_rename = false;
            let mut cancel_rename = false;

            egui::Window::new("Rename Notebook")
                .open(&mut rename_open)
                .resizable(false)
                .collapsible(false)
                .default_size([300.0, 120.0])
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center())
                .show(ctx, |ui| {
                    ui.label("Enter new name:");
                    let resp = ui.text_edit_singleline(&mut name);
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_rename = true;
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            do_rename = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_rename = true;
                        }
                    });
                });

            if cancel_rename {
                rename_open = false;
            }

            if do_rename {
                action = Some(LibraryAction::Rename {
                    path: path.clone(),
                    new_title: name.clone(),
                });
            } else if rename_open {
                self.renaming_item = Some((path, name));
            }
        }

        // --- SUB-DIALOG: DELETE CONFIRMATION ---
        if let Some(del_path) = self.delete_confirm.take() {
            let mut del_open = true;
            let mut do_delete = false;
            let mut cancel_del = false;

            egui::Window::new("Delete Notebook?")
                .open(&mut del_open)
                .resizable(false)
                .collapsible(false)
                .default_size([320.0, 130.0])
                .pivot(egui::Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center())
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete '{}'? This action cannot be undone.",
                        del_path.file_name().and_then(|s| s.to_str()).unwrap_or("this note")
                    ));

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let btn = egui::Button::new(egui::RichText::new("Delete Forever").color(Color32::WHITE))
                            .fill(Color32::from_rgb(190, 40, 40));
                        if ui.add(btn).clicked() {
                            do_delete = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_del = true;
                        }
                    });
                });

            if cancel_del {
                del_open = false;
            }

            if do_delete {
                action = Some(LibraryAction::Delete(del_path));
            } else if del_open {
                self.delete_confirm = Some(del_path);
            }
        }

        self.is_open = is_open;
        action
    }

    /// Renders an authentic realistic notebook card in the bookshelf grid.
    fn render_notebook_card(
        ui: &mut Ui,
        meta: &NotebookMeta,
        w: f32,
        h: f32,
        _p: &Palette,
        is_dark: bool,
    ) -> Option<LibraryAction> {
        let mut action: Option<LibraryAction> = None;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());

        if resp.clicked() {
            action = Some(LibraryAction::Open(meta.path.clone()));
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let (cover_main, cover_spine) = meta.cover_palette(is_dark);

            // Card shadow / elevation
            if resp.hovered() {
                painter.rect_filled(
                    rect.translate(Vec2::new(0.0, 4.0)),
                    Rounding::same(8.0),
                    Color32::from_black_alpha(45),
                );
            }

            // Outer cover with rounded corners
            painter.rect_filled(rect, Rounding::same(8.0), cover_main);

            // Leather spine on the left
            let spine_w = 16.0;
            let spine_rect = Rect::from_min_size(rect.min, Vec2::new(spine_w, h));
            painter.rect_filled(
                spine_rect,
                Rounding {
                    nw: 8.0,
                    sw: 8.0,
                    ne: 0.0,
                    se: 0.0,
                },
                cover_spine,
            );

            // Spine stitch line
            painter.vline(
                rect.min.x + spine_w,
                rect.y_range(),
                Stroke::new(1.0_f32, Color32::from_white_alpha(30)),
            );

            // Bookmark ribbon hanging from top right (Inkk signature brick red)
            let ribbon_x = rect.max.x - 24.0;
            let ribbon_h = 28.0;
            let ribbon_rect = Rect::from_min_size(Pos2::new(ribbon_x, rect.min.y), Vec2::new(10.0, ribbon_h));
            painter.rect_filled(ribbon_rect, Rounding::ZERO, Color32::from_rgb(181, 65, 46));
            // Notch at end of ribbon
            let triangle = vec![
                Pos2::new(ribbon_x, rect.min.y + ribbon_h),
                Pos2::new(ribbon_x + 10.0, rect.min.y + ribbon_h),
                Pos2::new(ribbon_x + 5.0, rect.min.y + ribbon_h - 4.0),
            ];
            painter.add(egui::Shape::convex_polygon(triangle, cover_main, Stroke::NONE));

            // Inner paper inset preview
            let paper_inset = Rect::from_min_max(
                Pos2::new(rect.min.x + 24.0, rect.min.y + 16.0),
                Pos2::new(rect.max.x - 12.0, rect.min.y + 115.0),
            );
            let paper_bg = if is_dark {
                Color32::from_rgb(38, 35, 30)
            } else {
                Color32::from_rgb(250, 246, 238)
            };
            painter.rect_filled(paper_inset, Rounding::same(4.0), paper_bg);
            painter.rect_stroke(paper_inset, Rounding::same(4.0), Stroke::new(1.0_f32, Color32::from_black_alpha(25)));

            // Ruling / Grid lines inside paper preview
            let line_color = if is_dark {
                Color32::from_rgb(60, 55, 48)
            } else {
                Color32::from_rgb(228, 220, 208)
            };
            match meta.paper_type {
                PaperType::Ruled | PaperType::Cornell => {
                    let mut y = paper_inset.min.y + 14.0;
                    while y < paper_inset.max.y - 4.0 {
                        painter.hline(paper_inset.x_range(), y, Stroke::new(1.0_f32, line_color));
                        y += 14.0;
                    }
                    if meta.paper_type == PaperType::Ruled {
                        // Red margin line
                        painter.vline(paper_inset.min.x + 16.0, paper_inset.y_range(), Stroke::new(1.0_f32, Color32::from_rgb(220, 100, 90)));
                    }
                }
                PaperType::Grid => {
                    let mut x = paper_inset.min.x + 12.0;
                    while x < paper_inset.max.x {
                        painter.vline(x, paper_inset.y_range(), Stroke::new(0.8_f32, line_color));
                        x += 12.0;
                    }
                    let mut y = paper_inset.min.y + 12.0;
                    while y < paper_inset.max.y {
                        painter.hline(paper_inset.x_range(), y, Stroke::new(0.8_f32, line_color));
                        y += 12.0;
                    }
                }
                PaperType::DotGrid => {
                    let mut x = paper_inset.min.x + 14.0;
                    while x < paper_inset.max.x {
                        let mut y = paper_inset.min.y + 14.0;
                        while y < paper_inset.max.y {
                            painter.circle_filled(Pos2::new(x, y), 1.2, line_color);
                            y += 14.0;
                        }
                        x += 14.0;
                    }
                }
                PaperType::Blank => {}
            }

            // Active Notebook Indicator badge
            if meta.is_active {
                let badge_rect = Rect::from_min_size(Pos2::new(paper_inset.min.x + 6.0, paper_inset.min.y + 6.0), Vec2::new(60.0, 16.0));
                painter.rect_filled(badge_rect, Rounding::same(8.0), Color32::from_rgb(34, 197, 94));
                painter.text(
                    badge_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "ACTIVE",
                    egui::FontId::proportional(9.0),
                    Color32::WHITE,
                );
            }

            // Title Banner
            let title_pos = Pos2::new(rect.min.x + 24.0, rect.min.y + 126.0);
            let mut title_trimmed = meta.title.clone();
            if title_trimmed.len() > 24 {
                title_trimmed.truncate(22);
                title_trimmed.push_str("...");
            }
            painter.text(
                title_pos,
                egui::Align2::LEFT_TOP,
                &title_trimmed,
                crate::theme::font_serif(13.0),
                Color32::WHITE,
            );

            // Subtext: Pages & Modified Time
            let sub_text = format!("{} page(s) - {}", meta.page_count, meta.format_modified());
            painter.text(
                Pos2::new(rect.min.x + 24.0, rect.min.y + 146.0),
                egui::Align2::LEFT_TOP,
                &sub_text,
                egui::FontId::proportional(10.5),
                Color32::from_white_alpha(175),
            );

            // Hover action pill strip at the bottom
            let actions_rect = Rect::from_min_max(
                Pos2::new(rect.min.x + 20.0, rect.max.y - 36.0),
                Pos2::new(rect.max.x - 8.0, rect.max.y - 8.0),
            );

            painter.rect_filled(actions_rect, Rounding::same(4.0), Color32::from_black_alpha(60));

            // Quick actions on card
            let open_txt = if meta.is_active { "Editing" } else { "Open" };
            painter.text(
                Pos2::new(actions_rect.min.x + 18.0, actions_rect.center().y),
                egui::Align2::CENTER_CENTER,
                open_txt,
                egui::FontId::proportional(11.0),
                if meta.is_active { Color32::from_rgb(130, 220, 150) } else { Color32::WHITE },
            );
        }

        // Context menu for card
        resp.context_menu(|ui| {
            if ui.button("Open Notebook").clicked() {
                action = Some(LibraryAction::Open(meta.path.clone()));
                ui.close_menu();
            }
            if ui.button("Duplicate").clicked() {
                action = Some(LibraryAction::Duplicate(meta.path.clone()));
                ui.close_menu();
            }
            if ui.button("Rename").clicked() {
                action = Some(LibraryAction::Rename {
                    path: meta.path.clone(),
                    new_title: meta.title.clone(),
                });
                ui.close_menu();
            }
            if ui.button("Export to PDF...").clicked() {
                action = Some(LibraryAction::ExportPdf(meta.path.clone()));
                ui.close_menu();
            }
            ui.separator();
            if ui.button(egui::RichText::new("Delete").color(Color32::from_rgb(220, 60, 60))).clicked() {
                action = Some(LibraryAction::Delete(meta.path.clone()));
                ui.close_menu();
            }
        });

        action
    }

    /// Renders a list row for compact table view.
    fn render_notebook_list_row(ui: &mut Ui, meta: &NotebookMeta, p: &Palette) -> Option<LibraryAction> {
        let mut action: Option<LibraryAction> = None;
        let row_h = 36.0;
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), Sense::click());

        if resp.clicked() {
            action = Some(LibraryAction::Open(meta.path.clone()));
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let bg = if meta.is_active {
                p.accent_soft
            } else if resp.hovered() {
                p.hover
            } else {
                p.surface_alt
            };
            painter.rect_filled(rect, Rounding::same(4.0), bg);
            painter.rect_stroke(rect, Rounding::same(4.0), Stroke::new(1.0_f32, p.border));

            // Status dot
            let dot_c = Pos2::new(rect.min.x + 16.0, rect.center().y);
            let dot_col = if meta.is_active { Color32::from_rgb(34, 197, 94) } else { p.border };
            painter.circle_filled(dot_c, 4.0, dot_col);

            // Title
            painter.text(
                Pos2::new(rect.min.x + 30.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                &meta.title,
                crate::theme::font_serif(13.0),
                p.text,
            );

            // Pages & Template
            let template_str = match meta.paper_type {
                PaperType::Ruled => "Ruled",
                PaperType::Grid => "Grid",
                PaperType::DotGrid => "Dot Grid",
                PaperType::Cornell => "Cornell",
                PaperType::Blank => "Blank",
            };
            painter.text(
                Pos2::new(rect.min.x + 280.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{} • {} pgs", template_str, meta.page_count),
                egui::FontId::proportional(11.5),
                p.text_muted,
            );

            // Modified
            painter.text(
                Pos2::new(rect.max.x - 140.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                meta.format_modified(),
                egui::FontId::proportional(11.0),
                p.text_muted,
            );

            // Size
            painter.text(
                Pos2::new(rect.max.x - 20.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                meta.format_size(),
                egui::FontId::monospace(10.0),
                p.text_muted,
            );
        }

        resp.context_menu(|ui| {
            if ui.button("Open").clicked() {
                action = Some(LibraryAction::Open(meta.path.clone()));
                ui.close_menu();
            }
            if ui.button("Duplicate").clicked() {
                action = Some(LibraryAction::Duplicate(meta.path.clone()));
                ui.close_menu();
            }
            if ui.button("Rename").clicked() {
                action = Some(LibraryAction::Rename {
                    path: meta.path.clone(),
                    new_title: meta.title.clone(),
                });
                ui.close_menu();
            }
            if ui.button("Export to PDF...").clicked() {
                action = Some(LibraryAction::ExportPdf(meta.path.clone()));
                ui.close_menu();
            }
            ui.separator();
            if ui.button(egui::RichText::new("Delete").color(Color32::from_rgb(220, 60, 60))).clicked() {
                action = Some(LibraryAction::Delete(meta.path.clone()));
                ui.close_menu();
            }
        });

        action
    }
}
