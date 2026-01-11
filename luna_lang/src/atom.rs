use crate::abstractions::{BigFloat, BigInteger, IString};
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    String(IString),
    Integer(BigInteger),
    Real(BigFloat),
    Symbol(Symbol),
    SExpression(SExpression),
}

impl Atom {
    // pub fn string(string: impl Into<IString>) -> Self {
    //     Self::String(string.into())
    // }
    //
    // pub fn symbol(symbol: impl Into<IString>) -> Self {
    //     Self::Symbol(symbol.into())
    // }

    /// Gives the symbol under which the properties of this expression would be stored in the
    /// symbol table.
    pub fn name(&self) -> Option<Symbol> {
        match self {
            Atom::SExpression(_) => self.head().as_symbol().cloned(),
            Atom::Symbol(name) => Some(*name),
            _ => None,
        }
    }

    pub fn head(&self) -> Self {
        match self {
            Self::String(_) => Symbol::new("String").into(),
            Self::Integer(_) => Symbol::new("Integer").into(),
            Self::Real(_) => Symbol::new("Real").into(),
            Self::Symbol(_) => Symbol::new("Symbol").into(),
            Self::SExpression(sexpr) => sexpr.head(),
        }
    }

    pub fn has_head(&self, head: impl Into<Atom>) -> bool {
        self.head() == head.into()
    }

    pub fn parts(&self) -> &[Self] {
        match self {
            Self::SExpression(sexpr) => sexpr.parts(),
            _ => &[],
        }
    }

    pub fn part(&self, idx: usize) -> Option<&Self> {
        match self {
            Self::SExpression(sexpr) => sexpr.part(idx),
            _ => None,
        }
    }

    /// Returns the length of the expression not counting the head.
    /// Only S-Expressions can have nonzero length.
    pub fn len(&self) -> usize {
        match self {
            Self::SExpression(sexpr) => sexpr.len(),
            _ => 0,
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
        matches!(self, Self::SExpression(_))
    }

    // TODO: Figure out a nice naming pattern here

    pub fn as_symbol(&self) -> Option<&Symbol> {
        match self {
            Self::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }

    pub fn as_sexpr_with_head(&self, head: impl Into<Atom>) -> Option<&SExpression> {
        if let Atom::SExpression(sexpr) = self {
            if sexpr.head() == head.into() {
                return Some(sexpr);
            }
        }

        None
    }
}

// TODO: This is unsafe
impl Eq for Atom {}

// TODO: This is grim - figure out a better way
impl Hash for Atom {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Atom::String(v) => {
                hasher.write(&[102, 206, 57, 172, 207, 100, 198, 133]);
                v.hash(hasher);
            }

            Atom::Integer(v) => {
                hasher.write(&[242, 99, 84, 113, 102, 46, 118, 94]);
                v.to_string().hash(hasher);
            }

            Atom::Real(v) => {
                hasher.write(&[195, 244, 76, 249, 227, 115, 88, 251]);
                v.to_string().hash(hasher);
            }

            Atom::Symbol(v) => {
                hasher.write(&[107, 10, 247, 23, 33, 221, 163, 156]);
                v.hash(hasher);
            }

            Atom::SExpression(v) => {
                hasher.write(&[72, 5, 244, 86, 5, 210, 69, 30]);
                for part in v.parts() {
                    part.hash(hasher);
                }
            }
        }
    }
}

// TODO: Implement a simple display method
impl fmt::Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl From<Symbol> for Atom {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}

impl From<SExpression> for Atom {
    fn from(value: SExpression) -> Self {
        Self::SExpression(value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(IString);

impl Symbol {
    pub fn new(name: impl Into<IString>) -> Self {
        Self(name.into())
    }
}

impl From<Symbol> for IString {
    fn from(value: Symbol) -> Self {
        value.0
    }
}

impl From<IString> for Symbol {
    fn from(value: IString) -> Self {
        Self(value)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Self(IString::from(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SExpression(Rc<Vec<Atom>>);

impl SExpression {
    pub fn new(head: impl Into<Atom>, children: impl Into<Vec<Atom>>) -> Self {
        let mut children = children.into();

        let mut new_children = Vec::with_capacity(children.len() + 1);
        new_children.push(head.into());
        new_children.append(&mut children);

        Self(Rc::new(new_children))
    }

    pub fn apply0(head: impl Into<Atom>) -> Self {
        Self(Rc::new(vec![head.into()]))
    }

    pub fn apply1(head: impl Into<Atom>, a: impl Into<Atom>) -> Self {
        Self(Rc::new(vec![head.into(), a.into()]))
    }

    pub fn apply2(head: impl Into<Atom>, a: impl Into<Atom>, b: impl Into<Atom>) -> Self {
        Self(Rc::new(vec![head.into(), a.into(), b.into()]))
    }

    pub fn head(&self) -> Atom {
        self.0[0].clone()
    }

    pub fn parts(&self) -> &[Atom] {
        &self.0
    }

    pub fn part(&self, idx: usize) -> Option<&Atom> {
        self.0.get(idx)
    }

    pub fn len(&self) -> usize {
        // Do not count the head
        self.0.len() - 1
    }
}
