use crate::frontend::ast::{
    Binding, BindingMode, CompoundAssignOperator, ElseBranch, Expression, ForPostStatement,
    ForStatement, HeaderStatement, IfStatement, IncDecOperator, Statement, SwitchClause,
    SwitchStatement,
};
use crate::frontend::token::TokenKind;

use super::{ParseError, Parser, assignment_target_from_expression};

impl<'a> Parser<'a> {
    pub(super) fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_kind(&TokenKind::Var) {
            let header = self.parse_var_decl_header_statement()?;
            let HeaderStatement::VarDecl {
                name,
                type_ref,
                value,
            } = header
            else {
                unreachable!("variable declaration parsing always returns var data");
            };
            return Ok(Statement::VarDecl {
                name,
                type_ref,
                value,
            });
        }

        if self.match_kind(&TokenKind::Return) {
            if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RightBrace) {
                return Ok(Statement::Return(Vec::new()));
            }

            return Ok(Statement::Return(self.parse_expression_list()?));
        }

        if self.match_kind(&TokenKind::If) {
            return Ok(Statement::If(self.parse_if_statement()?));
        }

        if self.match_kind(&TokenKind::Switch) {
            return Ok(Statement::Switch(self.parse_switch_statement()?));
        }

        if self.match_kind(&TokenKind::For) {
            return self.parse_for_statement();
        }

        if self.match_kind(&TokenKind::Break) {
            self.reject_labeled_control_statement("break")?;
            return Ok(Statement::Break);
        }

        if self.match_kind(&TokenKind::Continue) {
            self.reject_labeled_control_statement("continue")?;
            return Ok(Statement::Continue);
        }

        self.parse_expression_statement()
    }

    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        if self.check(&TokenKind::LeftBrace) {
            return Ok(Statement::For(ForStatement {
                init: None,
                condition: None,
                post: None,
                body: self.parse_block()?,
            }));
        }

        if self.match_kind(&TokenKind::Range) {
            let target = self.parse_expression()?;
            let body = self.parse_block()?;
            return Ok(Statement::RangeFor {
                bindings: Vec::new(),
                binding_mode: None,
                target,
                body,
            });
        }

        if self.is_range_header_start() {
            let first = self.parse_binding()?;
            let mut bindings = vec![first];
            if self.match_kind(&TokenKind::Comma) {
                bindings.push(self.parse_binding()?);
            }
            let binding_mode = if self.match_kind(&TokenKind::Define) {
                BindingMode::Define
            } else {
                self.expect(TokenKind::Assign)?;
                BindingMode::Assign
            };
            self.expect(TokenKind::Range)?;
            let target = self.parse_expression()?;
            let body = self.parse_block()?;
            return Ok(Statement::RangeFor {
                bindings,
                binding_mode: Some(binding_mode),
                target,
                body,
            });
        }

        if self.for_statement_uses_clause_form() {
            return Ok(Statement::For(self.parse_for_clause_statement()?));
        }

        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::For(ForStatement {
            init: None,
            condition: Some(condition),
            post: None,
            body,
        }))
    }

    fn parse_if_statement(&mut self) -> Result<IfStatement, ParseError> {
        let (header, condition) = self.parse_if_header()?;
        let then_block = self.parse_block()?;
        let else_branch = if self.match_kind(&TokenKind::Else) {
            if self.match_kind(&TokenKind::If) {
                Some(ElseBranch::If(Box::new(self.parse_if_statement()?)))
            } else {
                Some(ElseBranch::Block(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(IfStatement {
            header,
            condition,
            then_block,
            else_branch,
        })
    }

    fn parse_switch_statement(&mut self) -> Result<SwitchStatement, ParseError> {
        let (header, expression) = self.parse_switch_header()?;
        self.expect(TokenKind::LeftBrace)?;
        self.skip_semicolons();

        let mut clauses = Vec::new();
        while !self.check(&TokenKind::RightBrace) {
            clauses.push(self.parse_switch_clause()?);
            self.skip_semicolons();
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(SwitchStatement {
            header,
            expression,
            clauses,
        })
    }

    fn parse_switch_clause(&mut self) -> Result<SwitchClause, ParseError> {
        if self.match_kind(&TokenKind::Case) {
            let expressions = self.parse_case_expression_list()?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_switch_clause_body()?;
            return Ok(SwitchClause::Case { expressions, body });
        }

        if self.match_kind(&TokenKind::Default) {
            self.expect(TokenKind::Colon)?;
            return Ok(SwitchClause::Default(self.parse_switch_clause_body()?));
        }

        Err(self.error_at_current("expected `case` or `default` in switch body"))
    }

    fn parse_case_expression_list(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut expressions = vec![self.parse_expression()?];
        while self.match_kind(&TokenKind::Comma) {
            expressions.push(self.parse_expression()?);
        }
        Ok(expressions)
    }

    fn parse_switch_clause_body(&mut self) -> Result<crate::frontend::ast::Block, ParseError> {
        let mut statements = Vec::new();
        self.skip_semicolons();
        while !matches!(
            self.current_kind(),
            Some(TokenKind::RightBrace | TokenKind::Case | TokenKind::Default)
        ) {
            statements.push(self.parse_statement()?);
            self.skip_semicolons();
        }
        Ok(crate::frontend::ast::Block { statements })
    }

    fn parse_if_header(&mut self) -> Result<(Option<HeaderStatement>, Expression), ParseError> {
        if self.match_kind(&TokenKind::Var) {
            let header = self.parse_var_decl_header_statement()?;
            self.expect(TokenKind::Semicolon)?;
            let condition = self.parse_expression()?;
            return Ok((Some(header), condition));
        }

        let expression = self.parse_expression()?;
        if self.check(&TokenKind::Comma) {
            let bindings = self.parse_binding_list_from_expression(expression)?;
            if self.match_kind(&TokenKind::Define) {
                let values = self.parse_expression_list()?;
                self.expect(TokenKind::Semicolon)?;
                let condition = self.parse_expression()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok((
                            Some(HeaderStatement::MapLookup {
                                bindings,
                                binding_mode: BindingMode::Define,
                                target,
                                key,
                            }),
                            condition,
                        ));
                    }
                }
                return Ok((
                    Some(HeaderStatement::ShortVarDecl { bindings, values }),
                    condition,
                ));
            }
            if self.match_kind(&TokenKind::Assign) {
                let values = self.parse_expression_list()?;
                self.expect(TokenKind::Semicolon)?;
                let condition = self.parse_expression()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok((
                            Some(HeaderStatement::MapLookup {
                                bindings,
                                binding_mode: BindingMode::Assign,
                                target,
                                key,
                            }),
                            condition,
                        ));
                    }
                }
                return Ok((
                    Some(HeaderStatement::MultiAssign { bindings, values }),
                    condition,
                ));
            }
            return Err(self.error_at_current(
                "multi-binding `if` header requires `:=` or `=` after the left side",
            ));
        }
        if self.match_kind(&TokenKind::Define) {
            let values = self.parse_expression_list()?;
            self.expect(TokenKind::Semicolon)?;
            let condition = self.parse_expression()?;
            return Ok((
                Some(HeaderStatement::ShortVarDecl {
                    bindings: vec![binding_from_expression(expression)?],
                    values,
                }),
                condition,
            ));
        }
        if self.match_kind(&TokenKind::Assign) {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            let condition = self.parse_expression()?;
            return Ok((Some(HeaderStatement::Assign { target, value }), condition));
        }
        if let Some(operator) = self.match_compound_assign_operator() {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            let condition = self.parse_expression()?;
            return Ok((
                Some(HeaderStatement::CompoundAssign {
                    target,
                    operator,
                    value,
                }),
                condition,
            ));
        }
        if let Some(operator) = self.match_inc_dec_operator() {
            let target = assignment_target_from_expression(expression)?;
            self.expect(TokenKind::Semicolon)?;
            let condition = self.parse_expression()?;
            return Ok((
                Some(HeaderStatement::IncDec { target, operator }),
                condition,
            ));
        }

        if self.match_kind(&TokenKind::Semicolon) {
            let condition = self.parse_expression()?;
            return Ok((Some(HeaderStatement::Expr(expression)), condition));
        }

        Ok((None, expression))
    }

    fn parse_switch_header(
        &mut self,
    ) -> Result<(Option<HeaderStatement>, Option<Expression>), ParseError> {
        if self.check(&TokenKind::LeftBrace) {
            return Ok((None, None));
        }

        if self.match_kind(&TokenKind::Var) {
            let header = self.parse_var_decl_header_statement()?;
            self.expect(TokenKind::Semicolon)?;
            let expression = if self.check(&TokenKind::LeftBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok((Some(header), expression));
        }

        let expression = self.parse_expression()?;
        if self.check(&TokenKind::Comma) {
            let bindings = self.parse_binding_list_from_expression(expression)?;
            if self.match_kind(&TokenKind::Define) {
                let values = self.parse_expression_list()?;
                self.expect(TokenKind::Semicolon)?;
                let switch_expression = if self.check(&TokenKind::LeftBrace) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok((
                            Some(HeaderStatement::MapLookup {
                                bindings,
                                binding_mode: BindingMode::Define,
                                target,
                                key,
                            }),
                            switch_expression,
                        ));
                    }
                }
                return Ok((
                    Some(HeaderStatement::ShortVarDecl { bindings, values }),
                    switch_expression,
                ));
            }
            if self.match_kind(&TokenKind::Assign) {
                let values = self.parse_expression_list()?;
                self.expect(TokenKind::Semicolon)?;
                let switch_expression = if self.check(&TokenKind::LeftBrace) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok((
                            Some(HeaderStatement::MapLookup {
                                bindings,
                                binding_mode: BindingMode::Assign,
                                target,
                                key,
                            }),
                            switch_expression,
                        ));
                    }
                }
                return Ok((
                    Some(HeaderStatement::MultiAssign { bindings, values }),
                    switch_expression,
                ));
            }
            return Err(self.error_at_current(
                "multi-binding `switch` header requires `:=` or `=` after the left side",
            ));
        }
        if self.match_kind(&TokenKind::Define) {
            let values = self.parse_expression_list()?;
            self.expect(TokenKind::Semicolon)?;
            let switch_expression = if self.check(&TokenKind::LeftBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok((
                Some(HeaderStatement::ShortVarDecl {
                    bindings: vec![binding_from_expression(expression)?],
                    values,
                }),
                switch_expression,
            ));
        }
        if self.match_kind(&TokenKind::Assign) {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            let switch_expression = if self.check(&TokenKind::LeftBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok((
                Some(HeaderStatement::Assign { target, value }),
                switch_expression,
            ));
        }
        if let Some(operator) = self.match_compound_assign_operator() {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            let switch_expression = if self.check(&TokenKind::LeftBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok((
                Some(HeaderStatement::CompoundAssign {
                    target,
                    operator,
                    value,
                }),
                switch_expression,
            ));
        }
        if let Some(operator) = self.match_inc_dec_operator() {
            let target = assignment_target_from_expression(expression)?;
            self.expect(TokenKind::Semicolon)?;
            let switch_expression = if self.check(&TokenKind::LeftBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok((
                Some(HeaderStatement::IncDec { target, operator }),
                switch_expression,
            ));
        }

        if self.match_kind(&TokenKind::Semicolon) {
            let switch_expression = if self.check(&TokenKind::LeftBrace) {
                None
            } else {
                Some(self.parse_expression()?)
            };
            return Ok((Some(HeaderStatement::Expr(expression)), switch_expression));
        }

        Ok((None, Some(expression)))
    }

    fn parse_var_decl_header_statement(&mut self) -> Result<HeaderStatement, ParseError> {
        let name = self.expect_identifier()?;
        let (type_ref, value) = if self.match_kind(&TokenKind::Assign) {
            (None, Some(self.parse_expression()?))
        } else if self.check_type_start() {
            let type_ref = self.parse_type_ref()?;
            let value = if self.match_kind(&TokenKind::Assign) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            (Some(type_ref), value)
        } else {
            return Err(
                self.error_at_current("variable declaration requires a type or initializer")
            );
        };
        Ok(HeaderStatement::VarDecl {
            name,
            type_ref,
            value,
        })
    }

    fn parse_for_clause_statement(&mut self) -> Result<ForStatement, ParseError> {
        let init = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_for_init_statement()?)
        };
        self.expect(TokenKind::Semicolon)?;

        let condition = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenKind::Semicolon)?;

        let post = if self.check(&TokenKind::LeftBrace) {
            None
        } else {
            Some(self.parse_for_post_statement()?)
        };

        Ok(ForStatement {
            init,
            condition,
            post,
            body: self.parse_block()?,
        })
    }

    fn parse_for_init_statement(&mut self) -> Result<HeaderStatement, ParseError> {
        if self.match_kind(&TokenKind::Var) {
            return self.parse_var_decl_header_statement();
        }

        let expression = self.parse_expression()?;
        if self.check(&TokenKind::Comma) {
            let bindings = self.parse_binding_list_from_expression(expression)?;
            if self.match_kind(&TokenKind::Define) {
                let values = self.parse_expression_list()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok(HeaderStatement::MapLookup {
                            bindings,
                            binding_mode: BindingMode::Define,
                            target,
                            key,
                        });
                    }
                }
                return Ok(HeaderStatement::ShortVarDecl { bindings, values });
            }
            if self.match_kind(&TokenKind::Assign) {
                let values = self.parse_expression_list()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok(HeaderStatement::MapLookup {
                            bindings,
                            binding_mode: BindingMode::Assign,
                            target,
                            key,
                        });
                    }
                }
                return Ok(HeaderStatement::MultiAssign { bindings, values });
            }
            return Err(self.error_at_current(
                "multi-binding `for` init requires `:=` or `=` after the left side",
            ));
        }
        if self.match_kind(&TokenKind::Define) {
            return Ok(HeaderStatement::ShortVarDecl {
                bindings: vec![binding_from_expression(expression)?],
                values: self.parse_expression_list()?,
            });
        }
        if self.match_kind(&TokenKind::Assign) {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(HeaderStatement::Assign { target, value });
        }
        if let Some(operator) = self.match_compound_assign_operator() {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(HeaderStatement::CompoundAssign {
                target,
                operator,
                value,
            });
        }
        if let Some(operator) = self.match_inc_dec_operator() {
            let target = assignment_target_from_expression(expression)?;
            return Ok(HeaderStatement::IncDec { target, operator });
        }

        Ok(HeaderStatement::Expr(expression))
    }

    fn parse_for_post_statement(&mut self) -> Result<ForPostStatement, ParseError> {
        let expression = self.parse_expression()?;
        if self.check(&TokenKind::Comma) {
            let bindings = self.parse_binding_list_from_expression(expression)?;
            if self.match_kind(&TokenKind::Define) {
                return Err(self.error_at_current("for post statement does not support `:=`"));
            }
            if self.match_kind(&TokenKind::Assign) {
                let values = self.parse_expression_list()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok(ForPostStatement::MapLookup {
                            bindings,
                            target,
                            key,
                        });
                    }
                }
                return Ok(ForPostStatement::MultiAssign { bindings, values });
            }
            return Err(self.error_at_current(
                "multi-binding `for` post statement requires `=` after the left side",
            ));
        }
        if self.match_kind(&TokenKind::Define) {
            return Err(self.error_at_current("for post statement does not support `:=`"));
        }
        if self.match_kind(&TokenKind::Assign) {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(ForPostStatement::Assign { target, value });
        }
        if let Some(operator) = self.match_compound_assign_operator() {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(ForPostStatement::CompoundAssign {
                target,
                operator,
                value,
            });
        }
        if let Some(operator) = self.match_inc_dec_operator() {
            let target = assignment_target_from_expression(expression)?;
            return Ok(ForPostStatement::IncDec { target, operator });
        }

        Ok(ForPostStatement::Expr(expression))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expression = self.parse_expression()?;
        if self.check(&TokenKind::Comma) {
            let bindings = self.parse_binding_list_from_expression(expression)?;
            if self.match_kind(&TokenKind::Define) {
                let values = self.parse_expression_list()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok(Statement::MapLookup {
                            bindings,
                            binding_mode: BindingMode::Define,
                            target,
                            key,
                        });
                    }
                }
                return Ok(Statement::ShortVarDecl { bindings, values });
            }
            if self.match_kind(&TokenKind::Assign) {
                let values = self.parse_expression_list()?;
                if bindings.len() == 2 {
                    if let Ok((target, key)) = map_lookup_from_value_list(values.clone()) {
                        return Ok(Statement::MapLookup {
                            bindings,
                            binding_mode: BindingMode::Assign,
                            target,
                            key,
                        });
                    }
                }
                return Ok(Statement::MultiAssign { bindings, values });
            }
            return Err(self.error_at_current(
                "multi-binding statement requires `:=` or `=` after the left side",
            ));
        }
        if self.match_kind(&TokenKind::Define) {
            return Ok(Statement::ShortVarDecl {
                bindings: vec![binding_from_expression(expression)?],
                values: self.parse_expression_list()?,
            });
        }
        if self.match_kind(&TokenKind::Assign) {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(Statement::Assign { target, value });
        }
        if self.match_kind(&TokenKind::LeftArrow) {
            let value = self.parse_expression()?;
            return Ok(Statement::Send {
                channel: expression,
                value,
            });
        }
        if let Some(operator) = self.match_compound_assign_operator() {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(Statement::CompoundAssign {
                target,
                operator,
                value,
            });
        }
        if let Some(operator) = self.match_inc_dec_operator() {
            let target = assignment_target_from_expression(expression)?;
            return Ok(Statement::IncDec { target, operator });
        }
        Ok(Statement::Expr(expression))
    }

    fn match_inc_dec_operator(&mut self) -> Option<IncDecOperator> {
        if self.match_kind(&TokenKind::PlusPlus) {
            Some(IncDecOperator::Increment)
        } else if self.match_kind(&TokenKind::MinusMinus) {
            Some(IncDecOperator::Decrement)
        } else {
            None
        }
    }

    fn match_compound_assign_operator(&mut self) -> Option<CompoundAssignOperator> {
        if self.match_kind(&TokenKind::PlusAssign) {
            Some(CompoundAssignOperator::Add)
        } else if self.match_kind(&TokenKind::MinusAssign) {
            Some(CompoundAssignOperator::Subtract)
        } else if self.match_kind(&TokenKind::StarAssign) {
            Some(CompoundAssignOperator::Multiply)
        } else if self.match_kind(&TokenKind::SlashAssign) {
            Some(CompoundAssignOperator::Divide)
        } else {
            None
        }
    }

    fn parse_binding(&mut self) -> Result<Binding, ParseError> {
        let name = self.expect_identifier()?;
        if name == "_" {
            Ok(Binding::Blank)
        } else {
            Ok(Binding::Identifier(name))
        }
    }

    fn is_range_header_start(&self) -> bool {
        match (
            self.current_kind(),
            self.peek_kind(),
            self.peek_second_kind(),
            self.peek_third_kind(),
            self.peek_fourth_kind(),
        ) {
            (
                Some(TokenKind::Identifier(_)),
                Some(TokenKind::Define | TokenKind::Assign),
                Some(TokenKind::Range),
                _,
                _,
            ) => true,
            (
                Some(TokenKind::Identifier(_)),
                Some(TokenKind::Comma),
                Some(TokenKind::Identifier(_)),
                Some(TokenKind::Define | TokenKind::Assign),
                Some(TokenKind::Range),
            ) => true,
            _ => false,
        }
    }

    fn for_statement_uses_clause_form(&self) -> bool {
        for index in self.index..self.tokens.len() {
            match self.tokens[index].kind {
                TokenKind::LeftBrace | TokenKind::Eof => return false,
                TokenKind::Semicolon => return true,
                _ => {}
            }
        }
        false
    }

    fn reject_labeled_control_statement(&self, keyword: &str) -> Result<(), ParseError> {
        if matches!(
            self.current_kind(),
            Some(TokenKind::Semicolon | TokenKind::RightBrace | TokenKind::Eof)
        ) {
            return Ok(());
        }
        Err(self.error_at_current(&format!("labeled `{keyword}` statements are not supported")))
    }

    fn parse_binding_list_from_expression(
        &mut self,
        first: Expression,
    ) -> Result<Vec<Binding>, ParseError> {
        let mut bindings = vec![binding_from_expression(first)?];
        while self.match_kind(&TokenKind::Comma) {
            bindings.push(self.parse_binding()?);
        }
        Ok(bindings)
    }
}

fn binding_from_expression(expression: Expression) -> Result<Binding, ParseError> {
    match expression {
        Expression::Identifier(name) if name == "_" => Ok(Binding::Blank),
        Expression::Identifier(name) => Ok(Binding::Identifier(name)),
        _ => Err(ParseError::new(
            "multi-binding statement requires identifier or `_` bindings on the left side",
        )),
    }
}

fn map_lookup_from_value_list(
    values: Vec<Expression>,
) -> Result<(Expression, Expression), Vec<Expression>> {
    if values.len() != 1 {
        return Err(values);
    }

    let value = values.into_iter().next().expect("length was checked above");
    match value {
        Expression::Index { target, index } => Ok((*target, *index)),
        other => Err(vec![other]),
    }
}
