use std::fmt;

use crate::frontend::ast::{
    AssignmentTarget, BinaryOperator, Block, Expression, FunctionDecl, ImportDecl, MapLiteralEntry,
    Parameter, SourceFileAst, Statement, TypeRef,
};
use crate::frontend::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseError {}

pub fn parse_source_file(tokens: &[Token]) -> Result<SourceFileAst, ParseError> {
    Parser::new(tokens).parse_source_file()
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_source_file(&mut self) -> Result<SourceFileAst, ParseError> {
        self.expect_keyword(TokenKind::Package)?;
        let package_name = self.expect_identifier()?;
        self.skip_semicolons();

        let mut imports = Vec::new();
        while self.match_kind(&TokenKind::Import) {
            imports.push(self.parse_import_decl()?);
            self.skip_semicolons();
        }

        let mut functions = Vec::new();
        while !self.is_at_end() {
            functions.push(self.parse_function_decl()?);
            self.skip_semicolons();
        }

        Ok(SourceFileAst {
            package_name,
            imports,
            functions,
        })
    }

    fn parse_import_decl(&mut self) -> Result<ImportDecl, ParseError> {
        let path = self.expect_string()?;
        Ok(ImportDecl { path })
    }

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        self.expect_keyword(TokenKind::Func)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftParen)?;
        let parameters = self.parse_parameter_list()?;
        self.expect(TokenKind::RightParen)?;
        let return_type = if self.check_type_start() {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(FunctionDecl {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut parameters = Vec::new();
        if self.check(&TokenKind::RightParen) {
            return Ok(parameters);
        }

        loop {
            let name = self.expect_identifier()?;
            let type_ref = self.parse_type_ref()?;
            parameters.push(Parameter { name, type_ref });
            if !self.match_kind(&TokenKind::Comma) {
                break;
            }
        }

        Ok(parameters)
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        self.skip_semicolons();
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) {
            if self.is_at_end() {
                return Err(self.error_at_current("unexpected end of file inside block"));
            }
            statements.push(self.parse_statement()?);
            self.skip_semicolons();
        }

        self.expect(TokenKind::RightBrace)?;
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_kind(&TokenKind::Var) {
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
            return Ok(Statement::VarDecl {
                name,
                type_ref,
                value,
            });
        }

        if self.match_kind(&TokenKind::Return) {
            if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RightBrace) {
                return Ok(Statement::Return(None));
            }

            let value = self.parse_expression()?;
            return Ok(Statement::Return(Some(value)));
        }

        if self.match_kind(&TokenKind::If) {
            let condition = self.parse_expression()?;
            let then_block = self.parse_block()?;
            let else_block = if self.match_kind(&TokenKind::Else) {
                Some(self.parse_block()?)
            } else {
                None
            };
            return Ok(Statement::If {
                condition,
                then_block,
                else_block,
            });
        }

        if self.match_kind(&TokenKind::For) {
            let condition = self.parse_expression()?;
            let body = self.parse_block()?;
            return Ok(Statement::For { condition, body });
        }

        let expression = self.parse_expression()?;
        if self.match_kind(&TokenKind::Assign) {
            let target = assignment_target_from_expression(expression)?;
            let value = self.parse_expression()?;
            return Ok(Statement::Assign { target, value });
        }
        Ok(Statement::Expr(expression))
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_equality_expression()
    }

    fn parse_equality_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_comparison_expression()?;

        while let Some(operator) =
            self.match_binary_operator(&[TokenKind::EqualEqual, TokenKind::BangEqual])
        {
            let right = self.parse_comparison_expression()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_additive_expression()?;

        while let Some(operator) = self.match_binary_operator(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let right = self.parse_additive_expression()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_multiplicative_expression()?;

        while let Some(operator) = self.match_binary_operator(&[TokenKind::Plus, TokenKind::Minus])
        {
            let right = self.parse_multiplicative_expression()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_postfix_expression()?;

        while let Some(operator) = self.match_binary_operator(&[TokenKind::Star, TokenKind::Slash])
        {
            let right = self.parse_postfix_expression()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_primary_expression()?;

        loop {
            if self.match_kind(&TokenKind::Dot) {
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

            return Ok(expression);
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, ParseError> {
        match self.current_token().kind.clone() {
            TokenKind::Integer(value) => {
                self.advance();
                Ok(Expression::Integer(value))
            }
            TokenKind::Bool(value) => {
                self.advance();
                Ok(Expression::Bool(value))
            }
            TokenKind::String(value) => {
                self.advance();
                Ok(Expression::String(value))
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expression::Nil)
            }
            TokenKind::Identifier(name) => {
                self.advance();
                if is_supported_named_type(&name) && self.check(&TokenKind::LeftParen) {
                    self.parse_conversion_expression(TypeRef::Named(name))
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            TokenKind::LeftBracket if self.peek_kind() == Some(&TokenKind::RightBracket) => {
                self.parse_type_prefixed_expression()
            }
            TokenKind::Map => self.parse_type_prefixed_expression(),
            TokenKind::LeftParen => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expression)
            }
            _ => Err(self.error_at_current("expected expression")),
        }
    }

    fn parse_type_prefixed_expression(&mut self) -> Result<Expression, ParseError> {
        let type_ref = self.parse_type_ref()?;
        if self.check(&TokenKind::LeftBrace) {
            return self.parse_composite_literal_with_type(type_ref);
        }
        if self.check(&TokenKind::LeftParen) {
            return self.parse_conversion_expression(type_ref);
        }
        Err(self.error_at_current("expected `{` or `(` after type"))
    }

    fn parse_composite_literal_with_type(
        &mut self,
        type_ref: TypeRef,
    ) -> Result<Expression, ParseError> {
        match type_ref {
            TypeRef::Slice(element_type) => self.parse_slice_literal_with_type(*element_type),
            TypeRef::Map { key, value } => self.parse_map_literal_with_type(*key, *value),
            TypeRef::Named(name) => Err(ParseError::new(format!(
                "composite literal requires `slice` or `map` type, found `{name}`"
            ))),
        }
    }

    fn parse_slice_literal_with_type(
        &mut self,
        element_type: TypeRef,
    ) -> Result<Expression, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        let elements = self.parse_trailing_comma_expression_list(TokenKind::RightBrace)?;
        Ok(Expression::SliceLiteral {
            element_type: TypeRef::Slice(Box::new(element_type)),
            elements,
        })
    }

    fn parse_map_literal_with_type(
        &mut self,
        key: TypeRef,
        value: TypeRef,
    ) -> Result<Expression, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        let mut entries = Vec::new();
        if !self.match_kind(&TokenKind::RightBrace) {
            loop {
                let key_expression = self.parse_expression()?;
                self.expect(TokenKind::Colon)?;
                let value_expression = self.parse_expression()?;
                entries.push(MapLiteralEntry {
                    key: key_expression,
                    value: value_expression,
                });
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RightBrace) {
                    break;
                }
            }
            if !matches!(
                self.current_token().kind,
                TokenKind::RightBrace | TokenKind::Eof
            ) {
                return Err(self.error_at_current("expected `,` or `}` in map literal"));
            }
            self.expect(TokenKind::RightBrace)?;
        }
        Ok(Expression::MapLiteral {
            map_type: TypeRef::Map {
                key: Box::new(key),
                value: Box::new(value),
            },
            entries,
        })
    }

    fn parse_trailing_comma_expression_list(
        &mut self,
        end: TokenKind,
    ) -> Result<Vec<Expression>, ParseError> {
        let mut expressions = Vec::new();
        if self.match_kind(&end) {
            return Ok(expressions);
        }

        loop {
            expressions.push(self.parse_expression()?);
            if !self.match_kind(&TokenKind::Comma) {
                break;
            }
            if self.match_kind(&end) {
                return Ok(expressions);
            }
        }

        self.expect(end)?;
        Ok(expressions)
    }

    fn parse_conversion_expression(&mut self, type_ref: TypeRef) -> Result<Expression, ParseError> {
        self.expect(TokenKind::LeftParen)?;
        if self.check(&TokenKind::RightParen) {
            return Err(self.error_at_current("conversion requires exactly one argument"));
        }
        let value = self.parse_expression()?;
        if self.match_kind(&TokenKind::Comma) {
            self.expect(TokenKind::RightParen)?;
        } else {
            self.expect(TokenKind::RightParen)?;
        }
        Ok(Expression::Conversion {
            type_ref,
            value: Box::new(value),
        })
    }

    fn parse_make_expression(&mut self) -> Result<Expression, ParseError> {
        if !self.check_type_start() {
            return Err(self.error_at_current("builtin `make` requires a type argument"));
        }

        let type_ref = self.parse_type_ref()?;
        let mut arguments = Vec::new();
        if self.match_kind(&TokenKind::Comma) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen)?;
        Ok(Expression::Make {
            type_ref,
            arguments,
        })
    }

    fn parse_call_expression(&mut self, callee: Expression) -> Result<Expression, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.parse_expression()?);
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen)?;
        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        if self.match_kind(&TokenKind::LeftBracket) {
            self.expect(TokenKind::RightBracket)?;
            return Ok(TypeRef::Slice(Box::new(self.parse_type_ref()?)));
        }

        if self.match_kind(&TokenKind::Map) {
            self.expect(TokenKind::LeftBracket)?;
            let key = self.parse_type_ref()?;
            self.expect(TokenKind::RightBracket)?;
            let value = self.parse_type_ref()?;
            return Ok(TypeRef::Map {
                key: Box::new(key),
                value: Box::new(value),
            });
        }

        Ok(TypeRef::Named(self.expect_identifier()?))
    }

    fn check_type_start(&self) -> bool {
        self.check(&TokenKind::LeftBracket)
            || self.check(&TokenKind::Map)
            || matches!(self.current_token().kind, TokenKind::Identifier(_))
    }

    fn match_binary_operator(&mut self, candidates: &[TokenKind]) -> Option<BinaryOperator> {
        for candidate in candidates {
            if self.match_kind(candidate) {
                return Some(match candidate {
                    TokenKind::Plus => BinaryOperator::Add,
                    TokenKind::Minus => BinaryOperator::Subtract,
                    TokenKind::Star => BinaryOperator::Multiply,
                    TokenKind::Slash => BinaryOperator::Divide,
                    TokenKind::EqualEqual => BinaryOperator::Equal,
                    TokenKind::BangEqual => BinaryOperator::NotEqual,
                    TokenKind::Less => BinaryOperator::Less,
                    TokenKind::LessEqual => BinaryOperator::LessEqual,
                    TokenKind::Greater => BinaryOperator::Greater,
                    TokenKind::GreaterEqual => BinaryOperator::GreaterEqual,
                    _ => unreachable!("only operator tokens are provided"),
                });
            }
        }

        None
    }

    fn reject_full_slice_expression(&self) -> Result<(), ParseError> {
        if self.check(&TokenKind::Colon) {
            Err(self.error_at_current("full slice expressions are not supported"))
        } else {
            Ok(())
        }
    }

    fn expect_keyword(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        self.expect(expected)
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_at_current(&format!("expected `{}`", expected.render())))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current_token().kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(self.error_at_current("expected identifier")),
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match &self.current_token().kind {
            TokenKind::String(value) => {
                let value = value.clone();
                self.advance();
                Ok(value)
            }
            _ => Err(self.error_at_current("expected string literal")),
        }
    }

    fn match_kind(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        match (kind, &self.current_token().kind) {
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::Integer(_), TokenKind::Integer(_)) => true,
            (TokenKind::Bool(_), TokenKind::Bool(_)) => true,
            (TokenKind::String(_), TokenKind::String(_)) => true,
            _ => self.current_token().kind == *kind,
        }
    }

    fn skip_semicolons(&mut self) {
        while self.match_kind(&TokenKind::Semicolon) {}
    }

    fn error_at_current(&self, message: &str) -> ParseError {
        let token = self.current_token();
        ParseError::new(format!(
            "{} at {}:{} (found `{}`)",
            message,
            token.span.line,
            token.span.column,
            token.kind.render()
        ))
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.index.min(self.tokens.len().saturating_sub(1))]
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index + 1)
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        self.peek().map(|token| &token.kind)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.index += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token().kind, TokenKind::Eof)
    }
}

fn is_supported_named_type(name: &str) -> bool {
    matches!(name, "int" | "byte" | "bool" | "string")
}

#[cfg(test)]
mod tests;

fn assignment_target_from_expression(
    expression: Expression,
) -> Result<AssignmentTarget, ParseError> {
    match expression {
        Expression::Identifier(name) => Ok(AssignmentTarget::Identifier(name)),
        Expression::Index { target, index } => Ok(AssignmentTarget::Index {
            target: *target,
            index: *index,
        }),
        _ => Err(ParseError::new(
            "assignment target must be a variable name or index expression",
        )),
    }
}
