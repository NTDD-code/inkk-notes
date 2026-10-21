//! Function Registry for Inkk MathEngine: Built-in and User-defined Functions
use std::collections::HashMap;
use crate::math_engine::error::MathError;
use crate::math_engine::value::{format_scalar_display, MathValue};

pub use crate::math_engine::ast::{FunctionMetadata, UserFunctionAst};

#[derive(Clone)]
pub enum FunctionKind {
    Builtin(fn(&[MathValue], &mut EvalContext) -> Result<MathValue, MathError>),
    User(UserFunctionAst, FunctionMetadata),
}

#[derive(Clone)]
pub struct RegistryEntry {
    pub kind: FunctionKind,
    pub metadata: FunctionMetadata,
}

#[derive(Clone)]
pub struct FunctionRegistry {
    entries: HashMap<String, RegistryEntry>,
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionRegistry({} functions)", self.entries.len())
    }
}

pub struct EvalContext<'a> {
    pub step_counter: &'a mut usize,
    pub limits: &'a crate::math_engine::eval::ExecutionLimits,
    pub hold_on: &'a mut bool,
    pub grid_on: &'a mut bool,
    pub output_lines: &'a mut Vec<String>,
    pub canvas_commands: &'a mut Vec<crate::math_engine::eval::CanvasCommand>,
    pub current_figure_curves: &'a mut Vec<crate::math_engine::eval::CurveInfo>,
    pub rng_seed: &'a mut u64,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            entries: HashMap::new(),
        };
        reg.register_builtins();
        reg
    }

    pub fn get(&self, name: &str) -> Option<&RegistryEntry> {
        self.entries.get(name)
    }

    pub fn describe_function(&self, name: &str) -> Option<&FunctionMetadata> {
        self.entries.get(name).map(|entry| &entry.metadata)
    }

    pub fn register_builtin(
        &mut self,
        name: &str,
        min_args: usize,
        max_args: usize,
        num_outputs: usize,
        help: &str,
        examples: &[&str],
        func: fn(&[MathValue], &mut EvalContext) -> Result<MathValue, MathError>,
    ) {
        let metadata = FunctionMetadata {
            name: name.to_string(),
            min_args,
            max_args,
            num_outputs,
            help: help.to_string(),
            examples: examples.iter().map(|s| s.to_string()).collect(),
        };
        self.entries.insert(
            name.to_string(),
            RegistryEntry {
                kind: FunctionKind::Builtin(func),
                metadata,
            },
        );
    }

    pub fn register_user(&mut self, ast: UserFunctionAst, meta: FunctionMetadata) -> Result<(), MathError> {
        let name = &meta.name;
        // Conflict check with builtin
        if let Some(entry) = self.entries.get(name) {
            if matches!(entry.kind, FunctionKind::Builtin(_)) {
                return Err(MathError::CannotDefineBuiltin(name.clone()));
            }
            if let FunctionKind::User(ref old_ast, _) = entry.kind {
                if let (Some(lib_a), Some(lib_b)) = (&old_ast.library, &ast.library) {
                    if lib_a != lib_b {
                        return Err(MathError::FunctionConflict {
                            func: name.clone(),
                            lib_a: lib_a.clone(),
                            lib_b: lib_b.clone(),
                        });
                    }
                }
            }
        }
        self.entries.insert(
            name.clone(),
            RegistryEntry {
                kind: FunctionKind::User(ast, meta.clone()),
                metadata: meta,
            },
        );
        Ok(())
    }

    fn register_builtins(&mut self) {
        // --- Creation ---
        self.register_builtin(
            "zeros",
            1,
            2,
            1,
            "Create an array of all zeros.",
            &["zeros(3)", "zeros(2, 4)"],
            builtin_zeros,
        );
        self.register_builtin(
            "ones",
            1,
            2,
            1,
            "Create an array of all ones.",
            &["ones(3)", "ones(2, 4)"],
            builtin_ones,
        );
        self.register_builtin(
            "eye",
            1,
            2,
            1,
            "Identity matrix or diagonal matrix.",
            &["eye(3)", "eye(2, 4)"],
            builtin_eye,
        );
        self.register_builtin(
            "linspace",
            2,
            3,
            1,
            "Generate linearly spaced vector between a and b.",
            &["linspace(0, 10, 5)", "linspace(0, pi)"],
            builtin_linspace,
        );
        self.register_builtin(
            "diag",
            1,
            1,
            1,
            "Diagonal matrices and diagonals of matrix.",
            &["diag([1 2 3])", "diag(A)"],
            builtin_diag,
        );
        self.register_builtin(
            "rand",
            0,
            2,
            1,
            "Uniformly distributed random numbers in [0, 1).",
            &["rand()", "rand(3)", "rand(2, 4)"],
            builtin_rand,
        );

        // --- Size & Shape ---
        self.register_builtin(
            "size",
            1,
            2,
            1,
            "Array dimensions [rows cols].",
            &["size(A)", "size(A, 1)"],
            builtin_size,
        );
        self.register_builtin(
            "length",
            1,
            1,
            1,
            "Length of largest array dimension.",
            &["length(v)"],
            builtin_length,
        );
        self.register_builtin(
            "numel",
            1,
            1,
            1,
            "Number of array elements.",
            &["numel(A)"],
            builtin_numel,
        );
        self.register_builtin(
            "isempty",
            1,
            1,
            1,
            "True for empty array.",
            &["isempty([])", "isempty(A)"],
            builtin_isempty,
        );

        // --- Trigonometric & Math ---
        self.register_builtin("sin", 1, 1, 1, "Sine of argument in radians.", &["sin(pi/2)"], builtin_unary_math);
        self.register_builtin("cos", 1, 1, 1, "Cosine of argument in radians.", &["cos(pi)"], builtin_unary_math);
        self.register_builtin("tan", 1, 1, 1, "Tangent of argument in radians.", &["tan(0)"], builtin_unary_math);
        self.register_builtin("asin", 1, 1, 1, "Inverse sine in radians.", &["asin(1)"], builtin_unary_math);
        self.register_builtin("acos", 1, 1, 1, "Inverse cosine in radians.", &["acos(0)"], builtin_unary_math);
        self.register_builtin("atan", 1, 1, 1, "Inverse tangent in radians.", &["atan(1)"], builtin_unary_math);
        self.register_builtin("atan2", 2, 2, 1, "Four-quadrant inverse tangent atan2(y, x).", &["atan2(1, 1)"], builtin_atan2);
        self.register_builtin("sinh", 1, 1, 1, "Hyperbolic sine.", &["sinh(1)"], builtin_unary_math);
        self.register_builtin("cosh", 1, 1, 1, "Hyperbolic cosine.", &["cosh(0)"], builtin_unary_math);
        self.register_builtin("tanh", 1, 1, 1, "Hyperbolic tangent.", &["tanh(1)"], builtin_unary_math);
        self.register_builtin("exp", 1, 1, 1, "Exponential function e^x.", &["exp(1)"], builtin_unary_math);
        self.register_builtin("log", 1, 1, 1, "Natural logarithm ln(x).", &["log(e)"], builtin_unary_math);
        self.register_builtin("log10", 1, 1, 1, "Common (base 10) logarithm.", &["log10(100)"], builtin_unary_math);
        self.register_builtin("log2", 1, 1, 1, "Base 2 logarithm.", &["log2(8)"], builtin_unary_math);
        self.register_builtin("sqrt", 1, 1, 1, "Square root.", &["sqrt(16)"], builtin_unary_math);
        self.register_builtin("abs", 1, 1, 1, "Absolute value.", &["abs(-5)"], builtin_unary_math);
        self.register_builtin("floor", 1, 1, 1, "Round toward negative infinity.", &["floor(3.7)"], builtin_unary_math);
        self.register_builtin("ceil", 1, 1, 1, "Round toward positive infinity.", &["ceil(3.2)"], builtin_unary_math);
        self.register_builtin("round", 1, 1, 1, "Round to nearest integer.", &["round(3.5)"], builtin_unary_math);
        self.register_builtin("fix", 1, 1, 1, "Round toward zero.", &["fix(-3.7)"], builtin_unary_math);
        self.register_builtin("sign", 1, 1, 1, "Signum function (-1, 0, 1).", &["sign(-5)"], builtin_unary_math);
        self.register_builtin("mod", 2, 2, 1, "Modulus after division.", &["mod(10, 3)"], builtin_mod);
        self.register_builtin("rem", 2, 2, 1, "Remainder after division.", &["rem(10, 3)"], builtin_rem);
        self.register_builtin("max", 1, 2, 1, "Maximum elements.", &["max([1 5 3])", "max(A, B)"], builtin_max);
        self.register_builtin("min", 1, 2, 1, "Minimum elements.", &["min([1 5 3])", "min(A, B)"], builtin_min);

        // --- Statistics ---
        self.register_builtin("sum", 1, 1, 1, "Sum of array elements.", &["sum([1 2 3])"], builtin_sum);
        self.register_builtin("prod", 1, 1, 1, "Product of array elements.", &["prod([1 2 3 4])"], builtin_prod);
        self.register_builtin("mean", 1, 1, 1, "Average or mean value of arrays.", &["mean([1 2 3 4])"], builtin_mean);
        self.register_builtin("cumsum", 1, 1, 1, "Cumulative sum.", &["cumsum([1 2 3 4])"], builtin_cumsum);
        self.register_builtin("sort", 1, 1, 1, "Sort array elements ascending.", &["sort([3 1 4 2])"], builtin_sort);
        self.register_builtin("norm", 1, 1, 1, "Vector 2-norm.", &["norm([3 4])"], builtin_norm);

        // --- Linear Algebra ---
        self.register_builtin("det", 1, 1, 1, "Matrix determinant.", &["det([1 2; 3 4])"], builtin_det);
        self.register_builtin("inv", 1, 1, 1, "Matrix inverse.", &["inv([1 2; 3 4])"], builtin_inv);
        self.register_builtin("trace", 1, 1, 1, "Sum of diagonal elements.", &["trace(A)"], builtin_trace);
        self.register_builtin("rank", 1, 1, 1, "Matrix rank.", &["rank(A)"], builtin_rank);
        self.register_builtin("transpose", 1, 1, 1, "Matrix transpose.", &["transpose(A)"], builtin_transpose);

        // --- Printing ---
        self.register_builtin("disp", 1, 1, 0, "Display value or text.", &["disp(\"Hello\")", "disp(A)"], builtin_disp);

        // --- Plotting ---
        self.register_builtin("plot", 1, 20, 0, "2D line plot.", &["plot(y)", "plot(x, y)", "plot(x, y, \"r--\")", "plot(x1, y1, x2, y2)"], builtin_plot);
        self.register_builtin("title", 1, 1, 0, "Set plot title.", &["title(\"Sine Wave\")"], builtin_title);
        self.register_builtin("xlabel", 1, 1, 0, "Set x-axis label.", &["xlabel(\"Time\")"], builtin_xlabel);
        self.register_builtin("ylabel", 1, 1, 0, "Set y-axis label.", &["ylabel(\"Amplitude\")"], builtin_ylabel);
        self.register_builtin("legend", 1, 10, 0, "Add legend to axes.", &["legend(\"sin\", \"cos\")"], builtin_legend);
        self.register_builtin("xlim", 1, 1, 0, "Set x-axis limits.", &["xlim([-5 5])"], builtin_xlim);
        self.register_builtin("ylim", 1, 1, 0, "Set y-axis limits.", &["ylim([-2 2])"], builtin_ylim);
        self.register_builtin("clf", 0, 0, 0, "Clear current figure.", &["clf"], builtin_clf);

        // --- Tier 1B: Arithmetic / THCS ---
        self.register_builtin("gcd", 2, 2, 1, "Greatest common divisor.", &["gcd(12, 18)"], builtin_gcd);
        self.register_builtin("lcm", 2, 2, 1, "Least common multiple.", &["lcm(4, 6)"], builtin_lcm);
        self.register_builtin("factorial", 1, 1, 1, "Factorial function n!.", &["factorial(5)"], builtin_factorial);
        self.register_builtin("nchoosek", 2, 2, 1, "Binomial coefficient (combinations).", &["nchoosek(5, 2)"], builtin_nchoosek);
        self.register_builtin("isprime", 1, 1, 1, "True for prime numbers.", &["isprime(7)"], builtin_isprime);
        self.register_builtin("primes", 1, 1, 1, "Prime numbers less than or equal to n.", &["primes(20)"], builtin_primes);
        self.register_builtin("factor", 1, 1, 1, "Prime factors of integer.", &["factor(60)"], builtin_factor);
        self.register_builtin("nthroot", 2, 2, 1, "Real nth root of real numbers.", &["nthroot(-8, 3)"], builtin_nthroot);

        // --- Tier 1B: Degrees Trigonometry ---
        self.register_builtin("sind", 1, 1, 1, "Sine of argument in degrees.", &["sind(30)", "sind(180)"], builtin_unary_math);
        self.register_builtin("cosd", 1, 1, 1, "Cosine of argument in degrees.", &["cosd(90)"], builtin_unary_math);
        self.register_builtin("tand", 1, 1, 1, "Tangent of argument in degrees.", &["tand(45)"], builtin_unary_math);
        self.register_builtin("deg2rad", 1, 1, 1, "Convert degrees to radians.", &["deg2rad(180)"], builtin_unary_math);
        self.register_builtin("rad2deg", 1, 1, 1, "Convert radians to degrees.", &["rad2deg(pi)"], builtin_unary_math);

        // --- Tier 1B: Statistics ---
        self.register_builtin("std", 1, 1, 1, "Sample standard deviation (divided by n-1).", &["std([1 2 3 4])"], builtin_std);
        self.register_builtin("var", 1, 1, 1, "Sample variance (divided by n-1).", &["var([1 2 3 4])"], builtin_var);
        self.register_builtin("median", 1, 1, 1, "Median value of array.", &["median([3 1 2])"], builtin_median);
        self.register_builtin("cumprod", 1, 1, 1, "Cumulative product.", &["cumprod([1 2 3 4])"], builtin_cumprod);
        self.register_builtin("diff", 1, 1, 1, "Differences and approximate derivatives.", &["diff([1 4 9 16])"], builtin_diff);

        // --- Tier 1B: Polynomials ---
        self.register_builtin("polyfit", 3, 3, 1, "Polynomial curve fitting.", &["polyfit([1 2 3], [2 4 6], 1)"], builtin_polyfit);
        self.register_builtin("polyval", 2, 2, 1, "Polynomial evaluation.", &["polyval([1 0 -1], 3)"], builtin_polyval);
        self.register_builtin("roots", 1, 1, 1, "Real roots of polynomial (sorted descending).", &["roots([1 -3 2])"], builtin_roots);

        // --- Tier 1B: Numerical Calculus ---
        self.register_builtin("trapz", 1, 2, 1, "Trapezoidal numerical integration.", &["trapz([0 1 2], [0 1 4])"], builtin_trapz);

        // --- Tier 1B: 3D Vectors ---
        self.register_builtin("dot", 2, 2, 1, "Vector dot product.", &["dot([1 2 3], [4 5 6])"], builtin_dot);
        self.register_builtin("cross", 2, 2, 1, "Vector cross product for 3-element vectors.", &["cross([1 0 0], [0 1 0])"], builtin_cross);

        // --- Tier 1B: Matrix ---
        self.register_builtin("reshape", 3, 3, 1, "Reshape array (column-major).", &["reshape(1:6, 2, 3)"], builtin_reshape);
        self.register_builtin("fliplr", 1, 1, 1, "Flip matrix left to right.", &["fliplr(A)"], builtin_fliplr);
        self.register_builtin("flipud", 1, 1, 1, "Flip matrix up to down.", &["flipud(A)"], builtin_flipud);

        // --- Tier 1B: Random ---
        self.register_builtin("rng", 1, 1, 0, "Control random number generation seed.", &["rng(1)"], builtin_rng);
        self.register_builtin("randi", 1, 3, 1, "Uniformly distributed pseudorandom integers.", &["randi(10)", "randi(10, 2, 3)"], builtin_randi);

        // --- Tier 1B: Strings ---
        self.register_builtin("num2str", 1, 1, 1, "Convert numbers to string.", &["num2str(pi)", "num2str(5)"], builtin_num2str);

        // --- Tier 1B: Extended Plots ---
        self.register_builtin("bar", 1, 2, 0, "Bar graph.", &["bar([1 2 3])"], builtin_bar);
        self.register_builtin("hist", 1, 2, 0, "Histogram.", &["hist(v)", "hist(v, 20)"], builtin_hist);
        self.register_builtin("pie", 1, 1, 0, "Pie chart (cannot be combined with other plots).", &["pie([1 2 3])"], builtin_pie);
        self.register_builtin("scatter", 2, 2, 0, "Scatter plot.", &["scatter(x, y)"], builtin_scatter);
        self.register_builtin("xline", 1, 1, 0, "Vertical line with constant x-value.", &["xline(2)"], builtin_xline);
        self.register_builtin("yline", 1, 1, 0, "Horizontal line with constant y-value.", &["yline(0)"], builtin_yline);
        self.register_builtin("axis", 1, 1, 0, "Set axis limits or scaling.", &["axis equal", "axis([0 10 0 20])"], builtin_axis);
        self.register_builtin("text", 3, 3, 0, "Add text descriptions to data points.", &["text(x, y, \"Label\")"], builtin_text);

        // --- Tier 2 ---
        self.register_builtin("eig", 1, 1, 1, "Matrix eigenvalues.", &["eig(A)"], builtin_eig);
        self.register_builtin("fplot", 2, 2, 0, "Plot function.", &["fplot(@(x) sin(x), [-5 5])"], builtin_fplot);
        self.register_builtin("fprintf", 1, 20, 0, "Write formatted data to text output.", &["fprintf(\"val = %d\\n\", 42)"], builtin_fprintf);
        self.register_builtin("error", 1, 1, 0, "Display error message and abort execution.", &["error('Custom error')"], builtin_error);
    }
}

