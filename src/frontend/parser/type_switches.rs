use crate::frontend::ast::{
    Expression, HeaderStatement, Statement, TypeSwitchCase, TypeSwitchClause, TypeSwitchGuard,
    TypeSwitchStatement,
};
use crate::frontend::token::TokenKind;

use super::{ParseError, Parser};

impl<'a> Parser<'a> {
    pub(super) fn parse_switch_statement(&mut self) -> Result<Statement, ParseError> {
        if self.switch_header_contains_type_guard() {
            return Ok(Statement::TypeSwitch(self.parse_type_switch_statement()?));
        }
        Ok(Statement::Switch(self.parse_expression_switch_statement()?))
    }

    fn parse_type_switch_statement(&mut self) -> Result<TypeSwitchStatement, ParseError> {
        let (header, guard) = self.parse_type_switch_header()?;
        self.expect(TokenKind::LeftBrace)?;
        self.skip_semicolons();

        let mut clauses = Vec::new();
        while !self.check(&TokenKind::RightBrace) {
            clauses.push(self.parse_type_switch_clause()?);
            self.skip_semicolons();
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(TypeSwitchStatement {
            header,
            guard,
            clauses,
        })
    }

    fn parse_type_switch_clause(&mut self) -> Result<TypeSwitchClause, ParseError> {
        if self.match_kind(&TokenKind::Case) {
            let cases = self.parse_type_switch_case_list()?;
            self.expect(TokenKind::Colon)?;
            let body = self.parse_switch_clause_body()?;
            return Ok(TypeSwitchClause::Case { cases, body });
        }

        if self.match_kind(&TokenKind::Default) {
            self.expect(TokenKind::Colon)?;
            return Ok(TypeSwitchClause::Default(self.parse_switch_clause_body()?));
        }

        Err(self.error_at_current("expected `case` or `default` in type switch body"))
    }

    fn parse_type_switch_case_list(&mut self) -> Result<Vec<TypeSwitchCase>, ParseError> {
        let mut cases = vec![self.parse_type_switch_case()?];
        while self.match_kind(&TokenKind::Comma) {
            cases.push(self.parse_type_switch_case()?);
        }
        Ok(cases)
    }

    fn parse_type_switch_case(&mut self) -> Result<TypeSwitchCase, ParseError> {
        if self.match_kind(&TokenKind::Nil) {
            return Ok(TypeSwitchCase::Nil);
        }
        if !self.check_type_start() {
            return Err(self.error_at_current("type switch case requires a type or `nil`"));
        }
        Ok(TypeSwitchCase::Type(self.parse_type_ref()?))
    }

    fn parse_type_switch_header(
        &mut self,
    ) -> Result<(Option<HeaderStatement>, TypeSwitchGuard), ParseError> {
        if matches!(self.current_kind(), Some(TokenKind::Identifier(name)) if name == "_")
            && self.peek_kind() == Some(&TokenKind::Define)
        {
            return Err(ParseError::new(
                "type switch guard requires a named identifier before `:=`",
            ));
        }
        if let Ok(guard) = self.parse_type_switch_guard() {
            return Ok((None, guard));
        }

        let header = if self.match_kind(&TokenKind::Var) {
            self.parse_var_decl_header_statement()?
        } else {
            self.parse_for_init_statement()?
        };
        self.expect(TokenKind::Semicolon)?;
        let guard = self.parse_type_switch_guard()?;
        Ok((Some(header), guard))
    }

    fn parse_type_switch_guard(&mut self) -> Result<TypeSwitchGuard, ParseError> {
        let checkpoint = self.index;
        let binding = if matches!(self.current_kind(), Some(TokenKind::Identifier(_)))
            && self.peek_kind() == Some(&TokenKind::Define)
        {
            let binding = self.expect_identifier()?;
            if binding == "_" {
                return Err(ParseError::new(
                    "type switch guard requires a named identifier before `:=`",
                ));
            }
            self.expect(TokenKind::Define)?;
            Some(binding)
        } else {
            None
        };

        match self.parse_type_switch_guard_expression() {
            Ok(expression) => Ok(TypeSwitchGuard {
                binding,
                expression,
            }),
            Err(error) => {
                self.index = checkpoint;
                Err(error)
            }
        }
    }

    fn parse_type_switch_guard_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_primary_expression()?;

        loop {
            if self.match_kind(&TokenKind::Dot) {
                if self.match_kind(&TokenKind::LeftParen) {
                    if matches!(self.current_kind(), Some(TokenKind::Identifier(name)) if name == "type")
                        && self.peek_kind() == Some(&TokenKind::RightParen)
                    {
                        self.advance();
                        self.expect(TokenKind::RightParen)?;
                        return Ok(expression);
                    }
                    return Err(self.error_at_current("type switch guard requires `.(type)`"));
                }
                let member = self.expect_identifier()?;
                expression = Expression::Selector {
                    target: Box::new(expression),
                    member,
                };
                continue;
            }

            if self.match_kind(&TokenKind::LeftBracket) {
                if self.match_kind(&TokenKind::Colon) {
                    let high = if self.check(&TokenKind::RightBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expression()?))
                    };
                    self.reject_full_slice_expression()?;
                    self.expect(TokenKind::RightBracket)?;
                    expression = Expression::Slice {
                        target: Box::new(expression),
                        low: None,
                        high,
                    };
                    continue;
                }

                let first = self.parse_expression()?;
                if self.match_kind(&TokenKind::Colon) {
                    let high = if self.check(&TokenKind::RightBracket) {
                        None
                    } else {
                        Some(Box::new(self.parse_expression()?))
                    };
                    self.reject_full_slice_expression()?;
                    self.expect(TokenKind::RightBracket)?;
                    expression = Expression::Slice {
                        target: Box::new(expression),
                        low: Some(Box::new(first)),
                        high,
                    };
                    continue;
                }
                self.expect(TokenKind::RightBracket)?;
                expression = Expression::Index {
                    target: Box::new(expression),
                    index: Box::new(first),
                };
                continue;
            }

            if self.match_kind(&TokenKind::LeftParen) {
                expression = if matches!(&expression, Expression::Identifier(name) if name == "make")
                {
                    self.parse_make_expression()?
                } else {
                    self.parse_call_expression(expression)?
                };
                continue;
            }

            return Err(self.error_at_current("type switch guard requires `.(type)`"));
        }
    }

    fn switch_header_contains_type_guard(&self) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        for index in self.index..self.tokens.len().saturating_sub(3) {
            match self.tokens[index].kind {
                TokenKind::LeftParen => paren_depth += 1,
                TokenKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::LeftBracket => bracket_depth += 1,
                TokenKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::LeftBrace | TokenKind::Eof if paren_depth == 0 && bracket_depth == 0 => {
                    return false;
                }
                TokenKind::Dot if paren_depth == 0 && bracket_depth == 0 => {
                    if matches!(self.tokens[index + 1].kind, TokenKind::LeftParen)
                        && matches!(&self.tokens[index + 2].kind, TokenKind::Identifier(name) if name == "type")
                        && matches!(self.tokens[index + 3].kind, TokenKind::RightParen)
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}
