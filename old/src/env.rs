use crate::ast::Expr;
use crate::builtins::{
    APPLY, ATTRIBUTES, BLANK, BLANK_NULL_SEQUENCE, BLANK_SEQUENCE, FALSE, HEAD, IF, INTEGER, LIST,
    NULL, PATTERN, PLUS, SEQUENCE, SET, SYMBOL, TRUE,
};
use crate::eval::EvalError;
use crate::symbol::{SymbolDef, SymbolId};
use std::collections::HashMap;

pub type BuiltinFn = fn(Vec<Expr>, &mut Env) -> Result<Expr, EvalError>;

pub struct Env {
    symbols: HashMap<SymbolId, SymbolDef>,
    intern: HashMap<String, SymbolId>,
}

impl Env {
    pub fn new() -> Self {
        let mut env = Self {
            symbols: HashMap::new(),
            intern: HashMap::new(),
        };

        env.register(ATTRIBUTES.clone());
        env.register(HEAD.clone());
        env.register(PLUS.clone());
        env.register(SET.clone());

        // Unstructured (all currently in mod.rs)
        env.register(SYMBOL.clone());
        env.register(INTEGER.clone());
        env.register(SEQUENCE.clone());
        env.register(BLANK.clone());
        env.register(BLANK_SEQUENCE.clone());
        env.register(BLANK_NULL_SEQUENCE.clone());
        env.register(PATTERN.clone());
        env.register(TRUE.clone());
        env.register(FALSE.clone());
        env.register(NULL.clone());
        env.register(IF.clone());
        env.register(APPLY.clone());
        env.register(LIST.clone());

        env
    }

    pub fn symbol_def(&self, symbol_id: SymbolId) -> &SymbolDef {
        self.symbols.get(&symbol_id).expect("could not find symbol")
    }

    pub fn symbol_def_mut(&mut self, symbol_id: SymbolId) -> &mut SymbolDef {
        self.symbols
            .get_mut(&symbol_id)
            .expect("could not find symbol")
    }

    pub fn intern(&mut self, name: &str) -> SymbolId {
        if let Some(&symbol_id) = self.intern.get(name) {
            return symbol_id;
        }

        self.register(SymbolDef::new(name))
    }

    pub fn register(&mut self, symbol_def: SymbolDef) -> SymbolId {
        let name = symbol_def.name();
        let id = symbol_def.symbol_id();

        self.intern.insert(name.to_string(), id);
        self.symbols.insert(id, symbol_def);

        id
    }
}