// --- Built-in Implementation Functions ---

fn builtin_zeros(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let r = args[0].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
    let c = if args.len() >= 2 {
        args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize
    } else {
        r
    };
    Ok(MathValue::zeros(r, c))
}

fn builtin_ones(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let r = args[0].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
    let c = if args.len() >= 2 {
        args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize
    } else {
        r
    };
    Ok(MathValue::ones(r, c))
}

fn builtin_eye(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let r = args[0].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
    let c = if args.len() >= 2 {
        args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize
    } else {
        r
    };
    Ok(MathValue::eye(r, c))
}

fn builtin_linspace(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = args[0].as_scalar().ok_or(MathError::Other("linspace: a must be scalar".into()))?;
    let b = args[1].as_scalar().ok_or(MathError::Other("linspace: b must be scalar".into()))?;
    let n = if args.len() >= 3 {
        args[2].as_scalar().ok_or(MathError::Other("linspace: n must be scalar".into()))? as usize
    } else {
        100
    };

    if n < 1 {
        return Ok(MathValue::empty());
    }
    if n == 1 {
        return Ok(MathValue::scalar(b));
    }
    let step = (b - a) / ((n - 1) as f64);
    let mut data = Vec::with_capacity(n);
    for i in 0..n {
        data.push(a + (i as f64) * step);
    }
    Ok(MathValue::Matrix {
        rows: 1,
        cols: n,
        data,
    })
}

