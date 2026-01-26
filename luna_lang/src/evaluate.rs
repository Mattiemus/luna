use crate::{
    Attributes, Context, Expr, ExprKind, Matcher, Normal, RewriteRule, SolutionSet, Symbol,
    ValueType,
};

// TODO: Evaluation is currently a bit gross. We will want to introduce some kind of
//  enum to signal when evaluation has changed the expression etc.

pub fn evaluate(expr: Expr, context: &mut Context) -> Expr {
    let initial_context_state = context.state_version();

    match expr.kind() {
        ExprKind::Symbol(symbol) => {
            let unevaluated_rule =
                find_matching_definition(&expr, symbol, ValueType::OwnValue, context);

            match unevaluated_rule {
                None => expr,
                Some(unevaluated_rule) => {
                    let new_expr = unevaluated_rule.apply(expr, context);

                    if initial_context_state != context.state_version() {
                        evaluate(new_expr, context)
                    } else {
                        new_expr
                    }
                }
            }
        }

        ExprKind::Normal(normal) => {
            let head_eval = evaluate(normal.head().clone(), context);

            let attributes = match head_eval.name() {
                None => Attributes::empty(),
                Some(name) => context.get_attributes(&name),
            };

            let elements_eval = normal
                .elements()
                .iter()
                .enumerate()
                .map(|(i, elem)| {
                    if attributes.hold_all()
                        || attributes.hold_all_complete()
                        || (attributes.hold_first() && i == 0)
                        || (attributes.hold_rest() && i > 0)
                    {
                        return elem.clone();
                    }

                    evaluate(elem.clone(), context)
                })
                .collect::<Vec<_>>();

            //   * Unless h has attributes SequenceHold or HoldAllComplete, flatten out all Sequence objects that appear among the ei.
            //   * Unless h has attribute HoldAllComplete, strip the outermost of any Unevaluated wrappers that appear among the ei.
            //   * If h has attribute Flat, then flatten out all nested expressions with head h.
            //   * If h has attribute Listable, then thread through any ei that are lists.
            //   * If h has attribute Orderless, then sort the ei into order

            //   * Unless h has attribute HoldAllComplete, use any applicable transformation rules associated with f that you have defined for objects of the form h[f[e1,…],…].
            //   * Use any built‐in transformation rules associated with f for objects of the form h[f[e1,…],…].

            let new_expr = Expr::from(Normal::new(head_eval, elements_eval));

            if initial_context_state != context.state_version() {
                return evaluate(new_expr, context);
            }

            match new_expr.name() {
                None => new_expr,
                Some(name) => {
                    let unevaluated_rule =
                        find_matching_definition(&new_expr, &name, ValueType::DownValue, context);

                    match unevaluated_rule {
                        None => expr,
                        Some(unevaluated_rule) => {
                            let new_expr = unevaluated_rule.apply(expr, context);

                            if initial_context_state != context.state_version() {
                                evaluate(new_expr, context)
                            } else {
                                new_expr
                            }
                        }
                    }
                }
            }
        }

        // String, Integer, and Real remain unchanged
        _ => expr.clone(),
    }
}

struct UnevaluatedRule {
    rewrite_rule: RewriteRule,
    bindings: SolutionSet,
}

impl UnevaluatedRule {
    pub fn apply(self, expr: Expr, context: &mut Context) -> Expr {
        match &self.rewrite_rule {
            RewriteRule::Definitions { ground, .. } => replace_all(&self.bindings, ground),
            RewriteRule::BuiltIn { built_in, .. } => built_in(self.bindings, expr, context),
            RewriteRule::BuiltInMut { built_in, .. } => built_in(self.bindings, expr, context),
        }
    }
}

/// Replaces all instances of a variable with the value that it has been bound to, determined by
/// symbol name.
pub fn replace_all(bindings: &SolutionSet, expr: &Expr) -> Expr {
    match expr.kind() {
        ExprKind::Symbol(symbol) => {
            for (name, substitution) in bindings {
                if symbol == name {
                    return substitution.clone();
                }
            }

            expr.clone()
        }
        ExprKind::Normal(normal) => Expr::from(Normal::new(
            replace_all(bindings, normal.head()),
            normal
                .elements()
                .iter()
                .map(|elem| replace_all(bindings, elem))
                .collect::<Vec<_>>(),
        )),
        _ => expr.clone(),
    }
}

/// If any of the patterns in the vector of ``SymbolValue``'s match and satisfy any condition the
/// pattern may have, return the variable bindings and substitutions or built_in for the match. The value returned is
/// wrapped in an `UnevaluatedDefinitionMatch` and is left unevaluated.
fn find_matching_definition(
    ground: &Expr,
    symbol: &Symbol,
    value_type: ValueType,
    context: &mut Context,
) -> Option<UnevaluatedRule> {
    let rewrite_rules = context.get_values(symbol, value_type)?;

    for rewrite_rule in rewrite_rules {
        let mut matcher: Matcher =
            Matcher::new(rewrite_rule.pattern().clone(), ground.clone(), context);

        while let Some(bindings) = matcher.next() {
            if let Some(condition) = rewrite_rule.condition() {
                todo!();
                // if check_condition(c.clone(), &substitutions, context) {
                //   return wrap_definition_match(&symbol_value, substitutions);
                // }
            } else {
                return Some(UnevaluatedRule {
                    rewrite_rule: rewrite_rule.clone(),
                    bindings,
                });
            }
        }
    }

    None
}
