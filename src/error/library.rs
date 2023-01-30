//! Library of error codes.

#[macro_use]
mod r#macro;

/// Errors issued by parser.
pub mod parser {
    use crate::lexer::{keyword::Keyword, punctuation::Punctuation, Token};

    define_error! {
        /// Expected an item.
        deny ExpectedItem = "expected an item";

        /// Else may only be used directly after if conditional's body.
        ///
        /// ```notrust
        /// if x < 0 {
        ///     x = 0;
        /// }
        /// print("Hello world");
        /// else {
        ///     x -= 1;
        /// }
        /// ```
        deny ElseWithoutIf = "else may only be used directly after if conditional's body";

        /// Assignments can't be chained.
        ///
        /// ```notrust
        /// x = y = 5;
        /// ```
        deny ChainedAssignment = "assignments can't be chained";

        /// Invalid assigned was used in assignment statement.
        ///
        /// At the moment variables are the only valid assignees.
        ///
        /// ```notrust
        /// 5 = 6; ✗
        /// x = 6; 🗸
        /// ```
        deny InvalidAssignee = "assignments can't be chained";

        /// Punctuation is not allowed.
        deny InvalidPunctuation { punc: Punctuation }
        = "punctuation `{punc:?}` is not allowed";

        /// Keyword is not allowed in operator expression.
        deny KeywordNotAllowedInOperatorExpression { kw: Keyword }
        = "keyword `{kw}` is not allowed in operator expression";

        /// Function call may only.
        deny UnexpectedTokenInFunctionCall { token: Token }
        = "Expected one of the following: `)`, `,`, OPERAND; {token:?} encountered";
    }
}

/// Errors issued by lexer.
#[allow(unstable_name_collisions)]
pub mod lexer {
    use itertools::Itertools;

    use crate::{error::ExpectedToken, lexer::Token};

    define_error! {
        /// Token mismatch occured.
        deny TokenMismatch { expected: Vec<ExpectedToken>, found: Token }
        = match expected.as_slice() {
            [expected] => format!("expected {expected}, found {}", found.pretty_print()),
            [expected1, expected2] => format!("expected {expected1} or {expected2}, found {}", found.pretty_print()),
            _ => format!(
                "expected one of: {}, found {}",
                expected.into_iter()
                    .map(|x| x.to_string())
                    .intersperse(String::from(", "))
                    .collect::<String>(),
                found.pretty_print()
            ),
        };
    }

    define_error! {
        /// String literal wasn't terminated.
        deny UnterminatedString = "string literal wasn't terminated";

        /// Invalid identifier.
        ///
        /// identifier must contain only ascii alphanumeric and underscore characters.
        deny InvalidIdentifier = "identifier must contain only ascii alphanumeric and underscore characters";

        /// Invalid escape sentence in string.
        deny InvalidEscape = "invalid escape sentence";

        /// Parsed number is invalid.
        deny InvalidNumber = "invalid number";

        /// Valid punctuation sequence found, but it is unknown to the compiler.
        deny UnknownPunctuation { found: String }
        = "`{found}` is not a valid punctuation";

        /// Character not expected.
        ///
        /// Only ASCII is supported as the moment.
        deny UnexpectedCharacter { ch: char }
        = "character `{ch}` wasn't expected";

        /// End of file wasn't expected.
        deny UnexpectedEOF = "unexpected EOF";
    }
}
