//! MathValue: Matrices (column-major), Strings, Linear Algebra & Formatting
use crate::math_engine::error::MathError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathValue {
    Matrix {
        rows: usize,
        cols: usize,
        data: Vec<f64>, // column-major
    },
    Str(String),
    AnonymousFn {
        params: Vec<String>,
        body: Box<crate::math_engine::ast::Expr>,
    },
}

impl MathValue {
    // --- Constructors ---

    pub fn scalar(val: f64) -> Self {
        MathValue::Matrix {
            rows: 1,
            cols: 1,
            data: vec![val],
        }
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        MathValue::Matrix {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn ones(rows: usize, cols: usize) -> Self {
        MathValue::Matrix {
            rows,
            cols,
            data: vec![1.0; rows * cols],
        }
    }

    pub fn eye(rows: usize, cols: usize) -> Self {
        let mut data = vec![0.0; rows * cols];
        let n = rows.min(cols);
        for i in 0..n {
            data[i * rows + i] = 1.0;
        }
        MathValue::Matrix { rows, cols, data }
    }

    pub fn empty() -> Self {
        MathValue::Matrix {
            rows: 0,
            cols: 0,
            data: Vec::new(),
        }
    }

    pub fn string(s: impl Into<String>) -> Self {
        MathValue::Str(s.into())
    }

    pub fn from_rows(raw: Vec<Vec<f64>>) -> Result<Self, MathError> {
        let rows = raw.len();
        if rows == 0 {
            return Ok(Self::empty());
        }
        let cols = raw[0].len();
        for r in &raw {
            if r.len() != cols {
                return Err(MathError::Other(
                    "Inconsistent row lengths in matrix".into(),
                ));
            }
        }
        let mut data = vec![0.0; rows * cols];
        for (r_idx, row) in raw.iter().enumerate() {
            for (c_idx, &val) in row.iter().enumerate() {
                data[c_idx * rows + r_idx] = val;
            }
        }
        Ok(MathValue::Matrix { rows, cols, data })
    }

    // --- Accessors ---

    pub fn rows(&self) -> usize {
        match self {
            MathValue::Matrix { rows, .. } => *rows,
            MathValue::Str(_) => 1,
            MathValue::AnonymousFn { .. } => 1,
        }
    }

    pub fn cols(&self) -> usize {
        match self {
            MathValue::Matrix { cols, .. } => *cols,
            MathValue::Str(s) => s.len(),
            MathValue::AnonymousFn { .. } => 1,
        }
    }

    pub fn numel(&self) -> usize {
        match self {
            MathValue::Matrix { rows, cols, .. } => rows * cols,
            MathValue::Str(s) => s.len(),
            MathValue::AnonymousFn { .. } => 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MathValue::Matrix { rows, cols, .. } => *rows == 0 || *cols == 0,
            MathValue::Str(s) => s.is_empty(),
            MathValue::AnonymousFn { .. } => false,
        }
    }

    pub fn is_scalar(&self) -> bool {
        match self {
            MathValue::Matrix { rows, cols, .. } => *rows == 1 && *cols == 1,
            _ => false,
        }
    }

    pub fn as_scalar(&self) -> Option<f64> {
        match self {
            MathValue::Matrix { rows, cols, data } if *rows == 1 && *cols == 1 => Some(data[0]),
            _ => None,
        }
    }

    pub fn get(&self, r: usize, c: usize) -> Option<f64> {
        match self {
            MathValue::Matrix { rows, cols, data } => {
                if r < *rows && c < *cols {
                    Some(data[c * *rows + r])
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_linear(&self, idx: usize) -> Option<f64> {
        match self {
            MathValue::Matrix { data, .. } => data.get(idx).copied(),
            _ => None,
        }
    }

    pub fn to_vector(&self) -> Option<Vec<f64>> {
        match self {
            MathValue::Matrix { rows, cols, data } => {
                if *rows == 1 || *cols == 1 {
                    Some(data.clone())
                } else if *rows == 0 && *cols == 0 {
                    Some(Vec::new())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn to_2d_vec(&self) -> Vec<Vec<f64>> {
        match self {
            MathValue::Matrix { rows, cols, data } => {
                let mut out = vec![vec![0.0; *cols]; *rows];
                for r in 0..*rows {
                    for c in 0..*cols {
                        out[r][c] = data[c * *rows + r];
                    }
                }
                out
            }
            _ => Vec::new(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            MathValue::Matrix { .. } => "double",
            MathValue::Str(_) => "char",
            MathValue::AnonymousFn { .. } => "function_handle",
        }
    }

    pub fn size_string(&self) -> String {
        match self {
            MathValue::Matrix { rows, cols, .. } => format!("{}x{} double", rows, cols),
            MathValue::Str(s) => format!("1x{} char", s.len()),
            MathValue::AnonymousFn { .. } => "1x1 function_handle".to_string(),
        }
    }

    // --- Transpose ---

    pub fn transpose(&self) -> Result<Self, MathError> {
        match self {
            MathValue::Matrix { rows, cols, data } => {
                let new_rows = *cols;
                let new_cols = *rows;
                let mut new_data = vec![0.0; new_rows * new_cols];
                for r in 0..*rows {
                    for c in 0..*cols {
                        let old_idx = c * *rows + r;
                        let new_idx = r * new_rows + c;
                        new_data[new_idx] = data[old_idx];
                    }
                }
                Ok(MathValue::Matrix {
                    rows: new_rows,
                    cols: new_cols,
                    data: new_data,
                })
            }
            MathValue::Str(s) => {
                // String transpose in MATLAB becomes char matrix Nx1
                Ok(MathValue::Str(s.clone()))
            }
            MathValue::AnonymousFn { .. } => Ok(self.clone()),
        }
    }

    // --- Concatenation ---

    pub fn concat_horizontal(left: &Self, right: &Self) -> Result<Self, MathError> {
        match (left, right) {
            (
                MathValue::Matrix {
                    rows: r1,
                    cols: c1,
                    data: d1,
                },
                MathValue::Matrix {
                    rows: r2,
                    cols: c2,
                    data: d2,
                },
            ) => {
                if *r1 == 0 && *c1 == 0 {
                    return Ok(right.clone());
                }
                if *r2 == 0 && *c2 == 0 {
                    return Ok(left.clone());
                }
                if r1 != r2 {
                    return Err(MathError::ArraysIncompatibleSizes {
                        r1: *r1,
                        c1: *c1,
                        r2: *r2,
                        c2: *c2,
                    });
                }
                let mut new_data = Vec::with_capacity(d1.len() + d2.len());
                new_data.extend_from_slice(d1);
                new_data.extend_from_slice(d2);
                Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1 + *c2,
                    data: new_data,
                })
            }
            (MathValue::Str(s1), MathValue::Str(s2)) => Ok(MathValue::Str(format!("{}{}", s1, s2))),
            _ => Err(MathError::Other(
                "Cannot mix types or concatenate function handles".into(),
            )),
        }
    }

    pub fn concat_vertical(top: &Self, bottom: &Self) -> Result<Self, MathError> {
        match (top, bottom) {
            (
                MathValue::Matrix {
                    rows: r1,
                    cols: c1,
                    data: d1,
                },
                MathValue::Matrix {
                    rows: r2,
                    cols: c2,
                    data: d2,
                },
            ) => {
                if *r1 == 0 && *c1 == 0 {
                    return Ok(bottom.clone());
                }
                if *r2 == 0 && *c2 == 0 {
                    return Ok(top.clone());
                }
                if c1 != c2 {
                    return Err(MathError::ArraysIncompatibleSizes {
                        r1: *r1,
                        c1: *c1,
                        r2: *r2,
                        c2: *c2,
                    });
                }
                let new_rows = *r1 + *r2;
                let mut new_data = vec![0.0; new_rows * *c1];
                for c in 0..*c1 {
                    for r in 0..*r1 {
                        new_data[c * new_rows + r] = d1[c * *r1 + r];
                    }
                    for r in 0..*r2 {
                        new_data[c * new_rows + *r1 + r] = d2[c * *r2 + r];
                    }
                }
                Ok(MathValue::Matrix {
                    rows: new_rows,
                    cols: *c1,
                    data: new_data,
                })
            }
            _ => Err(MathError::Other(
                "Cannot vertically concatenate non-matrix types".into(),
            )),
        }
    }

    // --- Indexing ---

    pub fn index_1d(&self, k: usize) -> Result<f64, MathError> {
        if k < 1 {
            return Err(MathError::IndexMustBePositiveInteger);
        }
        let total = self.numel();
        if k > total {
            return Err(MathError::IndexOutOfBounds {
                requested: k,
                max: total,
            });
        }
        self.get_linear(k - 1).ok_or(MathError::IndexOutOfBounds {
            requested: k,
            max: total,
        })
    }

    pub fn index_2d(&self, r: usize, c: usize) -> Result<f64, MathError> {
        if r < 1 || c < 1 {
            return Err(MathError::IndexMustBePositiveInteger);
        }
        let (rows, cols) = (self.rows(), self.cols());
        if r > rows {
            return Err(MathError::IndexOutOfBounds {
                requested: r,
                max: rows,
            });
        }
        if c > cols {
            return Err(MathError::IndexOutOfBounds {
                requested: c,
                max: cols,
            });
        }
        self.get(r - 1, c - 1).ok_or(MathError::IndexOutOfBounds {
            requested: r,
            max: rows,
        })
    }

    pub fn slice_col(&self, c: usize) -> Result<Self, MathError> {
        if c < 1 {
            return Err(MathError::IndexMustBePositiveInteger);
        }
        let (rows, cols) = (self.rows(), self.cols());
        if c > cols {
            return Err(MathError::IndexOutOfBounds {
                requested: c,
                max: cols,
            });
        }
        let c_idx = c - 1;
        let mut col_data = Vec::with_capacity(rows);
        for r in 0..rows {
            col_data.push(self.get(r, c_idx).unwrap());
        }
        Ok(MathValue::Matrix {
            rows,
            cols: 1,
            data: col_data,
        })
    }

    pub fn slice_row(&self, r: usize) -> Result<Self, MathError> {
        if r < 1 {
            return Err(MathError::IndexMustBePositiveInteger);
        }
        let (rows, cols) = (self.rows(), self.cols());
        if r > rows {
            return Err(MathError::IndexOutOfBounds {
                requested: r,
                max: rows,
            });
        }
        let r_idx = r - 1;
        let mut row_data = Vec::with_capacity(cols);
        for c in 0..cols {
            row_data.push(self.get(r_idx, c).unwrap());
        }
        Ok(MathValue::Matrix {
            rows: 1,
            cols,
            data: row_data,
        })
    }

    pub fn assign_2d(&mut self, r: usize, c: usize, val: f64) -> Result<(), MathError> {
        if r < 1 || c < 1 {
            return Err(MathError::IndexMustBePositiveInteger);
        }
        match self {
            MathValue::Matrix { rows, cols, data } => {
                let req_r = r;
                let req_c = c;
                if req_r > *rows || req_c > *cols {
                    // Auto expand with zeros
                    let new_rows = (*rows).max(req_r);
                    let new_cols = (*cols).max(req_c);
                    let mut new_data = vec![0.0; new_rows * new_cols];
                    for col_idx in 0..*cols {
                        for row_idx in 0..*rows {
                            new_data[col_idx * new_rows + row_idx] =
                                data[col_idx * *rows + row_idx];
                        }
                    }
                    *rows = new_rows;
                    *cols = new_cols;
                    *data = new_data;
                }
                data[(c - 1) * *rows + (r - 1)] = val;
                Ok(())
            }
            _ => Err(MathError::Other("Cannot index into string".into())),
        }
    }

    pub fn assign_1d(&mut self, k: usize, val: f64) -> Result<(), MathError> {
        if k < 1 {
            return Err(MathError::IndexMustBePositiveInteger);
        }
        match self {
            MathValue::Matrix { rows, cols, data } => {
                if *rows == 0 && *cols == 0 {
                    *rows = 1;
                    *cols = k;
                    *data = vec![0.0; k];
                    data[k - 1] = val;
                    return Ok(());
                }
                if *rows == 1 {
                    // Row vector expansion
                    if k > *cols {
                        data.resize(k, 0.0);
                        *cols = k;
                    }
                    data[k - 1] = val;
                    return Ok(());
                }
                if *cols == 1 {
                    // Column vector expansion
                    if k > *rows {
                        data.resize(k, 0.0);
                        *rows = k;
                    }
                    data[k - 1] = val;
                    return Ok(());
                }
                // General 2D linear assignment
                let total = *rows * *cols;
                if k > total {
                    // Expand columns
                    let extra = k - total;
                    let extra_cols = (extra + *rows - 1) / *rows;
                    let new_cols = *cols + extra_cols;
                    data.resize(*rows * new_cols, 0.0);
                    *cols = new_cols;
                }
                data[k - 1] = val;
                Ok(())
            }
            _ => Err(MathError::Other("Cannot index into string".into())),
        }
    }

    // --- Linear Algebra ---

    pub fn det(&self) -> Result<f64, MathError> {
        let (rows, cols) = (self.rows(), self.cols());
        if rows != cols {
            return Err(MathError::MatrixMustBeSquare);
        }
        let n = rows;
        if n == 0 {
            return Ok(1.0);
        }
        let mut a = vec![vec![0.0; n]; n];
        for r in 0..n {
            for c in 0..n {
                a[r][c] = self.get(r, c).unwrap();
            }
        }
        let mut det = 1.0;
        for i in 0..n {
            let mut max_row = i;
            let mut max_val = a[i][i].abs();
            for k in (i + 1)..n {
                if a[k][i].abs() > max_val {
                    max_val = a[k][i].abs();
                    max_row = k;
                }
            }
            if max_val < 1e-14 {
                return Ok(0.0);
            }
            if max_row != i {
                a.swap(i, max_row);
                det = -det;
            }
            det *= a[i][i];
            let pivot = a[i][i];
            for j in (i + 1)..n {
                let factor = a[j][i] / pivot;
                for k in (i + 1)..n {
                    a[j][k] -= factor * a[i][k];
                }
            }
        }
        Ok(det)
    }

    pub fn inv(&self) -> Result<Self, MathError> {
        let (rows, cols) = (self.rows(), self.cols());
        if rows != cols {
            return Err(MathError::MatrixMustBeSquare);
        }
        let n = rows;
        if n == 0 {
            return Ok(Self::empty());
        }

        // Augmented matrix [A | I]
        let mut aug = vec![vec![0.0; 2 * n]; n];
        for r in 0..n {
            for c in 0..n {
                aug[r][c] = self.get(r, c).unwrap();
            }
            aug[r][n + r] = 1.0;
        }

        for i in 0..n {
            let mut max_row = i;
            let mut max_val = aug[i][i].abs();
            for k in (i + 1)..n {
                if aug[k][i].abs() > max_val {
                    max_val = aug[k][i].abs();
                    max_row = k;
                }
            }
            if max_val < 1e-12 {
                return Err(MathError::MatrixIsSingular);
            }
            if max_row != i {
                aug.swap(i, max_row);
            }
            let pivot = aug[i][i];
            for j in 0..2 * n {
                aug[i][j] /= pivot;
            }
            for k in 0..n {
                if k != i {
                    let factor = aug[k][i];
                    for j in 0..2 * n {
                        aug[k][j] -= factor * aug[i][j];
                    }
                }
            }
        }

        let mut data = vec![0.0; n * n];
        for r in 0..n {
            for c in 0..n {
                data[c * n + r] = aug[r][n + c];
            }
        }
        Ok(MathValue::Matrix {
            rows: n,
            cols: n,
            data,
        })
    }

    pub fn solve_linear(&self, b: &Self) -> Result<Self, MathError> {
        let (rows_a, cols_a) = (self.rows(), self.cols());
        if rows_a != cols_a {
            return Err(MathError::MatrixMustBeSquare);
        }
        let (rows_b, cols_b) = (b.rows(), b.cols());
        if rows_a != rows_b {
            return Err(MathError::InnerMatrixDimensionsMismatch {
                r1: rows_a,
                c1: cols_a,
                r2: rows_b,
                c2: cols_b,
            });
        }
        let n = rows_a;
        let m = cols_b;
        if n == 0 {
            return Ok(Self::empty());
        }

        // Augmented matrix [A | B]
        let mut aug = vec![vec![0.0; n + m]; n];
        for r in 0..n {
            for c in 0..n {
                aug[r][c] = self.get(r, c).unwrap();
            }
            for c in 0..m {
                aug[r][n + c] = b.get(r, c).unwrap();
            }
        }

        for i in 0..n {
            let mut max_row = i;
            let mut max_val = aug[i][i].abs();
            for k in (i + 1)..n {
                if aug[k][i].abs() > max_val {
                    max_val = aug[k][i].abs();
                    max_row = k;
                }
            }
            if max_val < 1e-12 {
                return Err(MathError::MatrixIsSingular);
            }
            if max_row != i {
                aug.swap(i, max_row);
            }
            let pivot = aug[i][i];
            for j in 0..n + m {
                aug[i][j] /= pivot;
            }
            for k in 0..n {
                if k != i {
                    let factor = aug[k][i];
                    for j in 0..n + m {
                        aug[k][j] -= factor * aug[i][j];
                    }
                }
            }
        }

        let mut data = vec![0.0; n * m];
        for r in 0..n {
            for c in 0..m {
                data[c * n + r] = aug[r][n + c];
            }
        }
        Ok(MathValue::Matrix {
            rows: n,
            cols: m,
            data,
        })
    }

    pub fn trace(&self) -> Result<f64, MathError> {
        let (rows, cols) = (self.rows(), self.cols());
        if rows != cols {
            return Err(MathError::MatrixMustBeSquare);
        }
        let mut sum = 0.0;
        for i in 0..rows {
            sum += self.get(i, i).unwrap();
        }
        Ok(sum)
    }

    pub fn rank(&self) -> usize {
        let (rows, cols) = (self.rows(), self.cols());
        if rows == 0 || cols == 0 {
            return 0;
        }
        let mut a = vec![vec![0.0; cols]; rows];
        let mut max_abs: f64 = 0.0;
        for r in 0..rows {
            for c in 0..cols {
                let v = self.get(r, c).unwrap();
                a[r][c] = v;
                if v.abs() > max_abs {
                    max_abs = v.abs();
                }
            }
        }
        let tol = (rows.max(cols) as f64) * max_abs * f64::EPSILON;
        let mut rank = 0;
        let mut col = 0;
        for row in 0..rows {
            if col >= cols {
                break;
            }
            let mut pivot = row;
            while col < cols {
                for r in row..rows {
                    if a[r][col].abs() > a[pivot][col].abs() {
                        pivot = r;
                    }
                }
                if a[pivot][col].abs() > tol {
                    break;
                }
                col += 1;
            }
            if col >= cols {
                break;
            }
            a.swap(row, pivot);
            let pivot_val = a[row][col];
            for r in (row + 1)..rows {
                let factor = a[r][col] / pivot_val;
                for c in col..cols {
                    a[r][c] -= factor * a[row][c];
                }
            }
            rank += 1;
            col += 1;
        }
        rank
    }

    // --- Formatting for Output ---

    pub fn to_display_string(&self) -> String {
        self.to_display_string_with_mode(crate::math_engine::ast::FormatMode::Short)
    }

    pub fn to_display_string_with_mode(&self, mode: crate::math_engine::ast::FormatMode) -> String {
        match self {
            MathValue::Str(s) => s.clone(),
            MathValue::AnonymousFn { params, .. } => format!("@({})", params.join(",")),
            MathValue::Matrix { rows, cols, data } => {
                if *rows == 0 || *cols == 0 {
                    return "[](0x0)".to_string();
                }
                if *rows == 1 && *cols == 1 {
                    return format_scalar_display_mode(data[0], mode);
                }

                // Check if any element requires decimal/scientific formatting
                let has_non_int = data.iter().any(|&x| {
                    !x.is_finite()
                        || x.fract() != 0.0
                        || x.abs() >= 1e5
                        || (x != 0.0 && x.abs() < 1e-4)
                });

                let mut formatted = vec![vec![String::new(); *cols]; *rows];
                for r in 0..*rows {
                    for c in 0..*cols {
                        let val = data[c * *rows + r];
                        formatted[r][c] = match mode {
                            crate::math_engine::ast::FormatMode::Rat => float_to_rat(val),
                            crate::math_engine::ast::FormatMode::Long => format!("{:.15}", val),
                            crate::math_engine::ast::FormatMode::Short => {
                                if has_non_int {
                                    format_matrix_float(val)
                                } else {
                                    format!("{:.0}", val)
                                }
                            }
                        };
                    }
                }

                // Calculate column widths
                let mut col_widths = vec![0usize; *cols];
                for c in 0..*cols {
                    for r in 0..*rows {
                        col_widths[c] = col_widths[c].max(formatted[r][c].len());
                    }
                }

                let mut lines = Vec::new();
                for r in 0..*rows {
                    let mut line = String::from("     "); // 5 spaces indent
                    for c in 0..*cols {
                        let s = &formatted[r][c];
                        let width = col_widths[c];
                        let pad = width.saturating_sub(s.len());
                        line.push_str(&" ".repeat(pad));
                        line.push_str(s);
                        if c + 1 < *cols {
                            line.push_str("     "); // 5 spaces between cols
                        }
                    }
                    lines.push(line);
                }
                lines.join("\n")
            }
        }
    }
}

// --- Float, Rational & Scientific Formatting Helpers ---

pub fn float_to_rat(x: f64) -> String {
    if !x.is_finite() {
        return format_scalar_display(x);
    }
    if x.fract() == 0.0 && x.abs() < 1e12 {
        return format!("{:.0}", x);
    }
    let sign = if x < 0.0 { "-" } else { "" };
    let abs_x = x.abs();

    let mut h0: i64 = 0;
    let mut h1: i64 = 1;
    let mut k0: i64 = 1;
    let mut k1: i64 = 0;

    let mut val = abs_x;
    let max_denominator: i64 = 10000;
    let eps: f64 = 1e-10;

    for _ in 0..50 {
        let a = val.floor() as i64;
        let h2 = a.saturating_mul(h1).saturating_add(h0);
        let k2 = a.saturating_mul(k1).saturating_add(k0);

        if k2 > max_denominator || k2 <= 0 {
            break;
        }

        h0 = h1;
        h1 = h2;
        k0 = k1;
        k1 = k2;

        let frac = val - a as f64;
        let diff = (abs_x - (h1 as f64 / k1 as f64)).abs();
        if frac < eps || diff < eps {
            break;
        }
        val = 1.0 / frac;
    }

    if k1 == 1 {
        format!("{}{}", sign, h1)
    } else if k1 > 1 && (abs_x - (h1 as f64 / k1 as f64)).abs() < 1e-6 {
        format!("{}{}/{}", sign, h1, k1)
    } else {
        format_scalar_display(x)
    }
}

pub fn format_scalar_display_mode(val: f64, mode: crate::math_engine::ast::FormatMode) -> String {
    match mode {
        crate::math_engine::ast::FormatMode::Rat => float_to_rat(val),
        crate::math_engine::ast::FormatMode::Long => {
            if val.is_nan() {
                "NaN".to_string()
            } else if val == f64::INFINITY {
                "Inf".to_string()
            } else if val == f64::NEG_INFINITY {
                "-Inf".to_string()
            } else if val.fract() == 0.0 && val.abs() < 1e12 {
                format!("{:.0}", val)
            } else {
                format!("{:.15}", val)
            }
        }
        crate::math_engine::ast::FormatMode::Short => format_scalar_display(val),
    }
}

pub fn num2str(val: f64) -> String {
    if !val.is_finite() {
        return format_scalar_display(val);
    }
    if val.fract() == 0.0 && val.abs() < 1e14 {
        return format!("{:.0}", val);
    }
    let abs_val = val.abs();
    let n = if abs_val > 0.0 {
        ((abs_val.log10().floor() as i32) + 5).max(5).min(16) as usize
    } else {
        5
    };
    format_g(val, n)
}

fn format_g(val: f64, prec: usize) -> String {
    // MATLAB num2str formatting using significant digits
    let s = format!("{:.*e}", prec - 1, val);
    if let Some(pos) = s.find('e') {
        let exp_val: i32 = s[pos + 1..].parse().unwrap_or(0);
        if exp_val >= -4 && exp_val < prec as i32 {
            // Fixed point
            let decimals = (prec as i32 - 1 - exp_val).max(0) as usize;
            let fixed = format!("{:.*}", decimals, val);
            let trimmed = fixed.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        } else {
            // Scientific
            let base = s[..pos].trim_end_matches('0').trim_end_matches('.');
            if exp_val >= 0 {
                format!("{}e+{:02}", base, exp_val)
            } else {
                format!("{}e-{:02}", base, exp_val.abs())
            }
        }
    } else {
        format!("{}", val)
    }
}

// --- Float & Scientific Formatting Helpers ---

pub fn format_scientific(val: f64) -> String {
    let s = format!("{:1.4e}", val);
    if let Some(pos) = s.find('e') {
        let base = &s[..pos];
        let exp_str = &s[pos + 1..];
        let exp_num: i32 = exp_str.parse().unwrap_or(0);
        if exp_num >= 0 {
            format!("{}e+{:02}", base, exp_num)
        } else {
            format!("{}e-{:02}", base, exp_num.abs())
        }
    } else {
        s
    }
}

pub fn format_scalar_display(val: f64) -> String {
    if val.is_nan() {
        return "NaN".to_string();
    }
    if val == f64::INFINITY {
        return "Inf".to_string();
    }
    if val == f64::NEG_INFINITY {
        return "-Inf".to_string();
    }
    let abs_val = val.abs();
    if abs_val >= 1e5 || (abs_val > 0.0 && abs_val < 1e-4) {
        format_scientific(val)
    } else if val.fract() == 0.0 {
        format!("{:.0}", val)
    } else {
        format!("{:.4}", val)
    }
}

pub fn format_matrix_float(val: f64) -> String {
    if val.is_nan() {
        return "NaN".to_string();
    }
    if val == f64::INFINITY {
        return "Inf".to_string();
    }
    if val == f64::NEG_INFINITY {
        return "-Inf".to_string();
    }
    let abs_val = val.abs();
    if abs_val >= 1e5 || (abs_val > 0.0 && abs_val < 1e-4) {
        format_scientific(val)
    } else {
        format!("{:.4}", val)
    }
}

// --- Operators Implementation ---

pub fn op_add(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| s + x).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| x + s).collect(),
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: d1.iter().zip(d2.iter()).map(|(&x, &y)| x + y).collect(),
            })
        }
        _ => Err(MathError::Other("Invalid operands for addition '+'".into())),
    }
}

pub fn op_sub(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| s - x).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| x - s).collect(),
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: d1.iter().zip(d2.iter()).map(|(&x, &y)| x - y).collect(),
            })
        }
        _ => Err(MathError::Other(
            "Invalid operands for subtraction '-'".into(),
        )),
    }
}

