#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinFunction {
    Print,
    Println,
    Len,
    Cap,
    Copy,
    Append,
    Make,
    Delete,
    Close,
    Clear,
    Panic,
}

impl BuiltinFunction {
    pub fn render(self) -> &'static str {
        match self {
            BuiltinFunction::Print => "print",
            BuiltinFunction::Println => "println",
            BuiltinFunction::Len => "len",
            BuiltinFunction::Cap => "cap",
            BuiltinFunction::Copy => "copy",
            BuiltinFunction::Append => "append",
            BuiltinFunction::Make => "make",
            BuiltinFunction::Delete => "delete",
            BuiltinFunction::Close => "close",
            BuiltinFunction::Clear => "clear",
            BuiltinFunction::Panic => "panic",
        }
    }
}
