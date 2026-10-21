//! Abstract Syntax Tree (AST) for Inkk MathEngine
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    pub line: usize,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMetadata {
    pub name: String,
    pub min_args: usize,
    pub max_args: usize,
    pub num_outputs: usize,
    pub help: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserFunctionAst {
    pub name: String,
    pub params: Vec<String>,
    pub returns: Vec<String>,
    pub body: Vec<Stmt>,
    pub local_functions: std::collections::HashMap<String, UserFunctionAst>,
    pub library: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    Assign {
        target: AssignTarget,
        expr: Expr,
        print: bool,
    },
    MultiAssign {
        targets: Vec<String>,
        expr: Expr,
        print: bool,
    },
    Expr {
        expr: Expr,
        print: bool,
    },
    Command(CommandStmt),
    For {
        var: String,
        expr: Expr,
        body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    FunctionDef {
        ast: UserFunctionAst,
        metadata: FunctionMetadata,
    },
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    Var(String),
    Index {
        var: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FormatMode {
    Short,
    Long,
    Rat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AxisOption {
    Equal,
    Limits(Vec<f64>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandStmt {
    Hold(bool),
    Grid(bool),
    Clear(Vec<String>),
    Clc,
    Close(Option<String>),
    Figure,
    Who,
    Whos,
    Axis(AxisOption),
    Format(FormatMode),
    Help(Option<String>),
    Which(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    pub line: usize,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    Number(f64),
    Str(String),
    Ident(String),
    End,
    ColonAll, // standalone ':' in indexing
    Range {
        start: Box<Expr>,
        step: Option<Box<Expr>>,
        end: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Matrix(Vec<Vec<Expr>>),
    CallOrIndex {
        name: String,
        args: Vec<Expr>,
    },
    AnonymousFn {
        params: Vec<String>,
        body: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Pos,
    Transpose,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    LeftDiv,
    Pow,
    DotMul,
    DotDiv,
    DotPow,
    // Comparisons
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}
