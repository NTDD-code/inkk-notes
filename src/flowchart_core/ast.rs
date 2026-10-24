use serde::{Deserialize, Serialize};

/// Source text span: line (1-based), column (1-based), byte offset and length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(line: usize, col: usize, start: usize, end: usize) -> Self {
        Self {
            line,
            col,
            start,
            end,
        }
    }

    pub fn merge(self, other: Span) -> Span {
        if self.start == 0 && self.end == 0 {
            return other;
        }
        if other.start == 0 && other.end == 0 {
            return self;
        }
        Span {
            line: self.line.min(other.line),
            col: if self.line <= other.line {
                self.col
            } else {
                other.col
            },
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// A value with its source span attached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

/// Standard flowchart block kinds conforming to ISO 5807.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockKind {
    Start,
    End,
    Process,
    Input,
    Output,
    Decision,
    Switch,
    LoopFor,
    LoopWhile,
    LoopRepeat,
    Subroutine,
    Connector,
    Offpage,
    Document,
    Database,
    ManualInput,
    Delay,
    Display,
    Parallel,
    Comment,
    Swimlane,
}

impl BlockKind {
    pub fn default_label(&self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::End => "End",
            Self::Process => "Process",
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Decision => "Condition?",
            Self::Switch => "Switch",
            Self::LoopFor => "For loop",
            Self::LoopWhile => "While condition",
            Self::LoopRepeat => "Repeat block",
            Self::Subroutine => "Call subroutine",
            Self::Connector => "Connector",
            Self::Offpage => "Offpage link",
            Self::Document => "Document",
            Self::Database => "Database",
            Self::ManualInput => "Manual input",
            Self::Delay => "Wait",
            Self::Display => "Display",
            Self::Parallel => "Parallel",
            Self::Comment => "Note",
            Self::Swimlane => "Lane",
        }
    }
}

/// Abstract Syntax Tree (AST) node representation for Flowchart DSL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstStmt {
    AlgorithmTitle(Spanned<String>),
    Start(Span),
    End(Span),
    Return(Spanned<Option<String>>),
    Process(Spanned<String>),
    Input(Spanned<String>),
    Output(Spanned<String>),
    Decision {
        condition: Spanned<String>,
        then_branch: Vec<AstStmt>,
        else_branch: Option<Vec<AstStmt>>,
        span: Span,
    },
    Switch {
        target: Spanned<String>,
        cases: Vec<AstCase>,
        default_case: Option<Vec<AstStmt>>,
        span: Span,
    },
    LoopFor {
        var: Spanned<String>,
        from: Spanned<String>,
        to: Spanned<String>,
        body: Vec<AstStmt>,
        span: Span,
    },
    LoopWhile {
        condition: Spanned<String>,
        body: Vec<AstStmt>,
        span: Span,
    },
    LoopRepeat {
        body: Vec<AstStmt>,
        until_cond: Spanned<String>,
        span: Span,
    },
    SubroutineCall {
        name: Spanned<String>,
        args: Option<Spanned<String>>,
        span: Span,
    },
    Goto(Spanned<String>),
    Label(Spanned<String>),
    OffpageLink(Spanned<String>),
    Document(Spanned<String>),
    Database(Spanned<String>),
    ManualInput(Spanned<String>),
    Delay(Spanned<String>),
    Display(Spanned<String>),
    Parallel {
        branches: Vec<Vec<AstStmt>>,
        span: Span,
    },
    Note(Spanned<String>),
    Swimlane {
        name: Spanned<String>,
        body: Vec<AstStmt>,
        span: Span,
    },
    RawNodeDef {
        id: Spanned<String>,
        kind: BlockKind,
        label: Spanned<String>,
        span: Span,
    },
    RawEdge {
        from: Spanned<String>,
        from_port: Option<Spanned<String>>,
        to: Spanned<String>,
        to_port: Option<Spanned<String>>,
        label: Option<Spanned<String>>,
        span: Span,
    },
    /// Error recovery token holding the invalid line segment so the rest of the script renders fine.
    ErrorRecovery {
        raw_text: String,
        message: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstCase {
    pub value: Spanned<String>,
    pub body: Vec<AstStmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowchartAst {
    pub title: Option<String>,
    pub statements: Vec<AstStmt>,
}
