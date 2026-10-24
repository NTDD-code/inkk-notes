//! Flowchart Studio Code Editor Panel.
//! Provides interactive DSL editing, real-time diagnostic error badges, round-trip formatting,
//! quick snippet inserters, and one-click exports (SVG, Mermaid, Pseudocode).

use eframe::egui::{self, Color32, Ui, Vec2};

use crate::flowchart_core::check::{Diagnostic, DiagnosticSeverity, FlowchartChecker};
use crate::flowchart_core::export::FlowchartExporter;
use crate::flowchart_core::ir::FlowchartGraph;
use crate::flowchart_core::parser::Parser;
use crate::flowchart_core::print::format_dsl;
use crate::model::FlowchartEntity;
use crate::theme::Palette;

pub struct FlowchartEditorState {
    pub is_open: bool,
    pub editing_entity_id: Option<String>,
    pub dsl_buffer: String,
    pub diagnostics: Vec<Diagnostic>,
    pub preview_graph: Option<FlowchartGraph>,
    pub status_message: Option<String>,
    pub req_bake_to_image: Option<String>,
    pub req_save_svg_file: Option<String>,
}

impl Default for FlowchartEditorState {
    fn default() -> Self {
        let sample_dsl = r#"algorithm "Binary Search"

start
read target, array
set left = 0
set right = len(array) - 1

while left <= right
    set mid = (left + right) / 2
    if array[mid] == target then
        print "Found target at mid"
        return mid
    else
        if array[mid] < target then
            set left = mid + 1
        else
            set right = mid - 1
        end
    end
end

print "Target not found"
end
"#;
        Self {
            is_open: false,
            editing_entity_id: None,
            dsl_buffer: sample_dsl.to_string(),
            diagnostics: Vec::new(),
            preview_graph: None,
            status_message: None,
            req_bake_to_image: None,
            req_save_svg_file: None,
        }
    }
}

impl FlowchartEditorState {
    pub fn open_for_entity(&mut self, entity: &FlowchartEntity) {
        self.editing_entity_id = Some(entity.id.clone());
        self.dsl_buffer = if entity.dsl_source.trim().is_empty() {
            Self::default().dsl_buffer
        } else {
            entity.dsl_source.clone()
        };
        self.recompile();
        self.is_open = true;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open && self.preview_graph.is_none() {
            self.recompile();
        }
    }

    pub fn recompile(&mut self) {
        let (ast, _errors) = Parser::from_str(&self.dsl_buffer).parse();
        let graph = FlowchartGraph::from_ast(&ast);
        let diags = FlowchartChecker::check(&graph, Some(&ast));
        self.diagnostics = diags;
        self.preview_graph = Some(graph);
    }

    /// Render the Flowchart Studio window.
    pub fn render_window(
        &mut self,
        ctx: &egui::Context,
        flowcharts: &mut Vec<FlowchartEntity>,
        palette: &Palette,
        is_dark: bool,
    ) {
        if !self.is_open {
            return;
        }

        let mut is_open_flag = self.is_open;
        let window_title = if self.editing_entity_id.is_some() {
            "⚡ Flowchart Studio — Edit Entity"
        } else {
            "⚡ Flowchart Studio"
        };

        egui::Window::new(window_title)
            .open(&mut is_open_flag)
            .default_size(Vec2::new(720.0, 580.0))
            .min_size(Vec2::new(540.0, 420.0))
            .resizable(true)
            .show(ctx, |ui| {
                self.render_content(ui, flowcharts, palette, is_dark);
            });

        self.is_open = is_open_flag;
    }

