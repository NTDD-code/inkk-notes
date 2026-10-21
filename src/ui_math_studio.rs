//! Inkk MathStudio: MATLAB-style Interactive IDE Panel, Command Window, and Workspace Inspector.
use eframe::egui::{self, Color32, Ui, Vec2};
use crate::math_engine::{CanvasCommand, EvalOutput, MathWorkspace};
use crate::model::{MatrixEntity, Page, PlotCurveData, PlotEntity, TableGridEntity};
use crate::theme::{self, Palette};
use crate::widgets;

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
}

impl Default for MathStudioState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: MathStudioTab::Console,
            script_input: "x = linspace(-2*pi, 2*pi, 200);\ny = sin(x) .* exp(-0.15*x);\nplot(x, y, \"DisplayName\", \"Damped Wave\")".to_string(),
            console_history: vec![
                ("Inkk MathStudio v1.0 [MATLAB-Compatible Engine Ready]".to_string(), false),
                ("Type commands below or click '▶ Run'. Try: plot(1:5) or A = [1 2; 3 4]".to_string(), false),
            ],
            workspace: MathWorkspace::new(),
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
            .default_height(260.0)
            .height_range(160.0..=500.0)
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(palette.surface).inner_margin(10.0))
            .show(ctx, |ui| {
                // Header Bar: Tabs + Close
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("🔬 MathStudio")
                            .strong()
                            .size(15.0)
                            .color(palette.accent),
                    );

                    widgets::divider(ui, palette);

                    let tabs = [
                        (MathStudioTab::Console, "⌨ Command Window"),
                        (MathStudioTab::Workspace, "📊 Workspace"),
                        (MathStudioTab::Libraries, "📚 Libraries"),
                        (MathStudioTab::QuickTools, "📐 Quick Tools"),
                    ];
                    for (tab, label) in tabs {
                        let active = self.active_tab == tab;
                        if widgets::chip(ui, palette, label, active).clicked() {
                            self.active_tab = tab;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::tool_button(ui, palette, theme::icon::CLOSE, false, "Close MathStudio").clicked() {
                            self.is_open = false;
                        }

                        // Presets Dropdown
                        egui::ComboBox::from_id_salt("math_presets")
                            .selected_text("💡 Presets...")
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(false, "Damped Sine Plot").clicked() {
                                    self.script_input = "x = linspace(-2*pi, 2*pi, 200);\ny = sin(x) .* exp(-0.15*x);\nplot(x, y, \"DisplayName\", \"Damped Wave\")".to_string();
                                }
                                if ui.selectable_label(false, "Trigonometric Comparison (hold on)").clicked() {
                                    self.script_input = "x = linspace(-pi, pi, 150);\nplot(x, sin(x), \"DisplayName\", \"sin(x)\");\nhold on;\nplot(x, cos(x), \"DisplayName\", \"cos(x)\")".to_string();
                                }
                                if ui.selectable_label(false, "Matrix Multiplication").clicked() {
                                    self.script_input = "A = [1 2 3; 4 5 6];\nB = [1 0; 0 1; 1 1];\nC = A * B".to_string();
                                }
                                if ui.selectable_label(false, "Cartesian Coordinate Axes").clicked() {
                                    self.script_input = "axes(-10, 10, -5, 5)".to_string();
                                }
                                if ui.selectable_label(false, "Create 4x5 Table Grid").clicked() {
                                    self.script_input = "grid(4, 5, 120, 36)".to_string();
                                }
                            });
                    });
                });

                ui.separator();

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
            // Left Column: Script / Command Input
            cols[0].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Script Editor (Shift+Enter to run)").size(12.0).color(palette.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(egui::RichText::new("▶ Run Script").strong().color(palette.on_accent))
                            .clicked()
                        {
                            run_requested = true;
                        }
                    });
                });

                let editor_h = ui.available_height() - 8.0;
                let text_edit = egui::TextEdit::multiline(&mut self.script_input)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(6)
                    .min_size(Vec2::new(100.0, editor_h));

                let resp = ui.add(text_edit);
                if resp.has_focus() && ui.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::Enter)) {
                    run_requested = true;
                }
            });

            // Right Column: Output Console
            cols[1].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Output Window").size(12.0).color(palette.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Clear Output").clicked() {
                            self.console_history.clear();
                        }
                    });
                });

                let bg = if is_dark { Color32::from_rgb(13, 17, 23) } else { Color32::from_rgb(248, 250, 252) };
                egui::Frame::none()
                    .fill(bg)
                    .rounding(4.0)
                    .inner_margin(8.0)
                    .stroke(egui::Stroke::new(1.0_f32, palette.border))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for (line, is_err) in &self.console_history {
                                    let color = if *is_err {
                                        palette.danger
                                    } else if line.starts_with("Hold mode") || line.starts_with("Inkk") {
                                        palette.accent
                                    } else {
                                        palette.text
                                    };
                                    ui.label(egui::RichText::new(line).monospace().size(12.0).color(color));
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
            ui.label(egui::RichText::new("Workspace Variables").size(13.0).strong().color(palette.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear Variables").clicked() {
                    self.workspace.variables.clear();
                    self.workspace.variables.insert("pi".to_string(), crate::math_engine::MathValue::scalar(std::f64::consts::PI));
                    self.workspace.variables.insert("e".to_string(), crate::math_engine::MathValue::scalar(std::f64::consts::E));
                    self.workspace.variables.insert("Inf".to_string(), crate::math_engine::MathValue::scalar(f64::INFINITY));
                    self.workspace.variables.insert("NaN".to_string(), crate::math_engine::MathValue::scalar(f64::NAN));
                    self.workspace.variables.insert("eps".to_string(), crate::math_engine::MathValue::scalar(f64::EPSILON));
                }
            });
        });

        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("workspace_table")
                .striped(true)
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Name").strong().color(palette.accent));
                    ui.label(egui::RichText::new("Type").strong().color(palette.accent));
                    ui.label(egui::RichText::new("Size").strong().color(palette.accent));
                    ui.label(egui::RichText::new("Value Preview").strong().color(palette.accent));
                    ui.end_row();

                    let mut names: Vec<String> = self.workspace.variables.keys().cloned().collect();
                    names.sort();

                    for name in names {
                        if let Some(val) = self.workspace.variables.get(&name) {
                            ui.label(egui::RichText::new(&name).monospace().strong());
                            ui.label(egui::RichText::new(val.type_name()).color(palette.text_muted));
                            ui.label(egui::RichText::new(val.size_string()).color(palette.text_muted));
                            let preview = val.to_display_string();
                            let first_line = preview.lines().next().unwrap_or("");
                            ui.label(egui::RichText::new(first_line).monospace());
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_libraries_tab(&mut self, ui: &mut Ui, palette: &Palette) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Function Libraries & Custom Functions").size(13.0).strong().color(palette.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Scan & Reload library/").clicked() {
                    let mgr = crate::math_engine::LibraryManager::new("library");
                    let reports = mgr.load_all(&mut self.workspace.evaluator);
                    for rep in reports {
                        for f in rep.files {
                            if f.success {
                                self.console_history.push((format!("Loaded function '{}' from library '{}'", f.function_name.unwrap_or_default(), rep.library_name), false));
                            } else if let Some(err) = f.error {
                                self.console_history.push((format!("Library load error: {}", err), true));
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("libraries_table")
                .striped(true)
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Function").strong().color(palette.accent));
                    ui.label(egui::RichText::new("Type / Library").strong().color(palette.accent));
                    ui.label(egui::RichText::new("Inputs / Outputs").strong().color(palette.accent));
                    ui.label(egui::RichText::new("Help Documentation").strong().color(palette.accent));
                    ui.end_row();

                    let mut funcs = Vec::new();
                    for name in ["sum2", "fact", "f", "rng2", "my_cube"] {
                        if let Some(meta) = self.workspace.describe_function(name) {
                            funcs.push((name.to_string(), meta.clone()));
                        }
                    }

                    if funcs.is_empty() {
                        ui.label(egui::RichText::new("No custom functions registered yet.").italics().color(palette.text_muted));
                        ui.label("");
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                    } else {
                        for (fname, meta) in funcs {
                            ui.label(egui::RichText::new(&fname).monospace().strong());
                            ui.label(egui::RichText::new("User Function").color(palette.text_muted));
                            ui.label(egui::RichText::new(format!("in: {} | out: {}", meta.max_args, meta.num_outputs)).color(palette.text_muted));
                            let help_first_line = meta.help.lines().next().unwrap_or("No help available");
                            ui.label(egui::RichText::new(help_first_line).italics());
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_quick_tools_tab(&mut self, ui: &mut Ui, page: &mut Page, palette: &Palette) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 8.0);

            if ui
                .button(egui::RichText::new("📐 Quick Coordinate Axes (Oxy)").strong())
                .on_hover_text("Insert clean Cartesian coordinate axes [-5..5, -5..5]")
                .clicked()
            {
                self.dispatch_command(CanvasCommand::QuickAxes { x_min: -5.0, x_max: 5.0, y_min: -5.0, y_max: 5.0 }, page);
                self.console_history.push(("Created Quick Coordinate Axes".to_string(), false));
            }

            if ui
                .button(egui::RichText::new("📊 Insert 3x3 Identity Matrix").strong())
                .on_hover_text("Insert matrix A = eye(3)")
                .clicked()
            {
                let data = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]];
                self.dispatch_command(CanvasCommand::AddMatrixBox { matrix: data }, page);
                self.console_history.push(("Created Matrix Box".to_string(), false));
            }

            if ui
                .button(egui::RichText::new("▦ Insert 4x4 Table Grid").strong())
                .on_hover_text("Generate structured 4x4 rows and columns")
                .clicked()
            {
                self.dispatch_command(CanvasCommand::TableGrid { rows: 4, cols: 4, col_width: 120.0, row_height: 38.0 }, page);
                self.console_history.push(("Created 4x4 Table Grid".to_string(), false));
            }

            if ui
                .button(egui::RichText::new("📈 Plot Sine Curve").strong())
                .on_hover_text("Generate standard sine wave plot")
                .clicked()
            {
                let script = "x = linspace(-pi, pi, 100); y = sin(x); plot(x, y, \"DisplayName\", \"sin(x)\")";
                let res = self.workspace.execute(script);
                self.apply_eval_output(res, page);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label(egui::RichText::new("Dọn dẹp / Xóa đồ thị").size(12.0).color(palette.text_muted));

            if ui
                .button(egui::RichText::new("🗑 Xóa tất cả đồ thị (clf)").color(palette.danger))
                .on_hover_text("Xoá toàn bộ các đồ thị hàm số và hệ trục toạ độ trên trang")
                .clicked()
            {
                let count = page.plots.len();
                page.plots.clear();
                self.console_history.push((format!("Đã xoá {} đồ thị", count), false));
            }

            if ui
                .button(egui::RichText::new("🗑 Xóa ma trận & bảng grid").color(palette.danger))
                .on_hover_text("Xoá toàn bộ các ma trận và bảng dữ liệu trên trang")
                .clicked()
            {
                let count = page.matrices.len() + page.grids.len();
                page.matrices.clear();
                page.grids.clear();
                self.console_history.push((format!("Đã xoá {} ma trận / bảng", count), false));
            }
        });
    }

    pub fn execute_script(&mut self, page: &mut Page) {
        let script = self.script_input.clone();
        self.console_history.push((format!(">> {}", script.lines().next().unwrap_or("")), false));

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
            CanvasCommand::PlotCurve { x, y, label, is_hold, color, .. } => {
                let pts: Vec<[f64; 2]> = x.into_iter().zip(y.into_iter()).map(|(a, b)| [a, b]).collect();
                if pts.is_empty() {
                    return;
                }

                let mut x_min = pts[0][0];
                let mut x_max = pts[0][0];
                let mut y_min = pts[0][1];
                let mut y_max = pts[0][1];

                for p in &pts {
                    if p[0] < x_min { x_min = p[0]; }
                    if p[0] > x_max { x_max = p[0]; }
                    if p[1] < y_min { y_min = p[1]; }
                    if p[1] > y_max { y_max = p[1]; }
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
                        id: format!("plot_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()),
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
                    });
                }
            }
            CanvasCommand::QuickAxes { x_min, x_max, y_min, y_max } => {
                let place_x = 100.0f32;
                let place_y = 150.0f32;
                page.plots.push(PlotEntity {
                    id: format!("axes_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()),
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
                });
            }
            CanvasCommand::TableGrid { rows, cols, col_width, row_height } => {
                let place_x = 80.0f32;
                let place_y = 120.0f32;
                let cells = vec![vec!["".to_string(); cols]; rows];
                page.grids.push(TableGridEntity {
                    id: format!("grid_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()),
                    x: place_x,
                    y: place_y,
                    rows,
                    cols,
                    col_width,
                    row_height,
                    cells,
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
                    id: format!("mat_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()),
                    x: place_x,
                    y: place_y,
                    width,
                    height,
                    label: "M =".to_string(),
                    data: matrix,
                });
            }
            CanvasCommand::ClearPlots => {
                let count = page.plots.len();
                page.plots.clear();
                self.console_history.push((format!("Đã xoá {} đồ thị", count), false));
            }
            CanvasCommand::ClearWorkspace => {
                self.workspace.variables.clear();
                self.workspace.variables.insert("pi".to_string(), crate::math_engine::MathValue::scalar(std::f64::consts::PI));
                self.workspace.variables.insert("e".to_string(), crate::math_engine::MathValue::scalar(std::f64::consts::E));
                self.workspace.variables.insert("Inf".to_string(), crate::math_engine::MathValue::scalar(f64::INFINITY));
                self.workspace.variables.insert("NaN".to_string(), crate::math_engine::MathValue::scalar(f64::NAN));
                self.workspace.variables.insert("eps".to_string(), crate::math_engine::MathValue::scalar(f64::EPSILON));
            }
            _ => {}
        }
    }
}