fn builtin_diag(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 1 || cols == 1 {
        // Vector -> Diagonal matrix
        let vec = val.to_vector().unwrap();
        let n = vec.len();
        let mut mat = MathValue::zeros(n, n);
        for i in 0..n {
            mat.assign_2d(i + 1, i + 1, vec[i])?;
        }
        Ok(mat)
    } else {
        // Matrix -> Column vector of diagonal elements
        let n = rows.min(cols);
        let mut data = Vec::with_capacity(n);
        for i in 0..n {
            data.push(val.get(i, i).unwrap());
        }
        Ok(MathValue::Matrix {
            rows: n,
            cols: 1,
            data,
        })
    }
}

// Linear congruential pseudo-random generator
fn builtin_rand(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let next_rand = |seed: &mut u64| -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (*seed >> 11) as f64;
        val / 9007199254740992.0
    };

    let (rows, cols) = if args.is_empty() {
        (1, 1)
    } else if args.len() == 1 {
        let n = args[0].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
        (n, n)
    } else {
        let r = args[0].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
        let c = args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
        (r, c)
    };

    let count = rows * cols;
    let mut data = Vec::with_capacity(count);
    for _ in 0..count {
        data.push(next_rand(ctx.rng_seed));
    }
    Ok(MathValue::Matrix { rows, cols, data })
}

fn builtin_size(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if args.len() == 2 {
        let dim = args[1].as_scalar().ok_or(MathError::Other("size: dim must be scalar".into()))? as usize;
        let res = match dim {
            1 => rows as f64,
            2 => cols as f64,
            _ => 1.0,
        };
        Ok(MathValue::scalar(res))
    } else {
        Ok(MathValue::Matrix {
            rows: 1,
            cols: 2,
            data: vec![rows as f64, cols as f64],
        })
    }
}

fn builtin_length(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let len = val.rows().max(val.cols());
    Ok(MathValue::scalar(len as f64))
}

fn builtin_numel(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    Ok(MathValue::scalar(args[0].numel() as f64))
}

fn builtin_isempty(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let empty = if args[0].is_empty() { 1.0 } else { 0.0 };
    Ok(MathValue::scalar(empty))
}

fn builtin_unary_math(_args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    // Unary math functions are dispatched based on caller or we inspect registry name
    // To handle all cleanly, we take function name from metadata or implement individually
    unreachable!("Handled via builtin_math_op")
}

impl FunctionRegistry {
    pub fn call(
        &self,
        name: &str,
        args: &[MathValue],
        ctx: &mut EvalContext,
    ) -> Result<MathValue, MathError> {
        let entry = self.entries.get(name).ok_or_else(|| MathError::UnrecognizedIdent(name.to_string()))?;

        if args.len() < entry.metadata.min_args {
            return Err(MathError::NotEnoughArguments(name.to_string()));
        }
        if args.len() > entry.metadata.max_args {
            return Err(MathError::TooManyArguments(name.to_string()));
        }

        match &entry.kind {
            FunctionKind::Builtin(f) => {
                // If it's unary math:
                if matches!(
                    name,
                    "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh" | "tanh"
                        | "exp" | "log" | "log10" | "log2" | "sqrt" | "abs" | "floor" | "ceil"
                        | "round" | "fix" | "sign" | "sind" | "cosd" | "tand" | "deg2rad" | "rad2deg"
                ) {
                    return apply_unary_math(name, &args[0]);
                }
                f(args, ctx)
            }
            FunctionKind::User(..) => {
                // Tier 3
                Err(MathError::Other("User functions executed via call frame".into()))
            }
        }
    }
}

fn apply_unary_math(name: &str, val: &MathValue) -> Result<MathValue, MathError> {
    let f = |x: f64| -> Result<f64, MathError> {
        match name {
            "sin" => Ok(x.sin()),
            "cos" => Ok(x.cos()),
            "tan" => Ok(x.tan()),
            "asin" => {
                if x < -1.0 || x > 1.0 {
                    Err(MathError::ComplexNotSupported)
                } else {
                    Ok(x.asin())
                }
            }
            "acos" => {
                if x < -1.0 || x > 1.0 {
                    Err(MathError::ComplexNotSupported)
                } else {
                    Ok(x.acos())
                }
            }
            "atan" => Ok(x.atan()),
            "sinh" => Ok(x.sinh()),
            "cosh" => Ok(x.cosh()),
            "tanh" => Ok(x.tanh()),
            "exp" => Ok(x.exp()),
            "log" => {
                if x < 0.0 {
                    Err(MathError::ComplexNotSupported)
                } else if x == 0.0 {
                    Ok(f64::NEG_INFINITY)
                } else {
                    Ok(x.ln())
                }
            }
            "log10" => {
                if x < 0.0 {
                    Err(MathError::ComplexNotSupported)
                } else if x == 0.0 {
                    Ok(f64::NEG_INFINITY)
                } else {
                    Ok(x.log10())
                }
            }
            "log2" => {
                if x < 0.0 {
                    Err(MathError::ComplexNotSupported)
                } else if x == 0.0 {
                    Ok(f64::NEG_INFINITY)
                } else {
                    Ok(x.log2())
                }
            }
            "sqrt" => {
                if x < 0.0 {
                    Err(MathError::ComplexNotSupported)
                } else {
                    Ok(x.sqrt())
                }
            }
            "abs" => Ok(x.abs()),
            "floor" => Ok(x.floor()),
            "ceil" => Ok(x.ceil()),
            "round" => Ok(x.round()),
            "fix" => Ok(x.trunc()),
            "sign" => {
                if x > 0.0 {
                    Ok(1.0)
                } else if x < 0.0 {
                    Ok(-1.0)
                } else {
                    Ok(0.0)
                }
            }
            "sind" => Ok(sind_f64(x)),
            "cosd" => Ok(cosd_f64(x)),
            "tand" => Ok(tand_f64(x)),
            "deg2rad" => Ok(x * (std::f64::consts::PI / 180.0)),
            "rad2deg" => Ok(x * (180.0 / std::f64::consts::PI)),
            _ => Ok(x),
        }
    };

    match val {
        MathValue::Matrix { rows, cols, data } => {
            let mut new_data = Vec::with_capacity(data.len());
            for &x in data {
                new_data.push(f(x)?);
            }
            Ok(MathValue::Matrix {
                rows: *rows,
                cols: *cols,
                data: new_data,
            })
        }
        _ => Err(MathError::Other("Cannot apply math function to string".into())),
    }
}

fn builtin_atan2(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let y = &args[0];
    let x = &args[1];
    match (y, x) {
        (MathValue::Matrix { rows: r1, cols: c1, data: d1 }, MathValue::Matrix { rows: r2, cols: c2, data: d2 }) => {
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes { r1: *r1, c1: *c1, r2: *r2, c2: *c2 });
            }
            let mut new_data = Vec::with_capacity(d1.len());
            for (&y_val, &x_val) in d1.iter().zip(d2.iter()) {
                new_data.push(y_val.atan2(x_val));
            }
            Ok(MathValue::Matrix { rows: *r1, cols: *c1, data: new_data })
        }
        _ => Err(MathError::Other("atan2 arguments must be matrices".into())),
    }
}

fn builtin_mod(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = &args[0];
    let b = &args[1];
    let calc = |x: f64, y: f64| -> f64 {
        if y == 0.0 {
            x
        } else {
            x - y * (x / y).floor()
        }
    };
    apply_binary_elementwise(a, b, calc)
}

fn builtin_rem(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = &args[0];
    let b = &args[1];
    let calc = |x: f64, y: f64| -> f64 {
        if y == 0.0 {
            f64::NAN
        } else {
            x - y * (x / y).trunc()
        }
    };
    apply_binary_elementwise(a, b, calc)
}

fn builtin_max(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if args.len() == 2 {
        let calc = |x: f64, y: f64| -> f64 { x.max(y) };
        return apply_binary_elementwise(&args[0], &args[1], calc);
    }
    // 1 argument: max along columns for matrix, or max of vector
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::empty());
    }
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let max_val = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        return Ok(MathValue::scalar(max_val));
    }
    let mut data = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut col_max = f64::NEG_INFINITY;
        for r in 0..rows {
            col_max = col_max.max(val.get(r, c).unwrap());
        }
        data.push(col_max);
    }
    Ok(MathValue::Matrix { rows: 1, cols, data })
}

fn builtin_min(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if args.len() == 2 {
        let calc = |x: f64, y: f64| -> f64 { x.min(y) };
        return apply_binary_elementwise(&args[0], &args[1], calc);
    }
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::empty());
    }
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let min_val = v.iter().copied().fold(f64::INFINITY, f64::min);
        return Ok(MathValue::scalar(min_val));
    }
    let mut data = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut col_min = f64::INFINITY;
        for r in 0..rows {
            col_min = col_min.min(val.get(r, c).unwrap());
        }
        data.push(col_min);
    }
    Ok(MathValue::Matrix { rows: 1, cols, data })
}

fn builtin_sum(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::scalar(0.0));
    }
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let s: f64 = v.iter().sum();
        return Ok(MathValue::scalar(s));
    }
    let mut data = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut col_sum = 0.0;
        for r in 0..rows {
            col_sum += val.get(r, c).unwrap();
        }
        data.push(col_sum);
    }
    Ok(MathValue::Matrix { rows: 1, cols, data })
}

