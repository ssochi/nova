use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    String(String),
    Slice(Vec<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Integer(0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(value) => write!(f, "{value}"),
            Value::Boolean(value) => write!(f, "{value}"),
            Value::String(value) => f.write_str(value),
            Value::Slice(elements) => write!(
                f,
                "[{}]",
                elements
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}
