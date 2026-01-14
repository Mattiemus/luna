pub mod kind;
pub mod normal;
pub mod symbol;

use crate::abstractions::{BigFloat, BigInteger};
use crate::expressions::kind::ExprKind;
use crate::expressions::normal::Normal;
use crate::symbol::Symbol;
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::sync::Arc;
use crate::OrdBigFloat;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Expr(Arc<ExprKind>);

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self(Arc::new(kind))
    }

    /// Returns the outermost symbol "tag" used in this expression.
    pub fn tag(&self) -> Option<&Symbol> {
        match *self.0 {
            ExprKind::Symbol(ref v) => Some(v),
            ExprKind::Normal(ref v) => v.head().tag(),
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

    pub fn try_head_symbol(&self) -> Option<&Symbol> {
        match *self.0 {
            ExprKind::Normal(ref v) => v.try_head_symbol(),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(*self.0, ExprKind::String(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(*self.0, ExprKind::Integer(_))
    }

    pub fn is_real(&self) -> bool {
        matches!(*self.0, ExprKind::Real(_))
    }

    pub fn is_symbol(&self) -> bool {
        matches!(*self.0, ExprKind::Symbol(_))
    }

    pub fn is_normal(&self) -> bool {
        matches!(*self.0, ExprKind::Normal(_))
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

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
