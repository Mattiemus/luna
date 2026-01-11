use std::ops::Add;

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]

pub enum Attribute {
    /// Symbol is read only and cannot be changed.
    ReadOnly,

    /// Attributes of the symbol cannot be changed.
    AttributesReadOnly,

    /// Application of the symbol satisfies the commutative property.
    ///
    /// This indicates that the order of the inputs do not matter.
    /// For example `f[x, y]` and `f[y, x]` are equivalent.
    ///
    /// During evaluation inputs will be sorted.
    /// For example `f[c, b, a]` will be evaluated into `f[a, b, c]`.
    Commutative,

    /// Application of the symbol satisfies the associative property.
    ///
    /// This indicates that nested applications of the symbol can be performed in any order.
    /// For example `f[x, f[y, z]]` and `f[f[x, y], z]` are equivalent.
    ///
    /// During evaluation inputs will be flattened.
    /// For example `f[x, f[y, z]]` will be evaluated into `f[x, y, z]`.
    Associative,
}

impl Add<Attribute> for Attribute {
    type Output = Attributes;

    fn add(self, other: Attribute) -> Self::Output {
        let mut out: Attributes = self.into();
        out.set(other);
        out
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Attributes(u32);

impl Attributes {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn has(&self, attribute: Attribute) -> bool {
        (self.0 & (1 << attribute as u32)) != 0
    }

    pub fn set(&mut self, attribute: Attribute) {
        self.0 = self.0 | (1 << attribute as u32)
    }

    pub fn set_all(&mut self, attributes: Attributes) {
        self.0 |= attributes.0;
    }

    pub fn read_only(&self) -> bool {
        self.has(Attribute::ReadOnly)
    }

    pub fn attributes_read_only(&self) -> bool {
        self.has(Attribute::AttributesReadOnly)
    }

    pub fn commutative(&self) -> bool {
        self.has(Attribute::Commutative)
    }

    pub fn associative(&self) -> bool {
        self.has(Attribute::Associative)
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self(0)
    }
}

impl Add for Attributes {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.set_all(other);
        self
    }
}

impl Add<Attribute> for Attributes {
    type Output = Self;

    fn add(mut self, other: Attribute) -> Self {
        self.set(other);
        self
    }
}

impl From<Attribute> for Attributes {
    fn from(attribute: Attribute) -> Self {
        Self(1u32 << attribute as u32)
    }
}