fn builtin_prod(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::scalar(1.0));
    }
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let p: f64 = v.iter().product();
        return Ok(MathValue::scalar(p));
    }
    let mut data = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut col_prod = 1.0;
        for r in 0..rows {
            col_prod *= val.get(r, c).unwrap();
        }
        data.push(col_prod);
    }
    Ok(MathValue::Matrix { rows: 1, cols, data })
}

fn builtin_mean(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::scalar(f64::NAN));
    }
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let s: f64 = v.iter().sum();
        return Ok(MathValue::scalar(s / v.len() as f64));
    }
    let mut data = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut col_sum = 0.0;
        for r in 0..rows {
            col_sum += val.get(r, c).unwrap();
        }
        data.push(col_sum / rows as f64);
    }
    Ok(MathValue::Matrix { rows: 1, cols, data })
}

fn builtin_cumsum(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::empty());
    }
    if rows == 1 {
        // Row vector
        let v = val.to_vector().unwrap();
        let mut cur = 0.0;
        let data: Vec<f64> = v.into_iter().map(|x| { cur += x; cur }).collect();
        return Ok(MathValue::Matrix { rows: 1, cols, data });
    }
    // Column vector or matrix
    let mut data = vec![0.0; rows * cols];
    for c in 0..cols {
        let mut cur = 0.0;
        for r in 0..rows {
            cur += val.get(r, c).unwrap();
            data[c * rows + r] = cur;
        }
    }
    Ok(MathValue::Matrix { rows, cols, data })
}

fn builtin_sort(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::empty());
    }
    if rows == 1 {
        let mut v = val.to_vector().unwrap();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        return Ok(MathValue::Matrix { rows: 1, cols, data: v });
    }
    let mut data = vec![0.0; rows * cols];
    for c in 0..cols {
        let mut col_vals: Vec<f64> = (0..rows).map(|r| val.get(r, c).unwrap()).collect();
        col_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        for r in 0..rows {
            data[c * rows + r] = col_vals[r];
        }
    }
    Ok(MathValue::Matrix { rows, cols, data })
}

fn builtin_norm(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let v = val.to_vector().ok_or(MathError::Other("norm: only vector norm is supported".into()))?;
    let sum_sq: f64 = v.iter().map(|&x| x * x).sum();
    Ok(MathValue::scalar(sum_sq.sqrt()))
}

fn builtin_det(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let d = args[0].det()?;
    Ok(MathValue::scalar(d))
}

fn builtin_inv(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    args[0].inv()
}

fn builtin_trace(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let t = args[0].trace()?;
    Ok(MathValue::scalar(t))
}

fn builtin_rank(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let r = args[0].rank();
    Ok(MathValue::scalar(r as f64))
}

fn builtin_transpose(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    args[0].transpose()
}

fn builtin_disp(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let text = match &args[0] {
        MathValue::Str(s) => s.clone(),
        MathValue::Matrix { rows: 1, cols: 1, data } => format_scalar_display(data[0]),
        _ => args[0].to_display_string(),
    };
    ctx.output_lines.push(text);
    Ok(MathValue::empty())
}

// --- Plotting Implementation ---

fn builtin_plot(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let default_palette: [[u8; 4]; 7] = [
        [0, 114, 189, 255],  // #0072BD
        [217, 83, 25, 255],   // #D95319
        [237, 177, 32, 255],  // #EDB120
        [126, 47, 142, 255],  // #7E2F8E
        [119, 172, 48, 255],  // #77AC30
        [77, 190, 238, 255],  // #4DBEEE
        [162, 20, 47, 255],   // #A2142F
    ];

    if args.len() == 1 {
        let y_vec = args[0].to_vector().ok_or(MathError::Other("plot(y): y must be vector".into()))?;
        let x_vec: Vec<f64> = (1..=y_vec.len()).map(|i| i as f64).collect();
        let curve_idx = ctx.current_figure_curves.len();
        let color = default_palette[curve_idx % default_palette.len()];
        let label = format!("data{}", curve_idx + 1);

        ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
            x: x_vec.clone(),
            y: y_vec.clone(),
            color,
            label: label.clone(),
            line_style: "-".to_string(),
            marker: "".to_string(),
            kind: crate::math_engine::eval::CurveKind::Plot,
        });

        ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::PlotCurve {
            x: x_vec,
            y: y_vec,
            label,
            color,
            line_style: "-".to_string(),
            marker: "".to_string(),
            is_hold: *ctx.hold_on,
        });
        return Ok(MathValue::empty());
    }

    let mut i = 0;
    while i < args.len() {
        let x_vec = args[i].to_vector().ok_or(MathError::Other("plot: x must be a vector".into()))?;
        if i + 1 >= args.len() {
            return Err(MathError::Other("plot: missing y for x vector".into()));
        }
        let y_vec = args[i + 1].to_vector().ok_or(MathError::Other("plot: y must be a vector".into()))?;
        if x_vec.len() != y_vec.len() {
            return Err(MathError::VectorsMustBeSameLength);
        }
        i += 2;

        let curve_idx = ctx.current_figure_curves.len();
        let default_color = default_palette[curve_idx % default_palette.len()];

        let mut color = default_color;
        let mut line_style = "-".to_string();
        let mut marker = "".to_string();
        let mut label = format!("data{}", curve_idx + 1);

        while i < args.len() {
            if let MathValue::Str(spec_or_key) = &args[i] {
                if spec_or_key.eq_ignore_ascii_case("DisplayName") {
                    if i + 1 < args.len() {
                        if let MathValue::Str(lbl) = &args[i + 1] {
                            label = lbl.clone();
                        }
                        i += 2;
                        continue;
                    }
                } else if let Ok(parsed) = parse_linespec(spec_or_key) {
                    if let Some(c) = parsed.color {
                        color = c;
                    }
                    if let Some(ls) = parsed.line_style {
                        line_style = ls;
                    }
                    if let Some(m) = parsed.marker {
                        marker = m;
                    }
                    i += 1;
                    continue;
                } else {
                    return Err(MathError::InvalidLineSpec(spec_or_key.clone()));
                }
            }
            break;
        }

        ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
            x: x_vec.clone(),
            y: y_vec.clone(),
            color,
            label: label.clone(),
            line_style: line_style.clone(),
            marker: marker.clone(),
            kind: crate::math_engine::eval::CurveKind::Plot,
        });

        ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::PlotCurve {
            x: x_vec,
            y: y_vec,
            label,
            color,
            line_style,
            marker,
            is_hold: *ctx.hold_on,
        });
    }

    Ok(MathValue::empty())
}

struct ParsedLineSpec {
    color: Option<[u8; 4]>,
    line_style: Option<String>,
    marker: Option<String>,
}

fn parse_linespec(s: &str) -> Result<ParsedLineSpec, MathError> {
    let mut color = None;
    let mut line_style = None;
    let mut marker = None;

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Two-character line styles: '--' or '-.'
        if i + 1 < chars.len() {
            let pair = format!("{}{}", chars[i], chars[i + 1]);
            if pair == "--" || pair == "-." {
                line_style = Some(pair);
                i += 2;
                continue;
            }
        }

        let c = chars[i];
        match c {
            'r' => color = Some([220, 38, 38, 255]),
            'g' => color = Some([22, 163, 74, 255]),
            'b' => color = Some([37, 99, 235, 255]),
            'k' => color = Some([0, 0, 0, 255]),
            'm' => color = Some([217, 70, 239, 255]),
            'c' => color = Some([6, 182, 212, 255]),
            'y' => color = Some([234, 179, 8, 255]),
            '-' | ':' => line_style = Some(c.to_string()),
            'o' | '+' | '*' | '.' | 'x' => marker = Some(c.to_string()),
            _ => return Err(MathError::InvalidLineSpec(s.to_string())),
        }
        i += 1;
    }

    Ok(ParsedLineSpec {
        color,
        line_style,
        marker,
    })
}

fn builtin_title(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if let MathValue::Str(t) = &args[0] {
        ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetTitle(t.clone()));
    }
    Ok(MathValue::empty())
}

fn builtin_xlabel(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if let MathValue::Str(t) = &args[0] {
        ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetXLabel(t.clone()));
    }
    Ok(MathValue::empty())
}

fn builtin_ylabel(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if let MathValue::Str(t) = &args[0] {
        ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetYLabel(t.clone()));
    }
    Ok(MathValue::empty())
}

fn builtin_legend(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let mut labels = Vec::new();
    for a in args {
        if let MathValue::Str(s) = a {
            labels.push(s.clone());
        }
    }
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetLegend(labels));
    Ok(MathValue::empty())
}

fn builtin_xlim(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let v = args[0].to_vector().ok_or(MathError::Other("xlim: must be [min max]".into()))?;
    if v.len() != 2 {
        return Err(MathError::Other("xlim requires 2 elements [min max]".into()));
    }
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetXLim(v[0], v[1]));
    Ok(MathValue::empty())
}

fn builtin_ylim(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let v = args[0].to_vector().ok_or(MathError::Other("ylim: must be [min max]".into()))?;
    if v.len() != 2 {
        return Err(MathError::Other("ylim requires 2 elements [min max]".into()));
    }
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetYLim(v[0], v[1]));
    Ok(MathValue::empty())
}

fn builtin_clf(_args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    ctx.current_figure_curves.clear();
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::ClearPlots);
    Ok(MathValue::empty())
}

