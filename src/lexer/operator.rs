macro_rules! define_operator {
    (
        $(
            $(#[doc = $doc:expr])?
            enum $name:ident {
                $($field:ident = $value:literal,)*
            }
        )*
    ) => {
        $(
            $(#[doc = $doc])?
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum $name {
                $($field,)*
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "{}",
                        match self {
                            $($name::$field => $value,)*
                        }
                    )
                }
            }

            impl TryFrom<crate::lexer::punctuation::Punctuation> for $name {
                type Error = ();

                fn try_from(value: crate::lexer::punctuation::Punctuation) -> Result<Self, Self::Error> {
                    Ok(match value.0 {
                        $($value => $name::$field,)*
                        _ => return Err(()),
                    })
                }
            }
        )*
    }
}

define_operator! {
    /// An operator with one operand.
    enum UnaryOp {
        Add = "+",
        Sub = "-",
        Not = "!",
    }

    /// An operator with two operands.
    enum BinaryOp {
        Add = "+",
        Sub = "-",
        Mul = "*",
        Div = "/",
        Mod = "%",
        Rsh = ">>",
        Lsh = "<<",
        BinAnd = "&",
        BinOr = "|",
        BinXor = "^",
        And = "&&",
        Or = "||",
        Eq = "==",
        Neq = "!=",
        More = ">",
        Less = "<",
        MoreEq = ">=",
        LessEq = "<=",
    }

    /// An operator with two operands: assignee and value.
    enum AssignOp {
        Assign = "=",
        AddAssign = "+=",
        SubAssign = "-=",
        MulAssign = "*=",
        DivAssign = "/=",
    }
}

impl BinaryOp {
    pub fn priority(&self) -> usize {
        use BinaryOp::*;
        match self {
            Mul | Div | Mod => 128,
            Add | Sub => 96,
            Rsh | Lsh => 64,
            BinAnd => 52,
            BinXor => 51,
            BinOr => 50,
            And => 31,
            Or => 30,
            Eq | Neq | More | Less | MoreEq | LessEq => 16,
        }
    }
}
