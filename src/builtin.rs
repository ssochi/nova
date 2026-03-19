#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinFunction {
    Print,
    Println,
    Len,
    Append,
}

impl BuiltinFunction {
    pub fn render(self) -> &'static str {
        match self {
            BuiltinFunction::Print => "print",
            BuiltinFunction::Println => "println",
            BuiltinFunction::Len => "len",
            BuiltinFunction::Append => "append",
        }
    }
}
