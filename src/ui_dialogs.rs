//! Modal interactive dialogs for generating and editing Matrices, Coordinate Axes, and Tables.
use crate::math_engine::{MathValue, MathWorkspace};
use crate::model::{MatrixEntity, Page, PlotEntity, TableGridEntity};
use crate::theme::Palette;
use eframe::egui::{self, Vec2};

// ============================================================================
// 1. Matrix Generator & Editor Dialog
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatrixInputMode {
    Code,
    UI,
}

pub struct MatrixDialogState {
    pub is_open: bool,
    pub mode: MatrixInputMode,

    // Code mode
    pub code_input: String,

    // UI mode
    pub rows: usize,
    pub cols: usize,
    pub cell_values: Vec<Vec<String>>,
    pub matrix_name: String,

    // Edit target
    pub editing_id: Option<String>,
    pub error_msg: Option<String>,
}

impl Default for MatrixDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: MatrixInputMode::UI,
            code_input: "[1 0 0; 0 1 0; 0 0 1]".to_string(),
            rows: 3,
            cols: 3,
            cell_values: vec![
                vec!["1".into(), "0".into(), "0".into()],
                vec!["0".into(), "1".into(), "0".into()],
                vec!["0".into(), "0".into(), "1".into()],
            ],
            matrix_name: "A".to_string(),
            editing_id: None,
            error_msg: None,
        }
    }
}

impl MatrixDialogState {
    pub fn open_new(&mut self) {
        self.is_open = true;
        self.editing_id = None;
        self.error_msg = None;
        self.ensure_grid_size();
    }

    pub fn open_edit(&mut self, matrix: &MatrixEntity) {
        self.is_open = true;
        self.editing_id = Some(matrix.id.clone());
        self.error_msg = None;
        self.rows = matrix.data.len().clamp(1, 12);
        self.cols = matrix
            .data
            .first()
            .map(|r| r.len())
            .unwrap_or(1)
            .clamp(1, 12);
        self.matrix_name = matrix.label.trim_end_matches([' ', '=']).to_string();

        self.cell_values = matrix
            .data
            .iter()
            .map(|row| {
                row.iter()
                    .map(|v| {
                        if v.fract() == 0.0 {
                            format!("{:.0}", v)
                        } else {
                            format!("{:.3}", v)
                        }
                    })
                    .collect()
            })
            .collect();

        if let Some(ref code) = matrix.source_code {
            self.code_input = code.clone();
        } else {
            self.code_input = Self::format_as_matlab(&matrix.data, &self.matrix_name);
        }

        self.ensure_grid_size();
    }

    fn ensure_grid_size(&mut self) {
        self.rows = self.rows.clamp(1, 12);
        self.cols = self.cols.clamp(1, 12);

        if self.cell_values.len() < self.rows {
            self.cell_values
                .resize(self.rows, vec!["".into(); self.cols]);
        } else if self.cell_values.len() > self.rows {
            self.cell_values.truncate(self.rows);
        }

        for row in &mut self.cell_values {
            if row.len() < self.cols {
                row.resize(self.cols, "".into());
            } else if row.len() > self.cols {
                row.truncate(self.cols);
            }
        }
    }

    pub fn format_as_matlab(data: &[Vec<f64>], name: &str) -> String {
        let mut out = if name.is_empty() {
            "[".to_string()
        } else {
            format!("{} = [", name)
        };

        for (r_idx, row) in data.iter().enumerate() {
            if r_idx > 0 {
                out.push_str("; ");
            }
            for (c_idx, val) in row.iter().enumerate() {
                if c_idx > 0 {
                    out.push(' ');
                }
                if val.fract() == 0.0 {
                    out.push_str(&format!("{:.0}", val));
                } else {
                    out.push_str(&format!("{:.2}", val));
                }
            }
        }
        out.push(']');
        out
    }

