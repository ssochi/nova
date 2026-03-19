use crate::builtin::BuiltinFunction;
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
        value: CheckedExpression,
    },
    Assign {
        slot: usize,
        name: String,
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
pub struct CheckedExpression {
    pub ty: Type,
    pub kind: CheckedExpressionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedExpressionKind {
    Integer(i64),
    Bool(bool),
    String(String),
    Local {
        slot: usize,
        name: String,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    String,
    Void,
}

impl Type {
    pub fn render(&self) -> &'static str {
        match self {
            Type::Int => "int",
            Type::Bool => "bool",
            Type::String => "string",
            Type::Void => "void",
        }
    }

    pub fn produces_value(&self) -> bool {
        *self != Type::Void
    }
}
