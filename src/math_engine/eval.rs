//! Evaluator with Call Stack, Scopes, Limits, and Plot Dispatching for Inkk MathEngine
use std::collections::HashMap;
use crate::math_engine::ast::*;
use crate::math_engine::error::MathError;
use crate::math_engine::parser::Parser;
use crate::math_engine::registry::{EvalContext, FunctionKind, FunctionRegistry};
use crate::math_engine::token::Lexer;
use crate::math_engine::value::{
    op_add, op_div, op_dot_div, op_dot_mul, op_dot_pow, op_mul, op_neg, op_not, op_pow,
    op_relational, op_sub, MathValue,
};

#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    pub max_steps: usize,
    pub max_call_depth: usize,
    pub max_matrix_elements: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_steps: 10_000_000,
            max_call_depth: 200,
            max_matrix_elements: 10_000_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub name: String,
    pub variables: HashMap<String, MathValue>,
    pub local_functions: HashMap<String, UserFunctionAst>,
    pub is_isolated: bool,
}

impl StackFrame {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variables: HashMap::new(),
            local_functions: HashMap::new(),
            is_isolated: false,
        }
    }

    pub fn new_function(
        name: impl Into<String>,
        local_functions: HashMap<String, UserFunctionAst>,
    ) -> Self {
        let mut frame = Self {
            name: name.into(),
            variables: HashMap::new(),
            local_functions,
            is_isolated: true,
        };
        frame.variables.insert("pi".to_string(), MathValue::scalar(std::f64::consts::PI));
        frame.variables.insert("e".to_string(), MathValue::scalar(std::f64::consts::E));
        frame.variables.insert("Inf".to_string(), MathValue::scalar(f64::INFINITY));
        frame.variables.insert("NaN".to_string(), MathValue::scalar(f64::NAN));
        frame.variables.insert("eps".to_string(), MathValue::scalar(f64::EPSILON));
        frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveKind {
    Plot,
    Bar,
    Hist,
    Pie,
    Scatter,
    Line,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurveInfo {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: [u8; 4],
    pub label: String,
    pub line_style: String,
    pub marker: String,
    pub kind: CurveKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasCommand {
    PlotCurve {
        x: Vec<f64>,
        y: Vec<f64>,
        label: String,
        color: [u8; 4],
        line_style: String,
        marker: String,
        is_hold: bool,
    },
    Bar {
        x: Vec<f64>,
        y: Vec<f64>,
        is_hold: bool,
    },
    Hist {
        v: Vec<f64>,
        nbins: usize,
        is_hold: bool,
    },
    Pie {
        v: Vec<f64>,
    },
    Scatter {
        x: Vec<f64>,
        y: Vec<f64>,
        is_hold: bool,
    },
    XLine(f64),
    YLine(f64),
    SetAxisEqual,
    SetAxisLimits(Vec<f64>),
    DrawText {
        x: f64,
        y: f64,
        text: String,
    },
    SetTitle(String),
    SetXLabel(String),
    SetYLabel(String),
    SetLegend(Vec<String>),
    SetXLim(f64, f64),
    SetYLim(f64, f64),
    SetGrid(bool),
    ClearPlots,
    ClearWorkspace,
    QuickAxes {
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    },
    TableGrid {
        rows: usize,
        cols: usize,
        col_width: f32,
        row_height: f32,
    },
    AddMatrixBox {
        matrix: Vec<Vec<f64>>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct EvalOutput {
    pub text_output: String,
    pub canvas_commands: Vec<CanvasCommand>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Evaluator {
    pub frames: Vec<StackFrame>,
    pub limits: ExecutionLimits,
    pub step_count: usize,
    pub hold_on: bool,
    pub grid_on: bool,
    pub rng_seed: u64,
    pub format_mode: crate::math_engine::ast::FormatMode,
    pub current_figure_curves: Vec<CurveInfo>,
    pub registry: FunctionRegistry,
}

impl Default for Evaluator {
    fn default() -> Self {
        let mut root_frame = StackFrame::new("base");
        // Pre-populate standard constants
        root_frame.variables.insert("pi".to_string(), MathValue::scalar(std::f64::consts::PI));
        root_frame.variables.insert("e".to_string(), MathValue::scalar(std::f64::consts::E));
        root_frame.variables.insert("Inf".to_string(), MathValue::scalar(f64::INFINITY));
        root_frame.variables.insert("NaN".to_string(), MathValue::scalar(f64::NAN));
        root_frame.variables.insert("eps".to_string(), MathValue::scalar(f64::EPSILON));

        Self {
            frames: vec![root_frame],
            limits: ExecutionLimits::default(),
            step_count: 0,
            hold_on: false,
            grid_on: false,
            rng_seed: 123456789,
            format_mode: crate::math_engine::ast::FormatMode::Short,
            current_figure_curves: Vec::new(),
            registry: FunctionRegistry::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StmtControl {
    Normal,
    Break,
    Continue,
    Return,
}

pub fn is_truthy(val: &MathValue) -> bool {
    match val {
        MathValue::Matrix { data, .. } => {
            !data.is_empty() && data.iter().all(|&x| x != 0.0 && !x.is_nan())
        }
        MathValue::Str(s) => !s.is_empty(),
        MathValue::AnonymousFn { .. } => true,
    }
}

impl Evaluator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_steps(&mut self) {
        self.step_count = 0;
    }

    pub fn describe_function(&self, name: &str) -> Option<&FunctionMetadata> {
        self.registry.describe_function(name)
    }

    pub fn lookup_var(&self, name: &str) -> Option<&MathValue> {
        for frame in self.frames.iter().rev() {
            if let Some(val) = frame.variables.get(name) {
                return Some(val);
            }
            if frame.is_isolated {
                break;
            }
        }
        None
    }

    pub fn set_var(&mut self, name: &str, val: MathValue) {
        if let Some(frame) = self.frames.last_mut() {
            frame.variables.insert(name.to_string(), val);
        }
    }

    pub fn delete_var(&mut self, name: &str) {
        if let Some(frame) = self.frames.last_mut() {
            frame.variables.remove(name);
        }
    }

    pub fn clear_all_vars(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.variables.clear();
            frame.variables.insert("pi".to_string(), MathValue::scalar(std::f64::consts::PI));
            frame.variables.insert("e".to_string(), MathValue::scalar(std::f64::consts::E));
            frame.variables.insert("Inf".to_string(), MathValue::scalar(f64::INFINITY));
            frame.variables.insert("NaN".to_string(), MathValue::scalar(f64::NAN));
            frame.variables.insert("eps".to_string(), MathValue::scalar(f64::EPSILON));
        }
    }

    pub fn execute_script(&mut self, script_str: &str) -> EvalOutput {
        self.reset_steps();
        let mut output = EvalOutput::default();

        // 1. Lexing
        let mut lexer = Lexer::new(script_str);
        let tokens = match lexer.tokenize() {
            Ok(toks) => toks,
            Err(e) => {
                let err_str = e.format_with_line(0);
                output.error = Some(err_str.clone());
                output.text_output = err_str;
                return output;
            }
        };

        // 2. Parsing & Evaluation statement-by-statement
        let comments = lexer.comments;
        let mut parser = Parser::new_with_comments(tokens, comments);
        let mut output_lines = Vec::new();
        let mut canvas_cmds = Vec::new();

        loop {
            let next_res = parser.parse_next_stmt();
            match next_res {
                Ok(Some(stmt)) => {
                    let stmt_line = stmt.line;
                    match self.eval_stmt(&stmt, &mut output_lines, &mut canvas_cmds) {
                        Ok(_) => {}
                        Err(e) => {
                            let err_str = e.format_with_line(stmt_line);
                            output.error = Some(err_str.clone());
                            output_lines.push(err_str);
                            break;
                        }
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    let err_str = e.format_with_line(0);
                    output.error = Some(err_str.clone());
                    output_lines.push(err_str);
                    break;
                }
            }
        }

        output.text_output = output_lines.join("\n");
        output.canvas_commands = canvas_cmds;
        output
    }

    fn check_limits(&mut self) -> Result<(), MathError> {
        self.step_count += 1;
        if self.step_count > self.limits.max_steps {
            return Err(MathError::ExecutionLimitExceeded);
        }
        if self.frames.len() > self.limits.max_call_depth {
            return Err(MathError::RecursionDepthExceeded);
        }
        Ok(())
    }

    fn eval_stmt(
        &mut self,
        stmt: &Stmt,
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<StmtControl, MathError> {
        self.check_limits()?;

        match &stmt.kind {
            StmtKind::Command(cmd) => {
                self.eval_command(cmd, output_lines, canvas_cmds)?;
                Ok(StmtControl::Normal)
            }
            StmtKind::Assign { target, expr, print } => {
                let val = self.eval_expr(expr, None, output_lines, canvas_cmds)?;

                // Check matrix element count limit
                if val.numel() > self.limits.max_matrix_elements {
                    return Err(MathError::Other("Matrix elements exceed limit".into()));
                }

                match target {
                    AssignTarget::Var(name) => {
                        let display = format_assign_output(name, &val, self.format_mode);
                        self.set_var(name, val);
                        if *print {
                            output_lines.push(display);
                        }
                    }
                    AssignTarget::Index { var, args } => {
                        let mut target_val = self.lookup_var(var).cloned().unwrap_or(MathValue::empty());
                        self.assign_indexed(&mut target_val, args, &val, output_lines, canvas_cmds)?;
                        let display = format_assign_output(var, &target_val, self.format_mode);
                        self.set_var(var, target_val);
                        if *print {
                            output_lines.push(display);
                        }
                    }
                }
                Ok(StmtControl::Normal)
            }
            StmtKind::MultiAssign { targets, expr, print } => {
                let num_targets = targets.len();
                let unpacked: Vec<MathValue> = if let ExprKind::CallOrIndex { name, args } = &expr.kind {
                    self.eval_call_multi(name, args, num_targets, output_lines, canvas_cmds)?
                } else {
                    let val = self.eval_expr(expr, None, output_lines, canvas_cmds)?;
                    match &val {
                        MathValue::Matrix { data, .. } if data.len() >= num_targets => {
                            data.iter().take(num_targets).map(|&x| MathValue::scalar(x)).collect()
                        }
                        _ => vec![val; num_targets],
                    }
                };
                for (name, item) in targets.iter().zip(unpacked.into_iter()) {
                    let display = format_assign_output(name, &item, self.format_mode);
                    self.set_var(name, item);
                    if *print {
                        output_lines.push(display);
                    }
                }
                Ok(StmtControl::Normal)
            }
            StmtKind::Expr { expr, print } => {
                let val = self.eval_expr(expr, None, output_lines, canvas_cmds)?;
                self.set_var("ans", val.clone());
                if *print {
                    let display = format_assign_output("ans", &val, self.format_mode);
                    output_lines.push(display);
                }
                Ok(StmtControl::Normal)
            }
            StmtKind::For { var, expr, body } => {
                let val = self.eval_expr(expr, None, output_lines, canvas_cmds)?;
                let items: Vec<MathValue> = match &val {
                    MathValue::Matrix { rows, cols, data } => {
                        if *rows == 1 || *cols == 1 {
                            data.iter().map(|&x| MathValue::scalar(x)).collect()
                        } else {
                            let mut col_mats = Vec::with_capacity(*cols);
                            for c in 0..*cols {
                                let mut col_data = Vec::with_capacity(*rows);
                                for r in 0..*rows {
                                    col_data.push(data[c * *rows + r]);
                                }
                                col_mats.push(MathValue::Matrix { rows: *rows, cols: 1, data: col_data });
                            }
                            col_mats
                        }
                    }
                    MathValue::Str(s) => {
                        s.chars().map(|c| MathValue::string(c.to_string())).collect()
                    }
                    _ => vec![val],
                };

                for item in items {
                    self.set_var(var, item);
                    let mut broken = false;
                    for s in body {
                        match self.eval_stmt(s, output_lines, canvas_cmds)? {
                            StmtControl::Break => {
                                broken = true;
                                break;
                            }
                            StmtControl::Continue => {
                                break;
                            }
                            StmtControl::Return => return Ok(StmtControl::Return),
                            StmtControl::Normal => {}
                        }
                    }
                    if broken {
                        break;
                    }
                }
                Ok(StmtControl::Normal)
            }
            StmtKind::While { cond, body } => {
                loop {
                    self.check_limits()?;
                    let c_val = self.eval_expr(cond, None, output_lines, canvas_cmds)?;
                    if !is_truthy(&c_val) {
                        break;
                    }
                    let mut broken = false;
                    for s in body {
                        match self.eval_stmt(s, output_lines, canvas_cmds)? {
                            StmtControl::Break => {
                                broken = true;
                                break;
                            }
                            StmtControl::Continue => {
                                break;
                            }
                            StmtControl::Return => return Ok(StmtControl::Return),
                            StmtControl::Normal => {}
                        }
                    }
                    if broken {
                        break;
                    }
                }
                Ok(StmtControl::Normal)
            }
            StmtKind::If { branches, else_branch } => {
                let mut executed = false;
                for (cond_expr, branch_body) in branches {
                    let c_val = self.eval_expr(cond_expr, None, output_lines, canvas_cmds)?;
                    if is_truthy(&c_val) {
                        executed = true;
                        for s in branch_body {
                            let ctrl = self.eval_stmt(s, output_lines, canvas_cmds)?;
                            if ctrl != StmtControl::Normal {
                                return Ok(ctrl);
                            }
                        }
                        break;
                    }
                }
                if !executed {
                    if let Some(eb) = else_branch {
                        for s in eb {
                            let ctrl = self.eval_stmt(s, output_lines, canvas_cmds)?;
                            if ctrl != StmtControl::Normal {
                                return Ok(ctrl);
                            }
                        }
                    }
                }
                Ok(StmtControl::Normal)
            }
            StmtKind::FunctionDef { ast, metadata } => {
                self.registry.register_user(ast.clone(), metadata.clone())?;
                Ok(StmtControl::Normal)
            }
            StmtKind::Return => Ok(StmtControl::Return),
            StmtKind::Break => Ok(StmtControl::Break),
            StmtKind::Continue => Ok(StmtControl::Continue),
        }
    }

    fn eval_command(
        &mut self,
        cmd: &CommandStmt,
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<(), MathError> {
        match cmd {
            CommandStmt::Hold(on) => {
                self.hold_on = *on;
            }
            CommandStmt::Grid(on) => {
                self.grid_on = *on;
                canvas_cmds.push(CanvasCommand::SetGrid(*on));
            }
            CommandStmt::Clear(vars) => {
                if vars.is_empty() {
                    self.clear_all_vars();
                } else {
                    for v in vars {
                        self.delete_var(v);
                    }
                }
            }
            CommandStmt::Clc => {
                output_lines.clear();
            }
            CommandStmt::Close(_target) => {
                self.current_figure_curves.clear();
                canvas_cmds.push(CanvasCommand::ClearPlots);
            }
            CommandStmt::Figure => {
                self.current_figure_curves.clear();
            }
            CommandStmt::Who => {
                if let Some(frame) = self.frames.last() {
                    let mut names: Vec<String> = frame.variables.keys().cloned().collect();
                    names.sort();
                    output_lines.push(format!("Your variables are:\n\n{}", names.join("  ")));
                }
            }
            CommandStmt::Whos => {
                if let Some(frame) = self.frames.last() {
                    let mut names: Vec<String> = frame.variables.keys().cloned().collect();
                    names.sort();
                    let mut out = String::from("  Name      Size            Bytes  Class\n\n");
                    for name in names {
                        if let Some(val) = frame.variables.get(&name) {
                            let size = val.size_string();
                            let bytes = val.numel() * 8;
                            let class = val.type_name();
                            out.push_str(&format!("  {:8}  {:14}  {:5}  {}\n", name, size, bytes, class));
                        }
                    }
                    output_lines.push(out.trim_end().to_string());
                }
            }
            CommandStmt::Format(mode) => {
                self.format_mode = *mode;
            }
            CommandStmt::Axis(opt) => {
                match opt {
                    AxisOption::Equal => canvas_cmds.push(CanvasCommand::SetAxisEqual),
                    AxisOption::Limits(lims) => canvas_cmds.push(CanvasCommand::SetAxisLimits(lims.clone())),
                }
            }
            CommandStmt::Help(opt) => {
                if let Some(name) = opt {
                    if let Some(meta) = self.registry.describe_function(name) {
                        if meta.examples.is_empty() {
                            output_lines.push(meta.help.clone());
                        } else {
                            let out = format!("--- HELP: {} ---\n{}\n\nExamples:\n{}", meta.name, meta.help, meta.examples.join("\n"));
                            output_lines.push(out);
                        }
                    } else {
                        output_lines.push(format!("No help found for '{}'.", name));
                    }
                } else {
                    output_lines.push("MathStudio: Type 'help <function>' to view help for a function.".to_string());
                }
            }
            CommandStmt::Which(name) => {
                if let Some(entry) = self.registry.get(name) {
                    match &entry.kind {
                        FunctionKind::Builtin(_) => {
                            output_lines.push(format!("'{}' is a built-in function.", name));
                        }
                        FunctionKind::User(ast, _) => {
                            if let Some(lib) = &ast.library {
                                output_lines.push(format!("'{}' is a function in library '{}'.", name, lib));
                            } else {
                                output_lines.push(format!("'{}' is a user-defined function.", name));
                            }
                        }
                    }
                } else if self.lookup_var(name).is_some() {
                    output_lines.push(format!("'{}' is a variable.", name));
                } else {
                    output_lines.push(format!("'{}' not found.", name));
                }
            }
        }
        Ok(())
    }

    pub fn eval_expr(
        &mut self,
        expr: &Expr,
        end_val: Option<usize>,
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<MathValue, MathError> {
        self.check_limits()?;

        match &expr.kind {
            ExprKind::Number(n) => Ok(MathValue::scalar(*n)),
            ExprKind::Str(s) => Ok(MathValue::string(s)),
            ExprKind::End => {
                let val = end_val.ok_or(MathError::Other("end only allowed in indexing context".into()))?;
                Ok(MathValue::scalar(val as f64))
            }
            ExprKind::ColonAll => {
                Ok(MathValue::empty()) // Handled specially in indexing
            }
            ExprKind::Ident(name) => {
                if let Some(val) = self.lookup_var(name) {
                    Ok(val.clone())
                } else {
                    Err(MathError::UnrecognizedIdent(name.clone()))
                }
            }
            ExprKind::Range { start, step, end } => {
                let a = self.eval_expr(start, end_val, output_lines, canvas_cmds)?.as_scalar().ok_or(MathError::Other("Range start must be scalar".into()))?;
                let st = if let Some(step_expr) = step {
                    self.eval_expr(step_expr, end_val, output_lines, canvas_cmds)?.as_scalar().ok_or(MathError::Other("Range step must be scalar".into()))?
                } else {
                    1.0
                };
                let b = self.eval_expr(end, end_val, output_lines, canvas_cmds)?.as_scalar().ok_or(MathError::Other("Range end must be scalar".into()))?;

                if st == 0.0 {
                    return Err(MathError::Other("Range step cannot be 0".into()));
                }

                let mut data = Vec::new();
                if st > 0.0 {
                    let mut cur = a;
                    while cur <= b + 1e-12 {
                        data.push(cur);
                        cur += st;
                    }
                } else {
                    let mut cur = a;
                    while cur >= b - 1e-12 {
                        data.push(cur);
                        cur += st;
                    }
                }

                Ok(MathValue::Matrix {
                    rows: 1,
                    cols: data.len(),
                    data,
                })
            }
            ExprKind::Unary { op, expr } => {
                let val = self.eval_expr(expr, end_val, output_lines, canvas_cmds)?;
                match op {
                    UnaryOp::Neg => op_neg(&val),
                    UnaryOp::Pos => Ok(val),
                    UnaryOp::Transpose => val.transpose(),
                    UnaryOp::Not => op_not(&val),
                }
            }
            ExprKind::Binary { op, left, right } => {
                let l_val = self.eval_expr(left, end_val, output_lines, canvas_cmds)?;
                let r_val = self.eval_expr(right, end_val, output_lines, canvas_cmds)?;

                match op {
                    BinaryOp::Add => op_add(&l_val, &r_val),
                    BinaryOp::Sub => op_sub(&l_val, &r_val),
                    BinaryOp::Mul => op_mul(&l_val, &r_val),
                    BinaryOp::Div => op_div(&l_val, &r_val),
                    BinaryOp::LeftDiv => l_val.solve_linear(&r_val),
                    BinaryOp::Pow => op_pow(&l_val, &r_val),
                    BinaryOp::DotMul => op_dot_mul(&l_val, &r_val),
                    BinaryOp::DotDiv => op_dot_div(&l_val, &r_val),
                    BinaryOp::DotPow => op_dot_pow(&l_val, &r_val),
                    BinaryOp::Eq => op_relational(&l_val, &r_val, |a, b| if (a - b).abs() < 1e-9 { 1.0 } else { 0.0 }),
                    BinaryOp::Ne => op_relational(&l_val, &r_val, |a, b| if (a - b).abs() >= 1e-9 { 1.0 } else { 0.0 }),
                    BinaryOp::Lt => op_relational(&l_val, &r_val, |a, b| if a < b { 1.0 } else { 0.0 }),
                    BinaryOp::Le => op_relational(&l_val, &r_val, |a, b| if a <= b + 1e-9 { 1.0 } else { 0.0 }),
                    BinaryOp::Gt => op_relational(&l_val, &r_val, |a, b| if a > b { 1.0 } else { 0.0 }),
                    BinaryOp::Ge => op_relational(&l_val, &r_val, |a, b| if a >= b - 1e-9 { 1.0 } else { 0.0 }),
                    BinaryOp::And => op_relational(&l_val, &r_val, |a, b| if a != 0.0 && b != 0.0 { 1.0 } else { 0.0 }),
                    BinaryOp::Or => op_relational(&l_val, &r_val, |a, b| if a != 0.0 || b != 0.0 { 1.0 } else { 0.0 }),
                }
            }
            ExprKind::Matrix(rows) => {
                if rows.is_empty() {
                    return Ok(MathValue::empty());
                }

                let mut row_values = Vec::new();
                for r in rows {
                    if r.is_empty() {
                        continue;
                    }
                    let mut row_val = self.eval_expr(&r[0], end_val, output_lines, canvas_cmds)?;
                    for elem in &r[1..] {
                        let next_val = self.eval_expr(elem, end_val, output_lines, canvas_cmds)?;
                        row_val = MathValue::concat_horizontal(&row_val, &next_val)?;
                    }
                    row_values.push(row_val);
                }

                if row_values.is_empty() {
                    return Ok(MathValue::empty());
                }

                let mut result = row_values[0].clone();
                for r in &row_values[1..] {
                    result = MathValue::concat_vertical(&result, r)?;
                }

                Ok(result)
            }
            ExprKind::AnonymousFn { params, body } => {
                Ok(MathValue::AnonymousFn {
                    params: params.clone(),
                    body: body.clone(),
                })
            }
            ExprKind::CallOrIndex { name, args } => {
                // 1. Check local functions in current frame
                let local_fn_opt = self.frames.last().and_then(|frame| {
                    frame.local_functions.get(name).map(|ast| (ast.clone(), frame.local_functions.clone()))
                });

                if let Some((mut ast, parent_locals)) = local_fn_opt {
                    let mut evaluated_args = Vec::with_capacity(args.len());
                    for a in args {
                        evaluated_args.push(self.eval_expr(a, end_val, output_lines, canvas_cmds)?);
                    }
                    ast.local_functions = parent_locals;
                    let meta = FunctionMetadata {
                        name: ast.name.clone(),
                        min_args: 0,
                        max_args: ast.params.len(),
                        num_outputs: ast.returns.len(),
                        help: String::new(),
                        examples: Vec::new(),
                    };
                    let outs = self.call_user_function(&ast, &meta, &evaluated_args, 1, output_lines, canvas_cmds)?;
                    return Ok(outs.into_iter().next().unwrap_or(MathValue::empty()));
                }

                // 2. Check if 'name' is an existing variable in workspace
                if let Some(var) = self.lookup_var(name).cloned() {
                    match &var {
                        MathValue::AnonymousFn { params, body } => {
                            if args.len() != params.len() {
                                return Err(MathError::Other(format!(
                                    "Anonymous function requires {} arguments, got {}",
                                    params.len(),
                                    args.len()
                                )));
                            }
                            let mut evaluated_args = Vec::with_capacity(args.len());
                            for a in args {
                                evaluated_args.push(self.eval_expr(a, end_val, output_lines, canvas_cmds)?);
                            }

                            let mut frame = StackFrame::new(name.clone());
                            for (p, v) in params.iter().zip(evaluated_args.into_iter()) {
                                frame.variables.insert(p.clone(), v);
                            }
                            self.frames.push(frame);

                            let res = self.eval_expr(body, None, output_lines, canvas_cmds);
                            self.frames.pop();
                            return res;
                        }
                        _ => return self.eval_indexing(&var, args, output_lines, canvas_cmds),
                    }
                }

                // 3. Check if 'name' is in function registry
                if let Some(entry) = self.registry.get(name).cloned() {
                    let mut evaluated_args = Vec::with_capacity(args.len());
                    for a in args {
                        evaluated_args.push(self.eval_expr(a, end_val, output_lines, canvas_cmds)?);
                    }

                    match entry.kind {
                        FunctionKind::User(ast, meta) => {
                            let outs = self.call_user_function(&ast, &meta, &evaluated_args, 1, output_lines, canvas_cmds)?;
                            return Ok(outs.into_iter().next().unwrap_or(MathValue::empty()));
                        }
                        FunctionKind::Builtin(_) => {
                            let mut ctx = EvalContext {
                                step_counter: &mut self.step_count,
                                limits: &self.limits,
                                hold_on: &mut self.hold_on,
                                grid_on: &mut self.grid_on,
                                output_lines,
                                canvas_commands: canvas_cmds,
                                current_figure_curves: &mut self.current_figure_curves,
                                rng_seed: &mut self.rng_seed,
                            };
                            return self.registry.call(name, &evaluated_args, &mut ctx);
                        }
                    }
                }

                Err(MathError::UnrecognizedIdent(name.clone()))
            }
        }
    }

    fn eval_indexing(
        &mut self,
        var: &MathValue,
        args: &[Expr],
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<MathValue, MathError> {
        let (rows, cols) = (var.rows(), var.cols());

        if args.len() == 1 {
            let arg_expr = &args[0];
            if matches!(arg_expr.kind, ExprKind::ColonAll) {
                // A(:) -> Flatten into column vector (column-major)
                match var {
                    MathValue::Matrix { data, .. } => {
                        return Ok(MathValue::Matrix {
                            rows: data.len(),
                            cols: 1,
                            data: data.clone(),
                        });
                    }
                    _ => return Ok(var.clone()),
                }
            }

            let idx_val = self.eval_expr(arg_expr, Some(var.numel()), output_lines, canvas_cmds)?;
            if let Some(k) = idx_val.as_scalar() {
                if k.fract() != 0.0 || k < 1.0 {
                    return Err(MathError::IndexMustBePositiveInteger);
                }
                let val = var.index_1d(k as usize)?;
                return Ok(MathValue::scalar(val));
            } else if let Some(indices) = idx_val.to_vector() {
                let mut data = Vec::with_capacity(indices.len());
                for idx in indices {
                    if idx.fract() != 0.0 || idx < 1.0 {
                        return Err(MathError::IndexMustBePositiveInteger);
                    }
                    data.push(var.index_1d(idx as usize)?);
                }
                return Ok(MathValue::Matrix {
                    rows: 1,
                    cols: data.len(),
                    data,
                });
            }
        } else if args.len() == 2 {
            let r_expr = &args[0];
            let c_expr = &args[1];

            let is_r_colon = matches!(r_expr.kind, ExprKind::ColonAll);
            let is_c_colon = matches!(c_expr.kind, ExprKind::ColonAll);

            if is_r_colon && is_c_colon {
                return Ok(var.clone());
            }

            if is_r_colon {
                let c_val = self.eval_expr(c_expr, Some(cols), output_lines, canvas_cmds)?;
                let c = c_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
                if c.fract() != 0.0 || c < 1.0 {
                    return Err(MathError::IndexMustBePositiveInteger);
                }
                return var.slice_col(c as usize);
            }

            if is_c_colon {
                let r_val = self.eval_expr(r_expr, Some(rows), output_lines, canvas_cmds)?;
                let r = r_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
                if r.fract() != 0.0 || r < 1.0 {
                    return Err(MathError::IndexMustBePositiveInteger);
                }
                return var.slice_row(r as usize);
            }

            let r_val = self.eval_expr(r_expr, Some(rows), output_lines, canvas_cmds)?;
            let c_val = self.eval_expr(c_expr, Some(cols), output_lines, canvas_cmds)?;

            let r = r_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
            let c = c_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
            if r.fract() != 0.0 || r < 1.0 || c.fract() != 0.0 || c < 1.0 {
                return Err(MathError::IndexMustBePositiveInteger);
            }

            let val = var.index_2d(r as usize, c as usize)?;
            return Ok(MathValue::scalar(val));
        }

        Err(MathError::Other("Indexing with more than 2 dimensions not supported".into()))
    }

    fn assign_indexed(
        &mut self,
        target: &mut MathValue,
        args: &[Expr],
        rhs: &MathValue,
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<(), MathError> {
        let rhs_scalar = rhs.as_scalar().ok_or(MathError::Other("Indexed assignment only supports scalar values currently".into()))?;

        if args.len() == 1 {
            let k_val = self.eval_expr(&args[0], Some(target.numel()), output_lines, canvas_cmds)?;
            let k = k_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
            if k.fract() != 0.0 || k < 1.0 {
                return Err(MathError::IndexMustBePositiveInteger);
            }
            target.assign_1d(k as usize, rhs_scalar)?;
            return Ok(());
        } else if args.len() == 2 {
            let r_val = self.eval_expr(&args[0], Some(target.rows()), output_lines, canvas_cmds)?;
            let c_val = self.eval_expr(&args[1], Some(target.cols()), output_lines, canvas_cmds)?;

            let r = r_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
            let c = c_val.as_scalar().ok_or(MathError::IndexMustBePositiveInteger)?;
            if r.fract() != 0.0 || r < 1.0 || c.fract() != 0.0 || c < 1.0 {
                return Err(MathError::IndexMustBePositiveInteger);
            }

            target.assign_2d(r as usize, c as usize, rhs_scalar)?;
            return Ok(());
        }

        Err(MathError::Other("Indexed assignment with > 2 dimensions not supported".into()))
    }

    pub fn call_user_function(
        &mut self,
        ast: &UserFunctionAst,
        meta: &FunctionMetadata,
        args: &[MathValue],
        nargout: usize,
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<Vec<MathValue>, MathError> {
        self.check_limits()?;

        if args.len() > meta.max_args {
            return Err(MathError::TooManyArguments(meta.name.clone()));
        }
        if args.len() < meta.min_args {
            return Err(MathError::NotEnoughArguments(meta.name.clone()));
        }

        if self.frames.len() >= self.limits.max_call_depth {
            return Err(MathError::RecursionDepthExceeded);
        }

        let mut frame = StackFrame::new_function(&ast.name, ast.local_functions.clone());
        for (param, val) in ast.params.iter().zip(args.iter()) {
            frame.variables.insert(param.clone(), val.clone());
        }
        frame.variables.insert("nargin".to_string(), MathValue::scalar(args.len() as f64));
        frame.variables.insert("nargout".to_string(), MathValue::scalar(nargout as f64));

        self.frames.push(frame);

        let mut exec_err = None;
        for stmt in &ast.body {
            match self.eval_stmt(stmt, output_lines, canvas_cmds) {
                Ok(StmtControl::Return) => break,
                Ok(_) => {}
                Err(e) => {
                    exec_err = Some(e);
                    break;
                }
            }
        }

        if let Some(err) = exec_err {
            self.frames.pop();
            return Err(err);
        }

        let mut outputs = Vec::new();
        let top = self.frames.last().unwrap();
        for ret_name in &ast.returns {
            if let Some(val) = top.variables.get(ret_name) {
                outputs.push(val.clone());
            } else {
                let err = MathError::OutputArgumentNotAssigned {
                    var: ret_name.clone(),
                    func: ast.name.clone(),
                };
                self.frames.pop();
                return Err(err);
            }
        }

        self.frames.pop();
        Ok(outputs)
    }

    pub fn eval_call_multi(
        &mut self,
        name: &str,
        args: &[Expr],
        nargout: usize,
        output_lines: &mut Vec<String>,
        canvas_cmds: &mut Vec<CanvasCommand>,
    ) -> Result<Vec<MathValue>, MathError> {
        let mut evaluated_args = Vec::with_capacity(args.len());
        for a in args {
            evaluated_args.push(self.eval_expr(a, None, output_lines, canvas_cmds)?);
        }

        // 1. Check local functions in current frame
        let local_fn_opt = self.frames.last().and_then(|frame| {
            frame.local_functions.get(name).map(|ast| (ast.clone(), frame.local_functions.clone()))
        });

        if let Some((mut ast, parent_locals)) = local_fn_opt {
            ast.local_functions = parent_locals;
            let meta = FunctionMetadata {
                name: ast.name.clone(),
                min_args: 0,
                max_args: ast.params.len(),
                num_outputs: ast.returns.len(),
                help: String::new(),
                examples: Vec::new(),
            };
            return self.call_user_function(&ast, &meta, &evaluated_args, nargout, output_lines, canvas_cmds);
        }

        // 2. Check variable in workspace
        if let Some(var) = self.lookup_var(name).cloned() {
            let single_val = match &var {
                MathValue::AnonymousFn { params, body } => {
                    let mut frame = StackFrame::new(name);
                    for (p, v) in params.iter().zip(evaluated_args.into_iter()) {
                        frame.variables.insert(p.clone(), v);
                    }
                    self.frames.push(frame);
                    let res = self.eval_expr(body, None, output_lines, canvas_cmds);
                    self.frames.pop();
                    res?
                }
                _ => self.eval_indexing(&var, args, output_lines, canvas_cmds)?,
            };
            let unpacked = match &single_val {
                MathValue::Matrix { data, .. } if data.len() >= nargout => {
                    data.iter().take(nargout).map(|&x| MathValue::scalar(x)).collect()
                }
                _ => vec![single_val; nargout],
            };
            return Ok(unpacked);
        }

        // 3. Special handling for size(A) in multi-assignment: [r, c] = size(A)
        if name == "size" && nargout == 2 && !evaluated_args.is_empty() {
            let (r, c) = (evaluated_args[0].rows(), evaluated_args[0].cols());
            return Ok(vec![MathValue::scalar(r as f64), MathValue::scalar(c as f64)]);
        }

        // 4. Check registry
        if let Some(entry) = self.registry.get(name).cloned() {
            match entry.kind {
                FunctionKind::Builtin(f) => {
                    let mut ctx = EvalContext {
                        step_counter: &mut self.step_count,
                        limits: &self.limits,
                        hold_on: &mut self.hold_on,
                        grid_on: &mut self.grid_on,
                        output_lines,
                        canvas_commands: canvas_cmds,
                        current_figure_curves: &mut self.current_figure_curves,
                        rng_seed: &mut self.rng_seed,
                    };
                    let single_val = f(&evaluated_args, &mut ctx)?;
                    let unpacked = match &single_val {
                        MathValue::Matrix { data, .. } if data.len() >= nargout => {
                            data.iter().take(nargout).map(|&x| MathValue::scalar(x)).collect()
                        }
                        _ => vec![single_val; nargout],
                    };
                    return Ok(unpacked);
                }
                FunctionKind::User(ast, meta) => {
                    return self.call_user_function(&ast, &meta, &evaluated_args, nargout, output_lines, canvas_cmds);
                }
            }
        }

        Err(MathError::UnrecognizedIdent(name.to_string()))
    }
}

fn format_assign_output(name: &str, val: &MathValue, mode: crate::math_engine::ast::FormatMode) -> String {
    match val {
        MathValue::Str(s) => format!("{} = {}", name, s),
        MathValue::Matrix { rows: 1, cols: 1, .. } => {
            format!("{} = {}", name, val.to_display_string_with_mode(mode))
        }
        MathValue::Matrix { rows: 0, cols: 0, .. } => {
            format!("{} = [](0x0)", name)
        }
        _ => {
            format!("{} =\n{}", name, val.to_display_string_with_mode(mode))
        }
    }
}
