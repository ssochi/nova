use super::signature::render_result_decl_list;
pub use super::signature::{ParameterDecl, ResultDecl, TypeRef};

mod render;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFileAst {
    pub package_name: String,
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FunctionDecl>,
}

impl SourceFileAst {
    pub fn render(&self) -> String {
        let mut lines = vec![format!("package {}", self.package_name)];
        for import in &self.imports {
            lines.push(import.render());
        }
        for function in &self.functions {
            lines.push(function.render(0));
        }
        format!("{}\n", lines.join("\n"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportDecl {
    Single(ImportSpec),
    Group(Vec<ImportSpec>),
}

impl ImportDecl {
    fn render(&self) -> String {
        match self {
            ImportDecl::Single(spec) => format!("import {}", spec.render()),
            ImportDecl::Group(specs) => {
                let mut lines = vec!["import (".to_string()];
                for spec in specs {
                    lines.push(format!("    {}", spec.render()));
                }
                lines.push(")".to_string());
                lines.join("\n")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSpec {
    pub binding: Option<String>,
    pub path: String,
}

impl ImportSpec {
    fn render(&self) -> String {
        match &self.binding {
            Some(binding) => format!("{binding} {}", render_string_literal(&self.path)),
            None => render_string_literal(&self.path),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<ParameterDecl>,
    pub results: Vec<ResultDecl>,
    pub body: Block,
}

impl FunctionDecl {
    fn render(&self, indent: usize) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(ParameterDecl::render)
            .collect::<Vec<_>>()
            .join(", ");
        let results = render_result_decl_list(&self.results);
        let mut lines = vec![format!(
            "{}func {}({}){} {{",
            indent_str(indent),
            self.name,
            parameters,
            results
        )];
        for statement in &self.body.statements {
            lines.push(statement.render(indent + 1));
        }
        lines.push(format!("{}}}", indent_str(indent)));
        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    ShortVarDecl {
        bindings: Vec<Binding>,
        values: Vec<Expression>,
    },
    MultiAssign {
        bindings: Vec<Binding>,
        values: Vec<Expression>,
    },
    VarDecl {
        name: String,
        type_ref: Option<TypeRef>,
        value: Option<Expression>,
    },
    Assign {
        target: AssignmentTarget,
        value: Expression,
    },
    Send {
        channel: Expression,
        value: Expression,
    },
    CompoundAssign {
        target: AssignmentTarget,
        operator: CompoundAssignOperator,
        value: Expression,
    },
    Expr(Expression),
    If(IfStatement),
    Switch(SwitchStatement),
    TypeSwitch(TypeSwitchStatement),
    For(ForStatement),
    RangeFor {
        bindings: Vec<Binding>,
        binding_mode: Option<BindingMode>,
        target: Expression,
        body: Block,
    },
    MapLookup {
        bindings: Vec<Binding>,
        binding_mode: BindingMode,
        target: Expression,
        key: Expression,
    },
    TypeAssert {
        bindings: Vec<Binding>,
        binding_mode: BindingMode,
        target: Expression,
        asserted_type: TypeRef,
    },
    IncDec {
        target: AssignmentTarget,
        operator: IncDecOperator,
    },
    Defer(Expression),
    Break,
    Continue,
    Return(Vec<Expression>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfStatement {
    pub header: Option<HeaderStatement>,
    pub condition: Expression,
    pub then_block: Block,
    pub else_branch: Option<ElseBranch>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeaderStatement {
    ShortVarDecl {
        bindings: Vec<Binding>,
        values: Vec<Expression>,
    },
    MultiAssign {
        bindings: Vec<Binding>,
        values: Vec<Expression>,
    },
    VarDecl {
        name: String,
        type_ref: Option<TypeRef>,
        value: Option<Expression>,
    },
    Assign {
        target: AssignmentTarget,
        value: Expression,
    },
    CompoundAssign {
        target: AssignmentTarget,
        operator: CompoundAssignOperator,
        value: Expression,
    },
    Expr(Expression),
    MapLookup {
        bindings: Vec<Binding>,
        binding_mode: BindingMode,
        target: Expression,
        key: Expression,
    },
    TypeAssert {
        bindings: Vec<Binding>,
        binding_mode: BindingMode,
        target: Expression,
        asserted_type: TypeRef,
    },
    IncDec {
        target: AssignmentTarget,
        operator: IncDecOperator,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElseBranch {
    Block(Block),
    If(Box<IfStatement>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchStatement {
    pub header: Option<HeaderStatement>,
    pub expression: Option<Expression>,
    pub clauses: Vec<SwitchClause>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSwitchStatement {
    pub header: Option<HeaderStatement>,
    pub guard: TypeSwitchGuard,
    pub clauses: Vec<TypeSwitchClause>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSwitchGuard {
    pub binding: Option<String>,
    pub expression: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchClause {
    Case {
        expressions: Vec<Expression>,
        body: Block,
    },
    Default(Block),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSwitchClause {
    Case {
        cases: Vec<TypeSwitchCase>,
        body: Block,
    },
    Default(Block),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeSwitchCase {
    Type(TypeRef),
    Nil,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForStatement {
    pub init: Option<HeaderStatement>,
    pub condition: Option<Expression>,
    pub post: Option<ForPostStatement>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForPostStatement {
    Assign {
        target: AssignmentTarget,
        value: Expression,
    },
    MultiAssign {
        bindings: Vec<Binding>,
        values: Vec<Expression>,
    },
    CompoundAssign {
        target: AssignmentTarget,
        operator: CompoundAssignOperator,
        value: Expression,
    },
    Expr(Expression),
    MapLookup {
        bindings: Vec<Binding>,
        target: Expression,
        key: Expression,
    },
    TypeAssert {
        bindings: Vec<Binding>,
        target: Expression,
        asserted_type: TypeRef,
    },
    IncDec {
        target: AssignmentTarget,
        operator: IncDecOperator,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncDecOperator {
    Increment,
    Decrement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompoundAssignOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingMode {
    Assign,
    Define,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Identifier(String),
    Blank,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentTarget {
    Identifier(String),
    Index {
        target: Expression,
        index: Expression,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Integer(i64),
    Bool(bool),
    String(String),
    Nil,
    Identifier(String),
    SliceLiteral {
        element_type: TypeRef,
        elements: Vec<Expression>,
    },
    MapLiteral {
        map_type: TypeRef,
        entries: Vec<MapLiteralEntry>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Index {
        target: Box<Expression>,
        index: Box<Expression>,
    },
    Slice {
        target: Box<Expression>,
        low: Option<Box<Expression>>,
        high: Option<Box<Expression>>,
    },
    Selector {
        target: Box<Expression>,
        member: String,
    },
    TypeAssertion {
        target: Box<Expression>,
        asserted_type: TypeRef,
    },
    Receive {
        channel: Box<Expression>,
    },
    Make {
        type_ref: TypeRef,
        arguments: Vec<Expression>,
    },
    Conversion {
        type_ref: TypeRef,
        value: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<CallArgument>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CallArgument {
    Expression(Expression),
    Spread(Expression),
}

impl CallArgument {
    fn render(&self) -> String {
        match self {
            CallArgument::Expression(expression) => expression.render(),
            CallArgument::Spread(expression) => format!("{}...", expression.render()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapLiteralEntry {
    pub key: Expression,
    pub value: Expression,
}

impl MapLiteralEntry {
    fn render(&self) -> String {
        format!("{}: {}", self.key.render(), self.value.render())
    }
}

impl Expression {
    pub fn render(&self) -> String {
        match self {
            Expression::Integer(value) => value.to_string(),
            Expression::Bool(value) => value.to_string(),
            Expression::String(value) => render_string_literal(value),
            Expression::Nil => "nil".to_string(),
            Expression::Identifier(name) => name.clone(),
            Expression::SliceLiteral {
                element_type,
                elements,
            } => format!(
                "{}{{{}}}",
                element_type.render(),
                elements
                    .iter()
                    .map(Expression::render)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::MapLiteral { map_type, entries } => format!(
                "{}{{{}}}",
                map_type.render(),
                entries
                    .iter()
                    .map(MapLiteralEntry::render)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Expression::Binary {
                left,
                operator,
                right,
            } => format!(
                "({} {} {})",
                left.render(),
                operator.render(),
                right.render()
            ),
            Expression::Index { target, index } => {
                format!("{}[{}]", target.render(), index.render())
            }
            Expression::Slice { target, low, high } => format!(
                "{}[{}:{}]",
                target.render(),
                low.as_ref().map(|value| value.render()).unwrap_or_default(),
                high.as_ref()
                    .map(|value| value.render())
                    .unwrap_or_default()
            ),
            Expression::Selector { target, member } => {
                format!("{}.{}", target.render(), member)
            }
            Expression::TypeAssertion {
                target,
                asserted_type,
            } => format!("{}.({})", target.render(), asserted_type.render()),
            Expression::Receive { channel } => format!("<-{}", channel.render()),
            Expression::Make {
                type_ref,
                arguments,
            } => {
                let arguments = std::iter::once(type_ref.render())
                    .chain(arguments.iter().map(Expression::render))
                    .collect::<Vec<_>>();
                format!("make({})", arguments.join(", "))
            }
            Expression::Conversion { type_ref, value } => {
                format!("{}({})", type_ref.render(), value.render())
            }
            Expression::Call { callee, arguments } => format!(
                "{}({})",
                callee.render(),
                arguments
                    .iter()
                    .map(CallArgument::render)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
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

impl BinaryOperator {
    pub fn render(&self) -> &'static str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::Greater => ">",
            BinaryOperator::GreaterEqual => ">=",
        }
    }
}

fn indent_str(indent: usize) -> String {
    "    ".repeat(indent)
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
