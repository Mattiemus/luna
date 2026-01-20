use crate::Symbol;
use crate::expressions::normal::Normal;
use crate::{BigFloat, BigInteger, OrdBigFloat};
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExprKind {
    String(String),
    Integer(BigInteger),
    Real(OrdBigFloat),
    Symbol(Symbol),
    Normal(Normal),
}

impl From<&str> for ExprKind {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for ExprKind {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<BigInteger> for ExprKind {
    fn from(value: BigInteger) -> Self {
        Self::Integer(value)
    }
}

impl From<BigFloat> for ExprKind {
    fn from(value: BigFloat) -> Self {
        Self::Real(OrdBigFloat::from(value))
    }
}

impl From<OrdBigFloat> for ExprKind {
    fn from(value: OrdBigFloat) -> Self {
        Self::Real(value)
    }
}

impl From<Symbol> for ExprKind {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}

impl From<Normal> for ExprKind {
    fn from(value: Normal) -> Self {
        Self::Normal(value)
    }
}

impl Hash for ExprKind {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        // We want two expressions to have different hashes even in cases where they have the same
        // representation (i.e. strings and symbols). Because of this we also hash a type-specific
        // prefix prior to hashing the data.

        // These prefixes are taken from expreduce (https://github.com/corywalker/expreduce).
        // These prefixes were picked at random. We use the same prefixes here.

        const STRING_PREFIX: [u8; 8] = [102, 206, 57, 172, 207, 100, 198, 133];
        const INTEGER_PREFIX: [u8; 8] = [242, 99, 84, 113, 102, 46, 118, 94];
        const REAL_PREFIX: [u8; 8] = [195, 244, 76, 249, 227, 115, 88, 251];
        const SYMBOL_PREFIX: [u8; 8] = [107, 10, 247, 23, 33, 221, 163, 156];
        const EXPR_PREFIX: [u8; 8] = [72, 5, 244, 86, 5, 210, 69, 30];

        match self {
            Self::String(v) => {
                hasher.write(&STRING_PREFIX);
                v.hash(hasher);
            }

            Self::Integer(v) => {
                hasher.write(&INTEGER_PREFIX);
                v.hash(hasher);
            }

            Self::Real(v) => {
                hasher.write(&REAL_PREFIX);
                v.hash(hasher);
            }

            Self::Symbol(v) => {
                hasher.write(&SYMBOL_PREFIX);
                v.hash(hasher);
            }

            Self::Normal(v) => {
                hasher.write(&EXPR_PREFIX);
                v.hash(hasher);
            }
        }
    }
}

impl fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(v) => write!(f, "\"{}\"", v),
            Self::Integer(v) => write!(f, "{}", v),
            Self::Real(v) => write!(f, "{}", v.as_float()),
            Self::Symbol(v) => write!(f, "{}", v),
            Self::Normal(v) => write!(f, "{}", v),
        }
    }
}