pub fn op_dot_mul(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| s * x).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| x * s).collect(),
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: d1.iter().zip(d2.iter()).map(|(&x, &y)| x * y).collect(),
            })
        }
        _ => Err(MathError::Other(
            "Invalid operands for element-wise multiplication '.*'".into(),
        )),
    }
}

pub fn op_mul(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            // If either is scalar, standard scalar multiplication
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| s * x).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| x * s).collect(),
                });
            }
            // Standard matrix multiplication
            if c1 != r2 {
                return Err(MathError::InnerMatrixDimensionsMismatch {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            let mut new_data = vec![0.0; *r1 * *c2];
            for j in 0..*c2 {
                for i in 0..*r1 {
                    let mut sum = 0.0;
                    for k in 0..*c1 {
                        let a_val = d1[k * *r1 + i];
                        let b_val = d2[j * *r2 + k];
                        sum += a_val * b_val;
                    }
                    new_data[j * *r1 + i] = sum;
                }
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c2,
                data: new_data,
            })
        }
        _ => Err(MathError::Other(
            "Invalid operands for matrix multiplication '*'".into(),
        )),
    }
}

pub fn op_dot_div(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| s / x).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| x / s).collect(),
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: d1.iter().zip(d2.iter()).map(|(&x, &y)| x / y).collect(),
            })
        }
        _ => Err(MathError::Other(
            "Invalid operands for element-wise division './'".into(),
        )),
    }
}

