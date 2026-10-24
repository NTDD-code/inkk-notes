use crate::flowchart_core::ast::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Algorithm,
    Start,
    End,
    Return,
    Set,
    Let,
    Do,
    Input,
    Read,
    Output,
    Print,
    Show,
    If,
    Then,
    Else,
    Switch,
    Case,
    Default,
    For,
    From,
    To,
    While,
    Repeat,
    Until,
    Call,
    Goto,
    Label,
    Link,
    Document,
    Database,
    ManualInput,
    Wait,
    Display,
    Parallel,
    And,
    Note,
    Lane,
    Node,

    // Symbols
    Arrow,     // ->
    DashArrow, // -[...] ->
    Colon,     // :
    Comma,     // ,
    LBracket,  // [
    RBracket,  // ]
    LParen,    // (
    RParen,    // )

    // Literals & Identifiers
    StringLit(String),
    Identifier(String),
    Number(String),
    Comment(String),

    // Layout
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    idx: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().collect(),
            idx: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.idx).map(|&(_, c)| c)
    }

    fn peek_next_char(&self) -> Option<char> {
        self.chars.get(self.idx + 1).map(|&(_, c)| c)
    }

    fn current_byte_pos(&self) -> usize {
        self.chars
            .get(self.idx)
            .map(|&(pos, _)| pos)
            .unwrap_or(self.input.len())
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, c)) = self.chars.get(self.idx) {
            self.idx += 1;
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

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.idx < self.chars.len() {
            let start_byte = self.current_byte_pos();
            let start_line = self.line;
            let start_col = self.col;

            let c = match self.peek_char() {
                Some(ch) => ch,
                None => break,
            };

            // Skip horizontal whitespace (preserve newlines for indentation/statement separation)
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
                continue;
            }

            if c == '\n' {
                self.advance();
                let end_byte = self.current_byte_pos();
                // Deduplicate consecutive newlines
                if let Some(last) = tokens.last() {
                    if last.kind == TokenKind::Newline {
                        continue;
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::Newline,
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }

            // Comments starting with # or //
            if c == '#' || (c == '/' && self.peek_next_char() == Some('/')) {
                let mut comment = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch == '\n' {
                        break;
                    }
                    comment.push(ch);
                    self.advance();
                }
                let end_byte = self.current_byte_pos();
                tokens.push(Token {
                    kind: TokenKind::Comment(comment.trim().to_string()),
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }

            // Arrows: -> or -[
            if c == '-' && self.peek_next_char() == Some('>') {
                self.advance();
                self.advance();
                let end_byte = self.current_byte_pos();
                tokens.push(Token {
                    kind: TokenKind::Arrow,
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }
            if c == '-' && self.peek_next_char() == Some('[') {
                self.advance();
                self.advance();
                let end_byte = self.current_byte_pos();
                tokens.push(Token {
                    kind: TokenKind::LBracket,
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }

            // Single punctuation symbols
            let single_punct = match c {
                ':' => Some(TokenKind::Colon),
                ',' => Some(TokenKind::Comma),
                '[' => Some(TokenKind::LBracket),
                ']' => Some(TokenKind::RBracket),
                '(' => Some(TokenKind::LParen),
                ')' => Some(TokenKind::RParen),
                _ => None,
            };

            if let Some(kind) = single_punct {
                self.advance();
                let end_byte = self.current_byte_pos();
                tokens.push(Token {
                    kind,
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }

            // String literals: "..." or '...'
            if c == '"' || c == '\'' {
                let quote = c;
                self.advance();
                let mut s = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch == quote {
                        self.advance();
                        break;
                    }
                    if ch == '\\' {
                        self.advance();
                        if let Some(esc) = self.peek_char() {
                            s.push(match esc {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                other => other,
                            });
                            self.advance();
                            continue;
                        }
                    }
                    if ch == '\n' {
                        break; // Unclosed string on newline
                    }
                    s.push(ch);
                    self.advance();
                }
                let end_byte = self.current_byte_pos();
                tokens.push(Token {
                    kind: TokenKind::StringLit(s),
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }

            // Identifiers, Keywords, Numbers
            if c.is_alphanumeric()
                || c == '_'
                || c == '.'
                || c == '>'
                || c == '<'
                || c == '='
                || c == '!'
            {
                let mut word = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch.is_whitespace()
                        || ch == ':'
                        || ch == ','
                        || ch == '['
                        || ch == ']'
                        || ch == '('
                        || ch == ')'
                        || ch == '#'
                        || ch == '"'
                        || ch == '\''
                    {
                        break;
                    }
                    word.push(ch);
                    self.advance();
                }
                let end_byte = self.current_byte_pos();

                let kind = match word.to_lowercase().as_str() {
                    "algorithm" => TokenKind::Algorithm,
                    "start" => TokenKind::Start,
                    "end" => TokenKind::End,
                    "return" => TokenKind::Return,
                    "set" => TokenKind::Set,
                    "let" => TokenKind::Let,
                    "do" => TokenKind::Do,
                    "input" => TokenKind::Input,
                    "read" => TokenKind::Read,
                    "output" => TokenKind::Output,
                    "print" => TokenKind::Print,
                    "show" => TokenKind::Show,
                    "if" => TokenKind::If,
                    "then" => TokenKind::Then,
                    "else" => TokenKind::Else,
                    "switch" => TokenKind::Switch,
                    "case" => TokenKind::Case,
                    "default" => TokenKind::Default,
                    "for" => TokenKind::For,
                    "from" => TokenKind::From,
                    "to" => TokenKind::To,
                    "while" => TokenKind::While,
                    "repeat" => TokenKind::Repeat,
                    "until" => TokenKind::Until,
                    "call" => TokenKind::Call,
                    "goto" => TokenKind::Goto,
                    "label" => TokenKind::Label,
                    "link" => TokenKind::Link,
                    "document" => TokenKind::Document,
                    "database" => TokenKind::Database,
                    "manual" => TokenKind::ManualInput,
                    "wait" | "delay" => TokenKind::Wait,
                    "display" => TokenKind::Display,
                    "parallel" => TokenKind::Parallel,
                    "and" => TokenKind::And,
                    "note" => TokenKind::Note,
                    "lane" => TokenKind::Lane,
                    "node" => TokenKind::Node,
                    _ => {
                        if word.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
                            TokenKind::Number(word)
                        } else {
                            TokenKind::Identifier(word)
                        }
                    }
                };

                tokens.push(Token {
                    kind,
                    span: Span::new(start_line, start_col, start_byte, end_byte),
                });
                continue;
            }

            // Fallback for unrecognized single characters
            let unknown = self.advance().unwrap_or(c);
            let end_byte = self.current_byte_pos();
            tokens.push(Token {
                kind: TokenKind::Identifier(unknown.to_string()),
                span: Span::new(start_line, start_col, start_byte, end_byte),
            });
        }

        let end_byte = self.current_byte_pos();
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.line, self.col, end_byte, end_byte),
        });

        tokens
    }
}
