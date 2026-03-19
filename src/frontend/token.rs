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
    Import,
    Func,
    Var,
    If,
    Else,
    Switch,
    Case,
    Default,
    For,
    Range,
    Return,
    Map,
    Nil,
    Identifier(String),
    Integer(i64),
    Bool(bool),
    String(String),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Define,
    Dot,
    Semicolon,
    Assign,
    EqualEqual,
    BangEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Eof,
}

impl TokenKind {
    pub fn can_end_statement(&self) -> bool {
        matches!(
            self,
            TokenKind::Identifier(_)
                | TokenKind::Integer(_)
                | TokenKind::Bool(_)
                | TokenKind::String(_)
                | TokenKind::Nil
                | TokenKind::RightParen
                | TokenKind::RightBracket
                | TokenKind::RightBrace
                | TokenKind::Return
        )
    }

    pub fn render(&self) -> String {
        match self {
            TokenKind::Package => "package".to_string(),
            TokenKind::Import => "import".to_string(),
            TokenKind::Func => "func".to_string(),
            TokenKind::Var => "var".to_string(),
            TokenKind::If => "if".to_string(),
            TokenKind::Else => "else".to_string(),
            TokenKind::Switch => "switch".to_string(),
            TokenKind::Case => "case".to_string(),
            TokenKind::Default => "default".to_string(),
            TokenKind::For => "for".to_string(),
            TokenKind::Range => "range".to_string(),
            TokenKind::Return => "return".to_string(),
            TokenKind::Map => "map".to_string(),
            TokenKind::Nil => "nil".to_string(),
            TokenKind::Identifier(value) => format!("identifier({value})"),
            TokenKind::Integer(value) => format!("integer({value})"),
            TokenKind::Bool(value) => format!("bool({value})"),
            TokenKind::String(value) => format!("string({})", render_string_literal(value)),
            TokenKind::LeftParen => "(".to_string(),
            TokenKind::RightParen => ")".to_string(),
            TokenKind::LeftBracket => "[".to_string(),
            TokenKind::RightBracket => "]".to_string(),
            TokenKind::LeftBrace => "{".to_string(),
            TokenKind::RightBrace => "}".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Colon => ":".to_string(),
            TokenKind::Define => ":=".to_string(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::Semicolon => ";".to_string(),
            TokenKind::Assign => "=".to_string(),
            TokenKind::EqualEqual => "==".to_string(),
            TokenKind::BangEqual => "!=".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::Minus => "-".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Slash => "/".to_string(),
            TokenKind::Less => "<".to_string(),
            TokenKind::LessEqual => "<=".to_string(),
            TokenKind::Greater => ">".to_string(),
            TokenKind::GreaterEqual => ">=".to_string(),
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

fn render_string_literal(value: &str) -> String {
    let mut rendered = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => rendered.push_str("\\\\"),
            '"' => rendered.push_str("\\\""),
            '\n' => rendered.push_str("\\n"),
            '\t' => rendered.push_str("\\t"),
            '\r' => rendered.push_str("\\r"),
            other => rendered.push(other),
        }
    }
    rendered.push('"');
    rendered
}
