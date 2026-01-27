use crate::{
    Attributes, Context, Expr, ExprKind, Matcher, Normal, SolutionSet, Symbol, SymbolValue,
    ValueType,
};

// TODO: Evaluation is currently a bit gross. We will want to introduce some kind of
//  enum to signal when evaluation has changed the expression etc.

pub enum EvalResult {
    Changed(Expr),
    Unchanged(Expr),
}

impl EvalResult {
    pub fn is_changed(&self) -> bool {
        matches!(self, EvalResult::Changed(_))
    }

    pub fn expr(&self) -> &Expr {
        match self {
            EvalResult::Unchanged(e) | EvalResult::Changed(e) => e,
        }
    }

    pub fn into_expr(self) -> Expr {
        match self {
            EvalResult::Unchanged(e) | EvalResult::Changed(e) => e,
        }
    }
}

pub fn evaluate(expr: Expr, context: &mut Context) -> Expr {
    let initial_context_state = context.state_version();

    match evaluate_step(expr, context) {
        EvalResult::Changed(new_expr) => evaluate(new_expr, context),
        EvalResult::Unchanged(new_expr) => {
            if initial_context_state != context.state_version() {
                evaluate(new_expr, context)
            } else {
                new_expr
            }
        }
    }
}

pub fn evaluate_step(expr: Expr, context: &mut Context) -> EvalResult {
    match expr.kind() {
        ExprKind::Symbol(symbol) => {
            match find_matching_definition(&expr, symbol, ValueType::OwnValue, context) {
                None => EvalResult::Unchanged(expr),
                Some(unevaluated_rule) => unevaluated_rule.apply(expr, context),
            }
        }
        ExprKind::Normal(normal) => {
            let mut changed = false;

            let head_eval = match evaluate_step(normal.head().clone(), context) {
                EvalResult::Changed(new_expr) => {
                    changed = true;
                    new_expr
                }
                EvalResult::Unchanged(new_expr) => new_expr,
            };

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

                    let result = evaluate_step(elem.clone(), context);
                    changed |= result.is_changed();
                    result.into_expr()
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

            if changed {
                return EvalResult::Changed(new_expr);
            }

            match new_expr.name() {
                None => EvalResult::Unchanged(new_expr),
                Some(name) => {
                    match find_matching_definition(&new_expr, &name, ValueType::DownValue, context)
                    {
                        None => EvalResult::Unchanged(expr),
                        Some(unevaluated_rule) => unevaluated_rule.apply(expr, context),
                    }
                }
            }
        }
        _ => EvalResult::Unchanged(expr),
    }
}

struct UnevaluatedRule {
    value: SymbolValue,
    bindings: SolutionSet,
}

impl UnevaluatedRule {
    pub fn apply(self, expr: Expr, context: &mut Context) -> EvalResult {
        match self.value {
            SymbolValue::Definitions { ground, .. } => replace_all(&self.bindings, ground),
            SymbolValue::BuiltIn { built_in, .. } => built_in(self.bindings, expr, context),
            SymbolValue::BuiltInMut { built_in, .. } => built_in(self.bindings, expr, context),
        }
    }
}

/// Replaces all instances of a variable with the value that it has been bound to, determined by
/// symbol name.
pub fn replace_all(bindings: &SolutionSet, expr: Expr) -> EvalResult {
    match expr.kind() {
        ExprKind::Symbol(symbol) => {
            for (name, substitution) in bindings {
                if symbol == name {
                    return EvalResult::Changed(substitution.clone());
                }
            }

            EvalResult::Unchanged(expr)
        }
        ExprKind::Normal(normal) => {
            let head = replace_all(bindings, normal.head().clone());
            let mut changed = head.is_changed();

            let elements = normal
                .elements()
                .iter()
                .map(|elem| {
                    let result = replace_all(bindings, elem.clone());
                    changed |= result.is_changed();
                    result.into_expr()
                })
                .collect::<Vec<_>>();

            let new_expr = Expr::from(Normal::new(head.into_expr(), elements));

            if changed {
                EvalResult::Changed(new_expr)
            } else {
                EvalResult::Unchanged(new_expr)
            }
        }
        _ => EvalResult::Unchanged(expr),
    }
}

/// Search through the context to find a matching `SymbolValue` entries for the given symbol. This
/// method also checks that any conditions are satisfied.
fn find_matching_definition(
    ground: &Expr,
    symbol: &Symbol,
    value_type: ValueType,
    context: &mut Context,
) -> Option<UnevaluatedRule> {
    let values = context.get_values(symbol, value_type)?;

    for value in values {
        let mut matcher: Matcher = Matcher::new(value.pattern().clone(), ground.clone(), context);

        while let Some(bindings) = matcher.next() {
            if let Some(condition) = value.condition() {
                todo!();
                // if check_condition(c.clone(), &substitutions, context) {
                //   return wrap_definition_match(&symbol_value, substitutions);
                // }
            } else {
                return Some(UnevaluatedRule {
                    value: value.clone(),
                    bindings,
                });
            }
        }
    }

    None
}
