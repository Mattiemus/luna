use crate::abstractions::IString;
use crate::{Atom, Attributes, Context, SExpression, SolutionSet, SymbolValue};
use std::rc::Rc;

pub type BuiltinFn = fn(Atom, SolutionSet, &Context) -> Atom;
pub type BuiltinFnMut = fn(Atom, SolutionSet, &mut Context) -> Atom;

// TODO
pub fn parse(s: &str) -> Result<Atom, ()> {
    Err(())
}

/// Registers all builtins
pub(crate) fn register_builtins(context: &mut Context) {
    // TODO: This should register built ins once we have a simple parser ready.
    // register_head_builtin(context);
}

/// Registers the `Head` builtin symbol.
///
/// - `Attributes[Head] = { ReadOnly, AttributesReadOnly }`
/// - `Head[expr_] := built-in[expr]`
/// - `Head[expr_, h_] := built-in[expr, h]`
pub(crate) fn register_head_builtin(context: &mut Context) {
    context
        .set_down_value(
            IString::from("Head"),
            SymbolValue::BuiltIn {
                pattern: parse("Head[expr_]").unwrap(),
                condition: None,
                built_in: |_, arguments, _| {
                    let expr = &arguments[&SExpression::make_pattern_blank("expr")];

                    expr.head()
                },
            },
        )
        .unwrap();

    context
        .set_down_value(
            IString::from("Head"),
            SymbolValue::BuiltIn {
                pattern: parse("Head[expr_, h_]").unwrap(),
                condition: None,
                built_in: |_, arguments, _| {
                    let expr = &arguments[&SExpression::make_pattern_blank("expr")];
                    let h = &arguments[&SExpression::make_pattern_blank("h")];

                    Atom::SExpression(Rc::new(vec![h.clone(), expr.head()]))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            IString::from("Head"),
            Attributes::READ_ONLY | Attributes::ATTRIBUTES_READ_ONLY,
        )
        .unwrap();
}
