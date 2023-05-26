use crate::hir::types::TypeId;

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
                    Ok(match value.as_str() {
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

impl UnaryOp {
    pub fn in_type(&self) -> TypeId {
        match self {
            UnaryOp::Add => TypeId::I32,
            UnaryOp::Sub => TypeId::I32,
            UnaryOp::Not => TypeId::BOOL,
        }
    }

    pub fn out_type(&self) -> TypeId {
        match self {
            UnaryOp::Add => TypeId::I32,
            UnaryOp::Sub => TypeId::I32,
            UnaryOp::Not => TypeId::BOOL,
        }
    }
}

impl BinaryOp {
    pub fn in_type(&self) -> TypeId {
        match self {
            BinaryOp::Add => TypeId::I32,
            BinaryOp::Sub => TypeId::I32,
            BinaryOp::Mul => TypeId::I32,
            BinaryOp::Div => TypeId::I32,
            BinaryOp::Mod => TypeId::I32,
            BinaryOp::Rsh => todo!(),
            BinaryOp::Lsh => todo!(),
            BinaryOp::BinAnd => todo!(),
            BinaryOp::BinOr => todo!(),
            BinaryOp::BinXor => todo!(),
            BinaryOp::And => TypeId::BOOL,
            BinaryOp::Or => TypeId::BOOL,
            BinaryOp::Eq => TypeId::I32,
            BinaryOp::Neq => TypeId::I32,
            BinaryOp::More => TypeId::I32,
            BinaryOp::Less => TypeId::I32,
            BinaryOp::MoreEq => TypeId::I32,
            BinaryOp::LessEq => TypeId::I32,
        }
    }

    pub fn out_type(&self) -> TypeId {
        match self {
            BinaryOp::Add => TypeId::I32,
            BinaryOp::Sub => TypeId::I32,
            BinaryOp::Mul => TypeId::I32,
            BinaryOp::Div => TypeId::I32,
            BinaryOp::Mod => TypeId::I32,
            BinaryOp::Rsh => todo!(),
            BinaryOp::Lsh => todo!(),
            BinaryOp::BinAnd => todo!(),
            BinaryOp::BinOr => todo!(),
            BinaryOp::BinXor => todo!(),
            BinaryOp::And => TypeId::BOOL,
            BinaryOp::Or => TypeId::BOOL,
            BinaryOp::Eq => TypeId::BOOL,
            BinaryOp::Neq => TypeId::BOOL,
            BinaryOp::More => TypeId::BOOL,
            BinaryOp::Less => TypeId::BOOL,
            BinaryOp::MoreEq => TypeId::BOOL,
            BinaryOp::LessEq => TypeId::BOOL,
        }
    }

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

impl AssignOp {
    pub fn to_respective_binary_op(&self) -> Option<BinaryOp> {
        match self {
            AssignOp::Assign => None,
            AssignOp::AddAssign => Some(BinaryOp::Add),
            AssignOp::SubAssign => Some(BinaryOp::Sub),
            AssignOp::MulAssign => Some(BinaryOp::Mul),
            AssignOp::DivAssign => Some(BinaryOp::Div),
        }
    }
}
