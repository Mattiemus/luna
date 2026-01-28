use crate::Symbol;
use std::cell::LazyCell;

pub struct BuiltinSymbols {
    pub Sequence: Symbol,
    pub Plus: Symbol,
    pub Blank: Symbol,
    pub BlankSequence: Symbol,
    pub BlankNullSequence: Symbol,
    pub Pattern: Symbol,
    pub Condition: Symbol,
    pub Set: Symbol,
    pub SetDelayed: Symbol,
    pub Head: Symbol,
    pub Hold: Symbol,
    pub Null: Symbol,
    pub String: Symbol,
    pub Integer: Symbol,
    pub Real: Symbol,
    pub Symbol: Symbol,
    pub Subtract: Symbol,
    pub Times: Symbol,
}

pub const BUILTIN_SYMBOLS: LazyCell<BuiltinSymbols> = LazyCell::new(|| BuiltinSymbols {
    Sequence: Symbol::new("Sequence"),
    Plus: Symbol::new("Plus"),
    Blank: Symbol::new("Blank"),
    BlankSequence: Symbol::new("BlankSequence"),
    BlankNullSequence: Symbol::new("BlankNullSequence"),
    Pattern: Symbol::new("Pattern"),
    Condition: Symbol::new("Condition"),
    Set: Symbol::new("Set"),
    SetDelayed: Symbol::new("SetDelayed"),
    Head: Symbol::new("Head"),
    Hold: Symbol::new("Hold"),
    Null: Symbol::new("Null"),
    String: Symbol::new("String"),
    Integer: Symbol::new("Integer"),
    Real: Symbol::new("Real"),
    Symbol: Symbol::new("Symbol"),
    Subtract: Symbol::new("Subtract"),
    Times: Symbol::new("Times"),
});

#[macro_export]
macro_rules! sym {
    ($name:ident) => {
        crate::BUILTIN_SYMBOLS.$name.clone()
    };
}