fn apply_binary_elementwise(
    a: &MathValue,
    b: &MathValue,
    op: fn(f64, f64) -> f64,
) -> Result<MathValue, MathError> {
    match (a, b) {
        (MathValue::Matrix { rows: r1, cols: c1, data: d1 }, MathValue::Matrix { rows: r2, cols: c2, data: d2 }) => {
            if *r1 == 1 && *c1 == 1 {
                let s = d1[0];
                return Ok(MathValue::Matrix {
                    rows: *r2,
                    cols: *c2,
                    data: d2.iter().map(|&x| op(s, x)).collect(),
                });
            }
            if *r2 == 1 && *c2 == 1 {
                let s = d2[0];
                return Ok(MathValue::Matrix {
                    rows: *r1,
                    cols: *c1,
                    data: d1.iter().map(|&x| op(x, s)).collect(),
                });
            }
            if r1 != r2 || c1 != c2 {
                return Err(MathError::ArraysIncompatibleSizes { r1: *r1, c1: *c1, r2: *r2, c2: *c2 });
            }
            Ok(MathValue::Matrix {
                rows: *r1,
                cols: *c1,
                data: d1.iter().zip(d2.iter()).map(|(&x, &y)| op(x, y)).collect(),
            })
        }
        _ => Err(MathError::Other("Operands must be numeric matrices".into())),
    }
}

// --- Tier 1B Helpers & Implementations ---

pub fn is_int(x: f64) -> bool {
    x.fract() == 0.0 && x.is_finite()
}

pub fn sind_f64(deg: f64) -> f64 {
    let m = deg.rem_euclid(360.0);
    if m == 0.0 || m == 180.0 || m == 360.0 {
        0.0
    } else if m == 90.0 {
        1.0
    } else if m == 270.0 {
        -1.0
    } else if (m - 30.0).abs() < 1e-12 || (m - 150.0).abs() < 1e-12 {
        0.5
    } else if (m - 210.0).abs() < 1e-12 || (m - 330.0).abs() < 1e-12 {
        -0.5
    } else {
        deg.to_radians().sin()
    }
}

pub fn cosd_f64(deg: f64) -> f64 {
    let m = deg.rem_euclid(360.0);
    if m == 90.0 || m == 270.0 {
        0.0
    } else if m == 0.0 || m == 360.0 {
        1.0
    } else if m == 180.0 {
        -1.0
    } else if (m - 60.0).abs() < 1e-12 || (m - 300.0).abs() < 1e-12 {
        0.5
    } else if (m - 120.0).abs() < 1e-12 || (m - 240.0).abs() < 1e-12 {
        -0.5
    } else {
        deg.to_radians().cos()
    }
}

pub fn tand_f64(deg: f64) -> f64 {
    let m = deg.rem_euclid(180.0);
    if m == 0.0 || m == 180.0 {
        0.0
    } else if m == 90.0 {
        f64::INFINITY
    } else if (m - 45.0).abs() < 1e-12 {
        1.0
    } else if (m - 135.0).abs() < 1e-12 {
        -1.0
    } else {
        deg.to_radians().tan()
    }
}

fn builtin_gcd(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = args[0].as_scalar().ok_or(MathError::ArgumentsMustBeIntegers)?;
    let b = args[1].as_scalar().ok_or(MathError::ArgumentsMustBeIntegers)?;
    if !is_int(a) || !is_int(b) {
        return Err(MathError::ArgumentsMustBeIntegers);
    }
    if a < 0.0 || b < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let mut ua = a as u64;
    let mut ub = b as u64;
    while ub != 0 {
        let t = ub;
        ub = ua % ub;
        ua = t;
    }
    Ok(MathValue::scalar(ua as f64))
}

fn builtin_lcm(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = args[0].as_scalar().ok_or(MathError::ArgumentsMustBeIntegers)?;
    let b = args[1].as_scalar().ok_or(MathError::ArgumentsMustBeIntegers)?;
    if !is_int(a) || !is_int(b) {
        return Err(MathError::ArgumentsMustBeIntegers);
    }
    if a < 0.0 || b < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let ua = a as u64;
    let ub = b as u64;
    if ua == 0 || ub == 0 {
        return Ok(MathValue::scalar(0.0));
    }
    let mut x = ua;
    let mut y = ub;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    let g = x;
    let res = (ua / g) * ub;
    Ok(MathValue::scalar(res as f64))
}

fn builtin_factorial(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let n = args[0].as_scalar().ok_or(MathError::InputMustBeNonnegativeInteger)?;
    if !is_int(n) || n < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let un = n as u64;
    let mut res = 1.0;
    for i in 1..=un {
        res *= i as f64;
    }
    Ok(MathValue::scalar(res))
}

fn builtin_nchoosek(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let n = args[0].as_scalar().ok_or(MathError::ArgumentsMustBeIntegers)?;
    let k = args[1].as_scalar().ok_or(MathError::ArgumentsMustBeIntegers)?;
    if !is_int(n) || !is_int(k) {
        return Err(MathError::ArgumentsMustBeIntegers);
    }
    if n < 0.0 || k < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    if k > n {
        return Err(MathError::KMustNotExceedN);
    }
    let un = n as u64;
    let uk = k as u64;
    let k_eff = uk.min(un - uk);
    let mut c = 1.0;
    for i in 1..=k_eff {
        c = c * (un - k_eff + i) as f64 / (i as f64);
    }
    Ok(MathValue::scalar(c.round()))
}

fn builtin_isprime(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let check = |x: f64| -> Result<f64, MathError> {
        if !is_int(x) || x < 0.0 {
            return Err(MathError::InputMustBeNonnegativeInteger);
        }
        let n = x as u64;
        if n < 2 {
            return Ok(0.0);
        }
        if n == 2 || n == 3 {
            return Ok(1.0);
        }
        if n % 2 == 0 || n % 3 == 0 {
            return Ok(0.0);
        }
        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 {
                return Ok(0.0);
            }
            i += 6;
        }
        Ok(1.0)
    };

    match &args[0] {
        MathValue::Matrix { rows, cols, data } => {
            let mut out = Vec::with_capacity(data.len());
            for &x in data {
                out.push(check(x)?);
            }
            Ok(MathValue::Matrix { rows: *rows, cols: *cols, data: out })
        }
        _ => Err(MathError::Other("isprime requires numeric input".into())),
    }
}

fn builtin_primes(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let n = args[0].as_scalar().ok_or(MathError::InputMustBeNonnegativeInteger)?;
    if !is_int(n) || n < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let un = n as usize;
    if un < 2 {
        return Ok(MathValue::Matrix { rows: 1, cols: 0, data: Vec::new() });
    }
    let mut is_p = vec![true; un + 1];
    is_p[0] = false;
    is_p[1] = false;
    let mut p = 2;
    while p * p <= un {
        if is_p[p] {
            let mut i = p * p;
            while i <= un {
                is_p[i] = false;
                i += p;
            }
        }
        p += 1;
    }
    let mut res = Vec::new();
    for i in 2..=un {
        if is_p[i] {
            res.push(i as f64);
        }
    }
    Ok(MathValue::Matrix {
        rows: 1,
        cols: res.len(),
        data: res,
    })
}

fn builtin_factor(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let n = args[0].as_scalar().ok_or(MathError::InputMustBeNonnegativeInteger)?;
    if !is_int(n) || n < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let mut un = n as u64;
    if un <= 1 {
        return Ok(MathValue::Matrix { rows: 1, cols: 1, data: vec![un as f64] });
    }
    let mut res = Vec::new();
    while un % 2 == 0 {
        res.push(2.0);
        un /= 2;
    }
    let mut d = 3;
    while d * d <= un {
        while un % d == 0 {
            res.push(d as f64);
            un /= d;
        }
        d += 2;
    }
    if un > 1 {
        res.push(un as f64);
    }
    Ok(MathValue::Matrix {
        rows: 1,
        cols: res.len(),
        data: res,
    })
}

fn builtin_nthroot(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let n = args[1].as_scalar().ok_or(MathError::Other("nthroot: n must be scalar".into()))?;
    if !is_int(n) || n == 0.0 {
        return Err(MathError::ArgumentsMustBeIntegers);
    }
    let n_int = n as i64;
    let is_odd = (n_int % 2) != 0;

    let compute = |x: f64| -> Result<f64, MathError> {
        if x < 0.0 {
            if is_odd {
                Ok(-((-x).powf(1.0 / n)))
            } else {
                Err(MathError::ComplexNotSupported)
            }
        } else {
            Ok(x.powf(1.0 / n))
        }
    };

    match &args[0] {
        MathValue::Matrix { rows, cols, data } => {
            let mut out = Vec::with_capacity(data.len());
            for &x in data {
                out.push(compute(x)?);
            }
            Ok(MathValue::Matrix { rows: *rows, cols: *cols, data: out })
        }
        _ => Err(MathError::Other("nthroot: x must be numeric".into())),
    }
}

fn builtin_var(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::scalar(0.0));
    }
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let n = v.len();
        if n <= 1 {
            return Ok(MathValue::scalar(0.0));
        }
        let mean = v.iter().sum::<f64>() / (n as f64);
        let var = v.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / ((n - 1) as f64);
        return Ok(MathValue::scalar(var));
    }
    // Matrix: column-wise -> 1 x cols
    let mut out = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut sum = 0.0;
        for r in 0..rows {
            sum += val.get(r, c).unwrap();
        }
        let mean = sum / (rows as f64);
        let mut sq_sum = 0.0;
        for r in 0..rows {
            sq_sum += (val.get(r, c).unwrap() - mean).powi(2);
        }
        let var = if rows > 1 { sq_sum / ((rows - 1) as f64) } else { 0.0 };
        out.push(var);
    }
    Ok(MathValue::Matrix { rows: 1, cols, data: out })
}

fn builtin_std(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let var_val = builtin_var(args, ctx)?;
    match var_val {
        MathValue::Matrix { rows, cols, data } => {
            Ok(MathValue::Matrix {
                rows,
                cols,
                data: data.into_iter().map(|x| x.sqrt()).collect(),
            })
        }
        _ => unreachable!(),
    }
}

