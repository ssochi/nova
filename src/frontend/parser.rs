use std::fmt;

use crate::frontend::ast::{
    AssignmentTarget, BinaryOperator, Block, Expression, FunctionDecl, ImportDecl, Parameter,
    SourceFileAst, Statement, TypeRef,
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
            TokenKind::LeftBracket if self.peek_kind() == Some(&TokenKind::RightBracket) => {
                self.parse_slice_literal()
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

    fn parse_slice_literal(&mut self) -> Result<Expression, ParseError> {
        let element_type = self.parse_type_ref()?;
        self.expect(TokenKind::LeftBrace)?;
        let mut elements = Vec::new();
        if !self.check(&TokenKind::RightBrace) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Expression::SliceLiteral {
            element_type,
            elements,
        })
    }

    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        if self.match_kind(&TokenKind::LeftBracket) {
            self.expect(TokenKind::RightBracket)?;
            return Ok(TypeRef::Slice(Box::new(self.parse_type_ref()?)));
        }

        Ok(TypeRef::Named(self.expect_identifier()?))
    }

    fn check_type_start(&self) -> bool {
        self.check(&TokenKind::LeftBracket)
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

#[cfg(test)]
mod tests {
    use super::parse_source_file;
    use crate::frontend::ast::{AssignmentTarget, Expression, Statement, TypeRef};
    use crate::frontend::lexer::lex;
    use crate::source::SourceFile;

    #[test]
    fn parse_slice_literal_and_index_expression() {
        let source = SourceFile {
            path: "test.go".into(),
            contents:
                "package main\n\nfunc main() {\n\tvar values = []int{1, 2}\n\tprintln(values[1])\n}\n"
                    .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let function = &ast.functions[0];

        match &function.body.statements[0] {
            Statement::VarDecl { value, .. } => match value.as_ref() {
                Some(Expression::SliceLiteral { element_type, .. }) => {
                    assert_eq!(
                        element_type,
                        &TypeRef::Slice(Box::new(TypeRef::Named("int".into())))
                    );
                }
                _ => panic!("expected slice literal"),
            },
            _ => panic!("expected variable declaration"),
        }

        match &function.body.statements[1] {
            Statement::Expr(Expression::Call { arguments, .. }) => {
                assert!(matches!(arguments[0], Expression::Index { .. }));
            }
            _ => panic!("expected call expression"),
        }
    }

    #[test]
    fn parse_slice_expression_and_index_assignment() {
        let source = SourceFile {
            path: "test.go".into(),
            contents: "package main\n\nfunc main() {\n\tvar values = []int{1, 2, 3}\n\tvar middle = values[1:3]\n\tvalues[:2][1] = 9\n}\n"
                .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let function = &ast.functions[0];

        match &function.body.statements[1] {
            Statement::VarDecl { value, .. } => match value.as_ref() {
                Some(Expression::Slice { low, high, .. }) => {
                    assert!(matches!(low, Some(value) if **value == Expression::Integer(1)));
                    assert!(matches!(high, Some(value) if **value == Expression::Integer(3)));
                }
                _ => panic!("expected slice expression"),
            },
            _ => panic!("expected variable declaration"),
        }

        match &function.body.statements[2] {
            Statement::Assign { target, .. } => match target {
                AssignmentTarget::Index { target, index } => {
                    assert_eq!(*index, Expression::Integer(1));
                    assert!(matches!(target, Expression::Slice { .. }));
                }
                _ => panic!("expected index assignment target"),
            },
            _ => panic!("expected assignment statement"),
        }
    }

    #[test]
    fn parse_typed_var_declarations_with_and_without_initializers() {
        let source = SourceFile {
            path: "test.go".into(),
            contents: "package main\n\nfunc main() {\n\tvar total int\n\tvar values []int = []int{1, 2}\n}\n"
                .to_string(),
        };

        let tokens = lex(&source).expect("lexing should succeed");
        let ast = parse_source_file(&tokens).expect("parsing should succeed");
        let function = &ast.functions[0];

        match &function.body.statements[0] {
            Statement::VarDecl {
                type_ref, value, ..
            } => {
                assert_eq!(type_ref, &Some(TypeRef::Named("int".into())));
                assert!(value.is_none());
            }
            _ => panic!("expected typed variable declaration"),
        }

        match &function.body.statements[1] {
            Statement::VarDecl {
                type_ref, value, ..
            } => {
                assert_eq!(
                    type_ref,
                    &Some(TypeRef::Slice(Box::new(TypeRef::Named("int".into()))))
                );
                assert!(matches!(value, Some(Expression::SliceLiteral { .. })));
            }
            _ => panic!("expected typed variable declaration with initializer"),
        }
    }
}

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