    /// Renders the Matrix Generator & Editor modal dialog window.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        page: &mut Page,
        workspace: &mut MathWorkspace,
        palette: &Palette,
    ) {
        if !self.is_open {
            return;
        }

        let title = if self.editing_id.is_some() {
            "✏️ Edit Matrix"
        } else {
            "📊 Insert Matrix"
        };

        let mut is_open = self.is_open;
        egui::Window::new(title)
            .open(&mut is_open)
            .resizable(true)
            .default_size([460.0, 420.0])
            .min_size([380.0, 320.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.mode, MatrixInputMode::UI, "🎛️ UI (Grid Inputs)");
                    ui.selectable_value(
                        &mut self.mode,
                        MatrixInputMode::Code,
                        "💻 Code (MATLAB Syntax)",
                    );
                });
                ui.separator();

                match self.mode {
                    MatrixInputMode::UI => {
                        ui.horizontal(|ui| {
                            ui.label("Name / Label:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.matrix_name)
                                    .desired_width(60.0)
                                    .hint_text("A"),
                            );

                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Rows:").strong());
                            let prev_r = self.rows;
                            if ui
                                .add(egui::DragValue::new(&mut self.rows).range(1..=10))
                                .changed()
                            {
                                if self.rows != prev_r {
                                    self.ensure_grid_size();
                                }
                            }

                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Cols:").strong());
                            let prev_c = self.cols;
                            if ui
                                .add(egui::DragValue::new(&mut self.cols).range(1..=10))
                                .changed()
                            {
                                if self.cols != prev_c {
                                    self.ensure_grid_size();
                                }
                            }
                        });

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("💡 Enter cell values (empty cells default to 0):")
                                .small()
                                .color(palette.text_muted),
                        );
                        ui.add_space(6.0);

                        self.ensure_grid_size();

                        egui::ScrollArea::both().max_height(200.0).show(ui, |ui| {
                            egui::Grid::new("matrix_ui_cells_grid")
                                .spacing([8.0, 8.0])
                                .show(ui, |ui| {
                                    for r in 0..self.rows {
                                        ui.label(
                                            egui::RichText::new(format!("R{}:", r + 1)).weak(),
                                        );
                                        for c in 0..self.cols {
                                            let cell = &mut self.cell_values[r][c];
                                            ui.add(
                                                egui::TextEdit::singleline(cell)
                                                    .desired_width(54.0)
                                                    .hint_text("0"),
                                            );
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                    MatrixInputMode::Code => {
                        ui.label(
                            "Enter MATLAB matrix syntax (e.g., [1 2 3; 4 5 6] or A = eye(3)):",
                        );
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .max_height(180.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.code_input)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_rows(6)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("[1 2 3; 4 5 6; 7 8 9]"),
                                );
                            });

                        ui.horizontal(|ui| {
                            if ui.button("3×3 Identity").clicked() {
                                self.code_input = "[1 0 0; 0 1 0; 0 0 1]".to_string();
                            }
                            if ui.button("2×2 Rotation").clicked() {
                                self.code_input =
                                    "[cos(pi/4) -sin(pi/4); sin(pi/4) cos(pi/4)]".to_string();
                            }
                            if ui.button("3×3 Magic").clicked() {
                                self.code_input = "magic(3)".to_string();
                            }
                        });
                    }
                }

                if let Some(ref err) = self.error_msg {
                    ui.add_space(4.0);
                    ui.colored_label(palette.danger, format!("⚠️ Error: {}", err));
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let export_label = if self.editing_id.is_some() {
                        "💾 Update on Canvas"
                    } else {
                        "🚀 Export to Canvas"
                    };

                    let btn = ui.add(
                        egui::Button::new(egui::RichText::new(export_label).strong().size(13.0))
                            .fill(palette.accent)
                            .min_size(Vec2::new(160.0, 30.0)),
                    );

                    if btn.clicked() {
                        match self.execute_export(page, workspace) {
                            Ok(_) => {
                                self.is_open = false;
                                self.error_msg = None;
                            }
                            Err(e) => {
                                self.error_msg = Some(e);
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        self.is_open = is_open;
    }

    fn execute_export(
        &mut self,
        page: &mut Page,
        workspace: &mut MathWorkspace,
    ) -> Result<(), String> {
        let (data, label, source_code) = match self.mode {
            MatrixInputMode::UI => {
                let mut data = Vec::with_capacity(self.rows);
                for r in 0..self.rows {
                    let mut row = Vec::with_capacity(self.cols);
                    for c in 0..self.cols {
                        let text = self.cell_values[r][c].trim();
                        let val: f64 = if text.is_empty() {
                            0.0 // Default to 0 as requested!
                        } else {
                            text.parse::<f64>().unwrap_or(0.0)
                        };
                        row.push(val);
                    }
                    data.push(row);
                }
                let lbl = if self.matrix_name.trim().is_empty() {
                    String::new()
                } else {
                    format!("{} =", self.matrix_name.trim())
                };
                let code = Self::format_as_matlab(&data, &self.matrix_name);
                (data, lbl, code)
            }
            MatrixInputMode::Code => {
                let script = self.code_input.trim();
                if script.is_empty() {
                    return Err("Please enter matrix code!".into());
                }

                let mut temp_ws = workspace.clone();
                let output = temp_ws.execute(script);
                if let Some(err) = output.error {
                    return Err(err);
                }

                let builtins = ["pi", "e", "Inf", "NaN", "eps"];
                let mut found_matrix: Option<(Vec<Vec<f64>>, String)> = None;

                // Check user defined variables (not in builtins, not ans)
                for (name, val) in &temp_ws.variables {
                    if !builtins.contains(&name.as_str()) && name != "ans" {
                        if let MathValue::Matrix { .. } = val {
                            found_matrix = Some((val.to_2d_vec(), format!("{} =", name)));
                            break;
                        }
                    }
                }

                // Fallback to ans
                if found_matrix.is_none() {
                    if let Some(ans_val) = temp_ws.variables.get("ans") {
                        if let MathValue::Matrix { .. } = ans_val {
                            found_matrix = Some((ans_val.to_2d_vec(), String::new()));
                        }
                    }
                }

                let (mat_data, lbl) = found_matrix.ok_or_else(|| {
                    "Execution finished but did not produce a valid matrix.".to_string()
                })?;

                (mat_data, lbl, script.to_string())
            }
        };

        let rows = data.len();
        let cols = data.first().map(|r| r.len()).unwrap_or(1);
        let width = (cols as f32 * 56.0 + 80.0).max(180.0);
        let height = (rows as f32 * 38.0 + 36.0).max(90.0);

        if let Some(ref edit_id) = self.editing_id {
            if let Some(mat) = page.matrices.iter_mut().find(|m| m.id == *edit_id) {
                mat.data = data;
                mat.label = label;
                mat.width = width;
                mat.height = height;
                mat.source_code = Some(source_code);
                return Ok(());
            }
        }

        // New matrix placement
        let place_x = 100.0f32;
        let place_y = 120.0f32 + (page.matrices.len() as f32) * 50.0;
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
            label,
            data,
            source_code: Some(source_code),
        });

        Ok(())
    }
}

// ============================================================================
// 2. Coordinate Axes Generator & Editor Dialog (1D, 2D, 3D)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxesDimension {
    D1, // 1D Number Line
    D2, // 2D Cartesian Oxy
    D3, // 3D Cartesian Oxyz
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxesInputMode {
    UI,
    Code,
}

pub struct AxesDialogState {
    pub is_open: bool,
    pub dimension: AxesDimension,
    pub mode: AxesInputMode,

    // 1D, 2D, 3D Parameters
    pub x_min: f64,
    pub x_max: f64,
    pub x_step: f64,
    pub x_label: String,

    pub y_min: f64,
    pub y_max: f64,
    pub y_step: f64,
    pub y_label: String,

    pub z_min: f64,
    pub z_max: f64,
    pub z_step: f64,
    pub z_label: String,

    pub show_grid: bool,
    pub title: String,

    // Code mode
    pub code_input: String,

    // Edit target
    pub editing_id: Option<String>,
    pub error_msg: Option<String>,
}

impl Default for AxesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            dimension: AxesDimension::D2,
            mode: AxesInputMode::UI,
            x_min: -10.0,
            x_max: 10.0,
            x_step: 2.0,
            x_label: "x".into(),

            y_min: -5.0,
            y_max: 5.0,
            y_step: 1.0,
            y_label: "y".into(),

            z_min: -5.0,
            z_max: 5.0,
            z_step: 1.0,
            z_label: "z".into(),

            show_grid: true,
            title: "Coordinate System".into(),

            code_input: "axes(-10:2:10, -5:1:5)".into(),
            editing_id: None,
            error_msg: None,
        }
    }
}

