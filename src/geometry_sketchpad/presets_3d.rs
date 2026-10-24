//! Pre-built 3D Spatial Presets for Geometry Sketchpad.
//! Implements Sphere + Inscribed Pyramid (User GSP Image 2),
//! 3D Cube, Pyramid, Sphere, Triangular Prism, Cylinder, and Cone.

use eframe::egui::{Color32, Pos2};
use super::model::{
    ActionKind, GeoActionButton, GeoLine, GeoMeasurement, GeoPoint, GeoPolygon,
    MeasureKind, SketchpadModel,
};

pub struct Presets3D;

impl Presets3D {
    /// Loads the classic 3D Sphere with Inscribed Triangular Pyramid (Tetrahedron)
    /// exactly as seen in Geometer's Sketchpad (User Image 2).
    pub fn load_sphere_with_inscribed_pyramid(sketch: &mut SketchpadModel) {
        sketch.clear();
        sketch.camera3d.pitch = 0.32;
        sketch.camera3d.yaw = -0.50;
        sketch.camera3d.zoom = 1.0;

        let r = 135.0f32;

        // Points in 3D:
        // North Pole (Apex S)
        let s = GeoPoint::new_3d("pt_S", "S", [0.0, r, 0.0]);
        // Base triangle ABC on horizontal equator plane (y = -r * 0.32)
        // A: front-right
        let a = GeoPoint::new_3d("pt_A", "A", [r * 0.75, -r * 0.32, r * 0.60]);
        // B: back-right
        let b = GeoPoint::new_3d("pt_B", "B", [r * 0.85, -r * 0.32, -r * 0.35]);
        // C: left
        let c = GeoPoint::new_3d("pt_C", "C", [-r * 0.90, -r * 0.32, 0.0]);
        // Center/Altitude point W and projection
        let w = GeoPoint::new_3d("pt_W", "W", [0.0, 0.0, 0.0]);
        let w_base = GeoPoint::new_3d("pt_W0", "", [0.0, -r * 0.32, 0.0]);
        let s_base = GeoPoint::new_3d("pt_S0", "", [0.0, -r * 0.32, 0.0]);

        sketch.points.extend(vec![s, a, b, c, w, w_base, s_base]);

        // Equator Circle & Meridian in 3D: sampled points to create smooth curves with dashed rear
        let steps = 32;
        let mut equator_ids = Vec::new();
        for i in 0..steps {
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let px = r * theta.cos();
            let pz = r * theta.sin();
            let pid = format!("eq_{}", i);
            let mut pt = GeoPoint::new_3d(&pid, "", [px, 0.0, pz]);
            pt.is_hidden = true;
            sketch.points.push(pt);
            equator_ids.push(pid);
        }
        for i in 0..steps {
            let next = (i + 1) % steps;
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            // Behind (Z < 0) is dashed
            let is_dashed = theta.sin() < 0.0;
            let mut line = GeoLine::new(format!("eq_edge_{}", i), &equator_ids[i], &equator_ids[next], is_dashed);
            line.color = Color32::from_rgb(34, 150, 60); // Green sphere line as in GSP!
            sketch.lines.push(line);
        }

        // Meridian circle (YZ plane)
        let mut meridian_ids = Vec::new();
        for i in 0..steps {
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let py = r * theta.cos();
            let pz = r * theta.sin();
            let pid = format!("mer_{}", i);
            let mut pt = GeoPoint::new_3d(&pid, "", [0.0, py, pz]);
            pt.is_hidden = true;
            sketch.points.push(pt);
            meridian_ids.push(pid);
        }
        for i in 0..steps {
            let next = (i + 1) % steps;
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let is_dashed = theta.sin() < 0.0;
            let mut line = GeoLine::new(format!("mer_edge_{}", i), &meridian_ids[i], &meridian_ids[next], is_dashed);
            line.color = Color32::from_rgb(34, 150, 60);
            sketch.lines.push(line);
        }

        // Base polygon ABC (light cyan filled interior)
        let base_poly = GeoPolygon {
            id: "poly_ABC".to_string(),
            vertices: vec!["pt_A".to_string(), "pt_B".to_string(), "pt_C".to_string()],
            fill_color: Some(Color32::from_rgba_unmultiplied(180, 240, 255, 120)),
            stroke_color: Color32::from_rgb(25, 60, 180),
            is_selected: false,
            is_hidden: false,
        };
        sketch.polygons.push(base_poly);

        // Pyramid edges:
        // SA (solid), SB (dashed hidden back edge), SC (dashed hidden back edge)
        let mut l_sa = GeoLine::new("l_sa", "pt_S", "pt_A", false);
        l_sa.color = Color32::from_rgb(25, 60, 180);
        let mut l_sb = GeoLine::new("l_sb", "pt_S", "pt_B", true);
        l_sb.color = Color32::from_rgb(25, 60, 180);
        let mut l_sc = GeoLine::new("l_sc", "pt_S", "pt_C", true);
        l_sc.color = Color32::from_rgb(25, 60, 180);

        // Base edges: AB (solid), BC (dashed), CA (dashed)
        let mut l_ab = GeoLine::new("l_ab", "pt_A", "pt_B", false);
        l_ab.color = Color32::from_rgb(25, 60, 180);
        let mut l_bc = GeoLine::new("l_bc", "pt_B", "pt_C", true);
        l_bc.color = Color32::from_rgb(25, 60, 180);
        let mut l_ca = GeoLine::new("l_ca", "pt_C", "pt_A", true);
        l_ca.color = Color32::from_rgb(25, 60, 180);

        // Altitudes: SW (dashed) and W to base (dashed)
        let mut l_sw = GeoLine::new("l_sw", "pt_S", "pt_S0", true);
        l_sw.color = Color32::from_rgb(50, 120, 220);
        let mut l_wb = GeoLine::new("l_wb", "pt_W", "pt_W0", true);
        l_wb.color = Color32::from_rgb(34, 150, 60);

        sketch.lines.extend(vec![l_sa, l_sb, l_sc, l_ab, l_bc, l_ca, l_sw, l_wb]);

        // Action Buttons on top-left of canvas (as in User Image 2):
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_spin".to_string(),
            label: "QUAY 3D".to_string(),
            kind: ActionKind::Toggle3DRotate,
            pos: Pos2::new(30.0, 45.0),
        });
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_oxy".to_string(),
            label: "Oxy".to_string(),
            kind: ActionKind::ViewPlaneOxy,
            pos: Pos2::new(30.0, 75.0),
        });
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_oxz".to_string(),
            label: "Oxz".to_string(),
            kind: ActionKind::ViewPlaneOxz,
            pos: Pos2::new(30.0, 105.0),
        });
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_oyz".to_string(),
            label: "Oyz".to_string(),
            kind: ActionKind::ViewPlaneOyz,
            pos: Pos2::new(30.0, 135.0),
        });

        // Measurements:
        sketch.measurements.push(GeoMeasurement {
            id: "m_unit".to_string(),
            kind: MeasureKind::Length,
            target_ids: vec!["pt_A".to_string(), "pt_B".to_string()],
            cached_value: 0.89,
            display_text: "Unit = 0.89 cm".to_string(),
            pos: Pos2::new(30.0, 20.0),
        });
    }

    /// Loads a classic 3D Cube (ABCD.A'B'C'D') with dashed hidden edges.
    pub fn load_cube_3d(sketch: &mut SketchpadModel) {
        sketch.clear();
        sketch.camera3d.pitch = 0.40;
        sketch.camera3d.yaw = -0.65;
        let s = 90.0f32;

        // Bottom vertices A, B, C, D (D is hidden back-left)
        let a = GeoPoint::new_3d("pt_A", "A", [-s, -s, s]);
        let b = GeoPoint::new_3d("pt_B", "B", [s, -s, s]);
        let c = GeoPoint::new_3d("pt_C", "C", [s, -s, -s]);
        let d = GeoPoint::new_3d("pt_D", "D", [-s, -s, -s]);

        // Top vertices A', B', C', D'
        let a_top = GeoPoint::new_3d("pt_A1", "A'", [-s, s, s]);
        let b_top = GeoPoint::new_3d("pt_B1", "B'", [s, s, s]);
        let c_top = GeoPoint::new_3d("pt_C1", "C'", [s, s, -s]);
        let d_top = GeoPoint::new_3d("pt_D1", "D'", [-s, s, -s]);

        sketch.points.extend(vec![a, b, c, d, a_top, b_top, c_top, d_top]);

        // Front Face (ABB'A')
        let front_poly = GeoPolygon {
            id: "poly_front".to_string(),
            vertices: vec!["pt_A".to_string(), "pt_B".to_string(), "pt_B1".to_string(), "pt_A1".to_string()],
            fill_color: Some(Color32::from_rgba_unmultiplied(220, 235, 255, 90)),
            stroke_color: Color32::from_rgb(30, 80, 200),
            is_selected: false,
            is_hidden: false,
        };
        // Top Face (A'B'C'D')
        let top_poly = GeoPolygon {
            id: "poly_top".to_string(),
            vertices: vec!["pt_A1".to_string(), "pt_B1".to_string(), "pt_C1".to_string(), "pt_D1".to_string()],
            fill_color: Some(Color32::from_rgba_unmultiplied(200, 225, 255, 120)),
            stroke_color: Color32::from_rgb(30, 80, 200),
            is_selected: false,
            is_hidden: false,
        };
        sketch.polygons.extend(vec![front_poly, top_poly]);

        // Edges:
        // Visible solid edges
        sketch.lines.push(GeoLine::new("l_ab", "pt_A", "pt_B", false));
        sketch.lines.push(GeoLine::new("l_bc", "pt_B", "pt_C", false));
        sketch.lines.push(GeoLine::new("l_a1b1", "pt_A1", "pt_B1", false));
        sketch.lines.push(GeoLine::new("l_b1c1", "pt_B1", "pt_C1", false));
        sketch.lines.push(GeoLine::new("l_c1d1", "pt_C1", "pt_D1", false));
        sketch.lines.push(GeoLine::new("l_d1a1", "pt_D1", "pt_A1", false));
        sketch.lines.push(GeoLine::new("l_aa1", "pt_A", "pt_A1", false));
        sketch.lines.push(GeoLine::new("l_bb1", "pt_B", "pt_B1", false));
        sketch.lines.push(GeoLine::new("l_cc1", "pt_C", "pt_C1", false));

        // Hidden rear edges (DASHED!)
        sketch.lines.push(GeoLine::new("l_ad", "pt_A", "pt_D", true));
        sketch.lines.push(GeoLine::new("l_cd", "pt_C", "pt_D", true));
        sketch.lines.push(GeoLine::new("l_dd1", "pt_D", "pt_D1", true));

        // Action Buttons
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_spin".to_string(),
            label: "QUAY 3D".to_string(),
            kind: ActionKind::Toggle3DRotate,
            pos: Pos2::new(30.0, 45.0),
        });
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_oxy".to_string(),
            label: "Oxy".to_string(),
            kind: ActionKind::ViewPlaneOxy,
            pos: Pos2::new(30.0, 75.0),
        });
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_oxz".to_string(),
            label: "Oxz".to_string(),
            kind: ActionKind::ViewPlaneOxz,
            pos: Pos2::new(30.0, 105.0),
        });
        sketch.action_buttons.push(GeoActionButton {
            id: "btn_oyz".to_string(),
            label: "Oyz".to_string(),
            kind: ActionKind::ViewPlaneOyz,
            pos: Pos2::new(30.0, 135.0),
        });
    }

    /// Loads 3D Square Pyramid (S.ABCD).
    pub fn load_square_pyramid_3d(sketch: &mut SketchpadModel) {
        sketch.clear();
        sketch.camera3d.pitch = 0.35;
        sketch.camera3d.yaw = -0.55;
        let s = 95.0f32;
        let h = 130.0f32;

        let apex = GeoPoint::new_3d("pt_S", "S", [0.0, h, 0.0]);
        let a = GeoPoint::new_3d("pt_A", "A", [-s, -40.0, s]);
        let b = GeoPoint::new_3d("pt_B", "B", [s, -40.0, s]);
        let c = GeoPoint::new_3d("pt_C", "C", [s, -40.0, -s]);
        let d = GeoPoint::new_3d("pt_D", "D", [-s, -40.0, -s]);
        let o = GeoPoint::new_3d("pt_O", "O", [0.0, -40.0, 0.0]);

        sketch.points.extend(vec![apex, a, b, c, d, o]);

        // Base polygon
        let base_poly = GeoPolygon {
            id: "poly_base".to_string(),
            vertices: vec!["pt_A".to_string(), "pt_B".to_string(), "pt_C".to_string(), "pt_D".to_string()],
            fill_color: Some(Color32::from_rgba_unmultiplied(255, 230, 200, 100)),
            stroke_color: Color32::from_rgb(180, 80, 20),
            is_selected: false,
            is_hidden: false,
        };
        sketch.polygons.push(base_poly);

        // Visible edges
        sketch.lines.push(GeoLine::new("l_ab", "pt_A", "pt_B", false));
        sketch.lines.push(GeoLine::new("l_bc", "pt_B", "pt_C", false));
        sketch.lines.push(GeoLine::new("l_sa", "pt_S", "pt_A", false));
        sketch.lines.push(GeoLine::new("l_sb", "pt_S", "pt_B", false));
        sketch.lines.push(GeoLine::new("l_sc", "pt_S", "pt_C", false));

        // Hidden rear edges (DASHED!)
        sketch.lines.push(GeoLine::new("l_ad", "pt_A", "pt_D", true));
        sketch.lines.push(GeoLine::new("l_cd", "pt_C", "pt_D", true));
        sketch.lines.push(GeoLine::new("l_sd", "pt_S", "pt_D", true));
        sketch.lines.push(GeoLine::new("l_so", "pt_S", "pt_O", true)); // Altitude

        sketch.action_buttons.push(GeoActionButton {
            id: "btn_spin".to_string(),
            label: "QUAY 3D".to_string(),
            kind: ActionKind::Toggle3DRotate,
            pos: Pos2::new(30.0, 45.0),
        });
    }

    /// Loads 3D Triangular Prism (ABC.A'B'C').
    pub fn load_triangular_prism_3d(sketch: &mut SketchpadModel) {
        sketch.clear();
        sketch.camera3d.pitch = 0.35;
        sketch.camera3d.yaw = -0.60;
        let r = 85.0f32;
        let h = 100.0f32;

        let a = GeoPoint::new_3d("pt_A", "A", [-r, -h * 0.5, r * 0.5]);
        let b = GeoPoint::new_3d("pt_B", "B", [r, -h * 0.5, r * 0.5]);
        let c = GeoPoint::new_3d("pt_C", "C", [0.0, -h * 0.5, -r]);

        let a1 = GeoPoint::new_3d("pt_A1", "A'", [-r, h * 0.5, r * 0.5]);
        let b1 = GeoPoint::new_3d("pt_B1", "B'", [r, h * 0.5, r * 0.5]);
        let c1 = GeoPoint::new_3d("pt_C1", "C'", [0.0, h * 0.5, -r]);

        sketch.points.extend(vec![a, b, c, a1, b1, c1]);

        // Top triangle
        sketch.polygons.push(GeoPolygon {
            id: "poly_top".to_string(),
            vertices: vec!["pt_A1".to_string(), "pt_B1".to_string(), "pt_C1".to_string()],
            fill_color: Some(Color32::from_rgba_unmultiplied(220, 245, 220, 110)),
            stroke_color: Color32::from_rgb(30, 140, 50),
            is_selected: false,
            is_hidden: false,
        });

        // Visible edges
        sketch.lines.push(GeoLine::new("l_ab", "pt_A", "pt_B", false));
        sketch.lines.push(GeoLine::new("l_a1b1", "pt_A1", "pt_B1", false));
        sketch.lines.push(GeoLine::new("l_b1c1", "pt_B1", "pt_C1", false));
        sketch.lines.push(GeoLine::new("l_c1a1", "pt_C1", "pt_A1", false));
        sketch.lines.push(GeoLine::new("l_aa1", "pt_A", "pt_A1", false));
        sketch.lines.push(GeoLine::new("l_bb1", "pt_B", "pt_B1", false));

        // Hidden rear edges
        sketch.lines.push(GeoLine::new("l_bc", "pt_B", "pt_C", true));
        sketch.lines.push(GeoLine::new("l_ca", "pt_C", "pt_A", true));
        sketch.lines.push(GeoLine::new("l_cc1", "pt_C", "pt_C1", true));

        sketch.action_buttons.push(GeoActionButton {
            id: "btn_spin".to_string(),
            label: "QUAY 3D".to_string(),
            kind: ActionKind::Toggle3DRotate,
            pos: Pos2::new(30.0, 45.0),
        });
    }

    /// Loads 3D Cylinder.
    pub fn load_cylinder_3d(sketch: &mut SketchpadModel) {
        sketch.clear();
        sketch.camera3d.pitch = 0.30;
        let r = 80.0f32;
        let h = 110.0f32;

        let top_c = GeoPoint::new_3d("pt_top", "O'", [0.0, h * 0.5, 0.0]);
        let bot_c = GeoPoint::new_3d("pt_bot", "O", [0.0, -h * 0.5, 0.0]);
        let left_top = GeoPoint::new_3d("pt_lt", "", [-r, h * 0.5, 0.0]);
        let left_bot = GeoPoint::new_3d("pt_lb", "", [-r, -h * 0.5, 0.0]);
        let right_top = GeoPoint::new_3d("pt_rt", "", [r, h * 0.5, 0.0]);
        let right_bot = GeoPoint::new_3d("pt_rb", "", [r, -h * 0.5, 0.0]);

        sketch.points.extend(vec![top_c, bot_c, left_top, left_bot, right_top, right_bot]);

        // Lateral lines
        sketch.lines.push(GeoLine::new("l_left", "pt_lt", "pt_lb", false));
        sketch.lines.push(GeoLine::new("l_right", "pt_rt", "pt_rb", false));
        sketch.lines.push(GeoLine::new("l_axis", "pt_top", "pt_bot", true)); // Axis dashed

        // Equator loops for top and bottom
        let steps = 24;
        let mut top_ids = Vec::new();
        let mut bot_ids = Vec::new();
        for i in 0..steps {
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let px = r * theta.cos();
            let pz = r * theta.sin();
            let t_id = format!("ct_{}", i);
            let b_id = format!("cb_{}", i);
            let mut pt_t = GeoPoint::new_3d(&t_id, "", [px, h * 0.5, pz]);
            let mut pt_b = GeoPoint::new_3d(&b_id, "", [px, -h * 0.5, pz]);
            pt_t.is_hidden = true;
            pt_b.is_hidden = true;
            sketch.points.push(pt_t);
            sketch.points.push(pt_b);
            top_ids.push(t_id);
            bot_ids.push(b_id);
        }
        for i in 0..steps {
            let next = (i + 1) % steps;
            sketch.lines.push(GeoLine::new(format!("lt_{}", i), &top_ids[i], &top_ids[next], false));
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let is_dashed = theta.sin() < 0.0;
            sketch.lines.push(GeoLine::new(format!("lb_{}", i), &bot_ids[i], &bot_ids[next], is_dashed));
        }

        sketch.action_buttons.push(GeoActionButton {
            id: "btn_spin".to_string(),
            label: "QUAY 3D".to_string(),
            kind: ActionKind::Toggle3DRotate,
            pos: Pos2::new(30.0, 45.0),
        });
    }

    /// Loads 3D Cone.
    pub fn load_cone_3d(sketch: &mut SketchpadModel) {
        sketch.clear();
        sketch.camera3d.pitch = 0.30;
        let r = 85.0f32;
        let h = 130.0f32;

        let apex = GeoPoint::new_3d("pt_S", "S", [0.0, h * 0.6, 0.0]);
        let bot_c = GeoPoint::new_3d("pt_O", "O", [0.0, -h * 0.4, 0.0]);
        let left_b = GeoPoint::new_3d("pt_L", "A", [-r, -h * 0.4, 0.0]);
        let right_b = GeoPoint::new_3d("pt_R", "B", [r, -h * 0.4, 0.0]);

        sketch.points.extend(vec![apex, bot_c, left_b, right_b]);

        // Lateral generators
        sketch.lines.push(GeoLine::new("l_sa", "pt_S", "pt_L", false));
        sketch.lines.push(GeoLine::new("l_sb", "pt_S", "pt_R", false));
        sketch.lines.push(GeoLine::new("l_so", "pt_S", "pt_O", true)); // Axis dashed
        sketch.lines.push(GeoLine::new("l_oa", "pt_O", "pt_L", true)); // Radius dashed

        let steps = 24;
        let mut bot_ids = Vec::new();
        for i in 0..steps {
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let px = r * theta.cos();
            let pz = r * theta.sin();
            let b_id = format!("cone_b_{}", i);
            let mut pt_b = GeoPoint::new_3d(&b_id, "", [px, -h * 0.4, pz]);
            pt_b.is_hidden = true;
            sketch.points.push(pt_b);
            bot_ids.push(b_id);
        }
        for i in 0..steps {
            let next = (i + 1) % steps;
            let theta = std::f32::consts::TAU * (i as f32 / steps as f32);
            let is_dashed = theta.sin() < 0.0;
            sketch.lines.push(GeoLine::new(format!("cone_edge_{}", i), &bot_ids[i], &bot_ids[next], is_dashed));
        }

        sketch.action_buttons.push(GeoActionButton {
            id: "btn_spin".to_string(),
            label: "QUAY 3D".to_string(),
            kind: ActionKind::Toggle3DRotate,
            pos: Pos2::new(30.0, 45.0),
        });
    }
}
