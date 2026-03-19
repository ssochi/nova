#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFileAst {
    pub package_name: String,
    pub functions: Vec<FunctionDecl>,
}

impl SourceFileAst {
    pub fn render(&self) -> String {
        let mut lines = vec![format!("package {}", self.package_name)];
        for function in &self.functions {
            lines.push(function.render(0));
        }
        format!("{}\n", lines.join("\n"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: String,
    pub body: Block,
}

impl FunctionDecl {
    fn render(&self, indent: usize) -> String {
        let mut lines = vec![format!("{}func {}() {{", indent_str(indent), self.name)];
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
    VarDecl { name: String, value: Expression },
    Assign { name: String, value: Expression },
    Expr(Expression),
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
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Call {
        callee: String,
        arguments: Vec<Expression>,
    },
}

impl Expression {
    pub fn render(&self) -> String {
        match self {
            Expression::Integer(value) => value.to_string(),
            Expression::Identifier(name) => name.clone(),
            Expression::Binary {
                left,
                operator,
                right,
            } => format!("({} {} {})", left.render(), operator.render(), right.render()),
            Expression::Call { callee, arguments } => format!(
                "{}({})",
                callee,
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
}

impl BinaryOperator {
    pub fn render(&self) -> &'static str {
        match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
        }
    }
}

fn indent_str(indent: usize) -> String {
    "    ".repeat(indent)
}
