//! StrokeAlgorithm.rs — Port of C++ INKK High-Performance Ink & Stroke Processing Engine
//!
//! Ported faithfully from `StrokeAlgorithm.h` and `StrokeAlgorithm.cpp`:
//! 1. One-Euro Adaptive Filter with Corner Detection (v5 architecture)
//! 2. Pen Dynamics (Fountain, Ballpoint, Pencil, Calligraphy, Highlighter)
//! 3. Centripetal Catmull-Rom to Cubic Bezier Curve Generation
//! 4. Decimation & Inertia End Point Flush

use crate::model::{PenType, Point};

const PI: f32 = std::f32::consts::PI;

// --- One-Euro filter tuning (AGGRESSIVE - Native feel like Scribble Ink) ---
// CỰC KỲ AGGRESSIVE: MIN_CUTOFF 2.2 Hz, BETA 0.07 - BÁM SÁT con trỏ, ZERO LAG!
// Chỉ lọc rung nhẹ nhưng GIỮ NGUYÊN mọi chuyển động thật → cảm giác NATIVE!
const ONE_EURO_MIN_CUTOFF: f32 = 2.2;
const ONE_EURO_BETA: f32 = 0.07;
const ONE_EURO_DCUTOFF: f32 = 0.8;

// --- Corner protection (base at 96 DPI / 100% scale) ---
// Góc quẹo thật (> 115 độ) mới kích hoạt boost, bảo vệ đường tròn và số 0, 2, 3 mượt mà
const CORNER_MIN_DIST_BASE: f32 = 2.4;
const CORNER_COS_THRESHOLD: f32 = -0.42;
const CORNER_CUTOFF_BOOST: f32 = 3.2;

// --- Decimation (base at 96 DPI / 100% scale) ---
// Thưa hơn (0.45 px) để nét GÓC CẠNH rõ ràng, CRISP như native, không bị tròn ủng
const DECIMATE_MIN_DIST_BASE: f32 = 0.45;
const DECIMATE_SPEED_SLOPE_BASE: f32 = 0.75;

// --- Speed normalization reference (base at 96 DPI) ---
const SPEED_REFERENCE_BASE: f32 = 1100.0;

// dt clamps (seconds)
const DT_MIN: f32 = 1.0 / 500.0; // 2ms
const DT_MAX: f32 = 1.0 / 20.0; // 50ms (stalled gap)

// --- Pressure response (Windows DirectInk-style) ---
// Cảm biến bút trả về P tuyến tính 0..1 nhưng cảm giác viết tự nhiên cần P^gamma.
// gamma < 1 => chạm nhẹ (P nhỏ) vẫn cho nét dày ngay, tránh nét "sợi chỉ" rồi bung dần.
pub const PRESSURE_GAMMA: f32 = 0.5;

/// Lọc rung lực nhấn (low-pass) + nén gamma — mô phỏng đường cong cảm nhận của DirectInk.
#[derive(Debug, Clone)]
pub struct PressureModel {
    last_smoothed_p: f32,
    /// Trọng số EMA: nhỏ = mượt hơn nhưng lag hơn
    alpha: f32,
}

impl Default for PressureModel {
    fn default() -> Self {
        Self {
            last_smoothed_p: 0.5,
            alpha: 0.45,
        }
    }
}

impl PressureModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self, raw_pressure: f32) {
        self.last_smoothed_p = raw_pressure.clamp(0.0, 1.0);
    }

    /// Trả về lực đã lọc rung (chưa áp gamma).
    pub fn filter(&mut self, raw_pressure: f32) -> f32 {
        let raw = raw_pressure.clamp(0.0, 1.0);
        let smoothed = self.alpha * raw + (1.0 - self.alpha) * self.last_smoothed_p;
        self.last_smoothed_p = smoothed;
        smoothed
    }

    pub fn smoothed(&self) -> f32 {
        self.last_smoothed_p
    }
}

/// Nén gamma của lực nhấn: ánh xạ lực tuyến tính của cảm biến sang cảm giác viết tự nhiên.
pub fn pressure_curve(pressure: f32) -> f32 {
    pressure.clamp(0.0, 1.0).powf(PRESSURE_GAMMA)
}

#[inline]
fn lowpass_alpha(cutoff_hz: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 1.0;
    }
    let tau = 1.0 / (2.0 * PI * cutoff_hz);
    1.0 / (1.0 + tau / dt)
}