impl AxesDialogState {
    pub fn open_new(&mut self) {
        self.is_open = true;
        self.editing_id = None;
        self.error_msg = None;
    }

    pub fn open_edit(&mut self, plot: &PlotEntity) {
        self.is_open = true;
        self.editing_id = Some(plot.id.clone());
        self.error_msg = None;
        self.x_min = plot.x_min;
        self.x_max = plot.x_max;
        self.y_min = plot.y_min;
        self.y_max = plot.y_max;
        self.x_step = plot.x_step.unwrap_or(2.0);
        self.y_step = plot.y_step.unwrap_or(1.0);
        self.show_grid = plot.show_grid;
        self.title = plot.title.clone();

        if let Some(ref dim) = plot.dimension {
            match dim.as_str() {
                "1D" => self.dimension = AxesDimension::D1,
                "3D" => self.dimension = AxesDimension::D3,
                _ => self.dimension = AxesDimension::D2,
            }
        } else {
            self.dimension = AxesDimension::D2;
        }

        if let Some(ref code) = plot.source_code {
            self.code_input = code.clone();
        }
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        page: &mut Page,
        workspace: &mut MathWorkspace,
        palette: &Palette,
    ) {
        if !self.is_open {
            return;
        }

        let win_title = if self.editing_id.is_some() {
            "✏️ Edit Coordinate Axes"
        } else {
            "📐 Coordinate Axes (1D / 2D / 3D)"
        };

        let mut is_open = self.is_open;
        egui::Window::new(win_title)
            .open(&mut is_open)
            .resizable(true)
            .default_size([480.0, 440.0])
            .min_size([400.0, 360.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.mode, AxesInputMode::UI, "🎛️ UI (Visual)");
                    ui.selectable_value(&mut self.mode, AxesInputMode::Code, "💻 Code (MATLAB)");
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Dimension:");
                    ui.selectable_value(
                        &mut self.dimension,
                        AxesDimension::D1,
                        "📏 1D (Number Line)",
                    );
                    ui.selectable_value(&mut self.dimension, AxesDimension::D2, "📐 2D (Oxy)");
                    ui.selectable_value(&mut self.dimension, AxesDimension::D3, "🧊 3D (Oxyz)");
                });
                ui.separator();