fn builtin_median(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 0 || cols == 0 {
        return Ok(MathValue::scalar(0.0));
    }
    let calc_median = |mut arr: Vec<f64>| -> f64 {
        arr.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = arr.len();
        if n % 2 == 1 {
            arr[n / 2]
        } else {
            (arr[n / 2 - 1] + arr[n / 2]) / 2.0
        }
    };

    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        return Ok(MathValue::scalar(calc_median(v)));
    }

    let mut out = Vec::with_capacity(cols);
    for c in 0..cols {
        let mut col_data = Vec::with_capacity(rows);
        for r in 0..rows {
            col_data.push(val.get(r, c).unwrap());
        }
        out.push(calc_median(col_data));
    }
    Ok(MathValue::Matrix { rows: 1, cols, data: out })
}

fn builtin_cumprod(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 1 || cols == 1 {
        let v = val.to_vector().unwrap();
        let mut out = Vec::with_capacity(v.len());
        let mut prod = 1.0;
        for x in v {
            prod *= x;
            out.push(prod);
        }
        return Ok(MathValue::Matrix { rows, cols, data: out });
    }
    // Column-wise cumulative product
    let mut data = vec![0.0; rows * cols];
    for c in 0..cols {
        let mut prod = 1.0;
        for r in 0..rows {
            prod *= val.get(r, c).unwrap();
            data[c * rows + r] = prod;
        }
    }
    Ok(MathValue::Matrix { rows, cols, data })
}

fn builtin_diff(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let val = &args[0];
    let (rows, cols) = (val.rows(), val.cols());
    if rows == 1 {
        let v = val.to_vector().unwrap();
        if v.len() <= 1 {
            return Ok(MathValue::Matrix { rows: 1, cols: 0, data: Vec::new() });
        }
        let mut out = Vec::with_capacity(v.len() - 1);
        for i in 0..(v.len() - 1) {
            out.push(v[i + 1] - v[i]);
        }
        return Ok(MathValue::Matrix { rows: 1, cols: out.len(), data: out });
    }
    if cols == 1 {
        let v = val.to_vector().unwrap();
        if v.len() <= 1 {
            return Ok(MathValue::Matrix { rows: 0, cols: 1, data: Vec::new() });
        }
        let mut out = Vec::with_capacity(v.len() - 1);
        for i in 0..(v.len() - 1) {
            out.push(v[i + 1] - v[i]);
        }
        return Ok(MathValue::Matrix { rows: out.len(), cols: 1, data: out });
    }
    // Matrix: diff down rows -> (rows - 1) x cols
    if rows <= 1 {
        return Ok(MathValue::Matrix { rows: 0, cols: 0, data: Vec::new() });
    }
    let new_rows = rows - 1;
    let mut data = vec![0.0; new_rows * cols];
    for c in 0..cols {
        for r in 0..new_rows {
            let diff_val = val.get(r + 1, c).unwrap() - val.get(r, c).unwrap();
            data[c * new_rows + r] = diff_val;
        }
    }
    Ok(MathValue::Matrix { rows: new_rows, cols, data })
}

fn builtin_polyfit(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let x = args[0].to_vector().ok_or(MathError::Other("polyfit: x must be a vector".into()))?;
    let y = args[1].to_vector().ok_or(MathError::Other("polyfit: y must be a vector".into()))?;
    let n_f = args[2].as_scalar().ok_or(MathError::Other("polyfit: n must be a scalar".into()))?;
    if !is_int(n_f) || n_f < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let n = n_f as usize;
    if x.len() != y.len() {
        return Err(MathError::VectorsMustBeSameLength);
    }
    let m = x.len();
    let k = n + 1; // number of coefficients
    if m < k {
        return Err(MathError::Other("polyfit: insufficient points for degree".into()));
    }

    // Build Vandermonde matrix V of size m x k: V[i, j] = x[i]^(n - j)
    let mut v_mat = MathValue::zeros(m, k);
    for i in 0..m {
        for j in 0..k {
            let p = (n - j) as i32;
            let val = x[i].powi(p);
            v_mat.assign_2d(i + 1, j + 1, val)?;
        }
    }

    let y_mat = MathValue::Matrix {
        rows: m,
        cols: 1,
        data: y,
    };

    let vt = v_mat.transpose()?;
    let vtv = crate::math_engine::value::op_mul(&vt, &v_mat)?;
    let vty = crate::math_engine::value::op_mul(&vt, &y_mat)?;
    let p_col = vtv.solve_linear(&vty)?;
    let p_vec = p_col.to_vector().unwrap();

    Ok(MathValue::Matrix {
        rows: 1,
        cols: p_vec.len(),
        data: p_vec,
    })
}

fn builtin_polyval(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let p = args[0].to_vector().ok_or(MathError::Other("polyval: p must be vector of coefficients".into()))?;
    let x_val = &args[1];

    let eval_p = |x: f64| -> f64 {
        let mut res = 0.0;
        for &c in &p {
            res = res * x + c;
        }
        res
    };

    match x_val {
        MathValue::Matrix { rows, cols, data } => {
            let mut out = Vec::with_capacity(data.len());
            for &x in data {
                out.push(eval_p(x));
            }
            Ok(MathValue::Matrix { rows: *rows, cols: *cols, data: out })
        }
        _ => Err(MathError::Other("polyval: x must be numeric".into())),
    }
}

fn builtin_roots(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let mut p = args[0].to_vector().ok_or(MathError::Other("roots: input must be vector of coefficients".into()))?;
    while !p.is_empty() && p[0].abs() < 1e-12 {
        p.remove(0);
    }
    if p.len() <= 1 {
        return Ok(MathValue::Matrix { rows: 0, cols: 1, data: Vec::new() });
    }
    let lead = p[0];
    for x in &mut p {
        *x /= lead;
    }

    let degree = p.len() - 1;
    if degree == 1 {
        let r = -p[1];
        return Ok(MathValue::Matrix { rows: 1, cols: 1, data: vec![r] });
    }
    if degree == 2 {
        let b = p[1];
        let c = p[2];
        let delta = b * b - 4.0 * c;
        if delta < -1e-9 {
            return Err(MathError::ComplexNotSupported);
        }
        let delta_clamped = delta.max(0.0);
        let sqrt_d = delta_clamped.sqrt();
        let r1 = (-b + sqrt_d) / 2.0;
        let r2 = (-b - sqrt_d) / 2.0;
        let mut roots = vec![r1, r2];
        roots.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        return Ok(MathValue::Matrix { rows: 2, cols: 1, data: roots });
    }

    // Companion matrix of size degree x degree
    let mut comp = vec![vec![0.0; degree]; degree];
    for j in 0..degree {
        comp[0][j] = -p[j + 1];
    }
    for i in 1..degree {
        comp[i][i - 1] = 1.0;
    }

    let mut roots = compute_real_eigenvalues(&comp)?;
    roots.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Ok(MathValue::Matrix {
        rows: roots.len(),
        cols: 1,
        data: roots,
    })
}