/// Dynamic thickness based on pen physics model (Chỉnh mềm mại, tự nhiên, đỡ cứng)
/// Lấy cảm hứng từ phân tích Scrble Ink & C++ INKK PenDynamics + DirectInk curves.
///
/// **pressure_curve** đã được áp ở ngoài (đã qua gamma), nhận vào đây là lực cong.
pub fn compute_thickness(
    pen_type: PenType,
    base_thickness: f32,
    speed_factor: f32,
    pressure_curve: f32,
) -> f32 {
    let p = pressure_curve.clamp(0.0, 1.0);
    // Scale nét theo Scrble Ink: min_scale + range * p
    match pen_type {
        PenType::Ballpoint => {
            // Bút bi: [0.25 .. 1.75] theo đường cong lực
            let v_factor = 1.08 - speed_factor * 0.18;
            let scale = 0.25 + (1.75 - 0.25) * p;
            (base_thickness * scale * v_factor).max(0.7)
        }
        PenType::Fountain => {
            // Bút máy: [0.20 .. 1.80] biến thiên mạnh hơn
            let v_factor = 1.18 - speed_factor * 0.35;
            let scale = 0.20 + (1.80 - 0.20) * p;
            (base_thickness * scale * v_factor).max(0.6)
        }
        PenType::Pencil => {
            // Bút chì: [0.25 .. 1.60] đều hơn
            let v_factor = 1.05 - speed_factor * 0.16;
            let scale = 0.25 + (1.60 - 0.25) * p;
            (base_thickness * scale * v_factor).max(0.6)
        }
        PenType::Highlighter => base_thickness,
        PenType::Calligraphy => {
            // Bút thư pháp: [0.22 .. 1.75]
            let v_factor = 1.14 - speed_factor * 0.30;
            let scale = 0.22 + (1.75 - 0.22) * p;
            (base_thickness * scale * v_factor).max(0.8)
        }
    }
}

/// 1. INK STABILIZER & FILTERING PIPELINE (Realtime Input Processing)
/// Direct port of C++ StrokeAlgorithm::StabilizerPipeline
#[derive(Debug, Clone)]
pub struct StabilizerPipeline {
    filtered_x: f32,
    filtered_y: f32,
    dx_prev: f32,
    dy_prev: f32,
    prev_raw_x: f32,
    prev_raw_y: f32,
    prev_dir_x: f32,
    prev_dir_y: f32,
    last_emit_x: f32,
    last_emit_y: f32,
    pressure_model: PressureModel,
    prev_speed: f32,
    is_initialized: bool,
    last_timestamp_sec: f64,
    dpi_scale: f32,
}

impl Default for StabilizerPipeline {
    fn default() -> Self {
        Self {
            filtered_x: 0.0,
            filtered_y: 0.0,
            dx_prev: 0.0,
            dy_prev: 0.0,
            prev_raw_x: 0.0,
            prev_raw_y: 0.0,
            prev_dir_x: 0.0,
            prev_dir_y: 0.0,
            last_emit_x: 0.0,
            last_emit_y: 0.0,
            pressure_model: PressureModel::new(),
            prev_speed: 0.0,
            is_initialized: false,
            last_timestamp_sec: -1.0,
            dpi_scale: 1.0,
        }
    }
}

