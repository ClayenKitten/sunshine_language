//! Operator expressions in different forms.
//!
//! Infix expression is parsed and validated, then [shunting yard algorithm]
//! is used to map expressions from [infix notation] to [reverse polish notation] with respect of operator precedence.
//! Reverse polish notation is then mapped into abstract syntax tree.
//!
//! [shunting yard algorithm]: https://en.wikipedia.org/wiki/Shunting_yard_algorithm
//! [infix notation]: https://en.wikipedia.org/wiki/Infix_notation
//! [reverse polish notation]: https://en.wikipedia.org/wiki/Reverse_Polish_notation

pub use infix_notation::*;
pub use polish_notation::*;

mod infix_notation {
    use std::collections::VecDeque;

    use crate::{
        ast::expression::Expression,
        lexer::punctuation::{BinaryOp, UnaryOp},
        parser::{FileParser, ParserError},
    };

    /// A sequence of operands and operators in [infix notation](https://en.wikipedia.org/wiki/Infix_notation).
    #[derive(Debug, PartialEq, Eq)]
    pub struct InfixExpr(pub VecDeque<InfixEntry>);

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
        pub fn parse(parser: &mut FileParser) -> Result<Self, ParserError> {
            let mut depth = 0usize;
            let mut output = VecDeque::<InfixEntry>::new();

            loop {
                use InfixEntry::*;
                println!("{:?}", parser.lexer.peek());
                match output.back() {
                    Some(Operand(_) | RightParenthesis) => {
                        if let Some(op) = parser.lexer.consume_binary_operator()? {
                            output.push_back(InfixEntry::BinaryOperator(op));
                        } else if parser.lexer.peek_punctuation(")") {
                            if depth > 0 {
                                parser.lexer.discard();
                                depth -= 1;
                                output.push_back(InfixEntry::RightParenthesis);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    None | Some(UnaryOperator(_) | BinaryOperator(_) | LeftParenthesis) => {
                        if let Some(op) = parser.lexer.consume_unary_operator()? {
                            output.push_back(UnaryOperator(op));
                        } else if parser.lexer.consume_punctuation("(")? {
                            depth += 1;
                            output.push_back(LeftParenthesis);
                        } else {
                            let operand = parser.parse_operand()?;
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

            Ok(InfixExpr(output))
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
                punctuation::{BinaryOp, UnaryOp},
            },
            parser::FileParser,
        };

        use super::InfixExpr;

        #[test]
        fn unary() {
            use super::InfixEntry::*;

            let mut parser = FileParser::new_test("-x");
            let parsed = InfixExpr::parse(&mut parser).expect("parsing failed");
            let expected = InfixExpr(
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
            let parsed = InfixExpr::parse(&mut parser).expect("parsing failed");
            let expected = InfixExpr(
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
            let parsed = InfixExpr::parse(&mut parser).expect("parsing failed");
            let expected = InfixExpr(
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
            let parsed = InfixExpr::parse(&mut parser).expect("parsing failed");
            let expected = InfixExpr(
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
}

mod polish_notation {
    use std::collections::VecDeque;

    use crate::{
        ast::expression::Expression,
        lexer::punctuation::{BinaryOp, UnaryOp},
    };

    use super::infix_notation::{InfixEntry, InfixExpr};

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
                    InfixEntry::UnaryOperator(op) => op_stack.push(Operator::Unary(op)),
                    InfixEntry::BinaryOperator(op) => {
                        while let Some(top_op) = op_stack.last() {
                            let top_priority = match top_op {
                                Operator::Unary(_) => 0,
                                Operator::Binary(op) => op.priority(),
                                Operator::LeftParenthesis => break,
                            };
                            if top_priority < op.priority() {
                                break;
                            }
                            output.push_back(op_stack.pop().unwrap().try_into().unwrap());
                        }
                        op_stack.push(Operator::Binary(op));
                    }
                    InfixEntry::LeftParenthesis => op_stack.push(Operator::LeftParenthesis),
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
        UnaryOperator(UnaryOp),
        BinaryOperator(BinaryOp),
    }

    impl TryFrom<Operator> for PolishEntry {
        type Error = ();

        fn try_from(value: Operator) -> Result<Self, Self::Error> {
            match value {
                Operator::Unary(op) => Ok(PolishEntry::UnaryOperator(op)),
                Operator::Binary(op) => Ok(PolishEntry::BinaryOperator(op)),
                Operator::LeftParenthesis => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Operator {
        Unary(UnaryOp),
        Binary(BinaryOp),
        LeftParenthesis,
    }
}
