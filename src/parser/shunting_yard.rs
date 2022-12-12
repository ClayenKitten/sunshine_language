//! [Shunting yard algorithm](https://en.wikipedia.org/wiki/Shunting_yard_algorithm) is used to
//! map expressions from infix notation to reverse polish notation or syntax tree.

use std::collections::VecDeque;

use crate::{
    ast::expression::Expression,
    lexer::punctuation::Punctuation,
    parser::{Parser, ParserError},
};

/// A sequence of operands and operators in infix notation.
#[derive(Debug, PartialEq, Eq)]
pub struct InfixExpr(VecDeque<InfixEntry>);

impl InfixExpr {
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
    pub fn parse(parser: &mut Parser) -> Result<Self, ParserError> {
        let mut depth = 0usize;
        let mut output = VecDeque::<InfixEntry>::new();
        let mut is_last_token_an_operand = false;

        loop {
            if is_last_token_an_operand {
                if let Some(op) = parser.lexer.consume_binary_operator()? {
                    is_last_token_an_operand = false;
                    output.push_back(InfixEntry::BinaryOperator(op));
                } else if parser.lexer.peek_punctuation(")") {
                    if depth > 0 {
                        parser.lexer.discard();
                        depth -= 1;
                        is_last_token_an_operand = true;
                        output.push_back(InfixEntry::RightParenthesis);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                if let Some(op) = parser.lexer.consume_unary_operator()? {
                    is_last_token_an_operand = false;
                    output.push_back(InfixEntry::UnaryOperator(op));
                    continue;
                } else if parser.lexer.consume_punctuation("(")? {
                    depth += 1;
                    is_last_token_an_operand = false;
                    output.push_back(InfixEntry::LeftParenthesis);
                } else {
                    let operand = parser.parse_operand()?;
                    output.push_back(InfixEntry::Operand(operand));
                    is_last_token_an_operand = true;
                }
            }
        }

        if depth != 0 {
            return Err(ParserError::UnclosedParenthesis);
        }

        match output.front() {
            Some(InfixEntry::UnaryOperator(_)) | Some(InfixEntry::BinaryOperator(_)) | None => {
                return Err(ParserError::ExpectedExpression)
            }
            _ => {}
        }

        Ok(InfixExpr(output))
    }
}

/// An entry of infix expression: operand, operator (unary or binary) or parenthesis.
#[derive(Debug, PartialEq, Eq)]
pub enum InfixEntry {
    Operand(Expression),
    UnaryOperator(Punctuation),
    BinaryOperator(Punctuation),
    LeftParenthesis,
    RightParenthesis,
}

/// A sequence of operands and operators in [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
#[derive(Debug, PartialEq, Eq)]
pub struct ReversePolishExpr(VecDeque<PolishEntry>);

impl ReversePolishExpr {
    pub fn from_infix(infix: InfixExpr) -> Self {
        let mut output = VecDeque::<PolishEntry>::with_capacity(infix.0.capacity());
        let mut op_stack = Vec::<Operator>::new();

        for entry in infix.0 {
            match entry {
                InfixEntry::Operand(operand) => {
                    output.push_back(PolishEntry::Operand(operand));
                }
                InfixEntry::UnaryOperator(op) => op_stack.push(Operator::Unary { punc: op }),
                InfixEntry::BinaryOperator(op) => {
                    while let Some(top_op) = op_stack.last() {
                        let top_priority = match top_op {
                            Operator::Unary { punc } => punc.priority(),
                            Operator::Binary { punc, .. } => punc.priority(),
                            Operator::LeftParenthesis => break,
                        };
                        if top_priority < op.priority() {
                            break;
                        }
                        output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                    }
                    op_stack.push(Operator::Binary {
                        punc: op,
                        priority: op.priority(),
                    })
                }
                InfixEntry::LeftParenthesis => {
                    op_stack.push(Operator::LeftParenthesis);
                }
                InfixEntry::RightParenthesis => {
                    while let Some(top_op) = op_stack.last() {
                        if top_op == &Operator::LeftParenthesis {
                            break;
                        }
                        output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                    }

                    // Either `op_stack` is empty or left parenthesis is on the top at that point.
                    if op_stack.pop().is_none() {
                        panic!(":(");
                    }
                }
            }
        }

        while let Some(op) = op_stack.pop() {
            output.push_back(op.try_into().unwrap());
        }

        ReversePolishExpr(output)
    }
    /// Convert an RPN to expression tree.
    pub fn into_tree(mut self) -> Expression {
        Self::get_node(&mut self.0)
    }

    fn get_node(buf: &mut VecDeque<PolishEntry>) -> Expression {
        match buf.pop_back().unwrap() {
            PolishEntry::Operand(expr) => expr,
            PolishEntry::UnaryOperator(punc) => {
                let value = Box::new(Self::get_node(buf));
                Expression::Unary { op: punc, value }
            }
            PolishEntry::BinaryOperator(punc) => {
                let right = Box::new(Self::get_node(buf));
                let left = Box::new(Self::get_node(buf));
                Expression::Binary {
                    op: punc,
                    left,
                    right,
                }
            }
        }
    }
}

impl From<InfixExpr> for ReversePolishExpr {
    fn from(val: InfixExpr) -> Self {
        ReversePolishExpr::from_infix(val)
    }
}

/// An entry of RPN expression: operand or operator (unary or binary).
#[derive(Debug, PartialEq, Eq)]
pub enum PolishEntry {
    Operand(Expression),
    UnaryOperator(Punctuation),
    BinaryOperator(Punctuation),
}

#[derive(Debug, PartialEq, Eq)]
enum Operator {
    Unary { punc: Punctuation },
    Binary { punc: Punctuation, priority: u8 },
    LeftParenthesis,
}

impl TryFrom<Operator> for PolishEntry {
    type Error = ();

    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        match value {
            Operator::Unary { punc } => Ok(PolishEntry::UnaryOperator(punc)),
            Operator::Binary { punc, .. } => Ok(PolishEntry::BinaryOperator(punc)),
            Operator::LeftParenthesis => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::expression::{Expression, Literal},
        input_stream::InputStream,
        lexer::{
            number::{Base, Number},
            punctuation::Punctuation,
            Lexer,
        },
        parser::{shunting_yard::InfixEntry, Parser},
    };

    use super::InfixExpr;

    #[test]
    fn infix_parsing() {
        use InfixEntry::*;

        let input = InputStream::new("1 + 2 - (3 * 4) / -5");
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let parsed = InfixExpr::parse(&mut parser).expect("parsing failed");
        let expected = InfixExpr(
            vec![
                Operand(make_num("1")),
                BinaryOperator(Punctuation("+")),
                Operand(make_num("2")),
                BinaryOperator(Punctuation("-")),
                LeftParenthesis,
                Operand(make_num("3")),
                BinaryOperator(Punctuation("*")),
                Operand(make_num("4")),
                RightParenthesis,
                BinaryOperator(Punctuation("/")),
                UnaryOperator(Punctuation("-")),
                Operand(make_num("5")),
            ]
            .into(),
        );
        assert_eq!(
            expected, parsed,
            "infix expression parsed incorrectly. Expected: {:#?}\nParsed: {:#?}",
            expected, parsed
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
