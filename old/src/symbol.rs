use crate::ast::Expr;
use crate::env::BuiltinFn;
use crate::pattern::Rule;
use bitflags::bitflags;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SymbolId(usize);

impl SymbolId {
    pub fn next() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let next_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self(next_id)
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Attributes: u32 {
        const HOLD_ALL     = 0b0001;
        const HOLD_FIRST   = 0b0010;
        const HOLD_REST    = 0b0100;
        const PROTECTED    = 0b1000;

        const FLAT         = 0b0001_0000;
        const ORDERLESS    = 0b0010_0000;
        const ONE_IDENTITY = 0b0100_0000;
        const LOCKED       = 0b1000_0000;

        const SEQUENCE_HOLD    = 0b0001_0000_0000;
        const LISTABLE         = 0b0010_0000_0000;
        const NUMERIC_FUNCTION = 0b0100_0000_0000;
    }
}

#[derive(Debug, Clone)]
pub struct SymbolDef {
    symbol_id: SymbolId,
    name: String,
    pub attributes: Attributes,
    pub value: Option<Expr>,
    pub downvalues: Vec<Rule>,
    pub upvalues: Vec<Rule>,
    pub builtin: Option<BuiltinFn>,
}

impl SymbolDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            symbol_id: SymbolId::next(),
            name: name.into(),
            attributes: Attributes::empty(),
            value: None,
            downvalues: Vec::new(),
            upvalues: Vec::new(),
            builtin: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol_id(&self) -> SymbolId {
        self.symbol_id
    }

    pub fn with_attributes(self, attributes: Attributes) -> Self {
        Self { attributes, ..self }
    }

    pub fn with_builtin(self, builtin_fn: BuiltinFn) -> Self {
        Self {
            builtin: Some(builtin_fn),
            ..self
        }
    }
}
