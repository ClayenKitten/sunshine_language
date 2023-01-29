//! Infix operator expressions.
use std::collections::VecDeque;

use crate::{
    ast::expression::Expression as AstExpression,
    ast::{expression::Expression, Identifier},
    lexer::operator::{AssignOp, BinaryOp, UnaryOp},
    parser::{FileParser, ParserError},
};

use super::MaybeAssignment;

/// A sequence of operands and operators in [infix notation](https://en.wikipedia.org/wiki/Infix_notation).
pub type InfixNotation = MaybeAssignment<VecDeque<InfixEntry>>;

impl FileParser {
    /// Parse and validate infix expression.
    ///
    /// Parsing continues while valid infix expression may be produced.
    /// For example, in the following snippet only marked parts of source are valid infix expressions:
    /// ```notrust
    /// if x > 5 { 5 + 2 - (10 - 2) }
    ///    ^^^^^   ^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// # Errors
    ///
    /// Error will only be produced if parenthesis mismatches or operator without following operand occurs.
    pub fn parse_infix(&mut self) -> Result<InfixNotation, ParserError> {
        let mut depth = 0usize;
        let mut output = VecDeque::<InfixEntry>::new();
        let mut assignment: Option<(Identifier, AssignOp)> = None;

        loop {
            use InfixEntry::*;

            'assignment: {
                if output.len() != 1 {
                    break 'assignment;
                }
                let Some(operator) = self.lexer.consume_assignment_operator()? else {
                    break 'assignment;
                };
                let Some(Operand(AstExpression::Var(assignee))) = output.pop_back() else {
                    break 'assignment;
                };
                assignment = Some((assignee, operator));
            }

            match output.back() {
                Some(Operand(_) | RightParenthesis) => {
                    if let Some(op) = self.lexer.consume_binary_operator()? {
                        output.push_back(BinaryOperator(op));
                    } else if self.lexer.peek_punctuation(")") {
                        if depth > 0 {
                            self.lexer.discard();
                            depth -= 1;
                            output.push_back(RightParenthesis);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                None | Some(UnaryOperator(_) | BinaryOperator(_) | LeftParenthesis) => {
                    if let Some(op) = self.lexer.consume_unary_operator()? {
                        output.push_back(UnaryOperator(op));
                    } else if self.lexer.consume_punctuation("(")? {
                        depth += 1;
                        output.push_back(LeftParenthesis);
                    } else {
                        let operand = self.parse_operand()?;
                        output.push_back(Operand(operand));
                    }
                }
            }
        }

        if depth != 0 {
            return Err(ParserError::UnclosedParenthesis);
        }

        match output.front() {
            Some(InfixEntry::BinaryOperator(_)) | None => {
                return Err(ParserError::ExpectedExpression)
            }
            _ => {}
        }

        Ok(match assignment {
            Some((assignee, operator)) => {
                self.lexer.expect_punctuation(";")?;
                InfixNotation::Assignment {
                    assignee,
                    operator,
                    expression: output,
                }
            }
            None => InfixNotation::Expression(output),
        })
    }
}

/// An entry of infix expression: operand, operator (unary or binary) or parenthesis.
#[derive(Debug, PartialEq, Eq)]
pub enum InfixEntry {
    Operand(Expression),
    UnaryOperator(UnaryOp),
    BinaryOperator(BinaryOp),
    LeftParenthesis,
    RightParenthesis,
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expression::{Expression, Literal},
            Identifier,
        },
        lexer::{
            number::{Base, Number},
            operator::{BinaryOp, UnaryOp},
        },
        parser::FileParser,
    };

    use super::InfixNotation;

    #[test]
    fn unary() {
        use super::InfixEntry::*;

        let mut parser = FileParser::new_test("-x");
        let parsed = parser.parse_infix().expect("parsing failed");
        let expected = InfixNotation::Expression(
            vec![
                UnaryOperator(UnaryOp::Sub),
                Operand(Expression::Var(Identifier(String::from("x")))),
            ]
            .into(),
        );
        assert!(
            expected == parsed,
            "infix expression parsed incorrectly. Expected:\n{:#?}\nParsed:\n{:#?}",
            expected,
            parsed
        );
    }

    #[test]
    fn binary() {
        use super::InfixEntry::*;

        let mut parser = FileParser::new_test("4 >= x");
        let parsed = parser.parse_infix().expect("parsing failed");
        let expected = InfixNotation::Expression(
            vec![
                Operand(make_num("4")),
                BinaryOperator(BinaryOp::MoreEq),
                Operand(Expression::Var(Identifier(String::from("x")))),
            ]
            .into(),
        );
        assert!(
            expected == parsed,
            "infix expression parsed incorrectly. Expected:\n{:#?}\nParsed:\n{:#?}",
            expected,
            parsed
        );
    }

    #[test]
    fn simple_compound() {
        use super::InfixEntry::*;

        let mut parser = FileParser::new_test("1 + -2");
        let parsed = parser.parse_infix().expect("parsing failed");
        let expected = InfixNotation::Expression(
            vec![
                Operand(make_num("1")),
                BinaryOperator(BinaryOp::Add),
                UnaryOperator(UnaryOp::Sub),
                Operand(make_num("2")),
            ]
            .into(),
        );
        assert!(
            expected == parsed,
            "infix expression parsed incorrectly. Expected:\n{:#?}\nParsed:\n{:#?}",
            expected,
            parsed
        );
    }

    #[test]
    fn complex_compound() {
        use super::InfixEntry::*;

        let mut parser = FileParser::new_test("1 + -2 - (3 * 4) / -5");
        let parsed = parser.parse_infix().expect("parsing failed");
        let expected = InfixNotation::Expression(
            vec![
                Operand(make_num("1")),
                BinaryOperator(BinaryOp::Add),
                UnaryOperator(UnaryOp::Sub),
                Operand(make_num("2")),
                BinaryOperator(BinaryOp::Sub),
                LeftParenthesis,
                Operand(make_num("3")),
                BinaryOperator(BinaryOp::Mul),
                Operand(make_num("4")),
                RightParenthesis,
                BinaryOperator(BinaryOp::Div),
                UnaryOperator(UnaryOp::Sub),
                Operand(make_num("5")),
            ]
            .into(),
        );
        assert!(
            expected == parsed,
            "infix expression parsed incorrectly. Expected:\n{:#?}\nParsed:\n{:#?}",
            expected,
            parsed
        );
    }

    fn make_num(n: &'static str) -> Expression {
        Expression::Literal(Literal::Number(Number {
            integer: n.to_string(),
            fraction: None,
            base: Base::Decimal,
        }))
    }
}
