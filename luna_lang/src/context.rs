use crate::builtins::register_builtins;
use crate::symbol::Symbol;
use crate::{Attributes, BuiltinFn, BuiltinFnMut, Expr};
use std::collections::HashMap;

pub struct Context {
    attributes: HashMap<Symbol, Attributes>,
    definitions: HashMap<Symbol, SymbolDefinition>,
    state_version: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            definitions: HashMap::new(),
            state_version: 0,
        }
    }

    pub fn new_global_context() -> Self {
        let mut context = Self {
            attributes: HashMap::new(),
            definitions: HashMap::new(),
            state_version: 0,
        };

        register_builtins(&mut context);

        // context.set_verbosity(1);
        context
    }

    pub fn get_attributes(&self, symbol: &Symbol) -> Attributes {
        match self.attributes.get(symbol) {
            None => Attributes::new(),
            Some(attributes) => *attributes,
        }
    }

    pub fn set_attributes(
        &mut self,
        symbol: &Symbol,
        new_attributes: Attributes,
    ) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.attributes_read_only() {
            return Err(format!("Symbol '{}' has read-only attributes", symbol));
        }

        self.attributes.insert(symbol.clone(), new_attributes);
        Ok(())
    }

    pub fn get_definition(&self, symbol: &Symbol) -> Option<&SymbolDefinition> {
        self.definitions.get(&symbol)
    }

    pub fn get_definition_mut(&mut self, symbol: &Symbol) -> &mut SymbolDefinition {
        self.definitions
            .entry(symbol.clone())
            .or_insert_with(|| SymbolDefinition::new())
    }

    pub fn set_own_value(&mut self, symbol: &Symbol, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let definition = self.get_definition_mut(symbol);
        if definition.own_values.contains(&value) {
            return Ok(());
        }

        definition.own_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn set_up_value(&mut self, symbol: &Symbol, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let definition = self.get_definition_mut(symbol);
        if definition.up_values.contains(&value) {
            return Ok(());
        }

        definition.up_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn set_down_value(&mut self, symbol: &Symbol, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let definition = self.get_definition_mut(symbol);
        if definition.down_values.contains(&value) {
            return Ok(());
        }

        definition.down_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn set_sub_value(&mut self, symbol: &Symbol, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let definition = self.get_definition_mut(symbol);
        if definition.sub_values.contains(&value) {
            return Ok(());
        }

        definition.sub_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn clear_symbol(&mut self, symbol: &Symbol) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol {} is read-only", symbol));
        }

        self.attributes.remove(&symbol);
        self.definitions.remove(&symbol);
        self.state_version += 1;

        Ok(())
    }
}

/// A `SymbolDefinition` contains all the transformation rules that apply to a given symbol.
pub struct SymbolDefinition {
    /// OwnValues define how the symbol appearing alone should be evaluated.
    /// They have the form `x :> expr` or `x = expr`.
    pub own_values: Vec<SymbolValue>,

    /// UpValues define how M-expressions having the symbol as an argument should be evaluated.
    /// They typically have the form `f[pattern, g[pattern], pattern] :> expr`.
    /// UpValues are applied before DownValues.
    pub up_values: Vec<SymbolValue>,

    /// DownValues define how M-expressions having the symbol as their head should be evaluated.
    /// They typically have the form `f[pattern] :> expr`.
    pub down_values: Vec<SymbolValue>,

    /// SubValues define how M-expressions having an M-expression with the symbol as a head should be evaluated.
    /// They typically have the form `f[pat][pat] :> exp`.
    pub sub_values: Vec<SymbolValue>,
}

impl SymbolDefinition {
    pub fn new() -> Self {
        Self {
            own_values: Vec::new(),
            up_values: Vec::new(),
            down_values: Vec::new(),
            sub_values: Vec::new(),
        }
    }
}

/// A `SymbolValue` is a wrapper for `RuleDelayed` used for storing the rule in a symbol table
/// as an own/up/down/sub value.
/// The wrapper provides convenience methods and stores the expression that originally created the value.
pub enum SymbolValue {
    Definitions {
        pattern: Expr,
        condition: Option<Expr>,
        ground: Expr,
    },
    BuiltIn {
        pattern: Expr,
        condition: Option<Expr>,
        built_in: BuiltinFn,
    },
    BuiltInMut {
        pattern: Expr,
        condition: Option<Expr>,
        built_in: BuiltinFnMut,
    },
}

// TODO: The structure of `SymbolValue` is not ideal for comparisons as it contains function
//  pointers. Ideally this will be split into two parts. One for the match pattern + condition,
//  and another containing the actual rule or builtin.

impl Eq for SymbolValue {}

impl PartialEq for SymbolValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Definitions {
                    pattern: p1,
                    condition: c1,
                    ground: g1,
                },
                Self::Definitions {
                    pattern: p2,
                    condition: c2,
                    ground: g2,
                },
            ) => p1 == p2 && c1 == c2 && g1 == g2,

            (
                Self::BuiltIn {
                    pattern: p1,
                    condition: c1,
                    built_in: _b1,
                },
                Self::BuiltIn {
                    pattern: p2,
                    condition: c2,
                    built_in: _b2,
                },
            ) => {
                p1 == p2 && c1 == c2 //&& (b1 as *const _ == b2 as *const _)
            }

            (
                Self::BuiltInMut {
                    pattern: p1,
                    condition: c1,
                    built_in: _b1,
                },
                Self::BuiltInMut {
                    pattern: p2,
                    condition: c2,
                    built_in: _b2,
                },
            ) => {
                p1 == p2 && c1 == c2 //&& (b1 as *const _ == b2 as *const _)
            }

            _ => false,
        }
    }
}