                match self.mode {
                    AxesInputMode::UI => {
                        ui.horizontal(|ui| {
                            ui.label("Title:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.title).desired_width(260.0),
                            );
                        });
                        ui.add_space(4.0);

                        match self.dimension {
                            AxesDimension::D1 => {
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("📏 1D Number Line").strong());
                                    ui.horizontal(|ui| {
                                        ui.label("X Min:");
                                        ui.add(egui::DragValue::new(&mut self.x_min).speed(0.5));
                                        ui.label("X Max:");
                                        ui.add(egui::DragValue::new(&mut self.x_max).speed(0.5));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Step:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.x_step)
                                                .speed(0.1)
                                                .range(0.01..=100.0),
                                        );
                                        ui.label("Axis Label:");
                                        ui.add(
                                            egui::TextEdit::singleline(&mut self.x_label)
                                                .desired_width(50.0),
                                        );
                                    });
                                });
                            }
                            AxesDimension::D2 => {
                                ui.group(|ui| {
                                    ui.label(
                                        egui::RichText::new("📐 Horizontal Axis (X)").strong(),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label("X Min:");
                                        ui.add(egui::DragValue::new(&mut self.x_min).speed(0.5));
                                        ui.label("X Max:");
                                        ui.add(egui::DragValue::new(&mut self.x_max).speed(0.5));
                                        ui.label("Step:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.x_step)
                                                .speed(0.1)
                                                .range(0.01..=100.0),
                                        );
                                    });
                                });

                                ui.group(|ui| {
                                    ui.label(egui::RichText::new("📐 Vertical Axis (Y)").strong());
                                    ui.horizontal(|ui| {
                                        ui.label("Y Min:");
                                        ui.add(egui::DragValue::new(&mut self.y_min).speed(0.5));
                                        ui.label("Y Max:");
                                        ui.add(egui::DragValue::new(&mut self.y_max).speed(0.5));
                                        ui.label("Step:");
                                        ui.add(
                                            egui::DragValue::new(&mut self.y_step)
                                                .speed(0.1)
                                                .range(0.01..=100.0),
                                        );
                                    });
                                });

                                ui.checkbox(&mut self.show_grid, "Show Coordinate Grid");
                            }
                            AxesDimension::D3 => {
                                ui.group(|ui| {
                                    ui.label(
                                        egui::RichText::new("🧊 3D Coordinate Space (Oxyz)")
                                            .strong(),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label("X Min:");
                                        ui.add(egui::DragValue::new(&mut self.x_min).speed(0.5));
                                        ui.label("X Max:");
                                        ui.add(egui::DragValue::new(&mut self.x_max).speed(0.5));
                                        ui.label("X Step:");
                                        ui.add(egui::DragValue::new(&mut self.x_step).speed(0.1));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Y Min:");
                                        ui.add(egui::DragValue::new(&mut self.y_min).speed(0.5));
                                        ui.label("Y Max:");
                                        ui.add(egui::DragValue::new(&mut self.y_max).speed(0.5));
                                        ui.label("Y Step:");
                                        ui.add(egui::DragValue::new(&mut self.y_step).speed(0.1));
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Z Min:");
                                        ui.add(egui::DragValue::new(&mut self.z_min).speed(0.5));
                                        ui.label("Z Max:");
                                        ui.add(egui::DragValue::new(&mut self.z_max).speed(0.5));
                                        ui.label("Z Step:");
                                        ui.add(egui::DragValue::new(&mut self.z_step).speed(0.1));
                                    });
                                });

                                ui.checkbox(&mut self.show_grid, "Show Ground Grid");
                            }
                        }
                    }
                    AxesInputMode::Code => {
                        ui.label("Enter axes commands (MATLAB style):");
                        ui.add_space(4.0);

                        ui.add(
                            egui::TextEdit::multiline(&mut self.code_input)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(5)
                                .desired_width(f32::INFINITY)
                                .hint_text("axis([-10 10 -5 5]) or axes(-10:2:10, -5:1:5)"),
                        );

                        ui.horizontal(|ui| {
                            if ui.button("1D: -10:10 (step 1)").clicked() {
                                self.dimension = AxesDimension::D1;
                                self.code_input = "axis([-10 10])".into();
                            }
                            if ui.button("2D: -10:10, -5:5").clicked() {
                                self.dimension = AxesDimension::D2;
                                self.code_input = "axis([-10 10 -5 5])".into();
                            }
                            if ui.button("3D: -5:5, -5:5, -5:5").clicked() {
                                self.dimension = AxesDimension::D3;
                                self.code_input = "axis([-5 5 -5 5 -5 5])".into();
                            }
                        });
                    }
                }

                if let Some(ref err) = self.error_msg {
                    ui.add_space(4.0);
                    ui.colored_label(palette.danger, format!("⚠️ Error: {}", err));
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let export_label = if self.editing_id.is_some() {
                        "💾 Update Coordinate Axes"
                    } else {
                        "🚀 Export to Canvas"
                    };

                    let btn = ui.add(
                        egui::Button::new(egui::RichText::new(export_label).strong().size(13.0))
                            .fill(palette.accent)
                            .min_size(Vec2::new(160.0, 30.0)),
                    );

                    if btn.clicked() {
                        match self.execute_export(page, workspace) {
                            Ok(_) => {
                                self.is_open = false;
                                self.error_msg = None;
                            }
                            Err(e) => {
                                self.error_msg = Some(e);
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        self.is_open = is_open;
    }

    fn execute_export(
        &mut self,
        page: &mut Page,
        _workspace: &mut MathWorkspace,
    ) -> Result<(), String> {
        if self.mode == AxesInputMode::Code {
            // Parse simple axis commands like axis([-10 10 -5 5])
            let script = self.code_input.trim();
            if let Some(caps) = script
                .strip_prefix("axis([")
                .and_then(|s| s.strip_suffix("])"))
            {
                let nums: Vec<f64> = caps
                    .split_whitespace()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect();
                if nums.len() == 2 {
                    self.dimension = AxesDimension::D1;
                    self.x_min = nums[0];
                    self.x_max = nums[1];
                } else if nums.len() == 4 {
                    self.dimension = AxesDimension::D2;
                    self.x_min = nums[0];
                    self.x_max = nums[1];
                    self.y_min = nums[2];
                    self.y_max = nums[3];
                } else if nums.len() >= 6 {
                    self.dimension = AxesDimension::D3;
                    self.x_min = nums[0];
                    self.x_max = nums[1];
                    self.y_min = nums[2];
                    self.y_max = nums[3];
                    self.z_min = nums[4];
                    self.z_max = nums[5];
                }
            }
        }

        if self.x_max <= self.x_min {
            return Err("X Max must be greater than X Min!".into());
        }
        if self.dimension != AxesDimension::D1 && self.y_max <= self.y_min {
            return Err("Y Max must be greater than Y Min!".into());
        }

        let dim_str = match self.dimension {
            AxesDimension::D1 => "1D",
            AxesDimension::D2 => "2D",
            AxesDimension::D3 => "3D",
        };

        let width = match self.dimension {
            AxesDimension::D1 => 520.0,
            AxesDimension::D2 => 480.0,
            AxesDimension::D3 => 480.0,
        };
        let height = match self.dimension {
            AxesDimension::D1 => 130.0,
            AxesDimension::D2 => 320.0,
            AxesDimension::D3 => 360.0,
        };

        let auto_title = if self.title.trim().is_empty() {
            match self.dimension {
                AxesDimension::D1 => "1D Number Line".to_string(),
                AxesDimension::D2 => "2D Cartesian Axes (Oxy)".to_string(),
                AxesDimension::D3 => "3D Coordinate Space (Oxyz)".to_string(),
            }
        } else {
            self.title.clone()
        };

        if let Some(ref edit_id) = self.editing_id {
            if let Some(plot) = page.plots.iter_mut().find(|p| p.id == *edit_id) {
                plot.x_min = self.x_min;
                plot.x_max = self.x_max;
                plot.y_min = self.y_min;
                plot.y_max = self.y_max;
                plot.dimension = Some(dim_str.to_string());
                plot.x_step = Some(self.x_step);
                plot.y_step = Some(self.y_step);
                plot.z_min = Some(self.z_min);
                plot.z_max = Some(self.z_max);
                plot.z_step = Some(self.z_step);
                plot.show_grid = self.show_grid;
                plot.title = auto_title;
                plot.width = width;
                plot.height = height;
                plot.source_code = Some(self.code_input.clone());
                return Ok(());
            }
        }

        let place_x = 80.0f32;
        let place_y = 120.0f32 + (page.plots.len() as f32) * 40.0;
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
            width,
            height,
            x_min: self.x_min,
            x_max: self.x_max,
            y_min: self.y_min,
            y_max: self.y_max,
            curves: Vec::new(),
            show_grid: self.show_grid,
            show_axes: true,
            title: auto_title,
            dimension: Some(dim_str.to_string()),
            x_step: Some(self.x_step),
            y_step: Some(self.y_step),
            z_min: Some(self.z_min),
            z_max: Some(self.z_max),
            z_step: Some(self.z_step),
            source_code: Some(self.code_input.clone()),
        });

        Ok(())
    }
}

