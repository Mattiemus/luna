use crate::Symbol;
use crate::builtins::register_builtins;
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

    pub fn state_version(&self) -> usize {
        self.state_version
    }

    pub fn get_attributes(&self, symbol: &Symbol) -> Attributes {
        self.attributes.get(symbol).cloned().unwrap_or_default()
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

    pub fn get_values(&self, symbol: &Symbol, value_type: ValueType) -> Option<&RewriteRuleSet> {
        let definition = self.get_definition(symbol)?;
        let rewrite_rules = definition.rules(value_type);

        Some(rewrite_rules)
    }

    pub fn set_value(
        &mut self,
        symbol: &Symbol,
        value_type: ValueType,
        rewrite_rule: RewriteRule,
    ) -> Result<(), String> {
        let attributes = self.get_attributes(symbol);
        if attributes.read_only() {
            return Err(format!("Symbol '{}' is read-only", symbol));
        }

        let definition = self.get_definition_mut(symbol);
        let rewrite_rules = definition.rules_mut(value_type);

        if rewrite_rules.has_rule(rewrite_rule.pattern(), rewrite_rule.condition()) {
            return Ok(());
        }

        rewrite_rules.push(rewrite_rule);
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ValueType {
    OwnValue,
    UpValue,
    DownValue,
    SubValue,
}

/// A `SymbolDefinition` contains all the transformation rules that apply to a given symbol.
pub struct SymbolDefinition {
    /// OwnValues define how the symbol appearing alone should be evaluated.
    /// They have the form `x :> expr` or `x = expr`.
    pub own_values: RewriteRuleSet,

    /// UpValues define how M-expressions having the symbol as an argument should be evaluated.
    /// They typically have the form `f[pattern, g[pattern], pattern] :> expr`.
    /// UpValues are applied before DownValues.
    pub up_values: RewriteRuleSet,

    /// DownValues define how M-expressions having the symbol as their head should be evaluated.
    /// They typically have the form `f[pattern] :> expr`.
    pub down_values: RewriteRuleSet,

    /// SubValues define how M-expressions having an M-expression with the symbol as a head should be evaluated.
    /// They typically have the form `f[pat][pat] :> exp`.
    pub sub_values: RewriteRuleSet,
}

impl SymbolDefinition {
    pub fn new() -> Self {
        Self {
            own_values: RewriteRuleSet::new(),
            up_values: RewriteRuleSet::new(),
            down_values: RewriteRuleSet::new(),
            sub_values: RewriteRuleSet::new(),
        }
    }

    pub fn rules(&self, value_type: ValueType) -> &RewriteRuleSet {
        match value_type {
            ValueType::OwnValue => &self.own_values,
            ValueType::UpValue => &self.up_values,
            ValueType::DownValue => &self.down_values,
            ValueType::SubValue => &self.sub_values,
        }
    }

    pub fn rules_mut(&mut self, value_type: ValueType) -> &mut RewriteRuleSet {
        match value_type {
            ValueType::OwnValue => &mut self.own_values,
            ValueType::UpValue => &mut self.up_values,
            ValueType::DownValue => &mut self.down_values,
            ValueType::SubValue => &mut self.sub_values,
        }
    }
}

/// A `RewriteRule` is used for storing the definition of an own/up/down/sub value within a symbol
/// definition table.
#[derive(Clone, Debug)]
pub enum RewriteRule {
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

impl RewriteRule {
    pub fn pattern(&self) -> &Expr {
        match self {
            RewriteRule::Definitions { pattern, .. } => pattern,
            RewriteRule::BuiltIn { pattern, .. } => pattern,
            RewriteRule::BuiltInMut { pattern, .. } => pattern,
        }
    }

    pub fn condition(&self) -> Option<&Expr> {
        match self {
            RewriteRule::Definitions { condition, .. } => condition.as_ref(),
            RewriteRule::BuiltIn { condition, .. } => condition.as_ref(),
            RewriteRule::BuiltInMut { condition, .. } => condition.as_ref(),
        }
    }
}

/// A `RewriteRuleSet` is a set of `RewriteRule` values.
pub struct RewriteRuleSet(Vec<RewriteRule>);

impl RewriteRuleSet {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn has_rule(&self, pattern: &Expr, condition: Option<&Expr>) -> bool {
        self.0
            .iter()
            .any(|r| r.pattern() == pattern && r.condition() == condition)
    }

    pub fn push(&mut self, rewrite_rule: RewriteRule) {
        self.0.push(rewrite_rule);
    }
}

impl<'a> IntoIterator for &'a RewriteRuleSet {
    type Item = &'a RewriteRule;
    type IntoIter = core::slice::Iter<'a, RewriteRule>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
