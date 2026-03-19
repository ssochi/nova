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
pub struct ImportDecl {
    pub path: String,
}

impl ImportDecl {
    fn render(&self) -> String {
        format!("import {}", render_string_literal(&self.path))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<TypeRef>,
    pub body: Block,
}

impl FunctionDecl {
    fn render(&self, indent: usize) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(Parameter::render)
            .collect::<Vec<_>>()
            .join(", ");
        let return_type = self
            .return_type
            .as_ref()
            .map(|value| format!(" {}", value.render()))
            .unwrap_or_default();
        let mut lines = vec![format!(
            "{}func {}({}){} {{",
            indent_str(indent),
            self.name,
            parameters,
            return_type
        )];
        for statement in &self.body.statements {
            lines.push(statement.render(indent + 1));
        }
        lines.push(format!("{}}}", indent_str(indent)));
        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub type_ref: TypeRef,
}

impl Parameter {
    fn render(&self) -> String {
        format!("{} {}", self.name, self.type_ref.render())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef {
    Named(String),
    Slice(Box<TypeRef>),
    Map {
        key: Box<TypeRef>,
        value: Box<TypeRef>,
    },
}

impl TypeRef {
    pub fn render(&self) -> String {
        match self {
            TypeRef::Named(name) => name.clone(),
            TypeRef::Slice(element) => format!("[]{}", element.render()),
            TypeRef::Map { key, value } => {
                format!("map[{}]{}", key.render(), value.render())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    VarDecl {
        name: String,
        type_ref: Option<TypeRef>,
        value: Option<Expression>,
    },
    Assign {
        target: AssignmentTarget,
        value: Expression,
    },
    Expr(Expression),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    For {
        condition: Expression,
        body: Block,
    },
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
    Return(Option<Expression>),
}

impl Statement {
    fn render(&self, indent: usize) -> String {
        match self {
            Statement::VarDecl {
                name,
                type_ref,
                value,
            } => {
                let mut rendered = format!("{}var {}", indent_str(indent), name);
                if let Some(type_ref) = type_ref {
                    rendered.push(' ');
                    rendered.push_str(&type_ref.render());
                }
                if let Some(value) = value {
                    rendered.push_str(" = ");
                    rendered.push_str(&value.render());
                }
                rendered
            }
            Statement::Assign { target, value } => {
                format!(
                    "{}{} = {}",
                    indent_str(indent),
                    target.render(),
                    value.render()
                )
            }
            Statement::Expr(expression) => {
                format!("{}{}", indent_str(indent), expression.render())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let mut lines = vec![format!(
                    "{}if {} {{",
                    indent_str(indent),
                    condition.render()
                )];
                for statement in &then_block.statements {
                    lines.push(statement.render(indent + 1));
                }
                lines.push(format!("{}}}", indent_str(indent)));
                if let Some(else_block) = else_block {
                    lines.push(format!("{}else {{", indent_str(indent)));
                    for statement in &else_block.statements {
                        lines.push(statement.render(indent + 1));
                    }
                    lines.push(format!("{}}}", indent_str(indent)));
                }
                lines.join("\n")
            }
            Statement::For { condition, body } => {
                let mut lines = vec![format!(
                    "{}for {} {{",
                    indent_str(indent),
                    condition.render()
                )];
                for statement in &body.statements {
                    lines.push(statement.render(indent + 1));
                }
                lines.push(format!("{}}}", indent_str(indent)));
                lines.join("\n")
            }
            Statement::RangeFor {
                bindings,
                binding_mode,
                target,
                body,
            } => {
                let header = render_range_header(bindings, *binding_mode, target);
                let mut lines = vec![format!("{}for {} {{", indent_str(indent), header)];
                for statement in &body.statements {
                    lines.push(statement.render(indent + 1));
                }
                lines.push(format!("{}}}", indent_str(indent)));
                lines.join("\n")
            }
            Statement::MapLookup {
                bindings,
                binding_mode,
                target,
                key,
            } => format!(
                "{}{}",
                indent_str(indent),
                render_map_lookup_statement(bindings, *binding_mode, target, key)
            ),
            Statement::Return(Some(expression)) => {
                format!("{}return {}", indent_str(indent), expression.render())
            }
            Statement::Return(None) => format!("{}return", indent_str(indent)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingMode {
    Assign,
    Define,
}

impl BindingMode {
    fn render(self) -> &'static str {
        match self {
            BindingMode::Assign => "=",
            BindingMode::Define => ":=",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Identifier(String),
    Blank,
}

impl Binding {
    fn render(&self) -> String {
        match self {
            Binding::Identifier(name) => name.clone(),
            Binding::Blank => "_".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssignmentTarget {
    Identifier(String),
    Index {
        target: Expression,
        index: Expression,
    },
}

impl AssignmentTarget {
    fn render(&self) -> String {
        match self {
            AssignmentTarget::Identifier(name) => name.clone(),
            AssignmentTarget::Index { target, index } => {
                format!("{}[{}]", target.render(), index.render())
            }
        }
    }
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
        arguments: Vec<Expression>,
    },
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
                    .map(Expression::render)
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

fn render_range_header(
    bindings: &[Binding],
    binding_mode: Option<BindingMode>,
    target: &Expression,
) -> String {
    if bindings.is_empty() {
        return format!("range {}", target.render());
    }

    let bindings = render_binding_list(bindings);
    let binding_mode = binding_mode.expect("non-empty range bindings require a binding mode");
    format!(
        "{bindings} {} range {}",
        binding_mode.render(),
        target.render()
    )
}

fn render_map_lookup_statement(
    bindings: &[Binding],
    binding_mode: BindingMode,
    target: &Expression,
    key: &Expression,
) -> String {
    format!(
        "{} {} {}[{}]",
        render_binding_list(bindings),
        binding_mode.render(),
        target.render(),
        key.render()
    )
}

fn render_binding_list(bindings: &[Binding]) -> String {
    bindings
        .iter()
        .map(Binding::render)
        .collect::<Vec<_>>()
        .join(", ")
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
