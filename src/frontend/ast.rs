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
}

impl TypeRef {
    pub fn render(&self) -> String {
        match self {
            TypeRef::Named(name) => name.clone(),
            TypeRef::Slice(element) => format!("[]{}", element.render()),
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
        value: Expression,
    },
    Assign {
        name: String,
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
    Return(Option<Expression>),
}

impl Statement {
    fn render(&self, indent: usize) -> String {
        match self {
            Statement::VarDecl { name, value } => {
                format!("{}var {} = {}", indent_str(indent), name, value.render())
            }
            Statement::Assign { name, value } => {
                format!("{}{} = {}", indent_str(indent), name, value.render())
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
            Statement::Return(Some(expression)) => {
                format!("{}return {}", indent_str(indent), expression.render())
            }
            Statement::Return(None) => format!("{}return", indent_str(indent)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Integer(i64),
    Bool(bool),
    String(String),
    Identifier(String),
    SliceLiteral {
        element_type: TypeRef,
        elements: Vec<Expression>,
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
    Selector {
        target: Box<Expression>,
        member: String,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

impl Expression {
    pub fn render(&self) -> String {
        match self {
            Expression::Integer(value) => value.to_string(),
            Expression::Bool(value) => value.to_string(),
            Expression::String(value) => render_string_literal(value),
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
            Expression::Selector { target, member } => {
                format!("{}.{}", target.render(), member)
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