pub fn compute_real_eigenvalues(matrix: &[Vec<f64>]) -> Result<Vec<f64>, MathError> {
    let n = matrix.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    if n == 1 {
        return Ok(vec![matrix[0][0]]);
    }
    if n == 2 {
        let a = matrix[0][0];
        let b = matrix[0][1];
        let c = matrix[1][0];
        let d = matrix[1][1];
        let tr = a + d;
        let det = a * d - b * c;
        let disc = tr * tr - 4.0 * det;
        if disc < -1e-9 {
            return Err(MathError::ComplexNotSupported);
        }
        let sq = disc.max(0.0).sqrt();
        return Ok(vec![(tr + sq) / 2.0, (tr - sq) / 2.0]);
    }

    // Upper Hessenberg reduction
    let mut h = matrix.to_vec();
    for k in 0..(n - 2) {
        let mut norm_sq = 0.0;
        for i in (k + 1)..n {
            norm_sq += h[i][k] * h[i][k];
        }
        if norm_sq < 1e-24 {
            continue;
        }
        let alpha = norm_sq.sqrt().copysign(h[k + 1][k]);
        let mut v = vec![0.0; n];
        for i in (k + 1)..n {
            v[i] = h[i][k];
        }
        v[k + 1] += alpha;
        let v_norm_sq: f64 = v.iter().map(|&x| x * x).sum();
        if v_norm_sq < 1e-24 {
            continue;
        }

        let factor = 2.0 / v_norm_sq;
        for j in 0..n {
            let mut dot = 0.0;
            for i in (k + 1)..n {
                dot += v[i] * h[i][j];
            }
            for i in (k + 1)..n {
                h[i][j] -= factor * dot * v[i];
            }
        }
        for i in 0..n {
            let mut dot = 0.0;
            for j in (k + 1)..n {
                dot += v[j] * h[i][j];
            }
            for j in (k + 1)..n {
                h[i][j] -= factor * dot * v[j];
            }
        }
    }

    // QR algorithm with Wilkinson shift on upper Hessenberg matrix
    let mut eigenvalues = Vec::new();
    let mut m = n;
    let mut iter = 0;
    let max_iter = 100 * n;

    while m > 2 && iter < max_iter {
        iter += 1;
        let sub = h[m - 1][m - 2].abs();
        let diag_scale = h[m - 1][m - 1].abs() + h[m - 2][m - 2].abs() + 1e-15;
        if sub < 1e-12 * diag_scale {
            eigenvalues.push(h[m - 1][m - 1]);
            m -= 1;
            continue;
        }

        let sub2 = h[m - 2][m - 3].abs();
        let diag_scale2 = h[m - 2][m - 2].abs() + h[m - 3][m - 3].abs() + 1e-15;
        if sub2 < 1e-12 * diag_scale2 {
            let a = h[m - 2][m - 2];
            let b = h[m - 2][m - 1];
            let c = h[m - 1][m - 2];
            let d = h[m - 1][m - 1];
            let tr = a + d;
            let det = a * d - b * c;
            let disc = tr * tr - 4.0 * det;
            if disc < -1e-7 {
                return Err(MathError::ComplexNotSupported);
            }
            let sq = disc.max(0.0).sqrt();
            eigenvalues.push((tr + sq) / 2.0);
            eigenvalues.push((tr - sq) / 2.0);
            m -= 2;
            continue;
        }

        let a = h[m - 2][m - 2];
        let b = h[m - 2][m - 1];
        let c = h[m - 1][m - 2];
        let d = h[m - 1][m - 1];
        let tr = a + d;
        let det = a * d - b * c;
        let disc = tr * tr - 4.0 * det;
        let shift = if disc >= 0.0 {
            let s1 = (tr + disc.sqrt()) / 2.0;
            let s2 = (tr - disc.sqrt()) / 2.0;
            if (s1 - d).abs() < (s2 - d).abs() { s1 } else { s2 }
        } else {
            d
        };

        for i in 0..m {
            h[i][i] -= shift;
        }

        let mut givens = Vec::new();
        for i in 0..(m - 1) {
            let x = h[i][i];
            let y = h[i + 1][i];
            let r = (x * x + y * y).sqrt();
            let (c, s) = if r > 1e-15 {
                (x / r, y / r)
            } else {
                (1.0, 0.0)
            };
            givens.push((c, s));
            for j in i..m {
                let tau1 = h[i][j];
                let tau2 = h[i + 1][j];
                h[i][j] = c * tau1 + s * tau2;
                h[i + 1][j] = -s * tau1 + c * tau2;
            }
        }
        for (i, &(c, s)) in givens.iter().enumerate() {
            let limit = (i + 2).min(m);
            for row in 0..limit {
                let tau1 = h[row][i];
                let tau2 = h[row][i + 1];
                h[row][i] = c * tau1 + s * tau2;
                h[row][i + 1] = -s * tau1 + c * tau2;
            }
        }
        for i in 0..m {
            h[i][i] += shift;
        }
    }

    if m == 2 {
        let a = h[0][0];
        let b = h[0][1];
        let c = h[1][0];
        let d = h[1][1];
        let tr = a + d;
        let det = a * d - b * c;
        let disc = tr * tr - 4.0 * det;
        if disc < -1e-7 {
            return Err(MathError::ComplexNotSupported);
        }
        let sq = disc.max(0.0).sqrt();
        eigenvalues.push((tr + sq) / 2.0);
        eigenvalues.push((tr - sq) / 2.0);
    } else if m == 1 {
        eigenvalues.push(h[0][0]);
    } else if m > 2 {
        for i in 0..m {
            eigenvalues.push(h[i][i]);
        }
    }

    Ok(eigenvalues)
}

fn builtin_trapz(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if args.len() == 1 {
        let y = args[0].to_vector().ok_or(MathError::Other("trapz: y must be a vector".into()))?;
        if y.len() < 2 {
            return Ok(MathValue::scalar(0.0));
        }
        let mut area = 0.0;
        for i in 0..(y.len() - 1) {
            area += (y[i] + y[i + 1]) * 0.5;
        }
        return Ok(MathValue::scalar(area));
    }
    let x = args[0].to_vector().ok_or(MathError::Other("trapz: x must be a vector".into()))?;
    let y = args[1].to_vector().ok_or(MathError::Other("trapz: y must be a vector".into()))?;
    if x.len() != y.len() {
        return Err(MathError::VectorsMustBeSameLength);
    }
    if x.len() < 2 {
        return Ok(MathValue::scalar(0.0));
    }
    let mut area = 0.0;
    for i in 0..(x.len() - 1) {
        let dx = x[i + 1] - x[i];
        area += (y[i] + y[i + 1]) * 0.5 * dx;
    }
    Ok(MathValue::scalar(area))
}

fn builtin_dot(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = args[0].to_vector().ok_or(MathError::Other("dot: a must be a vector".into()))?;
    let b = args[1].to_vector().ok_or(MathError::Other("dot: b must be a vector".into()))?;
    if a.len() != b.len() {
        return Err(MathError::VectorsMustBeSameLength);
    }
    let s: f64 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
    Ok(MathValue::scalar(s))
}

fn builtin_cross(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = args[0].to_vector().ok_or(MathError::CrossRequires3ElementVectors)?;
    let b = args[1].to_vector().ok_or(MathError::CrossRequires3ElementVectors)?;
    if a.len() != 3 || b.len() != 3 {
        return Err(MathError::CrossRequires3ElementVectors);
    }
    let c0 = a[1] * b[2] - a[2] * b[1];
    let c1 = a[2] * b[0] - a[0] * b[2];
    let c2 = a[0] * b[1] - a[1] * b[0];
    let (rows, cols) = if args[0].rows() == 1 {
        (1, 3)
    } else {
        (3, 1)
    };
    Ok(MathValue::Matrix {
        rows,
        cols,
        data: vec![c0, c1, c2],
    })
}

fn builtin_reshape(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = &args[0];
    let r_f = args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
    let c_f = args[2].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
    if !is_int(r_f) || !is_int(c_f) || r_f < 1.0 || c_f < 1.0 {
        return Err(MathError::IndexMustBePositiveInteger);
    }
    let r = r_f as usize;
    let c = c_f as usize;
    if r * c != a.numel() {
        return Err(MathError::Other("To RESHAPE the number of elements must not change.".into()));
    }
    match a {
        MathValue::Matrix { data, .. } => Ok(MathValue::Matrix {
            rows: r,
            cols: c,
            data: data.clone(),
        }),
        _ => Err(MathError::Other("reshape not supported on non-matrix types".into())),
    }
}

fn builtin_fliplr(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = &args[0];
    let (rows, cols) = (a.rows(), a.cols());
    match a {
        MathValue::Matrix { data, .. } => {
            let mut new_data = vec![0.0; rows * cols];
            for c in 0..cols {
                let target_c = cols - 1 - c;
                for r in 0..rows {
                    new_data[target_c * rows + r] = data[c * rows + r];
                }
            }
            Ok(MathValue::Matrix { rows, cols, data: new_data })
        }
        MathValue::Str(s) => {
            let rev: String = s.chars().rev().collect();
            Ok(MathValue::Str(rev))
        }
        _ => Ok(a.clone()),
    }
}

fn builtin_flipud(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = &args[0];
    let (rows, cols) = (a.rows(), a.cols());
    match a {
        MathValue::Matrix { data, .. } => {
            let mut new_data = vec![0.0; rows * cols];
            for c in 0..cols {
                for r in 0..rows {
                    let target_r = rows - 1 - r;
                    new_data[c * rows + target_r] = data[c * rows + r];
                }
            }
            Ok(MathValue::Matrix { rows, cols, data: new_data })
        }
        _ => Ok(a.clone()),
    }
}

fn builtin_rng(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let s = args[0].as_scalar().ok_or(MathError::InputMustBeNonnegativeInteger)?;
    if !is_int(s) || s < 0.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    *ctx.rng_seed = s as u64;
    Ok(MathValue::empty())
}

fn builtin_randi(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let n_f = args[0].as_scalar().ok_or(MathError::InputMustBeNonnegativeInteger)?;
    if !is_int(n_f) || n_f < 1.0 {
        return Err(MathError::InputMustBeNonnegativeInteger);
    }
    let n = n_f as u64;

    let (rows, cols) = if args.len() == 1 {
        (1, 1)
    } else if args.len() == 2 {
        let r = args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
        (r, r)
    } else {
        let r = args[1].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
        let c = args[2].as_scalar().ok_or(MathError::IndexMustBePositiveInteger)? as usize;
        (r, c)
    };

    let next_rand = |seed: &mut u64| -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (*seed >> 11) as f64;
        val / 9007199254740992.0
    };

    let count = rows * cols;
    let mut data = Vec::with_capacity(count);
    for _ in 0..count {
        let r = next_rand(ctx.rng_seed);
        let val = ((r * (n as f64)).floor() as u64).min(n - 1) + 1;
        data.push(val as f64);
    }
    Ok(MathValue::Matrix { rows, cols, data })
}

fn builtin_num2str(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    match &args[0] {
        MathValue::Matrix { rows, cols, data } => {
            if *rows == 1 && *cols == 1 {
                Ok(MathValue::Str(crate::math_engine::value::num2str(data[0])))
            } else {
                let mut lines = Vec::new();
                for r in 0..*rows {
                    let mut row_strs = Vec::new();
                    for c in 0..*cols {
                        row_strs.push(crate::math_engine::value::num2str(data[c * *rows + r]));
                    }
                    lines.push(row_strs.join("   "));
                }
                Ok(MathValue::Str(lines.join("\n")))
            }
        }
        MathValue::Str(s) => Ok(MathValue::Str(s.clone())),
        _ => Ok(MathValue::Str(args[0].to_display_string())),
    }
}