pub fn op_div(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    // In MATLAB: A / s when s is scalar
    match (a, b) {
        (
            MathValue::Matrix { rows, cols, data },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *rows,
                    cols: *cols,
                    data: data.iter().map(|&x| x / s).collect(),
                });
            }
            // If both are scalars
            if *rows == 1 && *cols == 1 && *r2 == 1 && *c2 == 1 {
                return Ok(MathValue::scalar(data[0] / d2[0]));
            }
            Err(MathError::Other("Matrix division A/B is only supported when B is scalar. Use A\\b for solving Ax=b.".into()))
        }
        _ => Err(MathError::Other("Invalid operands for division '/'".into())),
    }
}

pub fn op_dot_pow(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            let compute = |base: f64, exp: f64| -> Result<f64, MathError> {
                if base < 0.0 && exp.fract() != 0.0 {
                    return Err(MathError::ComplexNotSupported);
                }
                Ok(base.powf(exp))
            };

            if *r1 == 1 && *c1 == 1 {
                let base = d1[0];
                let mut res = Vec::with_capacity(d2.len());
                for &exp in d2 {
                    res.push(compute(base, exp)?);
                }
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: res,
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let exp = d2[0];
                let mut res = Vec::with_capacity(d1.len());
                for &base in d1 {
                    res.push(compute(base, exp)?);
                }
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: res,
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            let mut res = Vec::with_capacity(d1.len());
            for (&base, &exp) in d1.iter().zip(d2.iter()) {
                res.push(compute(base, exp)?);
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: res,
            })
        }
        _ => Err(MathError::Other(
            "Invalid operands for element-wise power '.^'".into(),
        )),
    }
}

