use super::*;

impl Statement {
    pub(super) fn render(&self, indent: usize) -> String {
        match self {
            Statement::ShortVarDecl { bindings, values } => format!(
                "{}{} := {}",
                indent_str(indent),
                render_binding_list(bindings),
                render_expression_list(values)
            ),
            Statement::MultiAssign { bindings, values } => format!(
                "{}{} = {}",
                indent_str(indent),
                render_binding_list(bindings),
                render_expression_list(values)
            ),
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
            Statement::Send { channel, value } => {
                format!(
                    "{}{} <- {}",
                    indent_str(indent),
                    channel.render(),
                    value.render()
                )
            }
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
            Statement::Expr(expression) => format!("{}{}", indent_str(indent), expression.render()),
            Statement::If(if_statement) => render_if_statement(if_statement, indent).join("\n"),
            Statement::Switch(switch_statement) => {
                render_switch_statement(switch_statement, indent).join("\n")
            }
            Statement::TypeSwitch(type_switch_statement) => {
                render_type_switch_statement(type_switch_statement, indent).join("\n")
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
            Statement::TypeAssert {
                bindings,
                binding_mode,
                target,
                asserted_type,
            } => format!(
                "{}{}",
                indent_str(indent),
                render_type_assert_statement(bindings, *binding_mode, target, asserted_type)
            ),
            Statement::IncDec { target, operator } => format!(
                "{}{}{}",
                indent_str(indent),
                target.render(),
                operator.render()
            ),
            Statement::Defer(expression) => {
                format!("{}defer {}", indent_str(indent), expression.render())
            }
            Statement::Break => format!("{}break", indent_str(indent)),
            Statement::Continue => format!("{}continue", indent_str(indent)),
            Statement::Return(values) if values.is_empty() => {
                format!("{}return", indent_str(indent))
            }
            Statement::Return(values) => {
                format!(
                    "{}return {}",
                    indent_str(indent),
                    render_expression_list(values)
                )
            }
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
            HeaderStatement::TypeAssert {
                bindings,
                binding_mode,
                target,
                asserted_type,
            } => render_type_assert_statement(bindings, *binding_mode, target, asserted_type),
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
            ForPostStatement::TypeAssert {
                bindings,
                target,
                asserted_type,
            } => render_type_assert_statement(bindings, BindingMode::Assign, target, asserted_type),
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

impl BindingMode {
    fn render(self) -> &'static str {
        match self {
            BindingMode::Assign => "=",
            BindingMode::Define => ":=",
        }
    }
}

impl Binding {
    fn render(&self) -> String {
        match self {
            Binding::Identifier(name) => name.clone(),
            Binding::Blank => "_".to_string(),
        }
    }
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

fn render_type_switch_statement(
    type_switch_statement: &TypeSwitchStatement,
    indent: usize,
) -> Vec<String> {
    let mut lines = vec![format!(
        "{}switch {}{{",
        indent_str(indent),
        render_type_switch_header(type_switch_statement)
    )];
    for clause in &type_switch_statement.clauses {
        match clause {
            TypeSwitchClause::Case { cases, body } => {
                lines.push(format!(
                    "{}case {}:",
                    indent_str(indent + 1),
                    cases
                        .iter()
                        .map(TypeSwitchCase::render)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                for statement in &body.statements {
                    lines.push(statement.render(indent + 2));
                }
            }
            TypeSwitchClause::Default(body) => {
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

fn render_type_switch_header(type_switch_statement: &TypeSwitchStatement) -> String {
    let guard = type_switch_statement.guard.render();
    match &type_switch_statement.header {
        Some(header) => format!("{}; {} ", header.render(), guard),
        None => format!("{guard} "),
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

fn render_type_assert_statement(
    bindings: &[Binding],
    binding_mode: BindingMode,
    target: &Expression,
    asserted_type: &TypeRef,
) -> String {
    format!(
        "{} {} {}.({})",
        render_binding_list(bindings),
        binding_mode.render(),
        target.render(),
        asserted_type.render()
    )
}

impl TypeSwitchGuard {
    fn render(&self) -> String {
        match &self.binding {
            Some(binding) => format!("{binding} := {}.(type)", self.expression.render()),
            None => format!("{}.(type)", self.expression.render()),
        }
    }
}

impl TypeSwitchCase {
    fn render(&self) -> String {
        match self {
            TypeSwitchCase::Type(type_ref) => type_ref.render(),
            TypeSwitchCase::Nil => "nil".to_string(),
        }
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
