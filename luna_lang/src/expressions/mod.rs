mod kind;
mod normal;
mod symbol;

use crate::OrdBigFloat;
use crate::abstractions::{BigFloat, BigInteger};
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::sync::Arc;

pub use kind::ExprKind;
pub use normal::Normal;
pub use symbol::Symbol;

/// Representation of an expression node. An expression can be either an "atomic" value (such as
/// a string, integer, real, or symbol) or a "normal" expression which is of the form
/// `f[a1, ..., an]` where `f` is the "head" and `a1, ..., an` represents zero or more `elements`
/// of the expression.
///
/// Internally an expression is an `Arc<ExprKind>`. This means cloning and comparisons are cheap by
/// design to facilitate performant pattern matching.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Expr(Arc<ExprKind>);

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self(Arc::new(kind))
    }

    pub fn kind(&self) -> &ExprKind {
        self.0.as_ref()
    }

    pub fn name(&self) -> Option<&Symbol> {
        match *self.0 {
            ExprKind::Symbol(ref symbol) => Some(symbol),
            ExprKind::Normal(ref normal) => normal.head().name(),
            _ => None,
        }
    }

    pub fn head(&self) -> Self {
        match *self.0 {
            ExprKind::String(_) => Self::from(Symbol::new("String")),
            ExprKind::Integer(_) => Self::from(Symbol::new("Integer")),
            ExprKind::Real(_) => Self::from(Symbol::new("Real")),
            ExprKind::Symbol(_) => Self::from(Symbol::new("Symbol")),
            ExprKind::Normal(ref v) => v.head().clone(),
        }
    }

    pub fn is_normal_head(&self, head: &Symbol) -> bool {
        match *self.0 {
            ExprKind::Normal(ref v) => v.has_head(head),
            _ => false,
        }
    }

    pub fn try_string(&self) -> Option<&String> {
        match *self.0 {
            ExprKind::String(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn try_integer(&self) -> Option<&BigInteger> {
        match *self.0 {
            ExprKind::Integer(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn try_real(&self) -> Option<&OrdBigFloat> {
        match *self.0 {
            ExprKind::Real(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn try_symbol(&self) -> Option<&Symbol> {
        match *self.0 {
            ExprKind::Symbol(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn try_normal(&self) -> Option<&Normal> {
        match *self.0 {
            ExprKind::Normal(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn try_normal_head(&self, head: &Symbol) -> Option<&Normal> {
        match *self.0 {
            ExprKind::Normal(ref v) if v.has_head(head) => Some(v),
            _ => None,
        }
    }
}

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl From<BigInteger> for Expr {
    fn from(value: BigInteger) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl From<BigFloat> for Expr {
    fn from(value: BigFloat) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl From<OrdBigFloat> for Expr {
    fn from(value: OrdBigFloat) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl From<Symbol> for Expr {
    fn from(value: Symbol) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl From<Normal> for Expr {
    fn from(value: Normal) -> Self {
        Self::new(ExprKind::from(value))
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
