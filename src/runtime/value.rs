use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i64),
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
        }
    }
}
