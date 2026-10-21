//! StrokeAlgorithm.rs — Port of C++ INKK High-Performance Ink & Stroke Processing Engine
//!
//! Ported faithfully from `StrokeAlgorithm.h` and `StrokeAlgorithm.cpp`:
//! 1. One-Euro Adaptive Filter with Corner Detection (v5 architecture)
//! 2. Pen Dynamics (Fountain, Ballpoint, Pencil, Calligraphy, Highlighter)
//! 3. Centripetal Catmull-Rom to Cubic Bezier Curve Generation
//! 4. Decimation & Inertia End Point Flush

use crate::model::{PenType, Point};

const PI: f32 = std::f32::consts::PI;

// --- One-Euro filter tuning ---
// Tăng ONE_EURO_BETA và MIN_CUTOFF để bút phản hồi tức thì, không bị lag/trễ khiến cảm giác viết bị "cứng đơ"
const ONE_EURO_MIN_CUTOFF: f32 = 1.2;
const ONE_EURO_BETA: f32 = 0.045;
const ONE_EURO_DCUTOFF: f32 = 1.5;

// --- Corner protection (base at 96 DPI / 100% scale) ---
const CORNER_MIN_DIST_BASE: f32 = 1.8;
const CORNER_COS_THRESHOLD: f32 = -0.15; // Phản xạ góc nhạy hơn
const CORNER_CUTOFF_BOOST: f32 = 5.0; // Boost nhanh khi quẹo góc chữ

// --- Decimation (base at 96 DPI / 100% scale) ---
// Giảm ngưỡng decimation để thu nhiều vi điểm hơn ở các khúc lượn cong
const DECIMATE_MIN_DIST_BASE: f32 = 0.15;
const DECIMATE_SPEED_SLOPE_BASE: f32 = 0.5;

// --- Speed normalization reference (base at 96 DPI) ---
const SPEED_REFERENCE_BASE: f32 = 1100.0;

// dt clamps (seconds)
const DT_MIN: f32 = 1.0 / 1000.0; // 1ms
const DT_MAX: f32 = 1.0 / 20.0; // 50ms (stalled gap)

#[inline]
fn lowpass_alpha(cutoff_hz: f32, dt: f32) -> f32 {
    if dt <= 0.0 {
        return 1.0;
    }
    let tau = 1.0 / (2.0 * PI * cutoff_hz);
    1.0 / (1.0 + tau / dt)
}

/// Dynamic thickness based on pen physics model
/// Tạo ra nét thanh nét đậm tự nhiên: Viết nhanh thì thanh mảnh vuốt nhọn, viết chậm/nhấn thì đậm đà
pub fn compute_thickness(
    pen_type: PenType,
    base_thickness: f32,
    speed_factor: f32,
    pressure: f32,
) -> f32 {
    let p = pressure.clamp(0.05, 1.0);
    match pen_type {
        PenType::Ballpoint => {
            // Bút bi chất lượng cao: Tốc độ cao thì hơi thanh (0.75x), nhấn mạnh thì nở nét (1.25x)
            let v_factor = 1.15 - speed_factor * 0.40;
            let p_factor = 0.70 + p * 0.60;
            (base_thickness * v_factor * p_factor).max(0.6)
        }
        PenType::Fountain => {
            // Bút máy: Độ biến thiên rất nhạy, thanh mảnh ở đỉnh chữ, đậm ở bụng chữ
            let v_factor = 1.30 - speed_factor * 0.65;
            let p_factor = 0.55 + p * 0.85;
            (base_thickness * v_factor * p_factor).max(0.5)
        }
        PenType::Pencil => {
            let v_factor = 1.10 - speed_factor * 0.25;
            let p_factor = 0.65 + p * 0.55;
            (base_thickness * v_factor * p_factor).max(0.6)
        }
        PenType::Highlighter => base_thickness,
        PenType::Calligraphy => {
            // Nét bút vát
            let p_factor = 0.75 + p * 0.55;
            base_thickness * p_factor
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
    smooth_pressure: f32,
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
            smooth_pressure: 0.5,
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
        self.smooth_pressure = start_pressure;
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
            let thickness = compute_thickness(pen_type, base_thickness, 0.0, raw_pressure);
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

        // --- PRESSURE: light single-pass smoothing ---
        self.smooth_pressure = 0.35 * raw_pressure + 0.65 * self.smooth_pressure;
        self.prev_speed = speed;
        let speed_factor = (speed / speed_reference).min(1.0);

        // --- ADAPTIVE DISTANCE SAMPLING (decimation) ---
        let dx = self.filtered_x - self.last_emit_x;
        let dy = self.filtered_y - self.last_emit_y;
        let min_dist = decimate_min_dist_base + speed_factor * decimate_speed_slope;

        if dx * dx + dy * dy > min_dist * min_dist || corner_boost > 1.0 {
            let dynamic_thickness =
                compute_thickness(pen_type, base_thickness, speed_factor, self.smooth_pressure);
            self.last_emit_x = self.filtered_x;
            self.last_emit_y = self.filtered_y;
            Some(Point::with_thickness(
                self.filtered_x,
                self.filtered_y,
                self.smooth_pressure,
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
        let flush_thresh = 0.2 * self.dpi_scale;
        if fdx * fdx + fdy * fdy > flush_thresh * flush_thresh {
            let dynamic_thickness =
                compute_thickness(pen_type, base_thickness, 0.0, self.smooth_pressure);
            self.last_emit_x = self.prev_raw_x;
            self.last_emit_y = self.prev_raw_y;
            Some(Point::with_thickness(
                self.prev_raw_x,
                self.prev_raw_y,
                self.smooth_pressure,
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
        self.smooth_pressure
    }
}
