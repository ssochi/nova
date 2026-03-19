#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionKind {
    StringToBytes,
    BytesToString,
}

impl ConversionKind {
    pub fn render(self) -> &'static str {
        match self {
            ConversionKind::StringToBytes => "string->[]byte",
            ConversionKind::BytesToString => "[]byte->string",
        }
    }
}