pub fn op_pow(a: &MathValue, b: &MathValue) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: 1,
                cols: 1,
                data: d1,
            },
            MathValue::Matrix {
                rows: 1,
                cols: 1,
                data: d2,
            },
        ) => {
            let base = d1[0];
            let exp = d2[0];
            if base < 0.0 && exp.fract() != 0.0 {
                return Err(MathError::ComplexNotSupported);
            }
            Ok(MathValue::scalar(base.powf(exp)))
        }
        (
            MathValue::Matrix { rows, cols, .. },
            MathValue::Matrix {
                rows: 1,
                cols: 1,
                data: d2,
            },
        ) => {
            if rows != cols {
                return Err(MathError::MatrixMustBeSquare);
            }
            let exp = d2[0];
            if exp.fract() != 0.0 {
                return Err(MathError::Other(
                    "Matrix power requires integer exponent".into(),
                ));
            }
            let n = exp as i64;
            if n == 0 {
                return Ok(MathValue::eye(*rows, *cols));
            }
            if n < 0 {
                let inv = a.inv()?;
                return op_pow(&inv, &MathValue::scalar((-n) as f64));
            }
            let mut res = a.clone();
            for _ in 1..n {
                res = op_mul(&res, a)?;
            }
            Ok(res)
        }
        _ => Err(MathError::Other(
            "Invalid operands for matrix power '^'".into(),
        )),
    }
}

