mod attributes;
mod head;
mod plus;
mod set;

use crate::ast::Expr;
use crate::eval::EvalError;
use crate::symbol::{Attributes, SymbolDef};
use std::sync::LazyLock;

pub use attributes::*;
pub use head::*;
pub use plus::*;
pub use set::*;

pub static SEQUENCE: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("Sequence").with_attributes(Attributes::PROTECTED));

pub static SYMBOL: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("Symbol").with_attributes(Attributes::PROTECTED));

pub static INTEGER: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("Integer").with_attributes(Attributes::PROTECTED));

pub static BLANK: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("Blank").with_attributes(Attributes::PROTECTED));

pub static BLANK_SEQUENCE: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("BlankSequence").with_attributes(Attributes::PROTECTED));

pub static BLANK_NULL_SEQUENCE: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("BlankNullSequence").with_attributes(Attributes::PROTECTED));

pub static PATTERN: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("Pattern").with_attributes(Attributes::PROTECTED | Attributes::HOLD_FIRST)
});

pub static LIST: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("List").with_attributes(Attributes::PROTECTED | Attributes::LOCKED)
});

pub static TRUE: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("True").with_attributes(Attributes::LOCKED | Attributes::PROTECTED)
});

pub static FALSE: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("False").with_attributes(Attributes::LOCKED | Attributes::PROTECTED)
});

pub static NULL: LazyLock<SymbolDef> =
    LazyLock::new(|| SymbolDef::new("Null").with_attributes(Attributes::PROTECTED));

pub static IF: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("If")
        .with_attributes(Attributes::PROTECTED | Attributes::HOLD_REST)
        .with_builtin(|args, env| {
            if args.len() != 3 {
                return Err(EvalError::Arity);
            }

            match args[0] {
                Expr::Symbol(symbol_id) if symbol_id == TRUE.symbol_id() => Ok(args[1].clone()),
                Expr::Symbol(symbol_id) if symbol_id == FALSE.symbol_id() => Ok(args[2].clone()),
                _ => Ok(Expr::apply_symbol(IF.symbol_id(), args)),
            }
        })
});

pub static APPLY: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("Apply")
        .with_attributes(Attributes::PROTECTED | Attributes::HOLD_REST)
        .with_builtin(|args, _| {
            if args.len() != 2 {
                return Err(EvalError::Arity);
            }

            let f = args[0].clone();
            let expr = args[1].clone();

            match expr {
                Expr::Apply(_, args) => Ok(Expr::apply(f, args)),
                _ => Ok(expr),
            }
        })
});
