//! Canvas toolbar / palette for standard ISO flowchart blocks.

use eframe::egui::Ui;

use crate::model::{FlowchartEntity, Page};
use crate::theme::Palette;

pub struct FlowchartPalette;

impl FlowchartPalette {
    /// Render horizontal block insertion buttons in toolbar or floating palette.
    pub fn render_toolbar_buttons(
        ui: &mut Ui,
        page: &mut Page,
        _palette: &Palette,
        open_studio: &mut bool,
    ) {
        ui.horizontal(|ui| {
            if ui.button("⚡ Flowchart").clicked() {
                *open_studio = true;
            }

            if ui.button("+ New Algorithm Flow").clicked() {
                let ent = FlowchartEntity {
                    dsl_source: r#"algorithm "Quick Flow"

start
input data
if isValid(data) then
    do "process data"
    output "success"
else
    output "invalid input"
end
end
"#
                    .to_string(),
                    title: "Quick Flow".to_string(),
                    x: 80.0,
                    y: 120.0,
                    ..Default::default()
                };
                page.flowcharts.push(ent);
            }
        });
    }
}
