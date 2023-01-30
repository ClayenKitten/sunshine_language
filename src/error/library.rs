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
pub mod lexer {
    use crate::lexer::{keyword::Keyword, punctuation::Punctuation, Token};

    define_error! {
        /// Punctuation was expected, but something else was found.
        deny ExpectedPunctuation { expected: Punctuation, found: Token }
        = "Expected punctuation {expected:?}, found {found:?}";

        /// Identifier was expected, but something else was found.
        deny ExpectedIdentifier { found: Token }
        = "Expected identifier, found {found:?}";

        /// Keyword was expected, but something else was found.
        deny ExpectedKeyword { keyword: Keyword, found: Token }
        = "Expected keyword `{keyword}`, found {found:?}";

        /// Expected one of the following tokens.
        deny ExpectedOneOf { possible: Vec<&'static str>, found: Token }
        = "expected one of: {possible:?}; {found:?} encountered";

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