// ============================================================================
// 3. Table Grid Generator & Editor Dialog
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableInputMode {
    UI,
    Code,
}

pub struct TableDialogState {
    pub is_open: bool,
    pub mode: TableInputMode,

    // UI mode
    pub rows: usize,
    pub cols: usize,
    pub cell_values: Vec<Vec<String>>,
    pub col_width: f32,
    pub row_height: f32,

    // Code mode
    pub code_input: String,

    // Edit target
    pub editing_id: Option<String>,
    pub error_msg: Option<String>,
}

impl Default for TableDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: TableInputMode::UI,
            rows: 3,
            cols: 3,
            cell_values: vec![
                vec!["ID".into(), "Item".into(), "Value".into()],
                vec!["1".into(), "A".into(), "100".into()],
                vec!["2".into(), "B".into(), "250".into()],
            ],
            col_width: 110.0,
            row_height: 36.0,
            code_input: "ID, Item, Value\n1, A, 100\n2, B, 250".into(),
            editing_id: None,
            error_msg: None,
        }
    }
}

impl TableDialogState {
    pub fn open_new(&mut self) {
        self.is_open = true;
        self.editing_id = None;
        self.error_msg = None;
        self.ensure_grid_size();
    }

    pub fn open_edit(&mut self, grid: &TableGridEntity) {
        self.is_open = true;
        self.editing_id = Some(grid.id.clone());
        self.error_msg = None;
        self.rows = grid.rows.clamp(1, 15);
        self.cols = grid.cols.clamp(1, 15);
        self.col_width = grid.col_width;
        self.row_height = grid.row_height;
        self.cell_values = grid.cells.clone();
        if let Some(ref code) = grid.source_code {
            self.code_input = code.clone();
        } else {
            self.code_input = Self::format_as_csv(&grid.cells);
        }
        self.ensure_grid_size();
    }

