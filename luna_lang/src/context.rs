use crate::Symbol;
use crate::builtins::register_builtins;
use crate::{Attributes, BuiltinFn, BuiltinFnMut, Expr};
use std::collections::HashMap;

pub struct Context {
    definitions: HashMap<Symbol, SymbolDefinition>,
    state_version: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            state_version: 0,
        }
    }

    pub fn new_global_context() -> Self {
        let mut context = Self {
            definitions: HashMap::new(),
            state_version: 0,
        };

        register_builtins(&mut context);

        // context.set_verbosity(1);
        context
    }

    pub fn state_version(&self) -> usize {
        self.state_version
    }

    pub fn get_definition(&self, symbol: &Symbol) -> Option<&SymbolDefinition> {
        self.definitions.get(&symbol)
    }

    pub fn get_definition_mut(&mut self, symbol: &Symbol) -> &mut SymbolDefinition {
        self.definitions
            .entry(symbol.clone())
            .or_insert_with(|| SymbolDefinition::new())
    }

    pub fn get_attributes(&self, symbol: &Symbol) -> Attributes {
        self.get_definition(symbol)
            .map(|definition| definition.attributes)
            .unwrap_or_default()
    }

    pub fn set_attributes(
        &mut self,
        symbol: &Symbol,
        new_attributes: Attributes,
    ) -> Result<(), String> {
        let definition = self.get_definition_mut(symbol);
        if definition.attributes.attributes_read_only() {
            return Err(format!("Symbol '{}' has read-only attributes", symbol));
        }

        definition.attributes = new_attributes;
        Ok(())
    }

    pub fn get_values(&self, symbol: &Symbol, value_type: ValueType) -> Option<&SymbolValueSet> {
        let definition = self.get_definition(symbol)?;
        let values = definition.values(value_type);

        Some(values)
    }

    pub fn set_value(
        &mut self,
        symbol: &Symbol,
        value_type: ValueType,
        value: SymbolValue,
    ) -> Result<(), String> {
        let definition = self.get_definition_mut(symbol);
        if definition.attributes.read_only() {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let values = definition.values_mut(value_type);
        if let Some(_) = values.put(value) {
            self.state_version += 1;
        }

        Ok(())
    }

    pub fn clear_symbol(&mut self, symbol: &Symbol) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol {} is read-only", symbol));
        }

        self.definitions.remove(&symbol);
        self.state_version += 1;

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ValueType {
    OwnValue,
    UpValue,
    DownValue,
    SubValue,
}

/// A `SymbolDefinition` contains all the transformation rules that apply to a given symbol.
pub struct SymbolDefinition {
    /// Symbol attributes.
    attributes: Attributes,

    /// OwnValues define how the symbol appearing alone should be evaluated.
    /// They have the form `x :> expr` or `x = expr`.
    own_values: SymbolValueSet,

    /// UpValues define how M-expressions having the symbol as an argument should be evaluated.
    /// They typically have the form `f[pattern, g[pattern], pattern] :> expr`.
    /// UpValues are applied before DownValues.
    up_values: SymbolValueSet,

    /// DownValues define how M-expressions having the symbol as their head should be evaluated.
    /// They typically have the form `f[pattern] :> expr`.
    down_values: SymbolValueSet,

    /// SubValues define how M-expressions having an M-expression with the symbol as a head should be evaluated.
    /// They typically have the form `f[pat][pat] :> exp`.
    sub_values: SymbolValueSet,
}

impl SymbolDefinition {
    pub fn new() -> Self {
        Self {
            attributes: Attributes::empty(),
            own_values: SymbolValueSet::new(),
            up_values: SymbolValueSet::new(),
            down_values: SymbolValueSet::new(),
            sub_values: SymbolValueSet::new(),
        }
    }

    pub fn attributes(&self) -> Attributes {
        self.attributes
    }

    pub fn set_attributes(&mut self, attributes: Attributes) {
        self.attributes = attributes;
    }

    pub fn values(&self, value_type: ValueType) -> &SymbolValueSet {
        match value_type {
            ValueType::OwnValue => &self.own_values,
            ValueType::UpValue => &self.up_values,
            ValueType::DownValue => &self.down_values,
            ValueType::SubValue => &self.sub_values,
        }
    }

    pub fn values_mut(&mut self, value_type: ValueType) -> &mut SymbolValueSet {
        match value_type {
            ValueType::OwnValue => &mut self.own_values,
            ValueType::UpValue => &mut self.up_values,
            ValueType::DownValue => &mut self.down_values,
            ValueType::SubValue => &mut self.sub_values,
        }
    }
}

/// A `SymbolValue` is used for storing the definition of an own/up/down/sub value within a symbol
/// definition table.
#[derive(Clone, Debug)]
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

impl SymbolValue {
    pub fn pattern(&self) -> &Expr {
        match self {
            SymbolValue::Definitions { pattern, .. } => pattern,
            SymbolValue::BuiltIn { pattern, .. } => pattern,
            SymbolValue::BuiltInMut { pattern, .. } => pattern,
        }
    }

    pub fn condition(&self) -> Option<&Expr> {
        match self {
            SymbolValue::Definitions { condition, .. } => condition.as_ref(),
            SymbolValue::BuiltIn { condition, .. } => condition.as_ref(),
            SymbolValue::BuiltInMut { condition, .. } => condition.as_ref(),
        }
    }
}

/// A `SymbolValueSet` is a set of `SymbolValue`s.
pub struct SymbolValueSet(Vec<SymbolValue>);

impl SymbolValueSet {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn put(&mut self, value: SymbolValue) -> Option<SymbolValue> {
        for existing in &mut self.0 {
            if existing.pattern() == value.pattern() && existing.condition() == value.condition() {
                return Some(std::mem::replace(existing, value));
            }
        }

        self.0.push(value);
        None
    }
}

impl<'a> IntoIterator for &'a SymbolValueSet {
    type Item = &'a SymbolValue;
    type IntoIter = core::slice::Iter<'a, SymbolValue>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
