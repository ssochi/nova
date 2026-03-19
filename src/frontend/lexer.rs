use std::fmt;

use crate::frontend::token::{Span, Token, TokenKind};
use crate::source::SourceFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    message: String,
}

impl LexError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LexError {}

pub fn lex(source: &SourceFile) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(&source.contents);
    lexer.lex_all()
}

struct Lexer<'a> {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    previous_can_end_statement: bool,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Lexer<'a> {
    fn new(contents: &'a str) -> Self {
        Self {
            chars: contents.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            previous_can_end_statement: false,
            _marker: std::marker::PhantomData,
        }
    }

    fn lex_all(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while let Some(character) = self.peek() {
            match character {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    let span = Span::new(self.line, self.column);
                    self.advance();
                    if self.previous_can_end_statement {
                        tokens.push(Token::new(TokenKind::Semicolon, span));
                        self.previous_can_end_statement = false;
                    }
                }
                '/' if self.peek_next() == Some('/') => {
                    self.skip_line_comment();
                }
                '0'..='9' => {
                    let token = self.lex_integer()?;
                    self.previous_can_end_statement = token.kind.can_end_statement();
                    tokens.push(token);
                }
                '"' => {
                    let token = self.lex_string()?;
                    self.previous_can_end_statement = token.kind.can_end_statement();
                    tokens.push(token);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let token = self.lex_identifier_or_keyword();
                    self.previous_can_end_statement = token.kind.can_end_statement();
                    tokens.push(token);
                }
                '(' => self.push_simple(TokenKind::LeftParen, &mut tokens),
                ')' => self.push_simple(TokenKind::RightParen, &mut tokens),
                '[' => self.push_simple(TokenKind::LeftBracket, &mut tokens),
                ']' => self.push_simple(TokenKind::RightBracket, &mut tokens),
                '{' => self.push_simple(TokenKind::LeftBrace, &mut tokens),
                '}' => self.push_simple(TokenKind::RightBrace, &mut tokens),
                ',' => self.push_simple(TokenKind::Comma, &mut tokens),
                ':' if self.peek_next() == Some('=') => {
                    self.push_double(TokenKind::Define, &mut tokens)
                }
                ':' => self.push_simple(TokenKind::Colon, &mut tokens),
                '.' => self.push_simple(TokenKind::Dot, &mut tokens),
                ';' => self.push_simple(TokenKind::Semicolon, &mut tokens),
                '=' if self.peek_next() == Some('=') => {
                    self.push_double(TokenKind::EqualEqual, &mut tokens)
                }
                '=' => self.push_simple(TokenKind::Assign, &mut tokens),
                '!' if self.peek_next() == Some('=') => {
                    self.push_double(TokenKind::BangEqual, &mut tokens)
                }
                '+' => self.push_simple(TokenKind::Plus, &mut tokens),
                '-' => self.push_simple(TokenKind::Minus, &mut tokens),
                '*' => self.push_simple(TokenKind::Star, &mut tokens),
                '<' if self.peek_next() == Some('=') => {
                    self.push_double(TokenKind::LessEqual, &mut tokens)
                }
                '<' => self.push_simple(TokenKind::Less, &mut tokens),
                '>' if self.peek_next() == Some('=') => {
                    self.push_double(TokenKind::GreaterEqual, &mut tokens)
                }
                '>' => self.push_simple(TokenKind::Greater, &mut tokens),
                '/' => self.push_simple(TokenKind::Slash, &mut tokens),
                other => {
                    return Err(LexError::new(format!(
                        "unexpected character `{other}` at {}:{}",
                        self.line, self.column
                    )));
                }
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.line, self.column),
        ));
        Ok(tokens)
    }

    fn push_simple(&mut self, kind: TokenKind, tokens: &mut Vec<Token>) {
        let token = Token::new(kind, Span::new(self.line, self.column));
        self.advance();
        self.previous_can_end_statement = token.kind.can_end_statement();
        tokens.push(token);
    }

    fn push_double(&mut self, kind: TokenKind, tokens: &mut Vec<Token>) {
        let token = Token::new(kind, Span::new(self.line, self.column));
        self.advance();
        self.advance();
        self.previous_can_end_statement = token.kind.can_end_statement();
        tokens.push(token);
    }

    fn lex_integer(&mut self) -> Result<Token, LexError> {
        let start = Span::new(self.line, self.column);
        let mut literal = String::new();
        while let Some(character @ '0'..='9') = self.peek() {
            literal.push(character);
            self.advance();
        }

        let value = literal.parse::<i64>().map_err(|error| {
            LexError::new(format!(
                "invalid integer `{literal}` at {}:{}: {error}",
                start.line, start.column
            ))
        })?;
        Ok(Token::new(TokenKind::Integer(value), start))
    }

    fn lex_string(&mut self) -> Result<Token, LexError> {
        let start = Span::new(self.line, self.column);
        self.advance();

        let mut literal = String::new();
        while let Some(character) = self.peek() {
            match character {
                '"' => {
                    self.advance();
                    return Ok(Token::new(TokenKind::String(literal), start));
                }
                '\\' => {
                    self.advance();
                    let escape = self.peek().ok_or_else(|| {
                        LexError::new(format!(
                            "unterminated string literal at {}:{}",
                            start.line, start.column
                        ))
                    })?;
                    let escaped = match escape {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        other => {
                            return Err(LexError::new(format!(
                                "unsupported escape `\\{other}` at {}:{}",
                                self.line, self.column
                            )));
                        }
                    };
                    self.advance();
                    literal.push(escaped);
                }
                '\n' => {
                    return Err(LexError::new(format!(
                        "unterminated string literal at {}:{}",
                        start.line, start.column
                    )));
                }
                other => {
                    literal.push(other);
                    self.advance();
                }
            }
        }

        Err(LexError::new(format!(
            "unterminated string literal at {}:{}",
            start.line, start.column
        )))
    }

    fn lex_identifier_or_keyword(&mut self) -> Token {
        let start = Span::new(self.line, self.column);
        let mut literal = String::new();
        while let Some(character) = self.peek() {
            if character.is_ascii_alphanumeric() || character == '_' {
                literal.push(character);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match literal.as_str() {
            "package" => TokenKind::Package,
            "import" => TokenKind::Import,
            "func" => TokenKind::Func,
            "var" => TokenKind::Var,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "for" => TokenKind::For,
            "range" => TokenKind::Range,
            "return" => TokenKind::Return,
            "map" => TokenKind::Map,
            "nil" => TokenKind::Nil,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Identifier(literal),
        };

        Token::new(kind, start)
    }

    fn skip_line_comment(&mut self) {
        while let Some(character) = self.peek() {
            self.advance();
            if character == '\n' {
                if self.previous_can_end_statement {
                    self.previous_can_end_statement = false;
                }
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.index += 1;
        if character == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(character)
    }
}
