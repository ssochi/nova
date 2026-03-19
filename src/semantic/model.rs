use crate::builtin::BuiltinFunction;
use crate::conversion::ConversionKind;
use crate::package::PackageFunction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedProgram {
    pub package_name: String,
    pub entry_function: usize,
    pub functions: Vec<CheckedFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedFunction {
    pub name: String,
    pub parameter_count: usize,
    pub return_type: Type,
    pub local_names: Vec<String>,
    pub body: CheckedBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedBlock {
    pub statements: Vec<CheckedStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedStatement {
    VarDecl {
        slot: usize,
        name: String,
        value: Option<CheckedExpression>,
    },
    Assign {
        target: CheckedAssignmentTarget,
        value: CheckedExpression,
    },
    Expr(CheckedExpression),
    If {
        condition: CheckedExpression,
        then_block: CheckedBlock,
        else_block: Option<CheckedBlock>,
    },
    For {
        condition: CheckedExpression,
        body: CheckedBlock,
    },
    Return(Option<CheckedExpression>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedAssignmentTarget {
    Local {
        slot: usize,
        name: String,
    },
    Index {
        target: Box<CheckedExpression>,
        index: Box<CheckedExpression>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedExpression {
    pub ty: Type,
    pub kind: CheckedExpressionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedExpressionKind {
    Integer(i64),
    Bool(bool),
    String(String),
    ZeroValue,
    SliceLiteral {
        elements: Vec<CheckedExpression>,
    },
    Local {
        slot: usize,
        name: String,
    },
    Index {
        target: Box<CheckedExpression>,
        index: Box<CheckedExpression>,
    },
    Slice {
        target: Box<CheckedExpression>,
        low: Option<Box<CheckedExpression>>,
        high: Option<Box<CheckedExpression>>,
    },
    MakeSlice {
        element_type: Type,
        length: Box<CheckedExpression>,
        capacity: Option<Box<CheckedExpression>>,
    },
    MakeMap {
        map_type: Type,
        hint: Option<Box<CheckedExpression>>,
    },
    Conversion {
        conversion: ConversionKind,
        value: Box<CheckedExpression>,
    },
    Binary {
        left: Box<CheckedExpression>,
        operator: CheckedBinaryOperator,
        right: Box<CheckedExpression>,
    },
    Call {
        target: CallTarget,
        arguments: Vec<CheckedExpression>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallTarget {
    Builtin(BuiltinFunction),
    PackageFunction(PackageFunction),
    UserDefined { function_index: usize, name: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckedBinaryOperator {
    Add,
    Concat,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Byte,
    Bool,
    String,
    Slice(Box<Type>),
    Map { key: Box<Type>, value: Box<Type> },
    Void,
}

impl Type {
    pub fn render(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Slice(element) => format!("[]{}", element.render()),
            Type::Map { key, value } => format!("map[{}]{}", key.render(), value.render()),
            Type::Void => "void".to_string(),
        }
    }

    pub fn produces_value(&self) -> bool {
        *self != Type::Void
    }

    pub fn slice_element_type(&self) -> Option<&Type> {
        match self {
            Type::Slice(element) => Some(element.as_ref()),
            _ => None,
        }
    }

    pub fn map_parts(&self) -> Option<(&Type, &Type)> {
        match self {
            Type::Map { key, value } => Some((key.as_ref(), value.as_ref())),
            _ => None,
        }
    }

    pub fn supports_equality(&self) -> bool {
        matches!(self, Type::Int | Type::Byte | Type::Bool | Type::String)
    }

    pub fn is_byte_slice(&self) -> bool {
        matches!(self, Type::Slice(element) if element.as_ref() == &Type::Byte)
    }

    pub fn supports_map_key(&self) -> bool {
        matches!(self, Type::Int | Type::Byte | Type::Bool | Type::String)
    }
}