impl StabilizerPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_dpi_scale(&mut self, scale: f32) {
        self.dpi_scale = if scale > 0.01 { scale } else { 1.0 };
    }

    #[allow(dead_code)]
    pub fn get_dpi_scale(&self) -> f32 {
        self.dpi_scale
    }

    pub fn reset(&mut self, start_x: f32, start_y: f32, start_pressure: f32) {
        self.filtered_x = start_x;
        self.filtered_y = start_y;
        self.dx_prev = 0.0;
        self.dy_prev = 0.0;
        self.prev_raw_x = start_x;
        self.prev_raw_y = start_y;
        self.prev_dir_x = 0.0;
        self.prev_dir_y = 0.0;
        self.last_emit_x = start_x;
        self.last_emit_y = start_y;
        self.pressure_model.reset(start_pressure);
        self.prev_speed = 0.0;
        self.is_initialized = true;
        self.last_timestamp_sec = -1.0;
    }

    /// Feeds raw input point from pointer/mouse/stylus.
    /// `timestamp_sec`: monotonic hardware or QPC timestamp in seconds.
    /// Returns Some(Point) if a stabilized and decimated point was emitted.
    pub fn process_point(
        &mut self,
        raw_x: f32,
        raw_y: f32,
        raw_pressure: f32,
        base_thickness: f32,
        pen_type: PenType,
        timestamp_sec: f64,
    ) -> Option<Point> {
        if !self.is_initialized {
            self.reset(raw_x, raw_y, raw_pressure);
            self.last_timestamp_sec = timestamp_sec;
            let thickness = compute_thickness(
                pen_type,
                base_thickness,
                0.0,
                pressure_curve(raw_pressure),
            );
            return Some(Point::with_thickness(raw_x, raw_y, raw_pressure, thickness));
        }

        if self.last_timestamp_sec < 0.0 {
            self.last_timestamp_sec = timestamp_sec;
        }

        let mut dt = (timestamp_sec - self.last_timestamp_sec) as f32;
        if !(dt > 0.0) {
            dt = DT_MIN;
        } else {
            dt = dt.clamp(DT_MIN, DT_MAX);
        }
        self.last_timestamp_sec = timestamp_sec;

        let corner_min_dist = CORNER_MIN_DIST_BASE * self.dpi_scale;
        let decimate_min_dist_base = DECIMATE_MIN_DIST_BASE * self.dpi_scale;
        let decimate_speed_slope = DECIMATE_SPEED_SLOPE_BASE * self.dpi_scale;
        let speed_reference = SPEED_REFERENCE_BASE * self.dpi_scale;

        // --- CORNER DETECTION (on the raw stream) ---
        let rdx = raw_x - self.prev_raw_x;
        let rdy = raw_y - self.prev_raw_y;
        let raw_dist = (rdx * rdx + rdy * rdy).sqrt();

        let mut corner_boost = 1.0;
        if raw_dist > corner_min_dist {
            let inv = 1.0 / raw_dist;
            let dir_x = rdx * inv;
            let dir_y = rdy * inv;
            if self.prev_dir_x != 0.0 || self.prev_dir_y != 0.0 {
                let cos_angle = dir_x * self.prev_dir_x + dir_y * self.prev_dir_y;
                if cos_angle < CORNER_COS_THRESHOLD {
                    corner_boost = CORNER_CUTOFF_BOOST;
                    self.dx_prev = 0.0;
                    self.dy_prev = 0.0;
                }
            }
            self.prev_dir_x = dir_x;
            self.prev_dir_y = dir_y;
        }
        self.prev_raw_x = raw_x;
        self.prev_raw_y = raw_y;

        // --- ONE-EURO FILTER (2D-coupled) ---
        let inst_dx = (raw_x - self.filtered_x) / dt;
        let inst_dy = (raw_y - self.filtered_y) / dt;
        let a_d = lowpass_alpha(ONE_EURO_DCUTOFF, dt);
        self.dx_prev = a_d * inst_dx + (1.0 - a_d) * self.dx_prev;
        self.dy_prev = a_d * inst_dy + (1.0 - a_d) * self.dy_prev;
        let speed = (self.dx_prev * self.dx_prev + self.dy_prev * self.dy_prev).sqrt();

        let cutoff = (ONE_EURO_MIN_CUTOFF + ONE_EURO_BETA * speed) * corner_boost;
        let a = lowpass_alpha(cutoff, dt);

        self.filtered_x = a * raw_x + (1.0 - a) * self.filtered_x;
        self.filtered_y = a * raw_y + (1.0 - a) * self.filtered_y;

        // --- PRESSURE: Lọc rung (EMA) + nén gamma (DirectInk-style response) ---
        let smoothed_pressure = self.pressure_model.filter(raw_pressure);
        let curved_pressure = pressure_curve(smoothed_pressure);
        self.prev_speed = speed;
        let speed_factor = (speed / speed_reference).min(1.0);

        // --- ADAPTIVE DISTANCE SAMPLING (decimation) ---
        let dx = self.filtered_x - self.last_emit_x;
        let dy = self.filtered_y - self.last_emit_y;
        let min_dist = decimate_min_dist_base + speed_factor * decimate_speed_slope;

        if dx * dx + dy * dy > min_dist * min_dist || corner_boost > 1.0 {
            let dynamic_thickness =
                compute_thickness(pen_type, base_thickness, speed_factor, curved_pressure);
            self.last_emit_x = self.filtered_x;
            self.last_emit_y = self.filtered_y;
            Some(Point::with_thickness(
                self.filtered_x,
                self.filtered_y,
                smoothed_pressure, // point giữ lực gốc để render/serialize, thickness đã qua gamma
                dynamic_thickness,
            ))
        } else {
            None
        }
    }

    /// Flushes final resting point on stroke end if inertia distance exceeds DPI-scaled threshold.
    pub fn flush_end_point(&mut self, base_thickness: f32, pen_type: PenType) -> Option<Point> {
        if !self.is_initialized {
            return None;
        }

        let fdx = self.prev_raw_x - self.last_emit_x;
        let fdy = self.prev_raw_y - self.last_emit_y;
        let flush_thresh = 0.15 * self.dpi_scale;
        if fdx * fdx + fdy * fdy > flush_thresh * flush_thresh {
            let smoothed_p = self.pressure_model.smoothed();
            let curved_p = pressure_curve(smoothed_p);
            let dynamic_thickness = compute_thickness(pen_type, base_thickness, 0.0, curved_p);
            self.last_emit_x = self.prev_raw_x;
            self.last_emit_y = self.prev_raw_y;
            Some(Point::with_thickness(
                self.prev_raw_x,
                self.prev_raw_y,
                smoothed_p,
                dynamic_thickness,
            ))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn current_pos(&self) -> (f32, f32) {
        (self.filtered_x, self.filtered_y)
    }

    #[allow(dead_code)]
    pub fn current_pressure(&self) -> f32 {
        self.pressure_model.smoothed()
    }
}
