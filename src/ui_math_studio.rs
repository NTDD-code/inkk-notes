//! Inkk MathStudio: MATLAB-style Interactive IDE Panel, Command Window, and Workspace Inspector.
use crate::math_engine::{CanvasCommand, EvalOutput, MathWorkspace};
use crate::model::{MatrixEntity, Page, PlotCurveData, PlotEntity, TableGridEntity};
use crate::syntax_highlight::syntax_highlight_matlab;
use crate::theme::{self, Palette};
use crate::vector_icon::VectorIcon;
use crate::widgets;
use eframe::egui::{self, Color32, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathStudioTab {
    Console,
    Workspace,
    Libraries,
    QuickTools,
}

pub struct MathStudioState {
    pub is_open: bool,
    pub active_tab: MathStudioTab,
    pub script_input: String,
    pub console_history: Vec<(String, bool)>, // (text, is_error)
    pub workspace: MathWorkspace,
    pub req_open_matrix_dialog: bool,
    pub req_open_axes_dialog: bool,
    pub req_open_table_dialog: bool,
}

impl Default for MathStudioState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: MathStudioTab::Console,
            script_input: "x = linspace(-2*pi, 2*pi, 200);\ny = sin(x) .* exp(-0.15*x);\nplot(x, y, \"DisplayName\", \"Damped Wave\")".to_string(),
            console_history: vec![
                ("MathStudio v1.0 — MATLAB-compatible engine ready".to_string(), false),
                ("Try: plot(1:10) or A = [1 2; 3 4] or type 'help'".to_string(), false),
            ],
            workspace: MathWorkspace::new(),
            req_open_matrix_dialog: false,
            req_open_axes_dialog: false,
            req_open_table_dialog: false,
        }
    }
}

