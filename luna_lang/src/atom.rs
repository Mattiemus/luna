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
    Symbol(IString),
    SExpression(Rc<Vec<Atom>>),
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
                for part in v.as_ref() {
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

impl Atom {
    pub fn head(&self) -> Self {
        match self {
            Self::SExpression(children) => children[0].clone(),
            atom => Symbol::from_static_str(atom.kind().into()),
        }
    }

    pub fn kind(&self) -> AtomKind {
        match self {
            Self::String(_) => AtomKind::String,
            Self::Integer(_) => AtomKind::Integer,
            Self::Real(_) => AtomKind::Real,
            Self::Symbol(_) => AtomKind::Symbol,
            Self::SExpression(_) => AtomKind::SExpression,
        }
    }

    pub fn head_kind(&self) -> AtomKind {
        match self {
            Self::SExpression(children) => children[0].kind(),
            _ => AtomKind::Symbol, // The head of any non-function is a symbol.
        }
    }

    /// Gives the symbol (as an `IString`) under which the properties of this
    /// expression would be stored in the symbol table.
    pub fn name(&self) -> Option<IString> {
        match self {
            Self::SExpression(_) => match self.head() {
                Self::Symbol(name) => Some(name),
                _ => None,
            },
            Self::Symbol(name) => Some(*name),
            _ => None,
        }
    }

    /// Returns the length of the expression not counting the head.
    /// Only S-Expressions can have nonzero length.
    pub fn len(&self) -> usize {
        match self {
            Self::String(_) | Self::Integer(_) | Self::Real(_) | Self::Symbol(_) => 0,
            Self::SExpression(children) => children.len() - 1, // Don't count the head.
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub mod Symbol {
    use crate::Atom;
    use crate::abstractions::IString;

    pub fn new(name: IString) -> Atom {
        Atom::Symbol(name)
    }

    pub fn from_static_str(name: &'static str) -> Atom {
        Atom::Symbol(IString::from(name))
    }

    pub fn from_str(name: &str) -> Atom {
        Atom::Symbol(IString::from(name))
    }
}

pub(crate) mod SExpression {
    use crate::Atom;
    use crate::Symbol;
    use std::rc::Rc;

    pub(crate) fn new(head: Atom, mut children: Vec<Atom>) -> Atom {
        let mut new_children = Vec::with_capacity(children.len() + 1);
        new_children.push(head);
        new_children.append(&mut children);

        Atom::SExpression(Rc::new(new_children))
    }

    /// Creates an expression of the form `Blank[]`
    pub(crate) fn make_blank() -> Atom {
        Atom::SExpression(Rc::new(vec![Symbol::from_static_str("Blank")]))
    }

    /// Creates an expression of the form `Pattern[name, Blank[]]`.
    /// The provided `name` is turned into a symbol.
    pub(crate) fn make_pattern_blank(name: &'static str) -> Atom {
        Atom::SExpression(Rc::new(vec![
            Symbol::from_static_str("Pattern"),
            Symbol::from_static_str(name),
            make_blank(),
        ]))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AtomKind {
    String,
    Integer,
    Real,
    Symbol,
    SExpression,
}

impl From<AtomKind> for &'static str {
    fn from(value: AtomKind) -> Self {
        match value {
            AtomKind::String => "String",
            AtomKind::Integer => "Integer",
            AtomKind::Real => "Real",
            AtomKind::Symbol => "Symbol",
            AtomKind::SExpression => "SExpression",
        }
    }
}
