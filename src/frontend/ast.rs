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
    pub parameters: Vec<Parameter>,
    pub return_types: Vec<TypeRef>,
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
        let return_types = render_result_type_list(&self.return_types);
        let mut lines = vec![format!(
            "{}func {}({}){} {{",
            indent_str(indent),
            self.name,
            parameters,
            return_types
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
    pub variadic: bool,
}

impl Parameter {
    fn render(&self) -> String {
        let prefix = if self.variadic { "..." } else { "" };
        format!("{} {}{}", self.name, prefix, self.type_ref.render())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef {
    Named(String),
    Slice(Box<TypeRef>),
    Chan(Box<TypeRef>),
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
            TypeRef::Chan(element) => format!("chan {}", element.render()),
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
    IncDec {
        target: AssignmentTarget,
        operator: IncDecOperator,
    },
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
pub enum SwitchClause {
    Case {
        expressions: Vec<Expression>,
        body: Block,
    },
    Default(Block),
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

impl Statement {
    fn render(&self, indent: usize) -> String {
        match self {
            Statement::ShortVarDecl { bindings, values } => {
                format!(
                    "{}{} := {}",
                    indent_str(indent),
                    render_binding_list(bindings),
                    render_expression_list(values)
                )
            }
            Statement::MultiAssign { bindings, values } => {
                format!(
                    "{}{} = {}",
                    indent_str(indent),
                    render_binding_list(bindings),
                    render_expression_list(values)
                )
            }
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
            Statement::Send { channel, value } => format!(
                "{}{} <- {}",
                indent_str(indent),
                channel.render(),
                value.render()
            ),
            Statement::CompoundAssign {
                target,
                operator,
                value,
            } => format!(
                "{}{} {} {}",
                indent_str(indent),
                target.render(),
                operator.render(),
                value.render()
            ),
            Statement::Expr(expression) => {
                format!("{}{}", indent_str(indent), expression.render())
            }
            Statement::If(if_statement) => render_if_statement(if_statement, indent).join("\n"),
            Statement::Switch(switch_statement) => {
                render_switch_statement(switch_statement, indent).join("\n")
            }
            Statement::For(for_statement) => render_for_statement(for_statement, indent).join("\n"),
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
            Statement::IncDec { target, operator } => format!(
                "{}{}{}",
                indent_str(indent),
                target.render(),
                operator.render()
            ),
            Statement::Break => format!("{}break", indent_str(indent)),
            Statement::Continue => format!("{}continue", indent_str(indent)),
            Statement::Return(values) if values.is_empty() => {
                format!("{}return", indent_str(indent))
            }
            Statement::Return(values) => format!(
                "{}return {}",
                indent_str(indent),
                render_expression_list(values)
            ),
        }
    }
}

impl HeaderStatement {
    fn render(&self) -> String {
        match self {
            HeaderStatement::ShortVarDecl { bindings, values } => {
                format!(
                    "{} := {}",
                    render_binding_list(bindings),
                    render_expression_list(values)
                )
            }
            HeaderStatement::MultiAssign { bindings, values } => {
                format!(
                    "{} = {}",
                    render_binding_list(bindings),
                    render_expression_list(values)
                )
            }
            HeaderStatement::VarDecl {
                name,
                type_ref,
                value,
            } => {
                let mut rendered = format!("var {name}");
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
            HeaderStatement::Assign { target, value } => {
                format!("{} = {}", target.render(), value.render())
            }
            HeaderStatement::CompoundAssign {
                target,
                operator,
                value,
            } => format!(
                "{} {} {}",
                target.render(),
                operator.render(),
                value.render()
            ),
            HeaderStatement::Expr(expression) => expression.render(),
            HeaderStatement::MapLookup {
                bindings,
                binding_mode,
                target,
                key,
            } => render_map_lookup_statement(bindings, *binding_mode, target, key),
            HeaderStatement::IncDec { target, operator } => {
                format!("{}{}", target.render(), operator.render())
            }
        }
    }
}

impl ForPostStatement {
    fn render(&self) -> String {
        match self {
            ForPostStatement::Assign { target, value } => {
                format!("{} = {}", target.render(), value.render())
            }
            ForPostStatement::MultiAssign { bindings, values } => {
                format!(
                    "{} = {}",
                    render_binding_list(bindings),
                    render_expression_list(values)
                )
            }
            ForPostStatement::CompoundAssign {
                target,
                operator,
                value,
            } => format!(
                "{} {} {}",
                target.render(),
                operator.render(),
                value.render()
            ),
            ForPostStatement::Expr(expression) => expression.render(),
            ForPostStatement::MapLookup {
                bindings,
                target,
                key,
            } => render_map_lookup_statement(bindings, BindingMode::Assign, target, key),
            ForPostStatement::IncDec { target, operator } => {
                format!("{}{}", target.render(), operator.render())
            }
        }
    }
}

impl IncDecOperator {
    fn render(self) -> &'static str {
        match self {
            IncDecOperator::Increment => "++",
            IncDecOperator::Decrement => "--",
        }
    }
}

impl CompoundAssignOperator {
    fn render(self) -> &'static str {
        match self {
            CompoundAssignOperator::Add => "+=",
            CompoundAssignOperator::Subtract => "-=",
            CompoundAssignOperator::Multiply => "*=",
            CompoundAssignOperator::Divide => "/=",
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

fn render_if_statement(if_statement: &IfStatement, indent: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "{}if {} {{",
        indent_str(indent),
        render_if_header(if_statement)
    )];
    for statement in &if_statement.then_block.statements {
        lines.push(statement.render(indent + 1));
    }
    lines.push(format!("{}}}", indent_str(indent)));
    match &if_statement.else_branch {
        Some(ElseBranch::Block(else_block)) => {
            lines.push(format!("{}else {{", indent_str(indent)));
            for statement in &else_block.statements {
                lines.push(statement.render(indent + 1));
            }
            lines.push(format!("{}}}", indent_str(indent)));
        }
        Some(ElseBranch::If(else_if)) => {
            let mut nested = render_if_statement(else_if, indent);
            let first = nested.remove(0);
            lines.push(format!("{}else {}", indent_str(indent), first.trim_start()));
            lines.extend(nested);
        }
        None => {}
    }
    lines
}

fn render_if_header(if_statement: &IfStatement) -> String {
    match &if_statement.header {
        Some(header) => format!("{}; {}", header.render(), if_statement.condition.render()),
        None => if_statement.condition.render(),
    }
}

fn render_switch_statement(switch_statement: &SwitchStatement, indent: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "{}switch {}{{",
        indent_str(indent),
        render_switch_header(switch_statement)
    )];
    for clause in &switch_statement.clauses {
        match clause {
            SwitchClause::Case { expressions, body } => {
                lines.push(format!(
                    "{}case {}:",
                    indent_str(indent + 1),
                    expressions
                        .iter()
                        .map(Expression::render)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                for statement in &body.statements {
                    lines.push(statement.render(indent + 2));
                }
            }
            SwitchClause::Default(body) => {
                lines.push(format!("{}default:", indent_str(indent + 1)));
                for statement in &body.statements {
                    lines.push(statement.render(indent + 2));
                }
            }
        }
    }
    lines.push(format!("{}}}", indent_str(indent)));
    lines
}

fn render_switch_header(switch_statement: &SwitchStatement) -> String {
    match (&switch_statement.header, &switch_statement.expression) {
        (Some(header), Some(expression)) => {
            format!("{}; {} ", header.render(), expression.render())
        }
        (Some(header), None) => format!("{}; ", header.render()),
        (None, Some(expression)) => format!("{} ", expression.render()),
        (None, None) => String::new(),
    }
}

fn render_for_statement(for_statement: &ForStatement, indent: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "{}for {}{{",
        indent_str(indent),
        render_for_header(for_statement)
    )];
    for statement in &for_statement.body.statements {
        lines.push(statement.render(indent + 1));
    }
    lines.push(format!("{}}}", indent_str(indent)));
    lines
}

fn render_for_header(for_statement: &ForStatement) -> String {
    if let (None, Some(condition), None) = (
        for_statement.init.as_ref(),
        for_statement.condition.as_ref(),
        for_statement.post.as_ref(),
    ) {
        return format!("{} ", condition.render());
    }

    if let (None, None, None) = (
        for_statement.init.as_ref(),
        for_statement.condition.as_ref(),
        for_statement.post.as_ref(),
    ) {
        return String::new();
    }

    let init = for_statement
        .init
        .as_ref()
        .map(HeaderStatement::render)
        .unwrap_or_default();
    let condition = for_statement
        .condition
        .as_ref()
        .map(Expression::render)
        .unwrap_or_default();
    let post = for_statement
        .post
        .as_ref()
        .map(ForPostStatement::render)
        .unwrap_or_default();
    format!("{init}; {condition}; {post} ")
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

fn render_result_type_list(result_types: &[TypeRef]) -> String {
    match result_types {
        [] => String::new(),
        [result_type] => format!(" {}", result_type.render()),
        _ => format!(
            " ({})",
            result_types
                .iter()
                .map(TypeRef::render)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_binding_list(bindings: &[Binding]) -> String {
    bindings
        .iter()
        .map(Binding::render)
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_expression_list(values: &[Expression]) -> String {
    values
        .iter()
        .map(Expression::render)
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
