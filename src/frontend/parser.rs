use std::fmt;

use crate::frontend::ast::{BinaryOperator, Block, Expression, FunctionDecl, SourceFileAst, Statement};
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

        let mut functions = Vec::new();
        while !self.is_at_end() {
            functions.push(self.parse_function_decl()?);
            self.skip_semicolons();
        }

        Ok(SourceFileAst {
            package_name,
            functions,
        })
    }

    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseError> {
        self.expect_keyword(TokenKind::Func)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftParen)?;
        self.expect(TokenKind::RightParen)?;
        let body = self.parse_block()?;
        Ok(FunctionDecl { name, body })
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

        let expression = self.parse_expression()?;
        Ok(Statement::Expr(expression))
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_additive_expression()
    }

    fn parse_additive_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expression = self.parse_multiplicative_expression()?;

        while let Some(operator) = self.match_binary_operator(&[TokenKind::Plus, TokenKind::Minus]) {
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
        let mut expression = self.parse_primary_expression()?;

        while let Some(operator) = self.match_binary_operator(&[TokenKind::Star, TokenKind::Slash]) {
            let right = self.parse_primary_expression()?;
            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, ParseError> {
        match self.current_token().kind.clone() {
            TokenKind::Integer(value) => {
                self.advance();
                Ok(Expression::Integer(value))
            }
            TokenKind::Identifier(name) => {
                self.advance();
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
                    Ok(Expression::Call {
                        callee: name,
                        arguments,
                    })
                } else {
                    Ok(Expression::Identifier(name))
                }
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
                    _ => unreachable!("only operator tokens are provided"),
                });
            }
        }

        None
    }

    fn check_identifier_assignment(&self) -> bool {
        matches!(
            (&self.current_token().kind, self.peek().map(|token| &token.kind)),
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
