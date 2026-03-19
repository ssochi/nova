use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Package,
    Func,
    Var,
    Return,
    Identifier(String),
    Integer(i64),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Eof,
}

impl TokenKind {
    pub fn can_end_statement(&self) -> bool {
        matches!(
            self,
            TokenKind::Identifier(_)
                | TokenKind::Integer(_)
                | TokenKind::RightParen
                | TokenKind::RightBrace
                | TokenKind::Return
        )
    }

    pub fn render(&self) -> String {
        match self {
            TokenKind::Package => "package".to_string(),
            TokenKind::Func => "func".to_string(),
            TokenKind::Var => "var".to_string(),
            TokenKind::Return => "return".to_string(),
            TokenKind::Identifier(value) => format!("identifier({value})"),
            TokenKind::Integer(value) => format!("integer({value})"),
            TokenKind::LeftParen => "(".to_string(),
            TokenKind::RightParen => ")".to_string(),
            TokenKind::LeftBrace => "{".to_string(),
            TokenKind::RightBrace => "}".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Semicolon => ";".to_string(),
            TokenKind::Assign => "=".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Eof => "<eof>".to_string(),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn render(&self) -> String {
        format!(
            "{:>4}:{:<4} {}",
            self.span.line,
            self.span.column,
            self.kind.render()
        )
    }
}
