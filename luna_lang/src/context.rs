use crate::abstractions::IString;
use crate::builtins::register_builtins;
use crate::{Atom, Attributes, BuiltinFn, BuiltinFnMut};
use std::collections::HashMap;

pub struct Context {
    name: IString,
    attributes: HashMap<IString, Attributes>,
    symbols: HashMap<IString, SymbolRecord>,
    state_version: usize,
}

impl Context {
    pub fn new_global_context() -> Self {
        let mut context = Self {
            name: IString::from("Global"),
            attributes: HashMap::new(),
            symbols: HashMap::new(),
            state_version: 0,
        };

        register_builtins(&mut context);

        // context.set_verbosity(1);
        context
    }

    pub fn get_attributes(&self, symbol: IString) -> Attributes {
        match self.attributes.get(&symbol) {
            None => Attributes::empty(),
            Some(attributes) => *attributes,
        }
    }

    pub fn set_attributes(
        &mut self,
        symbol: IString,
        new_attributes: Attributes,
    ) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.contains(Attributes::ATTRIBUTES_READ_ONLY) {
            return Err(format!("Symbol '{}' has read-only attributes", symbol));
        }

        self.attributes.insert(symbol, new_attributes);
        Ok(())
    }

    pub fn get_symbol(&self, symbol: IString) -> Option<&SymbolRecord> {
        self.symbols.get(&symbol)
    }

    pub fn get_symbol_mut(&mut self, symbol: IString) -> &mut SymbolRecord {
        self.symbols
            .entry(symbol)
            .or_insert_with(|| SymbolRecord::empty())
    }

    pub fn set_own_value(&mut self, symbol: IString, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.contains(Attributes::READ_ONLY) {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let record = self.get_symbol_mut(symbol);
        if record.own_values.contains(&value) {
            return Ok(());
        }

        record.own_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn set_up_value(&mut self, symbol: IString, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.contains(Attributes::READ_ONLY) {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let record = self.get_symbol_mut(symbol);
        if record.up_values.contains(&value) {
            return Ok(());
        }

        record.up_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn set_down_value(&mut self, symbol: IString, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.contains(Attributes::READ_ONLY) {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let record = self.get_symbol_mut(symbol);
        if record.down_values.contains(&value) {
            return Ok(());
        }

        record.down_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn set_sub_value(&mut self, symbol: IString, value: SymbolValue) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.contains(Attributes::READ_ONLY) {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let record = self.get_symbol_mut(symbol);
        if record.sub_values.contains(&value) {
            return Ok(());
        }

        record.sub_values.push(value);
        self.state_version += 1;

        Ok(())
    }

    pub fn clear_symbol(&mut self, symbol: IString) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.contains(Attributes::READ_ONLY) || attributes.contains(Attributes::PROTECTED)
        {
            return Err(format!("Symbol {} is read-only", symbol));
        }

        self.attributes.remove(&symbol);
        self.symbols.remove(&symbol);
        self.state_version += 1;

        Ok(())
    }
}

pub struct SymbolRecord {
    /// OwnValues define how the symbol appearing alone should be evaluated.
    /// They have the form `x :> expr` or `x = expr`.
    pub own_values: Vec<SymbolValue>,

    /// UpValues define how M-expressions having the symbol as an argument should be evaluated.
    /// They typically have the form `f[pattern, g[pattern], pattern] :> expr`
    /// UpValues are applied before DownValues.
    pub up_values: Vec<SymbolValue>,

    /// DownValues define how M-expressions having the symbol as their head should be evaluated.
    /// They typically have the form `f[pattern] :> expr`
    pub down_values: Vec<SymbolValue>,

    /// SubValues define how M-expressions having an M-expression with the symbol as a head should be evaluated.
    /// They typically have the form `f[pat][pat] :> exp`.
    pub sub_values: Vec<SymbolValue>,
}

impl SymbolRecord {
    pub fn empty() -> Self {
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
#[derive(PartialEq)]
pub enum SymbolValue {
    Definitions {
        /// The original (sub)expression used to create this `SymbolValue`.
        definition: Atom,
        /// Treated as if wrapped in HoldPattern
        lhs: Atom,
        rhs: Atom,
        condition: Option<Atom>,
    },
    BuiltIn {
        pattern: Atom,
        condition: Option<Atom>,
        built_in: BuiltinFn,
    },
    BuiltInMut {
        pattern: Atom,
        condition: Option<Atom>,
        built_in: BuiltinFnMut,
    },
}
