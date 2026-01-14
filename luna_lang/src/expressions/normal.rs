use crate::Expr;
use crate::Symbol;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Represents a normal expression of the form `f[...]`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Normal {
    head: Expr,
    elements: Vec<Expr>,
}

impl Normal {
    pub fn new(head: impl Into<Expr>, elements: impl Into<Vec<Expr>>) -> Self {
        Self {
            head: head.into(),
            elements: elements.into(),
        }
    }

    pub fn head(&self) -> &Expr {
        &self.head
    }

    pub fn elements(&self) -> &[Expr] {
        &self.elements
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn has_head(&self, head: &Symbol) -> bool {
        match self.head.try_symbol() {
            Some(head_symbol) => head_symbol == head,
            None => false,
        }
    }

    pub fn part(&self, idx: usize) -> Option<&Expr> {
        self.elements.get(idx)
    }

    pub fn try_head_symbol(&self) -> Option<&Symbol> {
        self.head.try_symbol()
    }
}

impl Hash for Normal {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.head.hash(hasher);

        for elem in &self.elements {
            elem.hash(hasher);
        }
    }
}

impl fmt::Display for Normal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[", self.head)?;

        for (idx, elem) in self.elements.iter().enumerate() {
            write!(f, "{}", elem)?;

            if idx != self.elements.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}
