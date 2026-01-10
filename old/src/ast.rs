use crate::builtins::{BLANK, BLANK_NULL_SEQUENCE, BLANK_SEQUENCE, INTEGER, PATTERN, SYMBOL};
use crate::symbol::SymbolId;
use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expr {
    Symbol(SymbolId),
    Integer(BigInt),
    Apply(Box<Expr>, Vec<Expr>),
}

impl Expr {
    pub fn symbol(symbol_id: SymbolId) -> Self {
        Self::Symbol(symbol_id)
    }

    pub fn integer(value: impl Into<BigInt>) -> Self {
        Self::Integer(value.into())
    }

    pub fn apply(head: Expr, args: impl Into<Vec<Expr>>) -> Self {
        Self::Apply(Box::new(head), args.into())
    }

    pub fn apply_symbol(symbol_id: SymbolId, args: impl Into<Vec<Expr>>) -> Self {
        Self::apply(Self::symbol(symbol_id), args)
    }

    pub fn head(&self) -> Self {
        match self {
            Self::Symbol(_) => Self::symbol(SYMBOL.symbol_id()),
            Self::Integer(_) => Self::symbol(INTEGER.symbol_id()),
            Self::Apply(head, _) => *head.clone(),
        }
    }

    pub fn head_symbol(&self) -> Option<SymbolId> {
        match self {
            Expr::Apply(head, _) => match &**head {
                Expr::Symbol(symbol_id) => Some(*symbol_id),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_apply(&self, symbol_id: SymbolId) -> Option<&Vec<Expr>> {
        match self {
            Expr::Apply(head, args) => match &**head {
                Expr::Symbol(head_symbol_id) if *head_symbol_id == symbol_id => Some(args),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn collect_symbols(expr: &Expr, out: &mut Vec<SymbolId>) {
    match expr {
        Expr::Symbol(symbol_id) => out.push(*symbol_id),
        Expr::Apply(head, args) => {
            collect_symbols(&**head, out);
            for a in args {
                collect_symbols(a, out);
            }
        }
        _ => {}
    }
}

pub fn is_pattern(expr: &Expr) -> Option<(SymbolId, &Expr)> {
    if let Some(args) = expr.is_apply(PATTERN.symbol_id()) {
        // Pattern[x, expr]
        if args.len() == 2 {
            if let Expr::Symbol(symbol_id) = &args[0] {
                return Some((*symbol_id, &args[1]));
            }
        }
    }

    None
}

pub fn is_blank(expr: &Expr) -> Option<Option<&Expr>> {
    if let Some(args) = expr.is_apply(BLANK.symbol_id()) {
        return match args.len() {
            0 => Some(None),           // Blank[]
            1 => Some(Some(&args[0])), // Blank[h]
            _ => None,                 // Blank[...]
        };
    }

    None
}

pub fn is_blank_sequence(expr: &Expr) -> Option<Option<&Expr>> {
    if let Some(args) = expr.is_apply(BLANK_SEQUENCE.symbol_id()) {
        return match args.len() {
            0 => Some(None),           // BlankSequence[]
            1 => Some(Some(&args[0])), // BlankSequence[h]
            _ => None,                 // BlankSequence[...]
        };
    }

    None
}

pub fn is_blank_null_sequence(expr: &Expr) -> Option<Option<&Expr>> {
    if let Some(args) = expr.is_apply(BLANK_NULL_SEQUENCE.symbol_id()) {
        return match args.len() {
            0 => Some(None),           // BlankNullSequence[]
            1 => Some(Some(&args[0])), // BlankNullSequence[h]
            _ => None,                 // BlankNullSequence[...]
        };
    }

    None
}
