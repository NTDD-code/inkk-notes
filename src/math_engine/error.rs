//! Mathematical Errors for Inkk MathEngine
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum MathError {
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },
    UnrecognizedIdent(String),
    FeatureNotSupported {
        feature: String,
        line: usize,
    },
    InnerMatrixDimensionsMismatch {
        r1: usize,
        c1: usize,
        r2: usize,
        c2: usize,
    },
    ArraysIncompatibleSizes {
        r1: usize,
        c1: usize,
        r2: usize,
        c2: usize,
    },
    IndexOutOfBounds {
        requested: usize,
        max: usize,
    },
    IndexMustBePositiveInteger,
    MatrixMustBeSquare,
    MatrixIsSingular,
    ComplexNotSupported,
    NotEnoughArguments(String),
    TooManyArguments(String),
    VectorsMustBeSameLength,
    InvalidLineSpec(String),
    ArgumentsMustBeIntegers,
    InputMustBeNonnegativeInteger,
    KMustNotExceedN,
    CrossRequires3ElementVectors,
    PieCannotBeCombined,
    OutputArgumentNotAssigned {
        var: String,
        func: String,
    },
    CannotDefineBuiltin(String),
    FunctionConflict {
        func: String,
        lib_a: String,
        lib_b: String,
    },
    ExecutionLimitExceeded,
    RecursionDepthExceeded,
    DivisionByZero,
    Other(String),
}

impl MathError {
    pub fn raw_message(&self) -> String {
        match self {
            MathError::ParseError { line, col, message } => {
                format!("Parse error: Unexpected token '{}' at line {}, col {}.", message, line, col)
            }
            MathError::UnrecognizedIdent(name) => {
                format!("Unrecognized function or variable '{}'.", name)
            }
            MathError::FeatureNotSupported { feature, .. } => {
                format!("Feature not supported in Inkk MathStudio: {}", feature)
            }
            MathError::InnerMatrixDimensionsMismatch { r1, c1, r2, c2 } => {
                format!("Inner matrix dimensions must agree ({}x{} * {}x{}).", r1, c1, r2, c2)
            }
            MathError::ArraysIncompatibleSizes { r1, c1, r2, c2 } => {
                format!("Arrays have incompatible sizes for this operation ({}x{} vs {}x{}).", r1, c1, r2, c2)
            }
            MathError::IndexOutOfBounds { requested, max } => {
                format!("Index exceeds array bounds (requested {}, max {}).", requested, max)
            }
            MathError::IndexMustBePositiveInteger => {
                "Index must be a positive integer.".to_string()
            }
            MathError::MatrixMustBeSquare => {
                "Matrix must be square.".to_string()
            }
            MathError::MatrixIsSingular => {
                "Matrix is singular to working precision.".to_string()
            }
            MathError::ComplexNotSupported => {
                "Complex results are not supported.".to_string()
            }
            MathError::NotEnoughArguments(f) => {
                format!("Not enough input arguments for '{}'.", f)
            }
            MathError::TooManyArguments(f) => {
                format!("Too many input arguments for '{}'.", f)
            }
            MathError::VectorsMustBeSameLength => {
                "Vectors must be the same length.".to_string()
            }
            MathError::InvalidLineSpec(s) => {
                format!("Invalid LineSpec '{}'.", s)
            }
            MathError::ArgumentsMustBeIntegers => {
                "Arguments must be integers.".to_string()
            }
            MathError::InputMustBeNonnegativeInteger => {
                "Input must be a nonnegative integer.".to_string()
            }
            MathError::KMustNotExceedN => {
                "k must not exceed n.".to_string()
            }
            MathError::CrossRequires3ElementVectors => {
                "cross requires 3-element vectors.".to_string()
            }
            MathError::PieCannotBeCombined => {
                "Pie charts cannot be combined with other plots.".to_string()
            }
            MathError::OutputArgumentNotAssigned { var, func } => {
                format!("Output argument '{}' was not assigned during the call to '{}'.", var, func)
            }
            MathError::CannotDefineBuiltin(name) => {
                format!("Cannot define '{}': name is a built-in function.", name)
            }
            MathError::FunctionConflict { func, lib_a, lib_b } => {
                format!("Function '{}' is defined in both '{}' and '{}'.", func, lib_a, lib_b)
            }
            MathError::ExecutionLimitExceeded => {
                "Execution limit exceeded.".to_string()
            }
            MathError::RecursionDepthExceeded => {
                "Maximum recursion depth exceeded.".to_string()
            }
            MathError::DivisionByZero => {
                "Division by zero.".to_string()
            }
            MathError::Other(msg) => msg.clone(),
        }
    }

    pub fn format_with_line(&self, line: usize) -> String {
        match self {
            MathError::ParseError { line: l, col, message } => {
                format!("Parse error: Unexpected token '{}' at line {}, col {}.", message, l, col)
            }
            MathError::FeatureNotSupported { feature, line: l } => {
                let err_line = if *l > 0 { *l } else { line };
                if err_line > 0 {
                    format!("Error (line {}): Feature not supported in Inkk MathStudio: {}", err_line, feature)
                } else {
                    format!("Error: Feature not supported in Inkk MathStudio: {}", feature)
                }
            }
            _ => {
                if line > 0 {
                    format!("Error (line {}): {}", line, self.raw_message())
                } else {
                    format!("Error: {}", self.raw_message())
                }
            }
        }
    }
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_message())
    }
}

impl std::error::Error for MathError {}