pub fn op_neg(a: &MathValue) -> Result<MathValue, MathError> {
    match a {
        MathValue::Matrix { rows, cols, data } => Ok(MathValue::Matrix {
            rows: *rows,
            cols: *cols,
            data: data.iter().map(|&x| -x).collect(),
        }),
        _ => Err(MathError::Other("Cannot negate non-numeric value".into())),
    }
}

pub fn op_not(a: &MathValue) -> Result<MathValue, MathError> {
    match a {
        MathValue::Matrix { rows, cols, data } => {
            let res: Vec<f64> = data
                .iter()
                .map(|&x| if x == 0.0 { 1.0 } else { 0.0 })
                .collect();
            Ok(MathValue::Matrix {
                rows: *rows,
                cols: *cols,
                data: res,
            })
        }
        _ => Err(MathError::Other(
            "Cannot apply '~' to non-numeric value".into(),
        )),
    }
}

pub fn op_relational(
    a: &MathValue,
    b: &MathValue,
    cmp: fn(f64, f64) -> f64,
) -> Result<MathValue, MathError> {
    match (a, b) {
        (
            MathValue::Matrix {
                rows: r1,
                cols: c1,
                data: d1,
            },
            MathValue::Matrix {
                rows: r2,
                cols: c2,
                data: d2,
            },
        ) => {
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| cmp(s, x)).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| cmp(x, s)).collect(),
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes {
                    r1: *r1,
                    c1: *c1,
                    r2: *r2,
                    c2: *c2,
                });
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: d1.iter().zip(d2.iter()).map(|(&x, &y)| cmp(x, y)).collect(),
            })
        }
        _ => Err(MathError::Other(
            "Comparison operands must be numeric".into(),
        )),
    }
}
