//! Operator expressions in different forms.
//!
//! Infix expression is parsed and validated, then [shunting yard algorithm]
//! is used to map expressions from [infix notation] to [reverse polish notation] with respect of operator precedence.
//! Reverse polish notation is then mapped into abstract syntax tree.
//!
//! [shunting yard algorithm]: https://en.wikipedia.org/wiki/Shunting_yard_algorithm
//! [infix notation]: https://en.wikipedia.org/wiki/Infix_notation
//! [reverse polish notation]: https://en.wikipedia.org/wiki/Reverse_Polish_notation

mod infix_notation;
mod polish_notation;

pub use infix_notation::*;
pub use polish_notation::*;

use crate::{
    ast::{expression::Expression, Identifier},
    lexer::operator::AssignOp,
};

/// A tree of expressions.
pub type Tree = MaybeAssignment<Expression>;

/// A generic type that may be either expression or assignment of an expression.
#[derive(Debug, PartialEq, Eq)]
pub enum MaybeAssignment<Expr> {
    Assignment {
        assignee: Identifier,
        operator: AssignOp,
        expression: Expr,
    },
    Expression(Expr),
}

impl<Expr> MaybeAssignment<Expr> {
    /// Modifies expression part of any variant and produces new value.
    ///
    /// Assignee and operator are unmodified.
    pub fn map_expr<F, N>(self, func: F) -> MaybeAssignment<N>
    where
        F: FnOnce(Expr) -> N,
    {
        match self {
            MaybeAssignment::Assignment {
                assignee,
                operator,
                expression,
            } => MaybeAssignment::Assignment {
                assignee,
                operator,
                expression: func(expression),
            },
            MaybeAssignment::Expression(expr) => MaybeAssignment::Expression(func(expr)),
        }
    }
}