    fn ensure_grid_size(&mut self) {
        self.rows = self.rows.clamp(1, 15);
        self.cols = self.cols.clamp(1, 15);

        if self.cell_values.len() < self.rows {
            self.cell_values
                .resize(self.rows, vec!["".into(); self.cols]);
        } else if self.cell_values.len() > self.rows {
            self.cell_values.truncate(self.rows);
        }

        for row in &mut self.cell_values {
            if row.len() < self.cols {
                row.resize(self.cols, "".into());
            } else if row.len() > self.cols {
                row.truncate(self.cols);
            }
        }
    }

    pub fn format_as_csv(cells: &[Vec<String>]) -> String {
        cells
            .iter()
            .map(|row| row.join(", "))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn render(&mut self, ctx: &egui::Context, page: &mut Page, palette: &Palette) {
        if !self.is_open {
            return;
        }

        let win_title = if self.editing_id.is_some() {
            "✏️ Edit Table"
        } else {
            "▦ Table Generator"
        };

        let mut is_open = self.is_open;
        egui::Window::new(win_title)
            .open(&mut is_open)
            .resizable(true)
            .default_size([480.0, 420.0])
            .min_size([400.0, 320.0])
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.mode, TableInputMode::UI, "🎛️ UI (Grid Inputs)");
                    ui.selectable_value(
                        &mut self.mode,
                        TableInputMode::Code,
                        "💻 Code (CSV / Markdown)",
                    );
                });
                ui.separator();

                match self.mode {
                    TableInputMode::UI => {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Rows:").strong());
                            let prev_r = self.rows;
                            if ui
                                .add(egui::DragValue::new(&mut self.rows).range(1..=15))
                                .changed()
                            {
                                if self.rows != prev_r {
                                    self.ensure_grid_size();
                                }
                            }

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("Cols:").strong());
                            let prev_c = self.cols;
                            if ui
                                .add(egui::DragValue::new(&mut self.cols).range(1..=15))
                                .changed()
                            {
                                if self.cols != prev_c {
                                    self.ensure_grid_size();
                                }
                            }

                            ui.add_space(8.0);
                            ui.label("Col Width:");
                            ui.add(
                                egui::DragValue::new(&mut self.col_width)
                                    .range(40.0..=300.0)
                                    .suffix("px"),
                            );
                        });

                        ui.add_space(6.0);
                        self.ensure_grid_size();

                        egui::ScrollArea::both().max_height(200.0).show(ui, |ui| {
                            egui::Grid::new("table_ui_cells_grid")
                                .spacing([6.0, 6.0])
                                .show(ui, |ui| {
                                    for r in 0..self.rows {
                                        ui.label(
                                            egui::RichText::new(format!("H{}:", r + 1)).weak(),
                                        );
                                        for c in 0..self.cols {
                                            let cell = &mut self.cell_values[r][c];
                                            ui.add(
                                                egui::TextEdit::singleline(cell)
                                                    .desired_width(70.0)
                                                    .hint_text("..."),
                                            );
                                        }
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                    TableInputMode::Code => {
                        ui.label("Enter table data as CSV or Markdown table:");
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .max_height(180.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.code_input)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_rows(6)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("Col1, Col2, Col3\nA, B, C\n1, 2, 3"),
                                );
                            });
                    }
                }

                if let Some(ref err) = self.error_msg {
                    ui.add_space(4.0);
                    ui.colored_label(palette.danger, format!("⚠️ Error: {}", err));
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let export_label = if self.editing_id.is_some() {
                        "💾 Update Table"
                    } else {
                        "🚀 Export to Canvas"
                    };

                    let btn = ui.add(
                        egui::Button::new(egui::RichText::new(export_label).strong().size(13.0))
                            .fill(palette.accent)
                            .min_size(Vec2::new(160.0, 30.0)),
                    );

                    if btn.clicked() {
                        match self.execute_export(page) {
                            Ok(_) => {
                                self.is_open = false;
                                self.error_msg = None;
                            }
                            Err(e) => {
                                self.error_msg = Some(e);
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.is_open = false;
                    }
                });
            });

        self.is_open = is_open;
    }

    fn execute_export(&mut self, page: &mut Page) -> Result<(), String> {
        let (cells, rows, cols) = match self.mode {
            TableInputMode::UI => {
                let r = self.rows;
                let c = self.cols;
                (self.cell_values.clone(), r, c)
            }
            TableInputMode::Code => {
                let lines: Vec<&str> = self
                    .code_input
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .collect();
                if lines.is_empty() {
                    return Err("Please enter table content!".into());
                }

                let mut parsed: Vec<Vec<String>> = Vec::new();
                for line in lines {
                    let parts: Vec<String> = if line.contains(',') {
                        line.split(',').map(|s| s.trim().to_string()).collect()
                    } else if line.contains('\t') {
                        line.split('\t').map(|s| s.trim().to_string()).collect()
                    } else if line.contains('|') {
                        line.split('|')
                            .filter(|s| !s.trim().is_empty() && !s.contains("---"))
                            .map(|s| s.trim().to_string())
                            .collect()
                    } else {
                        line.split_whitespace().map(|s| s.to_string()).collect()
                    };

                    if !parts.is_empty() {
                        parsed.push(parts);
                    }
                }

                if parsed.is_empty() {
                    return Err("Could not parse table data!".into());
                }

                let r = parsed.len();
                let c = parsed.iter().map(|row| row.len()).max().unwrap_or(1);
                // Equalize rows
                for row in &mut parsed {
                    row.resize(c, String::new());
                }
                (parsed, r, c)
            }
        };

        let col_w = self.col_width.max(40.0);
        let row_h = self.row_height.max(24.0);

        if let Some(ref edit_id) = self.editing_id {
            if let Some(grid) = page.grids.iter_mut().find(|g| g.id == *edit_id) {
                grid.cells = cells;
                grid.rows = rows;
                grid.cols = cols;
                grid.col_width = col_w;
                grid.row_height = row_h;
                grid.source_code = Some(self.code_input.clone());
                return Ok(());
            }
        }

        let place_x = 80.0f32;
        let place_y = 120.0f32 + (page.grids.len() as f32) * 40.0;
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
            col_width: col_w,
            row_height: row_h,
            cells,
            source_code: Some(self.code_input.clone()),
        });

        Ok(())
    }
}
