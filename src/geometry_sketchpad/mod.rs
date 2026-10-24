//! Inkk Geometry Sketchpad (GSP) & 3D Spatial Studio module.
//! Provides dynamic Euclidean constructions, 3D geometric solid modeling,
//! geometric transformations, live measurements, action buttons, and whiteboard exports.

pub mod cad_app;
pub mod cad_icons;
pub mod engine;
pub mod export;
pub mod model;
pub mod presets_3d;
pub mod ui_window;

pub use cad_app::CadStandaloneApp;
pub use model::{ActionKind, GeoPoint, PointConstraint, SketchpadModel, SketchpadTool};
pub use ui_window::GeometrySketchpadState;

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Pos2;

    #[test]
    fn test_midpoint_construction() {
        let mut model = SketchpadModel::new();
        let p1 = GeoPoint::new_2d("p1", "A", Pos2::new(10.0, 20.0));
        let p2 = GeoPoint::new_2d("p2", "B", Pos2::new(30.0, 60.0));
        let mut m = GeoPoint::new_2d("m", "M", Pos2::ZERO);
        m.constraint = PointConstraint::Midpoint("p1".to_string(), "p2".to_string());

        model.points.push(p1);
        model.points.push(p2);
        model.points.push(m);

        engine::GeometryEngine::update_geometry(&mut model);

        let mid = model.get_point("m").unwrap();
        assert!((mid.pos.x - 20.0).abs() < 1e-4);
        assert!((mid.pos.y - 40.0).abs() < 1e-4);
    }

    #[test]
    fn test_3d_presets_loading() {
        let mut model = SketchpadModel::new();
        presets_3d::Presets3D::load_sphere_with_inscribed_pyramid(&mut model);
        assert!(!model.points.is_empty());
        assert!(!model.lines.is_empty());
        assert!(!model.polygons.is_empty());
        assert!(!model.action_buttons.is_empty());

        engine::GeometryEngine::update_geometry(&mut model);
        for p in &model.points {
            if p.pos3d.is_some() {
                assert!(p.pos != Pos2::ZERO);
            }
        }
    }

    #[test]
    fn test_3d_cube_loading() {
        let mut model = SketchpadModel::new();
        presets_3d::Presets3D::load_cube_3d(&mut model);
        assert_eq!(model.points.len(), 8);
        assert_eq!(model.lines.len(), 12);
        let dashed_lines: Vec<_> = model.lines.iter().filter(|l| l.is_dashed).collect();
        assert_eq!(dashed_lines.len(), 3); // 3 hidden back edges!
    }

    #[test]
    fn test_svg_export() {
        let mut model = SketchpadModel::new();
        presets_3d::Presets3D::load_cube_3d(&mut model);
        engine::GeometryEngine::update_geometry(&mut model);
        let svg = export::SketchpadExporter::to_svg(&model, 600.0, 400.0, false);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("stroke-dasharray="));
        assert!(svg.contains("</svg>"));
    }
}
