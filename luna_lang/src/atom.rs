use crate::abstractions::{BigFloat, BigInteger, IString};
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};

#[derive(Clone, Debug)]
pub enum Atom {
    String(IString),
    Integer(BigInteger),
    Real(BigFloat),
    Symbol(IString),
    Expr(Arc<Vec<Atom>>),
}

impl Atom {
    pub fn string(string: impl Into<IString>) -> Self {
        Self::String(string.into())
    }

    // integer
    // real

    pub fn symbol(symbol: impl Into<IString>) -> Self {
        Self::Symbol(symbol.into())
    }

    pub fn expr(head: impl Into<Self>, children: impl Into<Vec<Self>>) -> Self {
        let mut children = children.into();

        let mut new_children = Vec::with_capacity(children.len() + 1);
        new_children.push(head.into());
        new_children.append(&mut children);

        Self::Expr(Arc::new(new_children))
    }

    pub fn apply0(head: impl Into<Self>) -> Self {
        Self::Expr(Arc::new(vec![head.into()]))
    }

    pub fn apply1(head: impl Into<Self>, a: impl Into<Self>) -> Self {
        Self::Expr(Arc::new(vec![head.into(), a.into()]))
    }

    pub fn apply2(head: impl Into<Self>, a: impl Into<Self>, b: impl Into<Self>) -> Self {
        Self::Expr(Arc::new(vec![head.into(), a.into(), b.into()]))
    }

    /// Gives the symbol under which the properties of this expression would be stored in the
    /// symbol table.
    pub fn name(&self) -> Option<IString> {
        match self {
            Self::Expr(expr) => match expr[0] {
                Self::Symbol(head) => Some(head),
                _ => None,
            },
            Self::Symbol(name) => Some(*name),
            _ => None,
        }
    }

    pub fn head(&self) -> Self {
        match self {
            Self::Symbol(_) => Self::symbol("Symbol"),
            Self::String(_) => Self::symbol("String"),
            Self::Integer(_) => Self::symbol("Integer"),
            Self::Real(_) => Self::symbol("Real"),
            Self::Expr(expr) => expr[0].clone(),
        }
    }

    pub fn has_symbol_head(&self, head: impl Into<IString>) -> bool {
        match self {
            Self::Symbol(_) => head.into() == "Symbol",
            Self::String(_) => head.into() == "String",
            Self::Integer(_) => head.into() == "Integer",
            Self::Real(_) => head.into() == "Real",
            Self::Expr(expr) => match expr[0] {
                Self::Symbol(expr_head) => expr_head == head.into(),
                _ => false,
            },
        }
    }

    pub fn children(&self) -> &[Self] {
        match self {
            Self::Symbol(_) => &[],
            Self::String(_) => &[],
            Self::Integer(_) => &[],
            Self::Real(_) => &[],
            Self::Expr(expr) => &expr[1..],
        }
    }

    /// Returns the length of the expression not counting the head.
    /// Only expressions can have nonzero length.
    pub fn len(&self) -> usize {
        match self {
            Self::Expr(expr) => expr.len() - 1,
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Expr(expr) => expr.len() == 1,
            _ => true,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    pub fn is_real(&self) -> bool {
        matches!(self, Self::Real(_))
    }

    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::Expr(_))
    }

    pub fn try_as_string(&self) -> Option<IString> {
        match self {
            Self::String(string) => Some(*string),
            _ => None,
        }
    }

    pub fn try_as_integer(&self) -> Option<BigInteger> {
        match self {
            Self::Integer(integer) => Some(integer.clone()),
            _ => None,
        }
    }

    pub fn try_as_real(&self) -> Option<BigFloat> {
        match self {
            Self::Real(float) => Some(float.clone()),
            _ => None,
        }
    }

    pub fn try_as_symbol(&self) -> Option<IString> {
        match self {
            Self::Symbol(symbol) => Some(*symbol),
            _ => None,
        }
    }

    pub fn try_as_expr(&self) -> Option<&[Self]> {
        match self {
            Self::Expr(expr) => Some(expr),
            _ => None,
        }
    }

    pub fn try_as_apply_symbol(&self, head: impl Into<IString>) -> Option<&[Self]> {
        match self {
            Self::Expr(expr) => match expr[0] {
                Self::Symbol(expr_head) if expr_head == head.into() => Some(expr),
                _ => None,
            },
            _ => None,
        }
    }
}

impl Eq for Atom {}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Real(a), Self::Real(b)) => a.total_cmp(b) == Ordering::Equal,
            (Self::Symbol(a), Self::Symbol(b)) => a == b,
            (Self::Expr(a), Self::Expr(b)) => a == b,
            _ => false,
        }
    }
}

impl Ord for Atom {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::String(a), Self::String(b)) => a.cmp(b),
            (Self::String(_), _) => Ordering::Less,
            (_, Self::String(_)) => Ordering::Greater,

            (Self::Integer(a), Self::Integer(b)) => a.cmp(b),
            (Self::Integer(_), _) => Ordering::Less,
            (_, Self::Integer(_)) => Ordering::Greater,

            (Self::Real(a), Self::Real(b)) => a.total_cmp(b),
            (Self::Real(_), _) => Ordering::Less,
            (_, Self::Real(_)) => Ordering::Greater,

            (Self::Symbol(a), Self::Symbol(b)) => a.cmp(b),
            (Self::Symbol(_), _) => Ordering::Less,
            (_, Self::Symbol(_)) => Ordering::Greater,

            (Self::Expr(a), Self::Expr(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Atom {
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
                v.to_string().hash(hasher);
            }

            Self::Real(v) => {
                hasher.write(&REAL_PREFIX);
                v.to_string().hash(hasher);
            }

            Self::Symbol(v) => {
                hasher.write(&SYMBOL_PREFIX);
                v.hash(hasher);
            }

            Self::Expr(v) => {
                hasher.write(&EXPR_PREFIX);
                for part in v.as_ref() {
                    part.hash(hasher);
                }
            }
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Atom::String(v) => write!(f, "\"{}\"", v),
            Atom::Integer(v) => write!(f, "{}", v),
            Atom::Real(v) => write!(f, "{}", v),
            Atom::Symbol(v) => write!(f, "{}", v),
            Atom::Expr(expr) => {
                let mut child_iter = expr.iter();
                let head = child_iter.next().unwrap();

                write!(f, "{}[", head)?;

                for (i, arg) in child_iter.enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{}", arg)?;
                }

                write!(f, "]")
            }
        }
    }
}
