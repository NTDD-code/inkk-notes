//! Recursive Descent Parser for Inkk MathEngine
use crate::math_engine::ast::*;
use crate::math_engine::error::MathError;
use crate::math_engine::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    comments: std::collections::HashMap<usize, String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            comments: std::collections::HashMap::new(),
        }
    }

    pub fn new_with_comments(
        tokens: Vec<Token>,
        comments: std::collections::HashMap<usize, String>,
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            comments,
        }
    }

    fn current(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            self.tokens.last().unwrap()
        }
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn advance(&mut self) -> &Token {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            tok
        } else {
            self.tokens.last().unwrap()
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn current_line(&self) -> usize {
        self.current().line
    }

    pub fn parse_next_stmt(&mut self) -> Result<Option<Stmt>, MathError> {
        while !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                continue;
            }
            if self.check(&TokenKind::Eof) {
                break;
            }
            let stmt = self.parse_stmt()?;
            return Ok(Some(stmt));
        }
        Ok(None)
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, MathError> {
        let tok = self.current();
        let tok_str = match &tok.kind {
            TokenKind::Number(n) => format!("{}", n),
            TokenKind::Str(s) => format!("\"{}\"", s),
            TokenKind::Ident(s) => s.clone(),
            TokenKind::Plus => "+".into(),
            TokenKind::Minus => "-".into(),
            TokenKind::Star => "*".into(),
            TokenKind::Slash => "/".into(),
            TokenKind::Backslash => "\\".into(),
            TokenKind::Caret => "^".into(),
            TokenKind::DotStar => ".*".into(),
            TokenKind::DotSlash => "./".into(),
            TokenKind::DotCaret => ".^".into(),
            TokenKind::Transpose => "'".into(),
            TokenKind::DotTranspose => ".'".into(),
            TokenKind::Colon => ":".into(),
            TokenKind::Equal => "=".into(),
            TokenKind::EqualEqual => "==".into(),
            TokenKind::NotEqual => "~=".into(),
            TokenKind::Less => "<".into(),
            TokenKind::LessEqual => "<=".into(),
            TokenKind::Greater => ">".into(),
            TokenKind::GreaterEqual => ">=".into(),
            TokenKind::And => "&".into(),
            TokenKind::Or => "|".into(),
            TokenKind::Tilde => "~".into(),
            TokenKind::At => "@".into(),
            TokenKind::For => "for".into(),
            TokenKind::While => "while".into(),
            TokenKind::If => "if".into(),
            TokenKind::ElseIf => "elseif".into(),
            TokenKind::Else => "else".into(),
            TokenKind::End => "end".into(),
            TokenKind::Break => "break".into(),
            TokenKind::Continue => "continue".into(),
            TokenKind::Function => "function".into(),
            TokenKind::Return => "return".into(),
            TokenKind::LParen => "(".into(),
            TokenKind::RParen => ")".into(),
            TokenKind::LBracket => "[".into(),
            TokenKind::RBracket => "]".into(),
            TokenKind::Semicolon => ";".into(),
            TokenKind::Comma => ",".into(),
            TokenKind::Newline => "\\n".into(),
            TokenKind::SpaceSep => " ".into(),
            TokenKind::Eof => "EOF".into(),
        };

        let msg = message.into();
        let display = if msg.is_empty() { tok_str } else { msg };

        Err(MathError::ParseError {
            line: tok.line,
            col: tok.col,
            message: display,
        })
    }

    pub fn parse_script(&mut self) -> Result<Script, MathError> {
        let mut stmts = Vec::new();

        while !self.check(&TokenKind::Eof) {
            // Skip leading whitespace / newlines / semicolons between statements
            if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                continue;
            }

            if self.check(&TokenKind::Eof) {
                break;
            }

            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        Ok(Script { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, MathError> {
        let line = self.current_line();

        // 0. Control flow & Function statements (Tier 2 & 3)
        match self.peek_kind() {
            TokenKind::For => return self.parse_for_stmt(line),
            TokenKind::While => return self.parse_while_stmt(line),
            TokenKind::If => return self.parse_if_stmt(line),
            TokenKind::Function => return self.parse_function_def(line),
            TokenKind::Return => {
                self.advance();
                self.parse_stmt_terminator();
                return Ok(Stmt { line, kind: StmtKind::Return });
            }
            TokenKind::Break => {
                self.advance();
                self.parse_stmt_terminator();
                return Ok(Stmt { line, kind: StmtKind::Break });
            }
            TokenKind::Continue => {
                self.advance();
                self.parse_stmt_terminator();
                return Ok(Stmt { line, kind: StmtKind::Continue });
            }
            _ => {}
        }

        // 1. Check for command syntax (hold, grid, clear, clc, close, figure, who, whos)
        if let TokenKind::Ident(name) = self.peek_kind() {
            let cmd_name = name.clone();
            if is_command_name(&cmd_name) {
                if let Some(cmd) = self.try_parse_command(&cmd_name)? {
                    return Ok(Stmt { line, kind: StmtKind::Command(cmd) });
                }
            }
        }

        // 2. Check for unsupported features (Spec Section 8)
        if let TokenKind::Ident(name) = self.peek_kind() {
            if is_unsupported_feature(name) {
                let feat_name = name.clone();
                let feat_line = self.current().line;
                self.advance();
                return Err(MathError::FeatureNotSupported {
                    feature: feat_name,
                    line: feat_line,
                });
            }
        }

        // 3. Check for Multi-Assignment: [r, c] = expr
        if let Some(stmt) = self.try_parse_multi_assignment(line)? {
            return Ok(stmt);
        }

        // 4. Check for Assignment: var = expr OR var(args) = expr
        if let Some(stmt) = self.try_parse_assignment(line)? {
            return Ok(stmt);
        }

        // 5. Standalone Expression
        let expr = self.parse_expr()?;
        let print = self.parse_stmt_terminator();

        Ok(Stmt {
            line,
            kind: StmtKind::Expr { expr, print },
        })
    }

    fn try_parse_command(&mut self, name: &str) -> Result<Option<CommandStmt>, MathError> {
        // Peek ahead to see if it's followed by '(' or '='
        if self.pos + 1 < self.tokens.len() {
            let next_tok = &self.tokens[self.pos + 1].kind;
            if *next_tok == TokenKind::LParen || *next_tok == TokenKind::Equal {
                return Ok(None);
            }
        }

        self.advance(); // consume command name
        let cmd = match name {
            "hold" => {
                let mut on = true;
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    if arg == "on" {
                        on = true;
                        self.advance();
                    } else if arg == "off" {
                        on = false;
                        self.advance();
                    }
                }
                self.parse_stmt_terminator();
                CommandStmt::Hold(on)
            }
            "grid" => {
                let mut on = true;
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    if arg == "on" {
                        on = true;
                        self.advance();
                    } else if arg == "off" {
                        on = false;
                        self.advance();
                    }
                }
                self.parse_stmt_terminator();
                CommandStmt::Grid(on)
            }
            "clear" => {
                let mut vars = Vec::new();
                while let TokenKind::Ident(v) = self.peek_kind() {
                    vars.push(v.clone());
                    self.advance();
                }
                self.parse_stmt_terminator();
                CommandStmt::Clear(vars)
            }
            "clc" => {
                self.parse_stmt_terminator();
                CommandStmt::Clc
            }
            "close" => {
                let mut target = None;
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    target = Some(arg.clone());
                    self.advance();
                }
                self.parse_stmt_terminator();
                CommandStmt::Close(target)
            }
            "figure" => {
                self.parse_stmt_terminator();
                CommandStmt::Figure
            }
            "who" => {
                self.parse_stmt_terminator();
                CommandStmt::Who
            }
            "whos" => {
                self.parse_stmt_terminator();
                CommandStmt::Whos
            }
            "format" => {
                let mut mode = FormatMode::Short;
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    match arg.as_str() {
                        "short" => {
                            mode = FormatMode::Short;
                            self.advance();
                        }
                        "long" => {
                            mode = FormatMode::Long;
                            self.advance();
                        }
                        "rat" => {
                            mode = FormatMode::Rat;
                            self.advance();
                        }
                        _ => {}
                    }
                }
                self.parse_stmt_terminator();
                CommandStmt::Format(mode)
            }
            "axis" => {
                let mut opt = AxisOption::Equal;
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    if arg == "equal" {
                        opt = AxisOption::Equal;
                        self.advance();
                    }
                }
                self.parse_stmt_terminator();
                CommandStmt::Axis(opt)
            }
            "help" => {
                let mut target = None;
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    target = Some(arg.clone());
                    self.advance();
                }
                self.parse_stmt_terminator();
                CommandStmt::Help(target)
            }
            "which" => {
                let mut target = String::new();
                if let TokenKind::Ident(arg) = self.peek_kind() {
                    target = arg.clone();
                    self.advance();
                }
                self.parse_stmt_terminator();
                CommandStmt::Which(target)
            }
            _ => return Ok(None),
        };

        Ok(Some(cmd))
    }

    fn try_parse_assignment(&mut self, line: usize) -> Result<Option<Stmt>, MathError> {
        let saved_pos = self.pos;

        if let TokenKind::Ident(name) = self.peek_kind() {
            let var_name = name.clone();
            self.advance();

            // Case 1: Simple assignment: var = expr
            if self.match_token(&TokenKind::Equal) {
                let expr = self.parse_expr()?;
                let print = self.parse_stmt_terminator();
                return Ok(Some(Stmt {
                    line,
                    kind: StmtKind::Assign {
                        target: AssignTarget::Var(var_name),
                        expr,
                        print,
                    },
                }));
            }

            // Case 2: Indexed assignment: var(args) = expr
            if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                if self.match_token(&TokenKind::RParen) && self.match_token(&TokenKind::Equal) {
                    let expr = self.parse_expr()?;
                    let print = self.parse_stmt_terminator();
                    return Ok(Some(Stmt {
                        line,
                        kind: StmtKind::Assign {
                            target: AssignTarget::Index {
                                var: var_name,
                                args,
                            },
                            expr,
                            print,
                        },
                    }));
                }
            }
        }

        // Backtrack if not assignment
        self.pos = saved_pos;
        Ok(None)
    }

    fn try_parse_multi_assignment(&mut self, line: usize) -> Result<Option<Stmt>, MathError> {
        let saved_pos = self.pos;
        if !self.match_token(&TokenKind::LBracket) {
            return Ok(None);
        }
        let mut targets = Vec::new();
        while let TokenKind::Ident(name) = self.peek_kind() {
            targets.push(name.clone());
            self.advance();
            if self.match_token(&TokenKind::Comma) || self.match_token(&TokenKind::SpaceSep) {
                continue;
            }
            break;
        }
        if !targets.is_empty() && self.match_token(&TokenKind::RBracket) && self.match_token(&TokenKind::Equal) {
            let expr = self.parse_expr()?;
            let print = self.parse_stmt_terminator();
            return Ok(Some(Stmt {
                line,
                kind: StmtKind::MultiAssign {
                    targets,
                    expr,
                    print,
                },
            }));
        }
        self.pos = saved_pos;
        Ok(None)
    }

    fn parse_for_stmt(&mut self, line: usize) -> Result<Stmt, MathError> {
        self.advance(); // consume 'for'
        let var_name = if let TokenKind::Ident(name) = self.peek_kind() {
            let n = name.clone();
            self.advance();
            n
        } else {
            return self.error("Expected identifier after 'for'");
        };
        if !self.match_token(&TokenKind::Equal) {
            return self.error("Expected '=' in for loop");
        }
        let expr = self.parse_expr()?;
        self.parse_stmt_terminator();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                continue;
            }
            if self.check(&TokenKind::End) {
                break;
            }
            body.push(self.parse_stmt()?);
        }
        if !self.match_token(&TokenKind::End) {
            return self.error("Expected 'end' to close for loop");
        }
        self.parse_stmt_terminator();
        Ok(Stmt {
            line,
            kind: StmtKind::For {
                var: var_name,
                expr,
                body,
            },
        })
    }

    fn parse_while_stmt(&mut self, line: usize) -> Result<Stmt, MathError> {
        self.advance(); // consume 'while'
        let cond = self.parse_expr()?;
        self.parse_stmt_terminator();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                continue;
            }
            if self.check(&TokenKind::End) {
                break;
            }
            body.push(self.parse_stmt()?);
        }
        if !self.match_token(&TokenKind::End) {
            return self.error("Expected 'end' to close while loop");
        }
        self.parse_stmt_terminator();
        Ok(Stmt {
            line,
            kind: StmtKind::While { cond, body },
        })
    }

    fn parse_if_stmt(&mut self, line: usize) -> Result<Stmt, MathError> {
        self.advance(); // consume 'if'
        let cond = self.parse_expr()?;
        self.parse_stmt_terminator();

        let mut branches = Vec::new();
        let mut cur_body = Vec::new();

        while !matches!(self.peek_kind(), TokenKind::ElseIf | TokenKind::Else | TokenKind::End | TokenKind::Eof) {
            if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                continue;
            }
            if matches!(self.peek_kind(), TokenKind::ElseIf | TokenKind::Else | TokenKind::End) {
                break;
            }
            cur_body.push(self.parse_stmt()?);
        }
        branches.push((cond, cur_body));

        while self.match_token(&TokenKind::ElseIf) {
            let elif_cond = self.parse_expr()?;
            self.parse_stmt_terminator();
            let mut elif_body = Vec::new();
            while !matches!(self.peek_kind(), TokenKind::ElseIf | TokenKind::Else | TokenKind::End | TokenKind::Eof) {
                if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                    continue;
                }
                if matches!(self.peek_kind(), TokenKind::ElseIf | TokenKind::Else | TokenKind::End) {
                    break;
                }
                elif_body.push(self.parse_stmt()?);
            }
            branches.push((elif_cond, elif_body));
        }

        let mut else_branch = None;
        if self.match_token(&TokenKind::Else) {
            self.parse_stmt_terminator();
            let mut el_body = Vec::new();
            while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
                if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                    continue;
                }
                if self.check(&TokenKind::End) {
                    break;
                }
                el_body.push(self.parse_stmt()?);
            }
            else_branch = Some(el_body);
        }

        if !self.match_token(&TokenKind::End) {
            return self.error("Expected 'end' to close if statement");
        }
        self.parse_stmt_terminator();

        Ok(Stmt {
            line,
            kind: StmtKind::If {
                branches,
                else_branch,
            },
        })
    }

    fn parse_function_def(&mut self, line: usize) -> Result<Stmt, MathError> {
        self.advance(); // consume 'function'

        let mut returns = Vec::new();
        let name: String;

        if self.match_token(&TokenKind::LBracket) {
            while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::Eof) {
                if let TokenKind::Ident(id) = self.peek_kind() {
                    returns.push(id.clone());
                    self.advance();
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SpaceSep);
                } else {
                    return self.error("Expected return variable name inside '['");
                }
            }
            if !self.match_token(&TokenKind::RBracket) {
                return self.error("Expected ']' closing return variable list");
            }
            if !self.match_token(&TokenKind::Equal) {
                return self.error("Expected '=' after return variable list");
            }
            if let TokenKind::Ident(fn_name) = self.peek_kind() {
                name = fn_name.clone();
                self.advance();
            } else {
                return self.error("Expected function name after '='");
            }
        } else if let TokenKind::Ident(first_id) = self.peek_kind().clone() {
            if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1].kind == TokenKind::Equal {
                self.advance(); // consume first_id
                returns.push(first_id);
                self.advance(); // consume '='
                if let TokenKind::Ident(fn_name) = self.peek_kind() {
                    name = fn_name.clone();
                    self.advance();
                } else {
                    return self.error("Expected function name after '='");
                }
            } else {
                name = first_id;
                self.advance();
            }
        } else {
            return self.error("Expected function name or return variable list");
        }

        let mut params = Vec::new();
        if self.match_token(&TokenKind::LParen) {
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::Eof) {
                if let TokenKind::Ident(p) = self.peek_kind() {
                    params.push(p.clone());
                    self.advance();
                    self.match_token(&TokenKind::Comma);
                } else {
                    return self.error("Expected parameter name in function definition");
                }
            }
            if !self.match_token(&TokenKind::RParen) {
                return self.error("Expected ')' after parameter list");
            }
        }

        self.parse_stmt_terminator();

        let mut help_lines = Vec::new();
        let mut check_line = line + 1;
        while let Some(cmt) = self.comments.get(&check_line) {
            help_lines.push(cmt.clone());
            check_line += 1;
        }
        let help = help_lines.join("\n");

        let mut body = Vec::new();
        let mut local_functions = std::collections::HashMap::new();

        while !self.check(&TokenKind::End) && !self.check(&TokenKind::Eof) {
            if self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {
                continue;
            }
            if self.check(&TokenKind::End) {
                break;
            }
            let stmt = self.parse_stmt()?;
            if let StmtKind::FunctionDef { ast: local_ast, .. } = &stmt.kind {
                local_functions.insert(local_ast.name.clone(), local_ast.clone());
            } else {
                body.push(stmt);
            }
        }

        if !self.match_token(&TokenKind::End) {
            return self.error("Expected 'end' at end of function definition");
        }
        self.parse_stmt_terminator();

        let max_args = params.len();
        let num_outputs = returns.len();
        let metadata = FunctionMetadata {
            name: name.clone(),
            min_args: 0,
            max_args,
            num_outputs,
            help,
            examples: Vec::new(),
        };

        let ast = UserFunctionAst {
            name,
            params,
            returns,
            body,
            local_functions,
            library: None,
        };

        Ok(Stmt {
            line,
            kind: StmtKind::FunctionDef { ast, metadata },
        })
    }

    fn parse_stmt_terminator(&mut self) -> bool {
        let mut print = true;
        while self.check(&TokenKind::Semicolon) || self.check(&TokenKind::Comma) || self.check(&TokenKind::Newline) {
            if self.match_token(&TokenKind::Semicolon) {
                print = false;
            } else {
                self.advance();
            }
        }
        print
    }

    // --- Expression Parsing (Precedence Climbing) ---

    pub fn parse_expr(&mut self) -> Result<Expr, MathError> {
        self.parse_logical_or()
    }

    // Level 0a: Logical OR (|)
    fn parse_logical_or(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let mut left = self.parse_logical_and()?;

        while self.match_token(&TokenKind::Or) {
            let right = self.parse_logical_and()?;
            left = Expr {
                line,
                kind: ExprKind::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    // Level 0b: Logical AND (&)
    fn parse_logical_and(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let mut left = self.parse_comparison()?;

        while self.match_token(&TokenKind::And) {
            let right = self.parse_comparison()?;
            left = Expr {
                line,
                kind: ExprKind::Binary {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    // Level 0c: Comparison operators (==, ~=, <, <=, >, >=)
    fn parse_comparison(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let mut left = self.parse_range()?;

        while matches!(
            self.peek_kind(),
            TokenKind::EqualEqual
                | TokenKind::NotEqual
                | TokenKind::Less
                | TokenKind::LessEqual
                | TokenKind::Greater
                | TokenKind::GreaterEqual
        ) {
            let op = match self.peek_kind() {
                TokenKind::EqualEqual => BinaryOp::Eq,
                TokenKind::NotEqual => BinaryOp::Ne,
                TokenKind::Less => BinaryOp::Lt,
                TokenKind::LessEqual => BinaryOp::Le,
                TokenKind::Greater => BinaryOp::Gt,
                TokenKind::GreaterEqual => BinaryOp::Ge,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_range()?;
            left = Expr {
                line,
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    // Level 1: Range operator (:)
    fn parse_range(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();

        // Standalone ':' (used in indexing like A(:, 2))
        if self.match_token(&TokenKind::Colon) {
            return Ok(Expr {
                line,
                kind: ExprKind::ColonAll,
            });
        }

        let start = self.parse_additive()?;

        if self.match_token(&TokenKind::Colon) {
            let second = self.parse_additive()?;
            if self.match_token(&TokenKind::Colon) {
                let third = self.parse_additive()?;
                Ok(Expr {
                    line,
                    kind: ExprKind::Range {
                        start: Box::new(start),
                        step: Some(Box::new(second)),
                        end: Box::new(third),
                    },
                })
            } else {
                Ok(Expr {
                    line,
                    kind: ExprKind::Range {
                        start: Box::new(start),
                        step: None,
                        end: Box::new(second),
                    },
                })
            }
        } else {
            Ok(start)
        }
    }

    // Level 2: Additive (+, -)
    fn parse_additive(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let mut left = self.parse_multiplicative()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op = if self.match_token(&TokenKind::Plus) {
                BinaryOp::Add
            } else {
                self.advance();
                BinaryOp::Sub
            };
            let right = self.parse_multiplicative()?;
            left = Expr {
                line,
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    // Level 3: Multiplicative (*, /, \, .*, ./)
    fn parse_multiplicative(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let mut left = self.parse_unary()?;

        while matches!(
            self.peek_kind(),
            TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Backslash
                | TokenKind::DotStar
                | TokenKind::DotSlash
        ) {
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Backslash => BinaryOp::LeftDiv,
                TokenKind::DotStar => BinaryOp::DotMul,
                TokenKind::DotSlash => BinaryOp::DotDiv,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr {
                line,
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            };
        }

        Ok(left)
    }

    // Level 4: Unary (+, -, ~)
    // Note: in MATLAB -2^2 is -(2^2) = -4, so unary - has LOWER precedence than power!
    fn parse_unary(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();

        if self.match_token(&TokenKind::Plus) {
            let expr = self.parse_unary()?;
            return Ok(Expr {
                line,
                kind: ExprKind::Unary {
                    op: UnaryOp::Pos,
                    expr: Box::new(expr),
                },
            });
        }

        if self.match_token(&TokenKind::Minus) {
            let expr = self.parse_power()?;
            return Ok(Expr {
                line,
                kind: ExprKind::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                },
            });
        }

        if self.match_token(&TokenKind::Tilde) {
            let expr = self.parse_power()?;
            return Ok(Expr {
                line,
                kind: ExprKind::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                },
            });
        }

        self.parse_power()
    }

    // Level 5: Power (^, .^) - right-associative
    fn parse_power(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let left = self.parse_postfix()?;

        if self.check(&TokenKind::Caret) || self.check(&TokenKind::DotCaret) {
            let op = if self.match_token(&TokenKind::Caret) {
                BinaryOp::Pow
            } else {
                self.advance();
                BinaryOp::DotPow
            };
            let right = self.parse_unary()?; // Allows 2^-1
            return Ok(Expr {
                line,
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            });
        }

        Ok(left)
    }

    // Level 6: Postfix Transpose (')
    fn parse_postfix(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();
        let mut expr = self.parse_primary()?;

        while self.match_token(&TokenKind::Transpose) || self.match_token(&TokenKind::DotTranspose) {
            expr = Expr {
                line,
                kind: ExprKind::Unary {
                    op: UnaryOp::Transpose,
                    expr: Box::new(expr),
                },
            };
        }

        Ok(expr)
    }

    // Level 7: Primary (Number, String, Ident, Parentheses, Matrix Literal)
    fn parse_primary(&mut self) -> Result<Expr, MathError> {
        let line = self.current_line();

        // 1. Parentheses
        if self.match_token(&TokenKind::LParen) {
            let expr = self.parse_expr()?;
            if !self.match_token(&TokenKind::RParen) {
                return self.error("Missing closing parenthesis ')'");
            }
            return Ok(expr);
        }

        // 2. Matrix Literal [ ... ]
        if self.match_token(&TokenKind::LBracket) {
            return self.parse_matrix_literal(line);
        }

        // 3. Number
        if let TokenKind::Number(n) = *self.peek_kind() {
            self.advance();
            return Ok(Expr {
                line,
                kind: ExprKind::Number(n),
            });
        }

        // 4. String literal
        if let TokenKind::Str(s) = self.peek_kind().clone() {
            self.advance();
            return Ok(Expr {
                line,
                kind: ExprKind::Str(s),
            });
        }

        // 5. 'end' keyword (in indexing)
        if self.match_token(&TokenKind::End) {
            return Ok(Expr {
                line,
                kind: ExprKind::End,
            });
        }

        // 5b. Anonymous function: @(x, y) expr
        if self.match_token(&TokenKind::At) {
            if !self.match_token(&TokenKind::LParen) {
                return self.error("Expected '(' after '@' for anonymous function");
            }
            let mut params = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    if let TokenKind::Ident(param) = self.peek_kind() {
                        params.push(param.clone());
                        self.advance();
                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        }
                    }
                    break;
                }
            }
            if !self.match_token(&TokenKind::RParen) {
                return self.error("Expected ')' after anonymous function parameter list");
            }
            let body = self.parse_expr()?;
            return Ok(Expr {
                line,
                kind: ExprKind::AnonymousFn {
                    params,
                    body: Box::new(body),
                },
            });
        }

        // 6. Identifier (variable or function call)
        if let TokenKind::Ident(name) = self.peek_kind().clone() {
            if is_unsupported_feature(&name) {
                let feat_line = self.current().line;
                self.advance();
                return Err(MathError::FeatureNotSupported {
                    feature: name,
                    line: feat_line,
                });
            }
            self.advance();

            // Check if it's a function call or indexing: name(args)
            if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                if !self.match_token(&TokenKind::RParen) {
                    return self.error(format!("Missing closing ')' for '{}'", name));
                }
                return Ok(Expr {
                    line,
                    kind: ExprKind::CallOrIndex { name, args },
                });
            }

            return Ok(Expr {
                line,
                kind: ExprKind::Ident(name),
            });
        }

        self.error("")
    }

    fn parse_matrix_literal(&mut self, line: usize) -> Result<Expr, MathError> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut cur_row: Vec<Expr> = Vec::new();

        // Skip leading whitespace / newlines / delimiters
        while self.match_token(&TokenKind::Newline) || self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Comma) {}

        if self.match_token(&TokenKind::RBracket) {
            return Ok(Expr {
                line,
                kind: ExprKind::Matrix(Vec::new()),
            });
        }

        loop {
            // Parse element expression
            let elem = self.parse_expr()?;
            cur_row.push(elem);

            // Skip space separators between row elements
            while self.match_token(&TokenKind::SpaceSep) {}

            // Check if comma follows
            if self.match_token(&TokenKind::Comma) {
                while self.match_token(&TokenKind::SpaceSep) {}
            }

            // Check row separators (; or \n)
            if self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Newline) {
                rows.push(std::mem::take(&mut cur_row));
                // Skip subsequent semicolons and newlines
                while self.match_token(&TokenKind::Semicolon) || self.match_token(&TokenKind::Newline) {}
            }

            if self.match_token(&TokenKind::RBracket) {
                if !cur_row.is_empty() {
                    rows.push(cur_row);
                }
                break;
            }

            if self.check(&TokenKind::Eof) {
                return self.error("Missing closing bracket ']' for matrix literal");
            }
        }

        Ok(Expr {
            line,
            kind: ExprKind::Matrix(rows),
        })
    }
}

fn is_command_name(name: &str) -> bool {
    matches!(
        name,
        "hold"
            | "grid"
            | "clear"
            | "clc"
            | "close"
            | "figure"
            | "who"
            | "whos"
            | "axis"
            | "format"
            | "help"
            | "which"
    )
}

fn is_unsupported_feature(name: &str) -> bool {
    matches!(
        name,
        "syms"
            | "plot3"
            | "surf"
            | "mesh"
            | "subplot"
            | "eval"
            | "system"
            | "fopen"
            | "load"
            | "save"
            | "readtable"
    )
}
