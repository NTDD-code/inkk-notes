//! Inkk Computational Math Engine: Spec v1 & Appendix v1.1
pub mod ast;
pub mod error;
pub mod eval;
pub mod library;
pub mod parser;
pub mod registry;
pub mod token;
pub mod value;

use std::collections::HashMap;
pub use error::MathError;
pub use eval::{CanvasCommand, CurveInfo, EvalOutput, Evaluator, ExecutionLimits, StackFrame};
pub use library::*;
pub use registry::{FunctionKind, FunctionMetadata, FunctionRegistry};
pub use value::MathValue;

#[derive(Debug, Clone)]
pub struct MathWorkspace {
    pub evaluator: Evaluator,
    pub variables: HashMap<String, MathValue>,
}

impl Default for MathWorkspace {
    fn default() -> Self {
        let evaluator = Evaluator::new();
        let variables = evaluator.frames[0].variables.clone();
        Self {
            evaluator,
            variables,
        }
    }
}

impl MathWorkspace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(&mut self, script: &str) -> EvalOutput {
        self.evaluator.frames[0].variables = self.variables.clone();
        let output = self.evaluator.execute_script(script);
        self.variables = self.evaluator.frames[0].variables.clone();
        output
    }

    pub fn describe_function(&self, name: &str) -> Option<&FunctionMetadata> {
        self.evaluator.registry.describe_function(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 25 Mandatory Tier 1 Tests (Section 10) ---

    #[test]
    fn test_01_precedence_arithmetic() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("2+3*4");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 14.0);
    }

    #[test]
    fn test_02_unary_minus_power() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("-2^2");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), -4.0);
    }

    #[test]
    fn test_03_negative_power() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("2^-1");
        assert!(res.error.is_none());
        let val = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert!((val - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_04_bracket_space_unary() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("[1 -2]");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(ans.rows(), 1);
        assert_eq!(ans.cols(), 2);
        assert_eq!(ans.get(0, 0).unwrap(), 1.0);
        assert_eq!(ans.get(0, 1).unwrap(), -2.0);
    }

    #[test]
    fn test_05_bracket_binary_minus() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("[1 - 2]");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(ans.rows(), 1);
        assert_eq!(ans.cols(), 1);
        assert_eq!(ans.as_scalar().unwrap(), -1.0);
    }

    #[test]
    fn test_06_matrix_multiplication() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; A*A");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(ans.to_2d_vec(), vec![vec![7.0, 10.0], vec![15.0, 22.0]]);
    }

    #[test]
    fn test_07_elementwise_multiplication() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; A.*A");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(ans.to_2d_vec(), vec![vec![1.0, 4.0], vec![9.0, 16.0]]);
    }

    #[test]
    fn test_08_matrix_transpose() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; A'");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(ans.to_2d_vec(), vec![vec![1.0, 3.0], vec![2.0, 4.0]]);
    }

    #[test]
    fn test_09_matrix_determinant() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; det(A)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert!((ans - (-2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_10_matrix_inverse() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; inv(A)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        let mat = ans.to_2d_vec();
        assert!((mat[0][0] - (-2.0)).abs() < 1e-9);
        assert!((mat[0][1] - 1.0).abs() < 1e-9);
        assert!((mat[1][0] - 1.5).abs() < 1e-9);
        assert!((mat[1][1] - (-0.5)).abs() < 1e-9);
    }

    #[test]
    fn test_11_matrix_left_divide() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; A\\[5;6]");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        let mat = ans.to_2d_vec();
        assert!((mat[0][0] - (-4.0)).abs() < 1e-9);
        assert!((mat[1][0] - 4.5).abs() < 1e-9);
    }

    #[test]
    fn test_12_range_and_sum() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("x=1:5; sum(x)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert_eq!(ans, 15.0);
    }

    #[test]
    fn test_13_index_end() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("x=1:5; x(end)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert_eq!(ans, 5.0);
    }

    #[test]
    fn test_14_index_2d() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; A(2,1)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert_eq!(ans, 3.0);
    }

    #[test]
    fn test_15_slice_column() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A=[1 2;3 4]; A(:,2)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(ans.to_2d_vec(), vec![vec![2.0], vec![4.0]]);
    }

    #[test]
    fn test_16_linspace() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("linspace(0,1,5)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().to_vector().unwrap();
        let expected = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        assert_eq!(ans.len(), 5);
        for (a, b) in ans.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-9);
        }
    }

    #[test]
    fn test_17_sin_pi() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("sin(pi)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert!((ans - 1.2246467991473532e-16).abs() < 1e-20);
        let disp = ws.variables.get("ans").unwrap().to_display_string();
        assert_eq!(disp, "1.2246e-16");
    }

    #[test]
    fn test_18_divide_by_zero() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("1/0");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap().as_scalar().unwrap();
        assert!(ans.is_infinite() && ans.is_sign_positive());
        assert_eq!(ws.variables.get("ans").unwrap().to_display_string(), "Inf");
    }

    #[test]
    fn test_19_inner_matrix_dimension_mismatch() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("[1 2 3]*[4 5 6]");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Inner matrix dimensions must agree (1x3 * 1x3)"));
    }

    #[test]
    fn test_20_unrecognized_ident() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("foo(1)");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Unrecognized function or variable 'foo'"));
    }

    #[test]
    fn test_21_complex_not_supported() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("sqrt(-1)");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Complex results are not supported"));
    }

    #[test]
    fn test_22_plot_vector_length_mismatch() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("plot(1:3, [1 2])");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Vectors must be the same length"));
    }

    #[test]
    fn test_23_unsupported_feature() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("syms x");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Feature not supported in Inkk MathStudio: syms"));
    }

    #[test]
    fn test_24_parse_error_keeps_previous_vars() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("x=5; y=x+;");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Parse error"));
        assert_eq!(ws.variables.get("x").unwrap().as_scalar().unwrap(), 5.0);
        assert!(!ws.variables.contains_key("y"));
    }

    #[test]
    fn test_25_hold_on_plot_colors() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("x=1:3; y=1:3; z=2:4; plot(x,y); hold on; plot(x,z)");
        assert!(res.error.is_none());
        assert_eq!(res.canvas_commands.len(), 2);
        if let CanvasCommand::PlotCurve { color: c1, .. } = &res.canvas_commands[0] {
            assert_eq!(*c1, [0, 114, 189, 255]); // #0072BD
        } else {
            panic!("Expected PlotCurve");
        }
        if let CanvasCommand::PlotCurve { color: c2, is_hold, .. } = &res.canvas_commands[1] {
            assert_eq!(*c2, [217, 83, 25, 255]); // #D95319
            assert!(*is_hold);
        } else {
            panic!("Expected PlotCurve with hold on");
        }
    }

    #[test]
    fn test_describe_function() {
        let ws = MathWorkspace::new();
        let meta = ws.describe_function("linspace").unwrap();
        assert_eq!(meta.name, "linspace");
        assert_eq!(meta.min_args, 2);
        assert_eq!(meta.max_args, 3);
        assert!(!meta.help.is_empty());
    }

    #[test]
    fn test_execution_limit_exceeded() {
        let mut ws = MathWorkspace::new();
        ws.evaluator.limits.max_steps = 10;
        let res = ws.execute("a=1; b=2; c=3; d=4; e=5; f=6; g=7; h=8; i=9; j=10; k=11; l=12;");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Execution limit exceeded"));
    }

    #[test]
    fn test_extra_tier1_builtins() {
        let mut ws = MathWorkspace::new();
        // Creation & Size
        let res = ws.execute("Z = zeros(2, 3); O = ones(2, 2); E = eye(3); sz = size(Z); len = length(Z); num = numel(Z); emp = isempty([]);");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("Z").unwrap().rows(), 2);
        assert_eq!(ws.variables.get("Z").unwrap().cols(), 3);
        assert_eq!(ws.variables.get("O").unwrap().to_2d_vec(), vec![vec![1.0, 1.0], vec![1.0, 1.0]]);
        assert_eq!(ws.variables.get("E").unwrap().to_2d_vec(), vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0]]);
        assert_eq!(ws.variables.get("sz").unwrap().to_vector().unwrap(), vec![2.0, 3.0]);
        assert_eq!(ws.variables.get("len").unwrap().as_scalar().unwrap(), 3.0);
        assert_eq!(ws.variables.get("num").unwrap().as_scalar().unwrap(), 6.0);
        assert_eq!(ws.variables.get("emp").unwrap().as_scalar().unwrap(), 1.0);

        // Stats & Math
        let res2 = ws.execute("v = [2 4 1 3]; p = prod(v); m = mean(v); cs = cumsum(v); s = sort(v); n = norm([3 4]);");
        assert!(res2.error.is_none());
        assert_eq!(ws.variables.get("p").unwrap().as_scalar().unwrap(), 24.0);
        assert_eq!(ws.variables.get("m").unwrap().as_scalar().unwrap(), 2.5);
        assert_eq!(ws.variables.get("cs").unwrap().to_vector().unwrap(), vec![2.0, 6.0, 7.0, 10.0]);
        assert_eq!(ws.variables.get("s").unwrap().to_vector().unwrap(), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(ws.variables.get("n").unwrap().as_scalar().unwrap(), 5.0);

        // Linear algebra: trace, rank
        let res3 = ws.execute("A = [1 2; 3 4]; tr = trace(A); rk = rank(A);");
        assert!(res3.error.is_none());
        assert_eq!(ws.variables.get("tr").unwrap().as_scalar().unwrap(), 5.0);
        assert_eq!(ws.variables.get("rk").unwrap().as_scalar().unwrap(), 2.0);

        // LineSpec and Name-Value plot
        let res4 = ws.execute("x = 1:5; y = x.^2; plot(x, y, \"DisplayName\", \"Square Curve\")");
        assert!(res4.error.is_none());
        if let CanvasCommand::PlotCurve { label, .. } = &res4.canvas_commands[0] {
            assert_eq!(label, "Square Curve");
        } else {
            panic!("Expected PlotCurve");
        }

        // Column-major colon linearization A(:)
        let res5 = ws.execute("A = [1 2; 3 4]; colA = A(:);");
        assert!(res5.error.is_none());
        assert_eq!(ws.variables.get("colA").unwrap().to_2d_vec(), vec![vec![1.0], vec![3.0], vec![2.0], vec![4.0]]);
    }

    // --- Tier 1B Tests (Section D) ---

    #[test]
    fn test_tier1b_arithmetic_thcs() {
        let mut ws = MathWorkspace::new();

        // gcd(12,18) -> 6
        let res = ws.execute("gcd(12, 18)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 6.0);

        // lcm(4,6) -> 12
        let res = ws.execute("lcm(4, 6)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 12.0);

        // factorial(5) -> 120
        let res = ws.execute("factorial(5)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 120.0);

        // nchoosek(5,2) -> 10
        let res = ws.execute("nchoosek(5, 2)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 10.0);

        // primes(20) -> [2 3 5 7 11 13 17 19]
        let res = ws.execute("primes(20)");
        assert!(res.error.is_none());
        assert_eq!(
            ws.variables.get("ans").unwrap().to_vector().unwrap(),
            vec![2.0, 3.0, 5.0, 7.0, 11.0, 13.0, 17.0, 19.0]
        );

        // factor(60) -> [2 2 3 5]
        let res = ws.execute("factor(60)");
        assert!(res.error.is_none());
        assert_eq!(
            ws.variables.get("ans").unwrap().to_vector().unwrap(),
            vec![2.0, 2.0, 3.0, 5.0]
        );

        // isprime(7) -> 1
        let res = ws.execute("isprime(7)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 1.0);

        // nthroot(-8,3) -> -2
        let res = ws.execute("nthroot(-8, 3)");
        assert!(res.error.is_none());
        assert!((ws.variables.get("ans").unwrap().as_scalar().unwrap() - (-2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_tier1b_degrees_trig() {
        let mut ws = MathWorkspace::new();

        // sind(30) -> 0.5000
        let res = ws.execute("sind(30)");
        assert!(res.error.is_none());
        assert!((ws.variables.get("ans").unwrap().as_scalar().unwrap() - 0.5).abs() < 1e-9);

        // cosd(90) -> 0 (đúng tuyệt đối)
        let res = ws.execute("cosd(90)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 0.0);
    }

    #[test]
    fn test_tier1b_statistics() {
        let mut ws = MathWorkspace::new();

        // mean([1 2 3 4]) -> 2.5000
        let res = ws.execute("mean([1 2 3 4])");
        assert!(res.error.is_none());
        assert!((ws.variables.get("ans").unwrap().as_scalar().unwrap() - 2.5).abs() < 1e-9);

        // median([3 1 2]) -> 2
        let res = ws.execute("median([3 1 2])");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 2.0);

        // std([1 2 3 4]) -> 1.2910
        let res = ws.execute("std([1 2 3 4])");
        assert!(res.error.is_none());
        assert!((ws.variables.get("ans").unwrap().as_scalar().unwrap() - 1.2909944487358056).abs() < 1e-9);

        // var([1 2 3 4]) -> 1.6667
        let res = ws.execute("var([1 2 3 4])");
        assert!(res.error.is_none());
        assert!((ws.variables.get("ans").unwrap().as_scalar().unwrap() - (5.0 / 3.0)).abs() < 1e-9);

        // cumprod([1 2 3 4]) -> [1 2 6 24]
        let res = ws.execute("cumprod([1 2 3 4])");
        assert!(res.error.is_none());
        assert_eq!(
            ws.variables.get("ans").unwrap().to_vector().unwrap(),
            vec![1.0, 2.0, 6.0, 24.0]
        );

        // diff([1 4 9 16]) -> [3 5 7]
        let res = ws.execute("diff([1 4 9 16])");
        assert!(res.error.is_none());
        assert_eq!(
            ws.variables.get("ans").unwrap().to_vector().unwrap(),
            vec![3.0, 5.0, 7.0]
        );
    }

    #[test]
    fn test_tier1b_polynomials() {
        let mut ws = MathWorkspace::new();

        // polyfit([1 2 3],[2 4 6],1) -> [2 0]
        let res = ws.execute("polyfit([1 2 3], [2 4 6], 1)");
        assert!(res.error.is_none());
        let p = ws.variables.get("ans").unwrap().to_vector().unwrap();
        assert!((p[0] - 2.0).abs() < 1e-9);
        assert!(p[1].abs() < 1e-9);

        // polyval([1 0 -1],3) -> 8
        let res = ws.execute("polyval([1 0 -1], 3)");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 8.0);

        // roots([1 -3 2]) -> [2; 1]
        let res = ws.execute("roots([1 -3 2])");
        assert!(res.error.is_none());
        let roots = ws.variables.get("ans").unwrap();
        assert_eq!(roots.rows(), 2);
        assert_eq!(roots.cols(), 1);
        let r_vec = roots.to_vector().unwrap();
        assert!((r_vec[0] - 2.0).abs() < 1e-9);
        assert!((r_vec[1] - 1.0).abs() < 1e-9);

        // roots([1 0 1]) -> Error: Complex results are not supported.
        let res = ws.execute("roots([1 0 1])");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Complex results are not supported."));
    }

    #[test]
    fn test_tier1b_calculus_and_vectors() {
        let mut ws = MathWorkspace::new();

        // trapz([0 1 2],[0 1 4]) -> 3
        let res = ws.execute("trapz([0 1 2], [0 1 4])");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 3.0);

        // dot([1 2 3],[4 5 6]) -> 32
        let res = ws.execute("dot([1 2 3], [4 5 6])");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 32.0);

        // cross([1 0 0],[0 1 0]) -> [0 0 1]
        let res = ws.execute("cross([1 0 0], [0 1 0])");
        assert!(res.error.is_none());
        assert_eq!(
            ws.variables.get("ans").unwrap().to_vector().unwrap(),
            vec![0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn test_tier1b_reshape_and_matrix() {
        let mut ws = MathWorkspace::new();

        // reshape(1:6,2,3) -> [1 3 5; 2 4 6]
        let res = ws.execute("reshape(1:6, 2, 3)");
        assert!(res.error.is_none());
        let ans = ws.variables.get("ans").unwrap();
        assert_eq!(
            ans.to_2d_vec(),
            vec![vec![1.0, 3.0, 5.0], vec![2.0, 4.0, 6.0]]
        );

        // A=[1 2;3 4]; A(:) -> [1;3;2;4]
        let res = ws.execute("A = [1 2; 3 4]; A(:)");
        assert!(res.error.is_none());
        assert_eq!(
            ws.variables.get("ans").unwrap().to_2d_vec(),
            vec![vec![1.0], vec![3.0], vec![2.0], vec![4.0]]
        );
    }

    #[test]
    fn test_tier1b_errors() {
        let mut ws = MathWorkspace::new();

        // nchoosek(3,5) -> Error: k must not exceed n.
        let res = ws.execute("nchoosek(3, 5)");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("k must not exceed n."));

        // factorial(-1) -> Error: Input must be a nonnegative integer.
        let res = ws.execute("factorial(-1)");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Input must be a nonnegative integer."));

        // gcd(1.5,3) -> Error: Arguments must be integers.
        let res = ws.execute("gcd(1.5, 3)");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Arguments must be integers."));
    }

    #[test]
    fn test_tier1b_strings_and_format() {
        let mut ws = MathWorkspace::new();

        // num2str(pi) -> 3.1416
        let res = ws.execute("num2str(pi)");
        assert!(res.error.is_none());
        if let MathValue::Str(s) = ws.variables.get("ans").unwrap() {
            assert_eq!(s, "3.1416");
        } else {
            panic!("Expected string");
        }

        // num2str(5) -> 5
        let res = ws.execute("num2str(5)");
        assert!(res.error.is_none());
        if let MathValue::Str(s) = ws.variables.get("ans").unwrap() {
            assert_eq!(s, "5");
        } else {
            panic!("Expected string");
        }

        // format rat; 0.75 -> ans = 3/4
        let res = ws.execute("format rat; 0.75");
        assert!(res.error.is_none());
        assert!(res.text_output.contains("ans = 3/4"));
    }

    #[test]
    fn test_tier1b_rng_and_plots() {
        let mut ws = MathWorkspace::new();

        // rng(1); a=rand(); rng(1); b=rand(); a==b -> 1 (tái lập được)
        let res = ws.execute("rng(1); a=rand(); rng(1); b=rand(); a==b");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 1.0);

        // bar([1 2 3]) -> Figure
        let res = ws.execute("bar([1 2 3])");
        assert!(res.error.is_none());
        assert!(res.canvas_commands.iter().any(|c| matches!(c, CanvasCommand::Bar { .. })));

        // pie cannot be combined with other plots
        let res = ws.execute("plot([1 2]); hold on; pie([1 2 3])");
        assert!(res.error.is_some());
        assert!(res.error.unwrap().contains("Pie charts cannot be combined with other plots."));
    }

    // --- Tier 2 Tests ---

    #[test]
    fn test_tier2_relational_and_logical() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("eq = ([1 2 3] == [1 0 3]); ne = ([1 2] ~= [1 0]); lt = ([1 2] < [2 1]);");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("eq").unwrap().to_vector().unwrap(), vec![1.0, 0.0, 1.0]);
        assert_eq!(ws.variables.get("ne").unwrap().to_vector().unwrap(), vec![0.0, 1.0]);
        assert_eq!(ws.variables.get("lt").unwrap().to_vector().unwrap(), vec![1.0, 0.0]);

        let res2 = ws.execute("a = ([1 0] & [1 1]); o = ([1 0] | [0 0]); n = ~[1 0 2];");
        assert!(res2.error.is_none());
        assert_eq!(ws.variables.get("a").unwrap().to_vector().unwrap(), vec![1.0, 0.0]);
        assert_eq!(ws.variables.get("o").unwrap().to_vector().unwrap(), vec![1.0, 0.0]);
        assert_eq!(ws.variables.get("n").unwrap().to_vector().unwrap(), vec![0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_tier2_for_and_while_loops() {
        let mut ws = MathWorkspace::new();

        // for loop sum
        let res = ws.execute("s = 0; for k = 1:5; s = s + k; end; s");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("s").unwrap().as_scalar().unwrap(), 15.0);

        // for with break
        let res2 = ws.execute("val = 0; for k = 1:10; if k == 4; break; end; val = k; end; val");
        assert!(res2.error.is_none());
        assert_eq!(ws.variables.get("val").unwrap().as_scalar().unwrap(), 3.0);

        // for with continue
        let res3 = ws.execute("c = 0; for k = 1:5; if k == 3; continue; end; c = c + 1; end; c");
        assert!(res3.error.is_none());
        assert_eq!(ws.variables.get("c").unwrap().as_scalar().unwrap(), 4.0);

        // while loop
        let res4 = ws.execute("k = 1; tot = 0; while k <= 4; tot = tot + k; k = k + 1; end; tot");
        assert!(res4.error.is_none());
        assert_eq!(ws.variables.get("tot").unwrap().as_scalar().unwrap(), 10.0);
    }

    #[test]
    fn test_tier2_if_elseif_else() {
        let mut ws = MathWorkspace::new();

        let res1 = ws.execute("x = 10; if x < 5; y = 1; elseif x == 10; y = 2; else; y = 3; end;");
        assert!(res1.error.is_none());
        assert_eq!(ws.variables.get("y").unwrap().as_scalar().unwrap(), 2.0);

        let res2 = ws.execute("x = 20; if x < 5; y = 1; elseif x == 10; y = 2; else; y = 3; end;");
        assert!(res2.error.is_none());
        assert_eq!(ws.variables.get("y").unwrap().as_scalar().unwrap(), 3.0);
    }

    #[test]
    fn test_tier2_anonymous_function_and_fplot() {
        let mut ws = MathWorkspace::new();

        // Single param
        let res = ws.execute("f = @(x) x.^2 - 1; ans = f(3);");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 8.0);

        // Multi param
        let res2 = ws.execute("g = @(x, y) 2*x + y; ans = g(3, 4);");
        assert!(res2.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 10.0);

        // fplot
        let res3 = ws.execute("fplot(@(x) x.^2, [-2 2])");
        assert!(res3.error.is_none());
        if let CanvasCommand::PlotCurve { x, y, .. } = &res3.canvas_commands[0] {
            assert_eq!(x.len(), 200);
            assert_eq!(y.len(), 200);
            assert!((x[0] - (-2.0)).abs() < 1e-9);
            assert!((x[199] - 2.0).abs() < 1e-9);
        } else {
            panic!("Expected PlotCurve");
        }
    }

    #[test]
    fn test_tier2_multi_assign() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("A = [1 2 3; 4 5 6]; [r, c] = size(A);");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("r").unwrap().as_scalar().unwrap(), 2.0);
        assert_eq!(ws.variables.get("c").unwrap().as_scalar().unwrap(), 3.0);
    }

    #[test]
    fn test_tier2_eig() {
        let mut ws = MathWorkspace::new();

        // Real eigenvalues
        let res = ws.execute("A = [2 0; 0 3]; e = eig(A);");
        assert!(res.error.is_none());
        let ev = ws.variables.get("e").unwrap().to_vector().unwrap();
        assert_eq!(ev, vec![3.0, 2.0]);

        // Complex eigenvalues -> error
        let res2 = ws.execute("A = [0 -1; 1 0]; eig(A)");
        assert!(res2.error.is_some());
        assert!(res2.error.unwrap().contains("Complex results are not supported."));
    }

    #[test]
    fn test_tier2_fprintf() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("fprintf(\"Count: %d, Rate: %.2f\\n\", 42, 3.14159)");
        assert!(res.error.is_none());
        assert!(res.text_output.contains("Count: 42, Rate: 3.14"));
    }

    #[test]
    fn test_tier2_multi_pair_plot() {
        let mut ws = MathWorkspace::new();
        let res = ws.execute("plot(1:5, 1:5, 1:5, (1:5).^2)");
        assert!(res.error.is_none());
        assert_eq!(res.canvas_commands.len(), 2);
    }

    // --- Tier 3 Tests (Section E5) ---

    #[test]
    fn test_tier3_sum2() {
        let mut ws = MathWorkspace::new();
        let code = "function s = sum2(a, b)\n    % Calculate sum of two numbers\n    s = a + b;\nend\nans = sum2(2, 3);";
        let res = ws.execute(code);
        assert!(res.error.is_none(), "Error: {:?}", res.error);
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 5.0);

        // Test help and describe_function
        let meta = ws.describe_function("sum2").expect("meta found");
        assert_eq!(meta.name, "sum2");
        assert_eq!(meta.help, "Calculate sum of two numbers");
        assert_eq!(meta.max_args, 2);
        assert_eq!(meta.num_outputs, 1);

        let help_res = ws.execute("help sum2");
        assert!(help_res.text_output.contains("Calculate sum of two numbers"));

        let which_res = ws.execute("which sum2");
        assert!(which_res.text_output.contains("'sum2' is a user-defined function."));
    }

    #[test]
    fn test_tier3_nargin_defaults() {
        let mut ws = MathWorkspace::new();
        let code = "function y = f(a, b)\n    if nargin < 2\n        b = 10;\n    end\n    y = a + b;\nend";
        let res1 = ws.execute(code);
        assert!(res1.error.is_none());

        let res2 = ws.execute("ans = f(1);");
        assert!(res2.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 11.0);

        let res3 = ws.execute("ans = f(1, 2);");
        assert!(res3.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 3.0);
    }

    #[test]
    fn test_tier3_multi_output_rng2() {
        let mut ws = MathWorkspace::new();
        let code = "function [a, b] = rng2(v)\n    a = min(v);\n    b = max(v);\nend\n[a, b] = rng2([3 1 2]);";
        let res = ws.execute(code);
        assert!(res.error.is_none(), "Error: {:?}", res.error);
        assert_eq!(ws.variables.get("a").unwrap().as_scalar().unwrap(), 1.0);
        assert_eq!(ws.variables.get("b").unwrap().as_scalar().unwrap(), 3.0);
    }

    #[test]
    fn test_tier3_unassigned_output_error() {
        let mut ws = MathWorkspace::new();
        let code = "function y = f(x)\n    % Does not assign y\n    z = x + 1;\nend\nf(5)";
        let res = ws.execute(code);
        assert!(res.error.is_some());
        assert_eq!(
            res.error.unwrap(),
            "Error (line 5): Output argument 'y' was not assigned during the call to 'f'."
        );
    }

    #[test]
    fn test_tier3_too_many_arguments() {
        let mut ws = MathWorkspace::new();
        let code = "function s = sum2(a, b)\n    s = a + b;\nend\nsum2(1, 2, 3)";
        let res = ws.execute(code);
        assert!(res.error.is_some());
        assert_eq!(
            res.error.unwrap(),
            "Error (line 4): Too many input arguments for 'sum2'."
        );
    }

    #[test]
    fn test_tier3_cannot_define_builtin() {
        let ws = MathWorkspace::new();
        let code = "function y = sin(x)\n    y = x;\nend";
        let load_res = LibraryManager::parse_function_source("sin.m", code, None, &ws.evaluator);
        assert!(load_res.is_err());
        let (_, err_msg) = load_res.unwrap_err();
        assert_eq!(err_msg, "Cannot define 'sin': name is a built-in function.");
    }

    #[test]
    fn test_tier3_recursive_fact() {
        let mut ws = MathWorkspace::new();
        let code = "function f = fact(n)\n    if n <= 1\n        f = 1;\n        return;\n    end\n    f = n * fact(n - 1);\nend\nans = fact(5);";
        let res = ws.execute(code);
        assert!(res.error.is_none(), "Error: {:?}", res.error);
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 120.0);
    }

    #[test]
    fn test_tier3_recursion_depth_exceeded() {
        let mut ws = MathWorkspace::new();
        ws.evaluator.limits.max_call_depth = 50;
        let code = "function f = fact(n)\n    f = fact(n - 1);\nend\nfact(1e6)";
        let res = ws.execute(code);
        assert!(res.error.is_some());
        assert_eq!(
            res.error.unwrap(),
            "Error (line 4): Maximum recursion depth exceeded."
        );
    }

    #[test]
    fn test_tier3_execution_limit_exceeded() {
        let mut ws = MathWorkspace::new();
        ws.evaluator.limits.max_steps = 10_000;
        let code = "function f()\n    while 1\n    end\nend\nf()";
        let res = ws.execute(code);
        assert!(res.error.is_some());
        assert_eq!(
            res.error.unwrap(),
            "Error (line 5): Execution limit exceeded."
        );
    }

    #[test]
    fn test_tier3_variable_scope_isolation() {
        let mut ws = MathWorkspace::new();
        let code = "x = 1;\nfunction g()\n    x = 5;\nend\ng();";
        let res = ws.execute(code);
        assert!(res.error.is_none(), "Error: {:?}", res.error);
        assert_eq!(ws.variables.get("x").unwrap().as_scalar().unwrap(), 1.0);
    }

    #[test]
    fn test_tier3_unsupported_eval_rejected() {
        let ws = MathWorkspace::new();
        let code = "function y = my_eval(str)\n    y = eval(str);\nend";
        let load_res = LibraryManager::parse_function_source("my_eval.m", code, None, &ws.evaluator);
        assert!(load_res.is_err());
        let (line, err_msg) = load_res.unwrap_err();
        assert_eq!(line, 2);
        assert_eq!(err_msg, "Feature not supported in Inkk MathStudio: eval");
    }

    #[test]
    fn test_tier3_local_functions_in_file() {
        let mut ws = MathWorkspace::new();
        let code = "function y = main_fn(x)\n    y = helper(x) * 2;\nend\nfunction h = helper(z)\n    h = z + 1;\nend";
        let (ast, meta) = LibraryManager::parse_function_source("main_fn.m", code, None, &ws.evaluator).expect("parse ok");
        ws.evaluator.registry.register_user(ast, meta).expect("register ok");

        // Calling public function works
        let res1 = ws.execute("ans = main_fn(3);");
        assert!(res1.error.is_none(), "Error: {:?}", res1.error);
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 8.0);

        // Calling local function directly from outside fails
        let res2 = ws.execute("helper(3);");
        assert!(res2.error.is_some());
        assert!(res2.error.unwrap().contains("Unrecognized function or variable 'helper'."));
    }

    #[test]
    fn test_tier3_library_collision() {
        let mut ws = MathWorkspace::new();
        let code = "function y = f(x)\n    y = x;\nend";
        let (ast1, meta1) = LibraryManager::parse_function_source("f.m", code, Some("libA"), &ws.evaluator).unwrap();
        ws.evaluator.registry.register_user(ast1, meta1).unwrap();

        let (ast2, meta2) = LibraryManager::parse_function_source("f.m", code, Some("libB"), &ws.evaluator).unwrap();
        let err = ws.evaluator.registry.register_user(ast2, meta2).unwrap_err();
        assert_eq!(
            err.raw_message(),
            "Function 'f' is defined in both 'libA' and 'libB'."
        );
    }

    #[test]
    fn test_tier3_whiteboard_embedded_functions() {
        let mut ws = MathWorkspace::new();
        let source = "function y = my_cube(x)\n    y = x .^ 3;\nend";
        let hash = compute_source_hash(source);
        let ef = EmbeddedFunction {
            name: "my_cube".to_string(),
            source: source.to_string(),
            library: None,
            version: Some("1.0.0".to_string()),
            hash,
        };

        let reports = load_embedded_functions(&mut ws.evaluator, &[ef]);
        assert_eq!(reports.len(), 1);
        assert!(reports[0].is_ok());

        let res = ws.execute("ans = my_cube(3);");
        assert!(res.error.is_none());
        assert_eq!(ws.variables.get("ans").unwrap().as_scalar().unwrap(), 27.0);
    }
}
