use crate::lexer::{TokenStream, Token};

/// Stream that outputs structured token tree rather than raw tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenTreeStream {
    stream: TokenStream,
}

impl TokenTreeStream {
    pub fn new(source: TokenStream) -> TokenTreeStream {
        todo!();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenTree {
    Group(TokenTreeStream),
    Token(Token),
}

pub struct Group(pub TokenTreeStream, pub Delimiter);

impl Group {
    
}

pub enum Delimiter {
    /// ( ... )
    Parenthesis,
    /// { ... }
    Brace,
    /// [ ... ]
    Bracket,
}
