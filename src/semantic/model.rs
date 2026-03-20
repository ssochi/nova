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
    pub variadic_element_type: Option<Type>,
    pub return_types: Vec<Type>,
    pub result_locals: Vec<CheckedResultLocal>,
    pub local_names: Vec<String>,
    pub body: CheckedBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedResultLocal {
    pub slot: usize,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedBlock {
    pub statements: Vec<CheckedStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedStatement {
    ShortVarDecl {
        bindings: Vec<CheckedBinding>,
        values: CheckedValueSource,
    },
    MultiAssign {
        bindings: Vec<CheckedBinding>,
        values: CheckedValueSource,
    },
    VarDecl {
        slot: usize,
        name: String,
        value: Option<CheckedExpression>,
    },
    Assign {
        target: CheckedAssignmentTarget,
        value: CheckedExpression,
    },
    Send {
        channel: CheckedExpression,
        value: CheckedExpression,
    },
    CompoundAssign {
        target: CheckedAssignmentTarget,
        operator: CheckedCompoundAssignOperator,
        value: CheckedExpression,
    },
    Expr(CheckedExpression),
    If(CheckedIfStatement),
    Switch(CheckedSwitchStatement),
    For(CheckedForStatement),
    RangeFor {
        source: CheckedExpression,
        key_binding: Option<CheckedBinding>,
        value_binding: Option<CheckedBinding>,
        body: CheckedBlock,
    },
    MapLookup {
        map: CheckedExpression,
        key: CheckedExpression,
        value_binding: CheckedBinding,
        ok_binding: CheckedBinding,
    },
    IncDec {
        target: CheckedAssignmentTarget,
        operator: CheckedIncDecOperator,
        operand_type: Type,
    },
    Break,
    Continue,
    Return(CheckedValueSource),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedIfStatement {
    pub header: Option<CheckedHeaderStatement>,
    pub condition: CheckedExpression,
    pub then_block: CheckedBlock,
    pub else_branch: Option<CheckedElseBranch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedHeaderStatement {
    ShortVarDecl {
        bindings: Vec<CheckedBinding>,
        values: CheckedValueSource,
    },
    MultiAssign {
        bindings: Vec<CheckedBinding>,
        values: CheckedValueSource,
    },
    VarDecl {
        slot: usize,
        name: String,
        value: Option<CheckedExpression>,
    },
    Assign {
        target: CheckedAssignmentTarget,
        value: CheckedExpression,
    },
    CompoundAssign {
        target: CheckedAssignmentTarget,
        operator: CheckedCompoundAssignOperator,
        value: CheckedExpression,
    },
    Expr(CheckedExpression),
    MapLookup {
        map: CheckedExpression,
        key: CheckedExpression,
        value_binding: CheckedBinding,
        ok_binding: CheckedBinding,
    },
    IncDec {
        target: CheckedAssignmentTarget,
        operator: CheckedIncDecOperator,
        operand_type: Type,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedElseBranch {
    Block(CheckedBlock),
    If(Box<CheckedIfStatement>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedSwitchStatement {
    pub header: Option<CheckedHeaderStatement>,
    pub expression: Option<CheckedExpression>,
    pub clauses: Vec<CheckedSwitchClause>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedSwitchClause {
    Case {
        expressions: Vec<CheckedExpression>,
        body: CheckedBlock,
    },
    Default(CheckedBlock),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedForStatement {
    pub init: Option<CheckedHeaderStatement>,
    pub condition: Option<CheckedExpression>,
    pub post: Option<CheckedForPostStatement>,
    pub body: CheckedBlock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedForPostStatement {
    Assign {
        target: CheckedAssignmentTarget,
        value: CheckedExpression,
    },
    MultiAssign {
        bindings: Vec<CheckedBinding>,
        values: CheckedValueSource,
    },
    CompoundAssign {
        target: CheckedAssignmentTarget,
        operator: CheckedCompoundAssignOperator,
        value: CheckedExpression,
    },
    Expr(CheckedExpression),
    MapLookup {
        map: CheckedExpression,
        key: CheckedExpression,
        value_binding: CheckedBinding,
        ok_binding: CheckedBinding,
    },
    IncDec {
        target: CheckedAssignmentTarget,
        operator: CheckedIncDecOperator,
        operand_type: Type,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckedIncDecOperator {
    Increment,
    Decrement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckedCompoundAssignOperator {
    Add,
    Concat,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedBinding {
    Local { slot: usize, name: String },
    Discard,
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
pub struct CheckedMapLiteralEntry {
    pub key: CheckedExpression,
    pub value: CheckedExpression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedExpressionKind {
    Integer(i64),
    Bool(bool),
    String(String),
    UntypedNil,
    ZeroValue,
    SliceLiteral {
        elements: Vec<CheckedExpression>,
    },
    MapLiteral {
        entries: Vec<CheckedMapLiteralEntry>,
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
    MakeChan {
        element_type: Type,
        buffer: Option<Box<CheckedExpression>>,
    },
    MakeMap {
        map_type: Type,
        hint: Option<Box<CheckedExpression>>,
    },
    Conversion {
        conversion: ConversionKind,
        value: Box<CheckedExpression>,
    },
    Receive {
        channel: Box<CheckedExpression>,
    },
    Binary {
        left: Box<CheckedExpression>,
        operator: CheckedBinaryOperator,
        right: Box<CheckedExpression>,
    },
    Call(CheckedCall),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedValueSource {
    Expressions(Vec<CheckedExpression>),
    Call(CheckedCall),
}

impl CheckedValueSource {
    pub fn result_types(&self) -> Vec<Type> {
        match self {
            CheckedValueSource::Expressions(values) => {
                values.iter().map(|value| value.ty.clone()).collect()
            }
            CheckedValueSource::Call(call) => call.result_types.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCall {
    pub target: CallTarget,
    pub arguments: CheckedCallArguments,
    pub result_types: Vec<Type>,
}

impl CheckedCall {
    pub fn argument_count(&self) -> usize {
        match &self.arguments {
            CheckedCallArguments::Expressions(arguments) => arguments.len(),
            CheckedCallArguments::ExpandedCall(call) => call.result_types.len(),
            CheckedCallArguments::Spread { arguments, .. } => arguments.len() + 1,
        }
    }

    pub fn argument_types(&self) -> Vec<Type> {
        match &self.arguments {
            CheckedCallArguments::Expressions(arguments) => arguments
                .iter()
                .map(|argument| argument.ty.clone())
                .collect(),
            CheckedCallArguments::ExpandedCall(call) => call.result_types.clone(),
            CheckedCallArguments::Spread { arguments, spread } => arguments
                .iter()
                .map(|argument| argument.ty.clone())
                .chain(std::iter::once(spread.ty.clone()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCallArguments {
    Expressions(Vec<CheckedExpression>),
    ExpandedCall(Box<CheckedCall>),
    Spread {
        arguments: Vec<CheckedExpression>,
        spread: Box<CheckedExpression>,
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
    UntypedNil,
    Slice(Box<Type>),
    Chan(Box<Type>),
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
            Type::UntypedNil => "nil".to_string(),
            Type::Slice(element) => format!("[]{}", element.render()),
            Type::Chan(element) => format!("chan {}", element.render()),
            Type::Map { key, value } => format!("map[{}]{}", key.render(), value.render()),
            Type::Void => "void".to_string(),
        }
    }

    pub fn produces_value(&self) -> bool {
        !matches!(self, Type::Void | Type::UntypedNil)
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

    pub fn channel_element_type(&self) -> Option<&Type> {
        match self {
            Type::Chan(element) => Some(element.as_ref()),
            _ => None,
        }
    }

    pub fn supports_equality(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::Byte | Type::Bool | Type::String | Type::Chan(_)
        )
    }

    pub fn supports_nil(&self) -> bool {
        matches!(self, Type::Slice(_) | Type::Chan(_) | Type::Map { .. })
    }

    pub fn is_byte_slice(&self) -> bool {
        matches!(self, Type::Slice(element) if element.as_ref() == &Type::Byte)
    }

    pub fn supports_map_key(&self) -> bool {
        matches!(self, Type::Int | Type::Byte | Type::Bool | Type::String)
    }

    pub fn supports_inc_dec(&self) -> bool {
        matches!(self, Type::Int | Type::Byte)
    }
}
