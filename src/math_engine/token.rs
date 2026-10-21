//! Lexer / Tokenizer for Inkk MathEngine
use crate::math_engine::error::MathError;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Str(String),
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Backslash,
    Caret,
    DotStar,
    DotSlash,
    DotCaret,
    Transpose,     // '
    DotTranspose,  // .'
    Colon,         // :
    Equal,         // =
    EqualEqual,    // ==
    NotEqual,      // ~=
    Less,          // <
    LessEqual,     // <=
    Greater,       // >
    GreaterEqual,  // >=
    And,           // &
    Or,            // |
    Tilde,         // ~
    At,            // @

    // Keywords
    For,
    While,
    If,
    ElseIf,
    Else,
    End,
    Break,
    Continue,
    Function,
    Return,

    // Delimiters
    LParen,        // (
    RParen,        // )
    LBracket,      // [
    RBracket,      // ]
    Semicolon,     // ;
    Comma,         // ,
    Newline,       // \n
    SpaceSep,      // whitespace acting as element separator inside [ ]

    // End of input
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer<'a> {
    chars: Vec<char>,
    len: usize,
    pos: usize,
    line: usize,
    col: usize,
    bracket_depth: usize,
    last_token_can_be_transposed: bool,
    pub comments: std::collections::HashMap<usize, String>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            len: input.chars().count(),
            pos: 0,
            line: 1,
            col: 1,
            bracket_depth: 0,
            last_token_can_be_transposed: false,
            comments: std::collections::HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.len {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.pos + 1 < self.len {
            Some(self.chars[self.pos + 1])
        } else {
            None
        }
    }

    fn peek_offset(&self, offset: usize) -> Option<char> {
        if self.pos + offset < self.len {
            Some(self.chars[self.pos + offset])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.len {
            let c = self.chars[self.pos];
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, MathError> {
        let mut tokens = Vec::new();

        while self.pos < self.len {
            let ch = match self.peek() {
                Some(c) => c,
                None => break,
            };

            // 1. Comments: % until end of line
            if ch == '%' {
                let comment_line = self.line;
                self.advance(); // consume '%'
                let mut c_text = String::new();
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    c_text.push(c);
                    self.advance();
                }
                self.comments.insert(comment_line, c_text.trim().to_string());
                continue;
            }

            // 2. Line continuation: ...
            if ch == '.' && self.peek_next() == Some('.') && self.peek_offset(2) == Some('.') {
                self.advance(); // .
                self.advance(); // .
                self.advance(); // .
                // Skip everything until newline, and consume the newline
                while let Some(c) = self.peek() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
                continue;
            }

            // 3. Newline
            if ch == '\n' {
                let start_line = self.line;
                let start_col = self.col;
                self.advance();
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = false;
                continue;
            }

            // 4. Whitespace handling
            if ch == ' ' || ch == '\t' || ch == '\r' {
                let mut has_whitespace = false;
                while let Some(c) = self.peek() {
                    if c == ' ' || c == '\t' || c == '\r' {
                        has_whitespace = true;
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Inside [ ], whitespace can act as SpaceSep (element delimiter)
                // only if the preceding token could end an expression operand (i.e. last_token_can_be_transposed)
                if self.bracket_depth > 0 && has_whitespace && self.last_token_can_be_transposed {
                    if let Some(next_c) = self.peek() {
                        let is_space_sep = match next_c {
                            '+' | '-' => {
                                // Rule: whitespace + sign (+/-) + NO whitespace after -> is new element
                                if let Some(after_sign) = self.peek_next() {
                                    after_sign != ' ' && after_sign != '\t' && after_sign != '\r' && after_sign != '\n'
                                } else {
                                    false
                                }
                            }
                            '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '(' | '[' | '"' => true,
                            '.' => {
                                // Could be a number starting with . or transpose
                                if let Some(after_dot) = self.peek_next() {
                                    after_dot.is_ascii_digit()
                                } else {
                                    false
                                }
                            }
                            '\'' => true, // string literal opening
                            _ => false,
                        };

                        if is_space_sep {
                            tokens.push(Token {
                                kind: TokenKind::SpaceSep,
                                line: self.line,
                                col: self.col,
                            });
                            self.last_token_can_be_transposed = false;
                        }
                    }
                }
                continue;
            }

            let start_line = self.line;
            let start_col = self.col;

            // 5. Element-wise operators & Dot transpose
            if ch == '.' {
                if let Some(next_c) = self.peek_next() {
                    if next_c == '*' {
                        self.advance();
                        self.advance();
                        tokens.push(Token { kind: TokenKind::DotStar, line: start_line, col: start_col });
                        self.last_token_can_be_transposed = false;
                        continue;
                    } else if next_c == '/' {
                        self.advance();
                        self.advance();
                        tokens.push(Token { kind: TokenKind::DotSlash, line: start_line, col: start_col });
                        self.last_token_can_be_transposed = false;
                        continue;
                    } else if next_c == '^' {
                        self.advance();
                        self.advance();
                        tokens.push(Token { kind: TokenKind::DotCaret, line: start_line, col: start_col });
                        self.last_token_can_be_transposed = false;
                        continue;
                    } else if next_c == '\'' && self.last_token_can_be_transposed {
                        self.advance();
                        self.advance();
                        tokens.push(Token { kind: TokenKind::DotTranspose, line: start_line, col: start_col });
                        self.last_token_can_be_transposed = true;
                        continue;
                    }
                }
            }

            // 6. Transpose (') vs String literal (')
            if ch == '\'' {
                if self.last_token_can_be_transposed {
                    self.advance();
                    tokens.push(Token { kind: TokenKind::Transpose, line: start_line, col: start_col });
                    self.last_token_can_be_transposed = true;
                    continue;
                } else {
                    // String literal with single quotes
                    self.advance();
                    let mut s = String::new();
                    while let Some(c) = self.peek() {
                        if c == '\'' {
                            self.advance();
                            if self.peek() == Some('\'') {
                                // Escaped quote ''
                                self.advance();
                                s.push('\'');
                            } else {
                                break;
                            }
                        } else if c == '\n' {
                            return Err(MathError::ParseError {
                                line: start_line,
                                col: start_col,
                                message: "Unterminated string literal".into(),
                            });
                        } else {
                            s.push(c);
                            self.advance();
                        }
                    }
                    tokens.push(Token { kind: TokenKind::Str(s), line: start_line, col: start_col });
                    self.last_token_can_be_transposed = true;
                    continue;
                }
            }

            // 7. Double-quoted String literal ("...")
            if ch == '"' {
                self.advance();
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '"' {
                        self.advance();
                        break;
                    } else if c == '\n' {
                        return Err(MathError::ParseError {
                            line: start_line,
                            col: start_col,
                            message: "Unterminated string literal".into(),
                        });
                    } else {
                        s.push(c);
                        self.advance();
                    }
                }
                tokens.push(Token { kind: TokenKind::Str(s), line: start_line, col: start_col });
                self.last_token_can_be_transposed = true;
                continue;
            }

            // 8. Numbers (integer, float, scientific notation)
            if ch.is_ascii_digit() || (ch == '.' && self.peek_next().map_or(false, |c| c.is_ascii_digit())) {
                let mut num_str = String::new();
                let mut has_dot = false;
                let mut has_exp = false;

                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(c);
                        self.advance();
                    } else if c == '.' && !has_dot && !has_exp {
                        // Check if it's not .. (range or continuation)
                        if self.peek_next() == Some('.') {
                            break;
                        }
                        has_dot = true;
                        num_str.push(c);
                        self.advance();
                    } else if (c == 'e' || c == 'E') && !has_exp {
                        has_exp = true;
                        num_str.push(c);
                        self.advance();
                        if let Some(sign) = self.peek() {
                            if sign == '+' || sign == '-' {
                                num_str.push(sign);
                                self.advance();
                            }
                        }
                    } else {
                        break;
                    }
                }

                let num: f64 = num_str.parse().map_err(|_| MathError::ParseError {
                    line: start_line,
                    col: start_col,
                    message: format!("Invalid number '{}'", num_str),
                })?;

                tokens.push(Token {
                    kind: TokenKind::Number(num),
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = true;
                continue;
            }

            // 9. Identifiers (variables, function names, end)
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                let (kind, can_transpose) = match ident.as_str() {
                    "for" => (TokenKind::For, false),
                    "while" => (TokenKind::While, false),
                    "if" => (TokenKind::If, false),
                    "elseif" => (TokenKind::ElseIf, false),
                    "else" => (TokenKind::Else, false),
                    "end" => (TokenKind::End, true),
                    "break" => (TokenKind::Break, false),
                    "continue" => (TokenKind::Continue, false),
                    "function" => (TokenKind::Function, false),
                    "return" => (TokenKind::Return, false),
                    _ => (TokenKind::Ident(ident), true),
                };

                tokens.push(Token {
                    kind,
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = can_transpose;
                continue;
            }

            // 10. Multi-character comparison & logical operators
            if ch == '=' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token {
                    kind: TokenKind::EqualEqual,
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = false;
                continue;
            }
            if ch == '~' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token {
                    kind: TokenKind::NotEqual,
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = false;
                continue;
            }
            if ch == '<' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token {
                    kind: TokenKind::LessEqual,
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = false;
                continue;
            }
            if ch == '>' && self.peek_next() == Some('=') {
                self.advance();
                self.advance();
                tokens.push(Token {
                    kind: TokenKind::GreaterEqual,
                    line: start_line,
                    col: start_col,
                });
                self.last_token_can_be_transposed = false;
                continue;
            }

            // 11. Single character operators & delimiters
            self.advance();
            let (kind, can_transpose) = match ch {
                '+' => (TokenKind::Plus, false),
                '-' => (TokenKind::Minus, false),
                '*' => (TokenKind::Star, false),
                '/' => (TokenKind::Slash, false),
                '\\' => (TokenKind::Backslash, false),
                '^' => (TokenKind::Caret, false),
                ':' => (TokenKind::Colon, false),
                '=' => (TokenKind::Equal, false),
                '<' => (TokenKind::Less, false),
                '>' => (TokenKind::Greater, false),
                '&' => (TokenKind::And, false),
                '|' => (TokenKind::Or, false),
                '~' => (TokenKind::Tilde, false),
                '@' => (TokenKind::At, false),
                '(' => (TokenKind::LParen, false),
                ')' => (TokenKind::RParen, true),
                '[' => {
                    self.bracket_depth += 1;
                    (TokenKind::LBracket, false)
                }
                ']' => {
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    (TokenKind::RBracket, true)
                }
                ';' => (TokenKind::Semicolon, false),
                ',' => (TokenKind::Comma, false),
                _ => {
                    return Err(MathError::ParseError {
                        line: start_line,
                        col: start_col,
                        message: format!("{}", ch),
                    });
                }
            };

            tokens.push(Token {
                kind,
                line: start_line,
                col: start_col,
            });
            self.last_token_can_be_transposed = can_transpose;
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            line: self.line,
            col: self.col,
        });

        Ok(tokens)
    }
}