impl MathStudioState {
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Renders the MathStudio dockable bottom panel.
    pub fn render_panel(
        &mut self,
        ctx: &egui::Context,
        page: &mut Page,
        palette: &Palette,
        is_dark: bool,
    ) {
        if !self.is_open {
            return;
        }

        egui::TopBottomPanel::bottom("math_studio_panel")
            .resizable(true)
            .default_height(320.0)
            .height_range(220.0..=560.0)
            .frame(egui::Frame::side_top_panel(&ctx.style())
                .fill(palette.surface)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0)))
            .show(ctx, |ui| {
                // Header: Brand + Tabs + Actions
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;

                    // Brand mark
                    ui.label(
                        egui::RichText::new("MathStudio")
                            .strong()
                            .size(14.0)
                            .color(palette.text),
                    );

                    ui.add_space(8.0);

                    // Tab navigation
                    let tabs = [
                        (MathStudioTab::Console, "Console"),
                        (MathStudioTab::Workspace, "Variables"),
                        (MathStudioTab::Libraries, "Functions"),
                        (MathStudioTab::QuickTools, "Tools"),
                    ];
                    for (tab, label) in tabs {
                        let active = self.active_tab == tab;
                        let bg = if active { palette.accent_soft } else { Color32::TRANSPARENT };
                        let text_color = if active { palette.accent } else { palette.text_muted };
                        let response = ui.add(
                            egui::Button::new(egui::RichText::new(label).size(13.0).color(text_color))
                                .fill(bg)
                                .stroke(egui::Stroke::NONE)
                                .rounding(theme::RADIUS_SMALL)
                                .min_size(Vec2::new(72.0, 26.0))
                        );
                        if response.clicked() {
                            self.active_tab = tab;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button
                        if widgets::tool_button(ui, palette, VectorIcon::Close, false, "Close").clicked() {
                            self.is_open = false;
                        }

                        ui.add_space(4.0);

                        // Examples menu
                        egui::ComboBox::from_id_salt("math_examples")
                            .selected_text(egui::RichText::new("Examples").size(13.0))
                            .width(110.0)
                            .show_ui(ui, |ui| {
                                ui.style_mut().spacing.item_spacing.y = 6.0;
                                if ui.selectable_label(false, "Damped Wave").clicked() {
                                    self.script_input = "x = linspace(-2*pi, 2*pi, 200);\ny = sin(x) .* exp(-0.15*x);\nplot(x, y, \"DisplayName\", \"Damped Wave\")".to_string();
                                }
                                if ui.selectable_label(false, "Trig Functions").clicked() {
                                    self.script_input = "x = linspace(-pi, pi, 150);\nplot(x, sin(x), \"DisplayName\", \"sin(x)\");\nhold on;\nplot(x, cos(x), \"DisplayName\", \"cos(x)\")".to_string();
                                }
                                if ui.selectable_label(false, "Matrix Multiply").clicked() {
                                    self.script_input = "A = [1 2 3; 4 5 6];\nB = [1 0; 0 1; 1 1];\nC = A * B".to_string();
                                }
                                if ui.selectable_label(false, "Coordinate Axes").clicked() {
                                    self.script_input = "axes(-10, 10, -5, 5)".to_string();
                                }
                                if ui.selectable_label(false, "Table Grid").clicked() {
                                    self.script_input = "grid(4, 5, 120, 36)".to_string();
                                }
                            });
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                match self.active_tab {
                    MathStudioTab::Console => self.render_console_tab(ui, page, palette, is_dark),
                    MathStudioTab::Workspace => self.render_workspace_tab(ui, palette),
                    MathStudioTab::Libraries => self.render_libraries_tab(ui, palette),
                    MathStudioTab::QuickTools => self.render_quick_tools_tab(ui, page, palette),
                }
            });
    }

    fn render_console_tab(
        &mut self,
        ui: &mut Ui,
        page: &mut Page,
        palette: &Palette,
        is_dark: bool,
    ) {
        let mut run_requested = false;

        ui.columns(2, |cols| {
            // Left: Editor
            cols[0].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Editor")
                            .size(12.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let run_btn = egui::Button::new(
                            egui::RichText::new("Run")
                                .strong()
                                .color(palette.on_accent),
                        )
                        .fill(palette.accent)
                        .rounding(theme::RADIUS_SMALL)
                        .min_size(Vec2::new(80.0, 28.0));

                        if ui.add(run_btn).on_hover_text("Shift+Enter").clicked() {
                            run_requested = true;
                        }
                    });
                });

                ui.add_space(6.0);

                let editor_h = ui.available_height();
                let editor_bg = if is_dark {
                    Color32::from_rgb(20, 24, 30)
                } else {
                    Color32::from_rgb(252, 252, 253)
                };

                egui::Frame::none()
                    .fill(editor_bg)
                    .rounding(theme::RADIUS_SMALL)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0_f32, palette.border))
                    .show(ui, |ui| {
                        let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                            let mut layout_job =
                                syntax_highlight_matlab(text, ui, palette, is_dark);
                            layout_job.wrap.max_width = wrap_width;
                            ui.fonts(|f| f.layout_job(layout_job))
                        };

                        let text_edit = egui::TextEdit::multiline(&mut self.script_input)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .min_size(Vec2::new(100.0, editor_h - 20.0))
                            .layouter(&mut layouter);

                        let resp = ui.add(text_edit);
                        if resp.has_focus()
                            && ui.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::Enter))
                        {
                            run_requested = true;
                        }
                    });
            });

            // Right: Output
            cols[1].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Output")
                            .size(12.0)
                            .strong()
                            .color(palette.text),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("Clear").size(12.0))
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::new(1.0_f32, palette.border))
                                    .rounding(theme::RADIUS_SMALL)
                                    .min_size(Vec2::new(60.0, 24.0)),
                            )
                            .clicked()
                        {
                            self.console_history.clear();
                        }
                    });
                });

                ui.add_space(6.0);

                let console_bg = if is_dark {
                    Color32::from_rgb(13, 17, 23)
                } else {
                    Color32::from_rgb(248, 250, 252)
                };
                egui::Frame::none()
                    .fill(console_bg)
                    .rounding(theme::RADIUS_SMALL)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0_f32, palette.border))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .max_height(ui.available_height())
                            .show(ui, |ui| {
                                for (line, is_err) in &self.console_history {
                                    let color = if *is_err {
                                        palette.danger
                                    } else if line.starts_with("Hold mode")
                                        || line.starts_with("Inkk")
                                    {
                                        palette.accent
                                    } else {
                                        palette.text
                                    };
                                    ui.label(
                                        egui::RichText::new(line)
                                            .monospace()
                                            .size(11.5)
                                            .color(color),
                                    );
                                }
                            });
                    });
            });
        });

        if run_requested {
            self.execute_script(page);
        }
    }

    fn render_workspace_tab(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Variables")
                    .size(13.0)
                    .strong()
                    .color(palette.text),
            );
            ui.label(
                egui::RichText::new("•")
                    .size(10.0)
                    .color(palette.text_muted),
            );
            ui.label(
                egui::RichText::new(format!("{} items", self.workspace.variables.len()))
                    .size(12.0)
                    .color(palette.text_muted),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let clear_btn = egui::Button::new(
                    egui::RichText::new("Clear All")
                        .size(12.0)
                        .color(palette.danger),
                )
                .fill(Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0_f32, palette.danger))
                .rounding(theme::RADIUS_SMALL)
                .min_size(Vec2::new(80.0, 26.0));

                if ui.add(clear_btn).clicked() {
                    self.workspace.variables.clear();
                    self.workspace.variables.insert(
                        "pi".to_string(),
                        crate::math_engine::MathValue::scalar(std::f64::consts::PI),
                    );
                    self.workspace.variables.insert(
                        "e".to_string(),
                        crate::math_engine::MathValue::scalar(std::f64::consts::E),
                    );
                    self.workspace.variables.insert(
                        "Inf".to_string(),
                        crate::math_engine::MathValue::scalar(f64::INFINITY),
                    );
                    self.workspace.variables.insert(
                        "NaN".to_string(),
                        crate::math_engine::MathValue::scalar(f64::NAN),
                    );
                    self.workspace.variables.insert(
                        "eps".to_string(),
                        crate::math_engine::MathValue::scalar(f64::EPSILON),
                    );
                }
            });
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("workspace_grid")
                .striped(true)
                .spacing([16.0, 8.0])
                .min_col_width(90.0)
                .show(ui, |ui| {
                    // Header
                    ui.label(
                        egui::RichText::new("Name")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("Type")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("Size")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("Value")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.end_row();

                    let mut names: Vec<String> = self.workspace.variables.keys().cloned().collect();
                    names.sort();

                    for name in names {
                        if let Some(val) = self.workspace.variables.get(&name) {
                            ui.label(
                                egui::RichText::new(&name)
                                    .monospace()
                                    .strong()
                                    .size(12.0)
                                    .color(palette.accent),
                            );
                            ui.label(
                                egui::RichText::new(val.type_name())
                                    .size(11.5)
                                    .color(palette.text_muted),
                            );
                            ui.label(
                                egui::RichText::new(val.size_string())
                                    .size(11.5)
                                    .color(palette.text_muted),
                            );
                            let preview = val.to_display_string();
                            let first_line = preview.lines().next().unwrap_or("");
                            ui.label(egui::RichText::new(first_line).monospace().size(11.5));
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_libraries_tab(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Function Library")
                    .size(13.0)
                    .strong()
                    .color(palette.text),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let reload_btn = egui::Button::new(egui::RichText::new("Reload").size(12.0))
                    .fill(palette.accent_soft)
                    .stroke(egui::Stroke::new(1.0_f32, palette.accent))
                    .rounding(theme::RADIUS_SMALL)
                    .min_size(Vec2::new(80.0, 26.0));

                if ui
                    .add(reload_btn)
                    .on_hover_text("Scan library/ folder")
                    .clicked()
                {
                    let mgr = crate::math_engine::LibraryManager::new("library");
                    let reports = mgr.load_all(&mut self.workspace.evaluator);
                    for rep in reports {
                        for f in rep.files {
                            if f.success {
                                self.console_history.push((
                                    format!(
                                        "Loaded '{}' from {}",
                                        f.function_name.unwrap_or_default(),
                                        rep.library_name
                                    ),
                                    false,
                                ));
                            } else if let Some(err) = f.error {
                                self.console_history.push((format!("Error: {}", err), true));
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("functions_grid")
                .striped(true)
                .spacing([16.0, 8.0])
                .min_col_width(110.0)
                .show(ui, |ui| {
                    // Header
                    ui.label(
                        egui::RichText::new("Function")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("Type")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("Signature")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.label(
                        egui::RichText::new("Description")
                            .size(11.0)
                            .strong()
                            .color(palette.text_muted),
                    );
                    ui.end_row();

                    let mut funcs = Vec::new();
                    for name in ["sum2", "fact", "f", "rng2", "my_cube"] {
                        if let Some(meta) = self.workspace.describe_function(name) {
                            funcs.push((name.to_string(), meta.clone()));
                        }
                    }

                    if funcs.is_empty() {
                        ui.vertical(|ui| {
                            ui.add_space(20.0);
                            ui.label(
                                egui::RichText::new("No custom functions loaded")
                                    .italics()
                                    .color(palette.text_muted),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "Place .m files in library/ folder and click Reload",
                                )
                                .size(11.0)
                                .color(palette.text_muted),
                            );
                        });
                    } else {
                        for (fname, meta) in funcs {
                            ui.label(
                                egui::RichText::new(&fname)
                                    .monospace()
                                    .strong()
                                    .size(12.0)
                                    .color(palette.accent),
                            );
                            ui.label(
                                egui::RichText::new("User")
                                    .size(11.5)
                                    .color(palette.text_muted),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} in, {} out",
                                    meta.max_args, meta.num_outputs
                                ))
                                .size(11.5)
                                .color(palette.text_muted),
                            );
                            let help_first_line =
                                meta.help.lines().next().unwrap_or("No description");
                            ui.label(egui::RichText::new(help_first_line).italics().size(11.5));
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_quick_tools_tab(&mut self, ui: &mut Ui, page: &mut Page, palette: &Palette) {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Quick Actions")
                    .size(13.0)
                    .strong()
                    .color(palette.text),
            );
            ui.add_space(10.0);

            // Tool grid
            egui::Grid::new("quick_tools_grid")
                .spacing([12.0, 12.0])
                .min_col_width(160.0)
                .show(ui, |ui| {
                    // Row 1: Insert objects with rich UI & MATLAB Code dialogs
                    if self
                        .vector_tool_card(
                            ui,
                            palette,
                            VectorIcon::Axes,
                            "Axes",
                            "1D / 2D / 3D Coordinate System",
                        )
                        .clicked()
                    {
                        self.req_open_axes_dialog = true;
                    }

                    if self
                        .vector_tool_card(
                            ui,
                            palette,
                            VectorIcon::Matrix,
                            "Matrix",
                            "MATLAB Code & UI Grid Editor",
                        )
                        .clicked()
                    {
                        self.req_open_matrix_dialog = true;
                    }

                    if self
                        .vector_tool_card(
                            ui,
                            palette,
                            VectorIcon::Table,
                            "Table",
                            "Spreadsheet Grid & CSV Editor",
                        )
                        .clicked()
                    {
                        self.req_open_table_dialog = true;
                    }

                    ui.end_row();

                    // Row 2: Clear actions
                    if self
                        .danger_card(ui, palette, "Clear Plots", "Remove all plots")
                        .clicked()
                    {
                        let count = page.plots.len();
                        page.plots.clear();
                        self.console_history
                            .push((format!("Cleared {} plot(s)", count), false));
                    }

                    if self
                        .danger_card(ui, palette, "Clear Tables", "Remove matrices & grids")
                        .clicked()
                    {
                        let count = page.matrices.len() + page.grids.len();
                        page.matrices.clear();
                        page.grids.clear();
                        self.console_history
                            .push((format!("Cleared {} item(s)", count), false));
                    }

                    ui.end_row();
                });
        });
    }

    fn vector_tool_card(
        &self,
        ui: &mut Ui,
        palette: &Palette,
        icon: VectorIcon,
        title: &str,
        desc: &str,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(160.0, 68.0), egui::Sense::click());
        let bg = if response.hovered() {
            palette.hover
        } else {
            palette.surface_alt
        };
        let stroke = egui::Stroke::new(
            1.0_f32,
            if response.hovered() {
                palette.accent
            } else {
                palette.border
            },
        );
        ui.painter().rect(rect, theme::RADIUS_SMALL, bg, stroke);

        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 12.0, rect.top() + 11.0),
            Vec2::splat(18.0),
        );
        icon.paint(ui.painter(), icon_rect, palette.accent, 1.8);

        ui.painter().text(
            egui::pos2(rect.left() + 36.0, rect.top() + 11.0),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::proportional(13.0),
            palette.text,
        );

        ui.painter().text(
            egui::pos2(rect.left() + 12.0, rect.top() + 36.0),
            egui::Align2::LEFT_TOP,
            desc,
            egui::FontId::proportional(11.0),
            palette.text_muted,
        );

        response
    }

    #[allow(dead_code)]
    fn tool_card(&self, ui: &mut Ui, palette: &Palette, title: &str, desc: &str) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(160.0, 68.0), egui::Sense::click());

        let bg = if response.hovered() {
            palette.hover
        } else {
            palette.surface_alt
        };

        ui.painter().rect(
            rect,
            theme::RADIUS_SMALL,
            bg,
            egui::Stroke::new(1.0_f32, palette.border),
        );

        let mut text_rect = rect;
        text_rect.set_left(rect.left() + 12.0);
        text_rect.set_top(rect.top() + 12.0);

        ui.painter().text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::proportional(13.0),
            palette.text,
        );

        text_rect.set_top(rect.top() + 32.0);
        ui.painter().text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            desc,
            egui::FontId::proportional(11.0),
            palette.text_muted,
        );

        response
    }

    fn danger_card(
        &self,
        ui: &mut Ui,
        palette: &Palette,
        title: &str,
        desc: &str,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(160.0, 68.0), egui::Sense::click());

        let bg = if response.hovered() {
            Color32::from_rgba_unmultiplied(
                palette.danger.r(),
                palette.danger.g(),
                palette.danger.b(),
                20,
            )
        } else {
            palette.surface_alt
        };

        let stroke_color = if response.hovered() {
            palette.danger
        } else {
            palette.border
        };

        ui.painter().rect(
            rect,
            theme::RADIUS_SMALL,
            bg,
            egui::Stroke::new(1.0_f32, stroke_color),
        );

        let mut text_rect = rect;
        text_rect.set_left(rect.left() + 12.0);
        text_rect.set_top(rect.top() + 12.0);

        ui.painter().text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            title,
            egui::FontId::proportional(13.0),
            palette.danger,
        );

        text_rect.set_top(rect.top() + 32.0);
        ui.painter().text(
            text_rect.left_top(),
            egui::Align2::LEFT_TOP,
            desc,
            egui::FontId::proportional(11.0),
            palette.text_muted,
        );

        response
    }

    pub fn execute_script(&mut self, page: &mut Page) {
        let script = self.script_input.clone();
        self.console_history
            .push((format!(">> {}", script.lines().next().unwrap_or("")), false));

        let res = self.workspace.execute(&script);
        self.apply_eval_output(res, page);
    }

    fn apply_eval_output(&mut self, res: EvalOutput, page: &mut Page) {
        if !res.text_output.is_empty() {
            for line in res.text_output.lines() {
                self.console_history.push((line.to_string(), false));
            }
        }
        if let Some(err) = res.error {
            self.console_history.push((err, true));
        }

        for cmd in res.canvas_commands {
            self.dispatch_command(cmd, page);
        }
    }

    fn dispatch_command(&mut self, cmd: CanvasCommand, page: &mut Page) {
        match cmd {
            CanvasCommand::PlotCurve {
                x,
                y,
                label,
                is_hold,
                color,
                ..
            } => {
                let pts: Vec<[f64; 2]> = x
                    .into_iter()
                    .zip(y.into_iter())
                    .map(|(a, b)| [a, b])
                    .collect();
                if pts.is_empty() {
                    return;
                }

                let mut x_min = pts[0][0];
                let mut x_max = pts[0][0];
                let mut y_min = pts[0][1];
                let mut y_max = pts[0][1];

                for p in &pts {
                    if p[0] < x_min {
                        x_min = p[0];
                    }
                    if p[0] > x_max {
                        x_max = p[0];
                    }
                    if p[1] < y_min {
                        y_min = p[1];
                    }
                    if p[1] > y_max {
                        y_max = p[1];
                    }
                }

                // Add margin to axis span
                let y_margin = ((y_max - y_min) * 0.1).max(0.1);
                y_min -= y_margin;
                y_max += y_margin;

                // If hold on is active and there's an existing plot, add curve to the existing plot!
                if is_hold && !page.plots.is_empty() {
                    let last_plot = page.plots.last_mut().unwrap();
                    last_plot.x_min = last_plot.x_min.min(x_min);
                    last_plot.x_max = last_plot.x_max.max(x_max);
                    last_plot.y_min = last_plot.y_min.min(y_min);
                    last_plot.y_max = last_plot.y_max.max(y_max);
                    last_plot.curves.push(PlotCurveData {
                        label,
                        color,
                        points: pts,
                    });
                } else {
                    // Create new plot
                    let place_x = 80.0f32;
                    let place_y = 120.0f32 + (page.plots.len() as f32) * 40.0;
                    page.plots.push(PlotEntity {
                        id: format!(
                            "plot_{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_nanos()
                        ),
                        x: place_x,
                        y: place_y,
                        width: 480.0,
                        height: 320.0,
                        x_min,
                        x_max,
                        y_min,
                        y_max,
                        curves: vec![PlotCurveData {
                            label,
                            color,
                            points: pts,
                        }],
                        show_grid: true,
                        show_axes: true,
                        title: "Function Plot".to_string(),
                        ..Default::default()
                    });
                }
            }
            CanvasCommand::QuickAxes {
                x_min,
                x_max,
                y_min,
                y_max,
            } => {
                let place_x = 100.0f32;
                let place_y = 150.0f32;
                page.plots.push(PlotEntity {
                    id: format!(
                        "axes_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ),
                    x: place_x,
                    y: place_y,
                    width: 450.0,
                    height: 320.0,
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                    curves: Vec::new(),
                    show_grid: true,
                    show_axes: true,
                    title: "Coordinate System (Oxy)".to_string(),
                    ..Default::default()
                });
            }
            CanvasCommand::TableGrid {
                rows,
                cols,
                col_width,
                row_height,
            } => {
                let place_x = 80.0f32;
                let place_y = 120.0f32;
                let cells = vec![vec!["".to_string(); cols]; rows];
                page.grids.push(TableGridEntity {
                    id: format!(
                        "grid_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ),
                    x: place_x,
                    y: place_y,
                    rows,
                    cols,
                    col_width,
                    row_height,
                    cells,
                    ..Default::default()
                });
            }
            CanvasCommand::AddMatrixBox { matrix } => {
                let place_x = 120.0f32;
                let place_y = 120.0f32;
                let cols = matrix.first().map(|r| r.len()).unwrap_or(2);
                let rows = matrix.len();
                let width = (cols as f32 * 50.0 + 80.0).max(180.0);
                let height = (rows as f32 * 36.0 + 30.0).max(90.0);

                page.matrices.push(MatrixEntity {
                    id: format!(
                        "mat_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ),
                    x: place_x,
                    y: place_y,
                    width,
                    height,
                    label: "M =".to_string(),
                    data: matrix,
                    ..Default::default()
                });
            }
            CanvasCommand::ClearPlots => {
                page.plots.clear();
            }
            CanvasCommand::SetTitle(t) => {
                if let Some(p) = page.plots.last_mut() {
                    p.title = t;
                }
            }
            CanvasCommand::SetXLabel(_lbl) => {}
            CanvasCommand::SetYLabel(_lbl) => {}
            CanvasCommand::SetGrid(show) => {
                if let Some(p) = page.plots.last_mut() {
                    p.show_grid = show;
                }
            }
            CanvasCommand::SetXLim(min, max) => {
                if let Some(p) = page.plots.last_mut() {
                    p.x_min = min;
                    p.x_max = max;
                }
            }
            CanvasCommand::SetYLim(min, max) => {
                if let Some(p) = page.plots.last_mut() {
                    p.y_min = min;
                    p.y_max = max;
                }
            }
            CanvasCommand::SetLegend(labels) => {
                if let Some(p) = page.plots.last_mut() {
                    for (c, l) in p.curves.iter_mut().zip(labels.into_iter()) {
                        c.label = l;
                    }
                }
            }
            CanvasCommand::SetAxisEqual => {
                if let Some(p) = page.plots.last_mut() {
                    let dx = (p.x_max - p.x_min).abs();
                    let dy = (p.y_max - p.y_min).abs();
                    let max_span = dx.max(dy);
                    let x_mid = (p.x_min + p.x_max) / 2.0;
                    let y_mid = (p.y_min + p.y_max) / 2.0;
                    p.x_min = x_mid - max_span / 2.0;
                    p.x_max = x_mid + max_span / 2.0;
                    p.y_min = y_mid - max_span / 2.0;
                    p.y_max = y_mid + max_span / 2.0;
                }
            }
            CanvasCommand::SetAxisLimits(lims) => {
                if lims.len() >= 4 {
                    if let Some(p) = page.plots.last_mut() {
                        p.x_min = lims[0];
                        p.x_max = lims[1];
                        p.y_min = lims[2];
                        p.y_max = lims[3];
                    }
                }
            }
            CanvasCommand::XLine(val) => {
                if let Some(p) = page.plots.last_mut() {
                    let y_min = p.y_min;
                    let y_max = p.y_max;
                    p.curves.push(PlotCurveData {
                        label: String::new(),
                        color: [120, 120, 120, 255],
                        points: vec![[val, y_min], [val, y_max]],
                    });
                }
            }
            CanvasCommand::YLine(val) => {
                if let Some(p) = page.plots.last_mut() {
                    let x_min = p.x_min;
                    let x_max = p.x_max;
                    p.curves.push(PlotCurveData {
                        label: String::new(),
                        color: [120, 120, 120, 255],
                        points: vec![[x_min, val], [x_max, val]],
                    });
                }
            }
            CanvasCommand::DrawText { x, y, text } => {
                if let Some(p) = page.plots.last_mut() {
                    p.curves.push(PlotCurveData {
                        label: text,
                        color: [0, 0, 0, 255],
                        points: vec![[x, y]],
                    });
                }
            }
            CanvasCommand::Scatter { x, y, is_hold } => {
                let pts: Vec<[f64; 2]> = x
                    .into_iter()
                    .zip(y.into_iter())
                    .map(|(a, b)| [a, b])
                    .collect();
                if !pts.is_empty() {
                    let mut x_min = pts[0][0];
                    let mut x_max = pts[0][0];
                    let mut y_min = pts[0][1];
                    let mut y_max = pts[0][1];
                    for p in &pts {
                        if p[0] < x_min {
                            x_min = p[0];
                        }
                        if p[0] > x_max {
                            x_max = p[0];
                        }
                        if p[1] < y_min {
                            y_min = p[1];
                        }
                        if p[1] > y_max {
                            y_max = p[1];
                        }
                    }
                    let y_margin = ((y_max - y_min) * 0.1).max(0.1);
                    y_min -= y_margin;
                    y_max += y_margin;
                    let x_margin = ((x_max - x_min) * 0.1).max(0.1);
                    x_min -= x_margin;
                    x_max += x_margin;

                    if is_hold && !page.plots.is_empty() {
                        let last = page.plots.last_mut().unwrap();
                        last.x_min = last.x_min.min(x_min);
                        last.x_max = last.x_max.max(x_max);
                        last.y_min = last.y_min.min(y_min);
                        last.y_max = last.y_max.max(y_max);
                        last.curves.push(PlotCurveData {
                            label: "scatter".into(),
                            color: [0, 114, 189, 255],
                            points: pts,
                        });
                    } else {
                        let place_x = 80.0f32;
                        let place_y = 120.0f32 + (page.plots.len() as f32) * 40.0;
                        page.plots.push(PlotEntity {
                            id: format!(
                                "scatter_{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos()
                            ),
                            x: place_x,
                            y: place_y,
                            width: 480.0,
                            height: 320.0,
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                            curves: vec![PlotCurveData {
                                label: "scatter".into(),
                                color: [0, 114, 189, 255],
                                points: pts,
                            }],
                            show_grid: true,
                            show_axes: true,
                            title: "Scatter Plot".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            CanvasCommand::Bar { x, y, is_hold } => {
                let mut pts: Vec<[f64; 2]> = Vec::new();
                for (xi, yi) in x.iter().zip(y.iter()) {
                    let hw = 0.35;
                    pts.push([xi - hw, 0.0]);
                    pts.push([xi - hw, *yi]);
                    pts.push([xi + hw, *yi]);
                    pts.push([xi + hw, 0.0]);
                }
                if !pts.is_empty() {
                    let mut x_min = pts[0][0];
                    let mut x_max = pts[0][0];
                    let mut y_min = pts[0][1];
                    let mut y_max = pts[0][1];
                    for p in &pts {
                        if p[0] < x_min {
                            x_min = p[0];
                        }
                        if p[0] > x_max {
                            x_max = p[0];
                        }
                        if p[1] < y_min {
                            y_min = p[1];
                        }
                        if p[1] > y_max {
                            y_max = p[1];
                        }
                    }
                    let y_min = y_min.min(0.0);
                    let y_max = y_max.max(1.0);
                    let x_min = x_min - 0.5;
                    let x_max = x_max + 0.5;

                    if is_hold && !page.plots.is_empty() {
                        let last = page.plots.last_mut().unwrap();
                        last.x_min = last.x_min.min(x_min);
                        last.x_max = last.x_max.max(x_max);
                        last.y_min = last.y_min.min(y_min);
                        last.y_max = last.y_max.max(y_max);
                        last.curves.push(PlotCurveData {
                            label: "bar".into(),
                            color: [0, 114, 189, 255],
                            points: pts,
                        });
                    } else {
                        let place_x = 80.0f32;
                        let place_y = 120.0f32 + (page.plots.len() as f32) * 40.0;
                        page.plots.push(PlotEntity {
                            id: format!(
                                "bar_{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos()
                            ),
                            x: place_x,
                            y: place_y,
                            width: 480.0,
                            height: 320.0,
                            x_min,
                            x_max,
                            y_min,
                            y_max,
                            curves: vec![PlotCurveData {
                                label: "bar".into(),
                                color: [0, 114, 189, 255],
                                points: pts,
                            }],
                            show_grid: true,
                            show_axes: true,
                            title: "Bar Chart".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            CanvasCommand::Hist { v, nbins, is_hold } => {
                if !v.is_empty() {
                    let mut min_val = v[0];
                    let mut max_val = v[0];
                    for &val in &v {
                        if val < min_val {
                            min_val = val;
                        }
                        if val > max_val {
                            max_val = val;
                        }
                    }
                    let span = (max_val - min_val).max(1e-5);
                    let bin_width = span / nbins as f64;
                    let mut counts = vec![0.0f64; nbins];
                    for &val in &v {
                        let mut b = ((val - min_val) / bin_width).floor() as usize;
                        if b >= nbins {
                            b = nbins - 1;
                        }
                        counts[b] += 1.0;
                    }

                    let mut pts: Vec<[f64; 2]> = Vec::new();
                    for (b, &count) in counts.iter().enumerate() {
                        let left = min_val + b as f64 * bin_width;
                        let right = left + bin_width;
                        pts.push([left, 0.0]);
                        pts.push([left, count]);
                        pts.push([right, count]);
                        pts.push([right, 0.0]);
                    }
                    let max_count = counts.iter().cloned().fold(0.0f64, f64::max).max(1.0);

                    if is_hold && !page.plots.is_empty() {
                        let last = page.plots.last_mut().unwrap();
                        last.x_min = last.x_min.min(min_val);
                        last.x_max = last.x_max.max(max_val);
                        last.y_min = 0.0;
                        last.y_max = last.y_max.max(max_count * 1.1);
                        last.curves.push(PlotCurveData {
                            label: "hist".into(),
                            color: [0, 114, 189, 255],
                            points: pts,
                        });
                    } else {
                        let place_x = 80.0f32;
                        let place_y = 120.0f32 + (page.plots.len() as f32) * 40.0;
                        page.plots.push(PlotEntity {
                            id: format!(
                                "hist_{}",
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos()
                            ),
                            x: place_x,
                            y: place_y,
                            width: 480.0,
                            height: 320.0,
                            x_min: min_val - bin_width * 0.2,
                            x_max: max_val + bin_width * 0.2,
                            y_min: 0.0,
                            y_max: max_count * 1.1,
                            curves: vec![PlotCurveData {
                                label: "hist".into(),
                                color: [0, 114, 189, 255],
                                points: pts,
                            }],
                            show_grid: true,
                            show_axes: true,
                            title: "Histogram".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            CanvasCommand::Pie { v } => {
                let place_x = 80.0f32;
                let place_y = 120.0f32 + (page.plots.len() as f32) * 40.0;
                let total: f64 = v.iter().sum();
                let mut curves = Vec::new();
                let mut start_angle = 0.0f64;
                for (idx, &val) in v.iter().enumerate() {
                    let slice_angle = if total > 0.0 {
                        (val / total) * 2.0 * std::f64::consts::PI
                    } else {
                        0.0
                    };
                    let mut pts = vec![[0.0, 0.0]];
                    let steps = 30;
                    for s in 0..=steps {
                        let a = start_angle + (s as f64 / steps as f64) * slice_angle;
                        pts.push([a.cos(), a.sin()]);
                    }
                    pts.push([0.0, 0.0]);
                    start_angle += slice_angle;
                    curves.push(PlotCurveData {
                        label: format!("Slice {}", idx + 1),
                        color: [0, 0, 0, 0],
                        points: pts,
                    });
                }
                page.plots.push(PlotEntity {
                    id: format!(
                        "pie_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                    ),
                    x: place_x,
                    y: place_y,
                    width: 400.0,
                    height: 320.0,
                    x_min: -1.2,
                    x_max: 1.2,
                    y_min: -1.2,
                    y_max: 1.2,
                    curves,
                    show_grid: false,
                    show_axes: false,
                    title: "Pie Chart".to_string(),
                    ..Default::default()
                });
            }
            CanvasCommand::ClearWorkspace => {
                self.workspace.variables.clear();
                self.workspace.variables.insert(
                    "pi".to_string(),
                    crate::math_engine::MathValue::scalar(std::f64::consts::PI),
                );
                self.workspace.variables.insert(
                    "e".to_string(),
                    crate::math_engine::MathValue::scalar(std::f64::consts::E),
                );
                self.workspace.variables.insert(
                    "Inf".to_string(),
                    crate::math_engine::MathValue::scalar(f64::INFINITY),
                );
                self.workspace.variables.insert(
                    "NaN".to_string(),
                    crate::math_engine::MathValue::scalar(f64::NAN),
                );
                self.workspace.variables.insert(
                    "eps".to_string(),
                    crate::math_engine::MathValue::scalar(f64::EPSILON),
                );
            }
        }
    }
}