fn builtin_bar(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let (x, y) = if args.len() == 1 {
        let y_vec = args[0].to_vector().ok_or(MathError::Other("bar(y): y must be a vector".into()))?;
        let x_vec: Vec<f64> = (1..=y_vec.len()).map(|i| i as f64).collect();
        (x_vec, y_vec)
    } else {
        let x_vec = args[0].to_vector().ok_or(MathError::Other("bar(x, y): x must be a vector".into()))?;
        let y_vec = args[1].to_vector().ok_or(MathError::Other("bar(x, y): y must be a vector".into()))?;
        if x_vec.len() != y_vec.len() {
            return Err(MathError::VectorsMustBeSameLength);
        }
        (x_vec, y_vec)
    };

    ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
        x: x.clone(),
        y: y.clone(),
        color: [0, 114, 189, 255],
        label: "bar".to_string(),
        line_style: "-".to_string(),
        marker: "".to_string(),
        kind: crate::math_engine::eval::CurveKind::Bar,
    });

    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::Bar {
        x,
        y,
        is_hold: *ctx.hold_on,
    });
    Ok(MathValue::empty())
}

fn builtin_hist(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let v = args[0].to_vector().ok_or(MathError::Other("hist: input must be a vector".into()))?;
    let nbins = if args.len() >= 2 {
        args[1].as_scalar().ok_or(MathError::Other("hist: nbins must be scalar".into()))? as usize
    } else {
        10
    };

    ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
        x: v.clone(),
        y: Vec::new(),
        color: [0, 114, 189, 255],
        label: "hist".to_string(),
        line_style: "-".to_string(),
        marker: "".to_string(),
        kind: crate::math_engine::eval::CurveKind::Hist,
    });

    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::Hist {
        v,
        nbins,
        is_hold: *ctx.hold_on,
    });
    Ok(MathValue::empty())
}

fn builtin_pie(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else {
        if !ctx.current_figure_curves.is_empty() {
            return Err(MathError::PieCannotBeCombined);
        }
    }

    let v = args[0].to_vector().ok_or(MathError::Other("pie: input must be a vector".into()))?;

    ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
        x: v.clone(),
        y: Vec::new(),
        color: [0, 114, 189, 255],
        label: "pie".to_string(),
        line_style: "".to_string(),
        marker: "".to_string(),
        kind: crate::math_engine::eval::CurveKind::Pie,
    });

    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::Pie { v });
    Ok(MathValue::empty())
}

fn builtin_scatter(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let x = args[0].to_vector().ok_or(MathError::Other("scatter: x must be a vector".into()))?;
    let y = args[1].to_vector().ok_or(MathError::Other("scatter: y must be a vector".into()))?;
    if x.len() != y.len() {
        return Err(MathError::VectorsMustBeSameLength);
    }

    ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
        x: x.clone(),
        y: y.clone(),
        color: [0, 114, 189, 255],
        label: "scatter".to_string(),
        line_style: "".to_string(),
        marker: "o".to_string(),
        kind: crate::math_engine::eval::CurveKind::Scatter,
    });

    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::Scatter {
        x,
        y,
        is_hold: *ctx.hold_on,
    });
    Ok(MathValue::empty())
}

fn builtin_xline(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let v = args[0].as_scalar().ok_or(MathError::Other("xline: argument must be scalar".into()))?;
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::XLine(v));
    Ok(MathValue::empty())
}

fn builtin_yline(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let v = args[0].as_scalar().ok_or(MathError::Other("yline: argument must be scalar".into()))?;
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::YLine(v));
    Ok(MathValue::empty())
}

fn builtin_axis(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    match &args[0] {
        MathValue::Str(s) if s == "equal" => {
            ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetAxisEqual);
        }
        MathValue::Matrix { data, .. } => {
            if data.len() == 4 {
                ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::SetAxisLimits(data.clone()));
            } else {
                return Err(MathError::Other("axis requires 4 elements [xmin xmax ymin ymax]".into()));
            }
        }
        _ => return Err(MathError::Other("axis: invalid argument".into())),
    }
    Ok(MathValue::empty())
}

fn builtin_text(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let x = args[0].as_scalar().ok_or(MathError::Other("text: x must be scalar".into()))?;
    let y = args[1].as_scalar().ok_or(MathError::Other("text: y must be scalar".into()))?;
    let text = match &args[2] {
        MathValue::Str(s) => s.clone(),
        val => val.to_display_string(),
    };
    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::DrawText { x, y, text });
    Ok(MathValue::empty())
}

// --- Tier 2 Implementations ---

fn builtin_eig(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let a = &args[0];
    let (rows, cols) = (a.rows(), a.cols());
    if rows != cols {
        return Err(MathError::MatrixMustBeSquare);
    }
    let mat = a.to_2d_vec();
    let mut eigenvalues = compute_real_eigenvalues(&mat)?;
    eigenvalues.sort_by(|x, y| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal));
    Ok(MathValue::Matrix {
        rows: eigenvalues.len(),
        cols: 1,
        data: eigenvalues,
    })
}

fn builtin_fplot(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let fn_val = &args[0];
    let interval = args[1].to_vector().ok_or(MathError::Other("fplot: interval must be [a b]".into()))?;
    if interval.len() != 2 {
        return Err(MathError::Other("fplot: interval requires [a b]".into()));
    }
    let a = interval[0];
    let b = interval[1];
    let n = 200;
    let step = (b - a) / ((n - 1) as f64);
    let mut x_vec = Vec::with_capacity(n);
    let mut y_vec = Vec::with_capacity(n);

    match fn_val {
        MathValue::AnonymousFn { params, body } => {
            if params.len() != 1 {
                return Err(MathError::Other("fplot: anonymous function must take 1 argument".into()));
            }
            let param_name = &params[0];
            let mut eval = crate::math_engine::eval::Evaluator::new();
            for i in 0..n {
                let x = a + (i as f64) * step;
                x_vec.push(x);
                eval.set_var(param_name, MathValue::scalar(x));
                let mut dummy_lines = Vec::new();
                let mut dummy_cmds = Vec::new();
                let y_val = eval.eval_expr(body, None, &mut dummy_lines, &mut dummy_cmds)?;
                let y = y_val.as_scalar().unwrap_or(0.0);
                y_vec.push(y);
            }
        }
        _ => return Err(MathError::Other("fplot requires function handle as first argument".into())),
    }

    if !*ctx.hold_on {
        ctx.current_figure_curves.clear();
    } else if ctx.current_figure_curves.iter().any(|c| c.kind == crate::math_engine::eval::CurveKind::Pie) {
        return Err(MathError::PieCannotBeCombined);
    }

    let default_palette: [[u8; 4]; 7] = [
        [0, 114, 189, 255],
        [217, 83, 25, 255],
        [237, 177, 32, 255],
        [126, 47, 142, 255],
        [119, 172, 48, 255],
        [77, 190, 238, 255],
        [162, 20, 47, 255],
    ];
    let curve_idx = ctx.current_figure_curves.len();
    let color = default_palette[curve_idx % default_palette.len()];

    ctx.current_figure_curves.push(crate::math_engine::eval::CurveInfo {
        x: x_vec.clone(),
        y: y_vec.clone(),
        color,
        label: format!("data{}", curve_idx + 1),
        line_style: "-".to_string(),
        marker: "".to_string(),
        kind: crate::math_engine::eval::CurveKind::Plot,
    });

    ctx.canvas_commands.push(crate::math_engine::eval::CanvasCommand::PlotCurve {
        x: x_vec,
        y: y_vec,
        label: format!("data{}", curve_idx + 1),
        color,
        line_style: "-".to_string(),
        marker: "".to_string(),
        is_hold: *ctx.hold_on,
    });

    Ok(MathValue::empty())
}

fn builtin_fprintf(args: &[MathValue], ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let fmt_str = match &args[0] {
        MathValue::Str(s) => s.clone(),
        _ => return Err(MathError::Other("fprintf: format must be a string".into())),
    };

    let mut out = String::new();
    let chars: Vec<char> = fmt_str.chars().collect();
    let mut i = 0;
    let mut arg_idx = 1;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                '\\' => out.push('\\'),
                c => {
                    out.push('\\');
                    out.push(c);
                }
            }
            i += 2;
            continue;
        }

        if chars[i] == '%' && i + 1 < chars.len() {
            if chars[i + 1] == '%' {
                out.push('%');
                i += 2;
                continue;
            }

            let mut spec = String::new();
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.' || chars[j] == '-') {
                spec.push(chars[j]);
                j += 1;
            }

            if j < chars.len() {
                let type_char = chars[j];
                let val_opt = args.get(arg_idx);
                arg_idx += 1;

                match type_char {
                    'd' | 'i' => {
                        let num = val_opt.and_then(|v| v.as_scalar()).unwrap_or(0.0) as i64;
                        out.push_str(&format!("{}", num));
                    }
                    'f' => {
                        let num = val_opt.and_then(|v| v.as_scalar()).unwrap_or(0.0);
                        if let Some(precision) = spec.strip_prefix('.').and_then(|p| p.parse::<usize>().ok()) {
                            out.push_str(&format!("{:.prec$}", num, prec = precision));
                        } else {
                            out.push_str(&format!("{:.6}", num));
                        }
                    }
                    'g' => {
                        let num = val_opt.and_then(|v| v.as_scalar()).unwrap_or(0.0);
                        out.push_str(&crate::math_engine::value::num2str(num));
                    }
                    's' => {
                        let s = match val_opt {
                            Some(MathValue::Str(s)) => s.clone(),
                            Some(val) => val.to_display_string(),
                            None => "".to_string(),
                        };
                        out.push_str(&s);
                    }
                    _ => {
                        out.push('%');
                        out.push(type_char);
                    }
                }
                i = j + 1;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    ctx.output_lines.push(out);
    Ok(MathValue::empty())
}

fn builtin_error(args: &[MathValue], _ctx: &mut EvalContext) -> Result<MathValue, MathError> {
    let msg = match &args[0] {
        MathValue::Str(s) => s.clone(),
        _ => args[0].to_display_string(),
    };
    Err(MathError::Other(msg))
}
