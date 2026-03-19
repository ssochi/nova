use std::fmt;

use crate::frontend::ast::{
    BinaryOperator, Block, Expression, FunctionDecl, ImportDecl, Parameter, SourceFileAst,
    Statement,
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
        let return_type = if matches!(self.current_token().kind, TokenKind::Identifier(_)) {
            Some(self.expect_identifier()?)
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
            let type_name = self.expect_identifier()?;
            parameters.push(Parameter { name, type_name });
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
            self.expect(TokenKind::Assign)?;
            let value = self.parse_expression()?;
            return Ok(Statement::VarDecl { name, value });
        }

        if self.match_kind(&TokenKind::Return) {
            if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RightBrace) {
                return Ok(Statement::Return(None));
            }

            let value = self.parse_expression()?;
            return Ok(Statement::Return(Some(value)));
        }

        if self.check_identifier_assignment() {
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Assign)?;
            let value = self.parse_expression()?;
            return Ok(Statement::Assign { name, value });
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

            if self.match_kind(&TokenKind::LeftParen) {
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
                expression = Expression::Call {
                    callee: Box::new(expression),
                    arguments,
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
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expression::Identifier(name))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expression = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expression)
            }
            _ => Err(self.error_at_current("expected expression")),
        }
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

    fn check_identifier_assignment(&self) -> bool {
        matches!(
            (
                &self.current_token().kind,
                self.peek().map(|token| &token.kind)
            ),
            (TokenKind::Identifier(_), Some(TokenKind::Assign))
        )
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

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.index += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token().kind, TokenKind::Eof)
    }
}
