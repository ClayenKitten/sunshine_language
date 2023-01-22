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
