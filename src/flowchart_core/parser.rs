use crate::flowchart_core::ast::*;
use crate::flowchart_core::lexer::{Lexer, Token, TokenKind};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Expected {expected}, found {found} at line {line}, col {col}")]
    Expected {
        expected: String,
        found: String,
        line: usize,
        col: usize,
    },
    #[error("Syntax error: {message}")]
    Custom { message: String, span: Span },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn from_str(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Self::new(tokens)
    }

    fn current(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("Tokens non empty"))
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn skip_newlines(&mut self) {
        while self.current().kind == TokenKind::Newline {
            self.advance();
        }
    }

    #[allow(dead_code)]
    fn peek(&self) -> &Token {
        self.current()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Read remainder of the statement line as a raw string until Newline, Comment, or EOF.
    fn consume_line_rest(&mut self) -> (String, Span) {
        let mut parts = Vec::new();
        let start_span = self.current().span;
        let mut end_span = start_span;

        while !matches!(self.current().kind, TokenKind::Newline | TokenKind::Eof) {
            let tok = self.advance().clone();
            end_span = tok.span;
            let text = match tok.kind {
                TokenKind::StringLit(s) => format!("\"{}\"", s),
                TokenKind::Identifier(s) | TokenKind::Number(s) => s,
                TokenKind::Colon => ":".to_string(),
                TokenKind::Comma => ",".to_string(),
                TokenKind::Arrow => "->".to_string(),
                TokenKind::LBracket => "[".to_string(),
                TokenKind::RBracket => "]".to_string(),
                TokenKind::LParen => "(".to_string(),
                TokenKind::RParen => ")".to_string(),
                TokenKind::Set => "set".to_string(),
                TokenKind::Let => "let".to_string(),
                TokenKind::Do => "do".to_string(),
                TokenKind::Input => "input".to_string(),
                TokenKind::Read => "read".to_string(),
                TokenKind::Output => "output".to_string(),
                TokenKind::Print => "print".to_string(),
                TokenKind::Show => "show".to_string(),
                TokenKind::Then => "then".to_string(),
                TokenKind::Else => "else".to_string(),
                TokenKind::And => "and".to_string(),
                TokenKind::To => "to".to_string(),
                TokenKind::From => "from".to_string(),
                TokenKind::Comment(_) => break, // Inline comment stops line parsing
                _ => "".to_string(),
            };
            if !text.is_empty() {
                parts.push(text);
            }
        }

        let full_text = parts.join(" ");
        (full_text, start_span.merge(end_span))
    }

    pub fn parse(&mut self) -> (FlowchartAst, Vec<ParseError>) {
        let mut title = None;
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.check(&TokenKind::Eof) {
            self.skip_newlines();
            if self.check(&TokenKind::Eof) {
                break;
            }

            match self.parse_stmt() {
                Ok(Some(stmt)) => {
                    if let AstStmt::AlgorithmTitle(t) = &stmt {
                        title = Some(t.value.clone());
                    }
                    statements.push(stmt);
                }
                Ok(None) => {}
                Err(err) => {
                    self.errors.push(err);
                    // Error Recovery: Consume line until next newline so parser doesn't cascade fail
                    let (raw_line, span) = self.consume_line_rest();
                    statements.push(AstStmt::ErrorRecovery {
                        raw_text: raw_line,
                        message: "Syntax error on this line".to_string(),
                        span,
                    });
                }
            }
            self.skip_newlines();
        }

        (FlowchartAst { title, statements }, self.errors.clone())
    }

    fn parse_stmt(&mut self) -> Result<Option<AstStmt>, ParseError> {
        let tok = self.current().clone();
        match tok.kind {
            TokenKind::Newline => {
                self.advance();
                Ok(None)
            }
            TokenKind::Comment(c) => {
                self.advance();
                Ok(Some(AstStmt::Note(Spanned::new(c, tok.span))))
            }
            TokenKind::Algorithm => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                let clean = raw.trim().trim_matches('"').trim_matches('\'').to_string();
                Ok(Some(AstStmt::AlgorithmTitle(Spanned::new(
                    clean,
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Start => {
                self.advance();
                self.consume_line_rest();
                Ok(Some(AstStmt::Start(tok.span)))
            }
            TokenKind::End => {
                // If this is a standalone end, return end node
                self.advance();
                self.consume_line_rest();
                Ok(Some(AstStmt::End(tok.span)))
            }
            TokenKind::Return => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                let ret_val = if raw.is_empty() { None } else { Some(raw) };
                Ok(Some(AstStmt::Return(Spanned::new(
                    ret_val,
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Set | TokenKind::Let | TokenKind::Do => {
                let prefix = match tok.kind {
                    TokenKind::Do => "do",
                    _ => "set",
                };
                self.advance();
                let (raw, span) = self.consume_line_rest();
                let text = if prefix == "do" {
                    raw.trim_matches('"').to_string()
                } else {
                    raw
                };
                Ok(Some(AstStmt::Process(Spanned::new(
                    text,
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Input | TokenKind::Read => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Input(Spanned::new(
                    raw,
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Output | TokenKind::Print | TokenKind::Show => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                let clean = raw.trim_matches('"').to_string();
                Ok(Some(AstStmt::Output(Spanned::new(
                    clean,
                    tok.span.merge(span),
                ))))
            }
            TokenKind::If => {
                self.advance();
                let mut cond_parts = Vec::new();
                let cond_start = self.current().span;
                let mut cond_end = cond_start;

                while !matches!(
                    self.current().kind,
                    TokenKind::Then | TokenKind::Newline | TokenKind::Eof
                ) {
                    let t = self.advance().clone();
                    cond_end = t.span;
                    match t.kind {
                        TokenKind::Identifier(s) | TokenKind::Number(s) => cond_parts.push(s),
                        TokenKind::StringLit(s) => cond_parts.push(format!("\"{}\"", s)),
                        _ => {}
                    }
                }

                if self.check(&TokenKind::Then) {
                    self.advance();
                }
                self.skip_newlines();

                let condition_text = cond_parts.join(" ");
                let condition_span = cond_start.merge(cond_end);

                let mut then_branch = Vec::new();
                let mut else_branch = None;

                while !self.check(&TokenKind::End)
                    && !self.check(&TokenKind::Else)
                    && !self.check(&TokenKind::Eof)
                {
                    self.skip_newlines();
                    if self.check(&TokenKind::End)
                        || self.check(&TokenKind::Else)
                        || self.check(&TokenKind::Eof)
                    {
                        break;
                    }
                    if let Some(s) = self.parse_stmt()? {
                        then_branch.push(s);
                    }
                    self.skip_newlines();
                }

                if self.match_token(&TokenKind::Else) {
                    self.skip_newlines();
                    let mut else_stmts = Vec::new();
                    while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                        self.skip_newlines();
                        if self.check(&TokenKind::End) || self.check(&TokenKind::Eof) {
                            break;
                        }
                        if let Some(s) = self.parse_stmt()? {
                            else_stmts.push(s);
                        }
                        self.skip_newlines();
                    }
                    else_branch = Some(else_stmts);
                }

                let end_span = if self.match_token(&TokenKind::End) {
                    self.current().span
                } else {
                    tok.span
                };

                Ok(Some(AstStmt::Decision {
                    condition: Spanned::new(condition_text, condition_span),
                    then_branch,
                    else_branch,
                    span: tok.span.merge(end_span),
                }))
            }
            TokenKind::Switch => {
                self.advance();
                let (target_text, target_span) = self.consume_line_rest();
                self.skip_newlines();

                let mut cases = Vec::new();
                let mut default_case = None;

                while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                    if self.check(&TokenKind::End) || self.check(&TokenKind::Eof) {
                        break;
                    }

                    if self.match_token(&TokenKind::Case) {
                        let (case_val, case_span) = self.consume_line_rest();
                        self.skip_newlines();
                        let mut body = Vec::new();
                        while !self.check(&TokenKind::Case)
                            && !self.check(&TokenKind::Default)
                            && !self.check(&TokenKind::End)
                            && !self.check(&TokenKind::Eof)
                        {
                            self.skip_newlines();
                            if self.check(&TokenKind::Case)
                                || self.check(&TokenKind::Default)
                                || self.check(&TokenKind::End)
                                || self.check(&TokenKind::Eof)
                            {
                                break;
                            }
                            if let Some(s) = self.parse_stmt()? {
                                body.push(s);
                            }
                            self.skip_newlines();
                        }
                        cases.push(AstCase {
                            value: Spanned::new(case_val, case_span),
                            body,
                            span: case_span,
                        });
                    } else if self.match_token(&TokenKind::Default) {
                        self.consume_line_rest();
                        self.skip_newlines();
                        let mut body = Vec::new();
                        while !self.check(&TokenKind::Case)
                            && !self.check(&TokenKind::End)
                            && !self.check(&TokenKind::Eof)
                        {
                            self.skip_newlines();
                            if self.check(&TokenKind::Case)
                                || self.check(&TokenKind::End)
                                || self.check(&TokenKind::Eof)
                            {
                                break;
                            }
                            if let Some(s) = self.parse_stmt()? {
                                body.push(s);
                            }
                            self.skip_newlines();
                        }
                        default_case = Some(body);
                    } else {
                        // Unexpected token inside switch
                        break;
                    }
                }

                let end_span = if self.match_token(&TokenKind::End) {
                    self.current().span
                } else {
                    tok.span
                };

                Ok(Some(AstStmt::Switch {
                    target: Spanned::new(target_text, target_span),
                    cases,
                    default_case,
                    span: tok.span.merge(end_span),
                }))
            }
            TokenKind::For => {
                self.advance();
                let var_tok = self.advance().clone();
                let var_name = match var_tok.kind {
                    TokenKind::Identifier(s) => s,
                    other => format!("{:?}", other),
                };

                if self.check(&TokenKind::From) {
                    self.advance();
                }

                let mut from_parts = Vec::new();
                let from_start = self.current().span;
                let mut from_end = from_start;
                while !self.check(&TokenKind::To)
                    && !self.check(&TokenKind::Newline)
                    && !self.check(&TokenKind::Eof)
                {
                    let t = self.advance().clone();
                    from_end = t.span;
                    match t.kind {
                        TokenKind::Identifier(s) | TokenKind::Number(s) => from_parts.push(s),
                        _ => {}
                    }
                }

                if self.check(&TokenKind::To) {
                    self.advance();
                }

                let mut to_parts = Vec::new();
                let to_start = self.current().span;
                let mut to_end = to_start;
                while !self.check(&TokenKind::Newline) && !self.check(&TokenKind::Eof) {
                    let t = self.advance().clone();
                    to_end = t.span;
                    match t.kind {
                        TokenKind::Identifier(s) | TokenKind::Number(s) => to_parts.push(s),
                        _ => {}
                    }
                }
                self.skip_newlines();

                let mut body = Vec::new();
                while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                    if self.check(&TokenKind::End) || self.check(&TokenKind::Eof) {
                        break;
                    }
                    if let Some(s) = self.parse_stmt()? {
                        body.push(s);
                    }
                    self.skip_newlines();
                }

                let end_span = if self.match_token(&TokenKind::End) {
                    self.current().span
                } else {
                    tok.span
                };

                Ok(Some(AstStmt::LoopFor {
                    var: Spanned::new(var_name, var_tok.span),
                    from: Spanned::new(from_parts.join(" "), from_start.merge(from_end)),
                    to: Spanned::new(to_parts.join(" "), to_start.merge(to_end)),
                    body,
                    span: tok.span.merge(end_span),
                }))
            }
            TokenKind::While => {
                self.advance();
                let (cond, cond_span) = self.consume_line_rest();
                self.skip_newlines();

                let mut body = Vec::new();
                while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                    if self.check(&TokenKind::End) || self.check(&TokenKind::Eof) {
                        break;
                    }
                    if let Some(s) = self.parse_stmt()? {
                        body.push(s);
                    }
                    self.skip_newlines();
                }

                let end_span = if self.match_token(&TokenKind::End) {
                    self.current().span
                } else {
                    tok.span
                };

                Ok(Some(AstStmt::LoopWhile {
                    condition: Spanned::new(cond, cond_span),
                    body,
                    span: tok.span.merge(end_span),
                }))
            }
            TokenKind::Repeat => {
                self.advance();
                self.skip_newlines();

                let mut body = Vec::new();
                while !self.check(&TokenKind::Until) && !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                    if self.check(&TokenKind::Until) || self.check(&TokenKind::Eof) {
                        break;
                    }
                    if let Some(s) = self.parse_stmt()? {
                        body.push(s);
                    }
                    self.skip_newlines();
                }

                let mut cond_text = String::new();
                let mut cond_span = tok.span;
                if self.match_token(&TokenKind::Until) {
                    let (c, sp) = self.consume_line_rest();
                    cond_text = c;
                    cond_span = sp;
                }

                Ok(Some(AstStmt::LoopRepeat {
                    body,
                    until_cond: Spanned::new(cond_text, cond_span),
                    span: tok.span.merge(cond_span),
                }))
            }
            TokenKind::Call => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::SubroutineCall {
                    name: Spanned::new(raw, span),
                    args: None,
                    span: tok.span.merge(span),
                }))
            }
            TokenKind::Goto => {
                self.advance();
                let (label_name, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Goto(Spanned::new(
                    label_name.trim().to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Label => {
                self.advance();
                let (label_name, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Label(Spanned::new(
                    label_name.trim().to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Link => {
                self.advance();
                let (target, span) = self.consume_line_rest();
                Ok(Some(AstStmt::OffpageLink(Spanned::new(
                    target.trim_matches('"').to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Document => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Document(Spanned::new(
                    raw.trim_matches('"').to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Database => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Database(Spanned::new(
                    raw.trim_matches('"').to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::ManualInput => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::ManualInput(Spanned::new(
                    raw.trim_matches('"').to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Wait => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Delay(Spanned::new(
                    raw.trim_matches('"').to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Display => {
                self.advance();
                let (raw, span) = self.consume_line_rest();
                Ok(Some(AstStmt::Display(Spanned::new(
                    raw.trim_matches('"').to_string(),
                    tok.span.merge(span),
                ))))
            }
            TokenKind::Parallel => {
                self.advance();
                self.skip_newlines();
                let mut branches = Vec::new();
                let mut cur_branch = Vec::new();

                while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                    if self.check(&TokenKind::End) || self.check(&TokenKind::Eof) {
                        break;
                    }
                    if self.match_token(&TokenKind::And) {
                        branches.push(cur_branch);
                        cur_branch = Vec::new();
                        self.skip_newlines();
                        continue;
                    }
                    if let Some(s) = self.parse_stmt()? {
                        cur_branch.push(s);
                    }
                    self.skip_newlines();
                }
                branches.push(cur_branch);

                let end_span = if self.match_token(&TokenKind::End) {
                    self.current().span
                } else {
                    tok.span
                };

                Ok(Some(AstStmt::Parallel {
                    branches,
                    span: tok.span.merge(end_span),
                }))
            }
            TokenKind::Lane => {
                self.advance();
                let (name, span) = self.consume_line_rest();
                self.skip_newlines();
                let mut body = Vec::new();
                while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                    self.skip_newlines();
                    if self.check(&TokenKind::End) || self.check(&TokenKind::Eof) {
                        break;
                    }
                    if let Some(s) = self.parse_stmt()? {
                        body.push(s);
                    }
                    self.skip_newlines();
                }
                let end_span = if self.match_token(&TokenKind::End) {
                    self.current().span
                } else {
                    tok.span
                };

                Ok(Some(AstStmt::Swimlane {
                    name: Spanned::new(name.trim_matches('"').to_string(), span),
                    body,
                    span: tok.span.merge(end_span),
                }))
            }
            TokenKind::Node => {
                // Raw node declaration: node <id>: <kind> "<label>"
                self.advance();
                let id_tok = self.advance().clone();
                let id_str = match id_tok.kind {
                    TokenKind::Identifier(s) => s,
                    _ => "node".to_string(),
                };
                if self.check(&TokenKind::Colon) {
                    self.advance();
                }
                let (rest, rest_span) = self.consume_line_rest();
                let parts: Vec<&str> = rest.split_whitespace().collect();
                let kind_str = parts.first().copied().unwrap_or("process");
                let label = if parts.len() > 1 {
                    parts[1..].join(" ").trim_matches('"').to_string()
                } else {
                    kind_str.to_string()
                };

                let kind = match kind_str.to_lowercase().as_str() {
                    "start" => BlockKind::Start,
                    "end" => BlockKind::End,
                    "decision" => BlockKind::Decision,
                    "input" | "read" => BlockKind::Input,
                    "output" | "print" => BlockKind::Output,
                    "document" => BlockKind::Document,
                    "database" => BlockKind::Database,
                    "call" => BlockKind::Subroutine,
                    _ => BlockKind::Process,
                };

                Ok(Some(AstStmt::RawNodeDef {
                    id: Spanned::new(id_str, id_tok.span),
                    kind,
                    label: Spanned::new(label, rest_span),
                    span: tok.span.merge(rest_span),
                }))
            }
            TokenKind::Identifier(ref id) => {
                let from_id = id.clone();
                let from_span = tok.span;
                self.advance();

                // Check for Raw Edge: a -> b or a -[Yes]-> b
                if self.match_token(&TokenKind::Arrow) {
                    let to_tok = self.advance().clone();
                    let to_id = match to_tok.kind {
                        TokenKind::Identifier(s) => s,
                        _ => "target".to_string(),
                    };
                    return Ok(Some(AstStmt::RawEdge {
                        from: Spanned::new(from_id, from_span),
                        from_port: None,
                        to: Spanned::new(to_id, to_tok.span),
                        to_port: None,
                        label: None,
                        span: from_span.merge(to_tok.span),
                    }));
                }

                // Check for labeled edge: a -[Yes]-> b
                let is_bracket = self.match_token(&TokenKind::LBracket);
                if is_bracket {
                    let mut label_parts = Vec::new();
                    while !self.check(&TokenKind::RBracket)
                        && !self.check(&TokenKind::Newline)
                        && !self.check(&TokenKind::Eof)
                    {
                        let tok = self.advance();
                        match tok.kind {
                            TokenKind::Identifier(s) | TokenKind::StringLit(s) => {
                                label_parts.push(s)
                            }
                            _ => {}
                        }
                    }
                    if self.check(&TokenKind::RBracket) {
                        self.advance();
                    }
                    if self.match_token(&TokenKind::Arrow) {
                        let to_tok = self.advance().clone();
                        let to_id = match to_tok.kind {
                            TokenKind::Identifier(s) => s,
                            _ => "target".to_string(),
                        };
                        return Ok(Some(AstStmt::RawEdge {
                            from: Spanned::new(from_id, from_span),
                            from_port: None,
                            to: Spanned::new(to_id, to_tok.span),
                            to_port: None,
                            label: Some(Spanned::new(label_parts.join(" "), to_tok.span)),
                            span: from_span.merge(to_tok.span),
                        }));
                    }
                }

                // Fallback: treat unrecognized identifier line as a Process block
                let (rest, rest_span) = self.consume_line_rest();
                let full_line = if rest.is_empty() {
                    from_id
                } else {
                    format!("{} {}", from_id, rest)
                };
                Ok(Some(AstStmt::Process(Spanned::new(
                    full_line,
                    from_span.merge(rest_span),
                ))))
            }
            _ => {
                // Unknown statement: consume and record error recovery
                let (line, span) = self.consume_line_rest();
                Ok(Some(AstStmt::ErrorRecovery {
                    raw_text: line,
                    message: "Unrecognized statement".to_string(),
                    span,
                }))
            }
        }
    }
}
