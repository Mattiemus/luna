use crate::abstractions::{BigFloat, BigInteger, IString};
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};

#[derive(Clone, PartialEq, Debug)]
pub enum Atom {
    String(IString),
    Integer(BigInteger),
    Real(BigFloat),
    Symbol(IString),
    SExpr(Arc<Vec<Atom>>),
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

    pub fn sexpr(head: impl Into<Self>, children: impl Into<Vec<Self>>) -> Self {
        let mut children = children.into();

        let mut new_children = Vec::with_capacity(children.len() + 1);
        new_children.push(head.into());
        new_children.append(&mut children);

        Self::SExpr(Arc::new(new_children))
    }

    pub fn apply0(head: impl Into<Self>) -> Self {
        Self::SExpr(Arc::new(vec![head.into()]))
    }

    pub fn apply1(head: impl Into<Self>, a: impl Into<Self>) -> Self {
        Self::SExpr(Arc::new(vec![head.into(), a.into()]))
    }

    pub fn apply2(head: impl Into<Self>, a: impl Into<Self>, b: impl Into<Self>) -> Self {
        Self::SExpr(Arc::new(vec![head.into(), a.into(), b.into()]))
    }

    /// Gives the symbol under which the properties of this expression would be stored in the
    /// symbol table.
    pub fn name(&self) -> Option<IString> {
        match self {
            Self::SExpr(_) => self.head().as_symbol(),
            Self::Symbol(name) => Some(*name),
            _ => None,
        }
    }

    pub fn head(&self) -> Self {
        match self {
            Self::Symbol(_) => Atom::symbol("Symbol"),
            Self::String(_) => Atom::symbol("String"),
            Self::Integer(_) => Atom::symbol("Integer"),
            Self::Real(_) => Atom::symbol("Real"),
            Self::SExpr(sexpr) => sexpr[0].clone(),
        }
    }

    pub fn has_symbol_head(&self, head: impl Into<IString>) -> bool {
        match self.head() {
            Self::Symbol(symbol) => symbol == head.into(),
            _ => false,
        }
    }

    pub fn parts(&self) -> &[Self] {
        static SYMBOL_PARTS: LazyLock<Vec<Atom>> = LazyLock::new(|| vec![Atom::symbol("Symbol")]);
        static STRING_PARTS: LazyLock<Vec<Atom>> = LazyLock::new(|| vec![Atom::symbol("String")]);
        static INTEGER_PARTS: LazyLock<Vec<Atom>> = LazyLock::new(|| vec![Atom::symbol("Integer")]);
        static REAL_PARTS: LazyLock<Vec<Atom>> = LazyLock::new(|| vec![Atom::symbol("Real")]);

        match self {
            Self::Symbol(_) => &SYMBOL_PARTS,
            Self::String(_) => &STRING_PARTS,
            Self::Integer(_) => &INTEGER_PARTS,
            Self::Real(_) => &REAL_PARTS,
            Self::SExpr(sexpr) => sexpr,
        }
    }

    /// Returns the length of the expression not counting the head.
    /// Only S-Expressions can have nonzero length.
    pub fn len(&self) -> usize {
        match self {
            Self::SExpr(sexpr) => sexpr.len() - 1,
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::SExpr(sexpr) => sexpr.len() == 1,
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

    pub fn is_sexpr(&self) -> bool {
        matches!(self, Self::SExpr(_))
    }

    // TODO: Figure out a nice naming pattern here

    pub fn as_symbol(&self) -> Option<IString> {
        match self {
            Self::Symbol(symbol) => Some(*symbol),
            _ => None,
        }
    }
}

// TODO: This is unsafe
impl Eq for Atom {}

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
        const SEXPR_PREFIX: [u8; 8] = [72, 5, 244, 86, 5, 210, 69, 30];

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

            Self::SExpr(v) => {
                hasher.write(&SEXPR_PREFIX);
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
            Atom::SExpr(sexpr) => {
                let mut child_iter = sexpr.iter();
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