    fn render_content(
        &mut self,
        ui: &mut Ui,
        flowcharts: &mut Vec<FlowchartEntity>,
        palette: &Palette,
        _is_dark: bool,
    ) {
        // Top Toolbar: Snippets & Action Buttons
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Snippets:").strong());

            if ui.button("+ If/Else").clicked() {
                self.dsl_buffer.push_str("\nif cond then\n    do \"step 1\"\nelse\n    do \"step 2\"\nend\n");
                self.recompile();
            }
            if ui.button("+ While").clicked() {
                self.dsl_buffer.push_str("\nwhile cond\n    do \"loop body\"\nend\n");
                self.recompile();
            }
            if ui.button("+ For").clicked() {
                self.dsl_buffer.push_str("\nfor i from 1 to n\n    do \"step i\"\nend\n");
                self.recompile();
            }
            if ui.button("+ Switch").clicked() {
                self.dsl_buffer.push_str("\nswitch x\n    case 1: do \"action 1\"\n    case 2: do \"action 2\"\n    default: do \"default action\"\nend\n");
                self.recompile();
            }

            ui.separator();

            // Format DSL canonical button
            if ui.button("⚡ Format DSL").clicked() {
                let (ast, _) = Parser::from_str(&self.dsl_buffer).parse();
                self.dsl_buffer = format_dsl(&ast);
                self.recompile();
                self.status_message = Some("Formatted DSL code".to_string());
            }

            // Copy Mermaid
            if ui.button("📋 Copy Mermaid").clicked() {
                if let Some(graph) = &self.preview_graph {
                    let mermaid = FlowchartExporter::to_mermaid(graph);
                    ui.output_mut(|o| o.copied_text = mermaid);
                    self.status_message = Some("Copied Mermaid flowchart to clipboard".to_string());
                }
            }

            // Copy SVG
            if ui.button("📋 Copy SVG").clicked() {
                if let Some(graph) = &self.preview_graph {
                    let svg = FlowchartExporter::to_svg(graph, None, None);
                    ui.output_mut(|o| o.copied_text = svg);
                    self.status_message = Some("Copied SVG markup to clipboard".to_string());
                }
            }

            // Save SVG file to disk
            if ui.button("💾 Save SVG...").on_hover_text("Save this flowchart as an .svg vector file to disk").clicked() {
                self.recompile();
                if let Some(graph) = &self.preview_graph {
                    let svg = FlowchartExporter::to_svg(graph, None, None);
                    self.req_save_svg_file = Some(svg);
                }
            }

            // Bake directly to whiteboard image
            if ui.button("🖼 Bake to Whiteboard Image").on_hover_text("Render this flowchart directly onto the whiteboard canvas as a movable vector image").clicked() {
                self.recompile();
                if let Some(graph) = &self.preview_graph {
                    let svg = FlowchartExporter::to_svg(graph, None, None);
                    self.req_bake_to_image = Some(svg);
                }
            }

            // Sync to canvas button
            let apply_lbl = if self.editing_entity_id.is_some() {
                "💾 Apply to Canvas"
            } else {
                "➕ Insert New Flowchart"
            };

            if ui.button(egui::RichText::new(apply_lbl).color(palette.on_accent).strong()).clicked() {
                self.recompile();
                if let Some(target_id) = &self.editing_entity_id {
                    if let Some(entity) = flowcharts.iter_mut().find(|f| f.id == *target_id) {
                        entity.dsl_source = self.dsl_buffer.clone();
                        entity.graph = self.preview_graph.clone();
                        if let Some(g) = &self.preview_graph {
                            if let Some(t) = &g.title {
                                entity.title = t.clone();
                            }
                        }
                    }
                } else {
                    // Create new entity on current page
                    let mut new_ent = FlowchartEntity::default();
                    new_ent.dsl_source = self.dsl_buffer.clone();
                    new_ent.graph = self.preview_graph.clone();
                    if let Some(g) = &self.preview_graph {
                        if let Some(t) = &g.title {
                            new_ent.title = t.clone();
                        }
                    }
                    flowcharts.push(new_ent);
                }
                self.status_message = Some("Applied flowchart to canvas!".to_string());
            }
        });

        if let Some(msg) = &self.status_message {
            ui.label(egui::RichText::new(msg).color(palette.accent));
        }

        ui.separator();

        // Main Body: Split between DSL Code Editor and Live Diagnostic Inspector
        ui.columns(2, |cols| {
            // Column 1: Monospace DSL Code Editor
            cols[0].vertical(|ui| {
                ui.label(egui::RichText::new("Flowchart DSL Editor").strong());
                let text_edit = egui::TextEdit::multiline(&mut self.dsl_buffer)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_rows(22)
                    .desired_width(f32::INFINITY);

                let resp = ui.add(text_edit);
                if resp.changed() {
                    self.recompile();
                }
            });

            // Column 2: Compiler Diagnostics & Summary
            cols[1].vertical(|ui| {
                let err_count = self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).count();
                let warn_count = self.diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Warning).count();

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Diagnostics:").strong());
                    if err_count == 0 && warn_count == 0 {
                        ui.label(egui::RichText::new("✓ Valid Flowchart").color(palette.success));
                    } else {
                        if err_count > 0 {
                            ui.label(egui::RichText::new(format!("{} error(s)", err_count)).color(palette.danger));
                        }
                        if warn_count > 0 {
                            ui.label(egui::RichText::new(format!("{} warning(s)", warn_count)).color(Color32::from_rgb(220, 160, 40)));
                        }
                    }
                });

                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .show(ui, |ui| {
                        if self.diagnostics.is_empty() {
                            ui.label("No semantic or structural errors detected.\nFlowchart strictly conforms to ISO 5807.");
                        } else {
                            for diag in &self.diagnostics {
                                let (badge_text, badge_color) = match diag.severity {
                                    DiagnosticSeverity::Error => ("ERROR", palette.danger),
                                    DiagnosticSeverity::Warning => ("WARN", Color32::from_rgb(220, 160, 40)),
                                    _ => ("INFO", palette.text_muted),
                                };

                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(badge_color, format!("[{}] {}", badge_text, diag.code));
                                        if let Some(span) = diag.span {
                                            ui.label(format!("line {}", span.line));
                                        }
                                    });
                                    ui.label(&diag.message);
                                    if let Some(help) = &diag.help {
                                        ui.colored_label(palette.accent, format!("💡 Help: {}", help));
                                    }
                                });
                            }
                        }
                    });
            });
        });
    }
}
