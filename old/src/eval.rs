use crate::ast::Expr;
use crate::builtins::{LIST, SEQUENCE};
use crate::env::Env;
use crate::pattern::{Bindings, match_expr};
use crate::symbol::Attributes;
use std::cmp::Ordering;

#[derive(Debug)]
pub enum EvalError {
    Arity,
}

pub fn eval(mut expr: Expr, env: &mut Env) -> Result<Expr, EvalError> {
    loop {
        let next = eval_one(expr.clone(), env)?;

        if next == expr {
            return Ok(expr);
        }

        expr = next;
    }
}

pub fn eval_one(expr: Expr, env: &mut Env) -> Result<Expr, EvalError> {
    // The following is the sequence of steps that the Wolfram Language follows in evaluating an
    // expression like h[e1,e2…]. Every time the expression changes, the Wolfram Language effectively
    // starts the evaluation sequence over again.
    //
    // - If the expression is a raw object (e.g., Integer, String, etc.), leave it unchanged.
    // - Evaluate the head h of the expression.
    // - Evaluate each element ei of the expression in turn.
    //   If h is a symbol with attributes HoldFirst, HoldRest, HoldAll, or HoldAllComplete, then
    //   skip evaluation of certain elements.
    // - Unless h has attributes SequenceHold or HoldAllComplete, flatten out all Sequence objects
    //   that appear among the ei.
    // - Unless h has attribute HoldAllComplete, strip the outermost of any Unevaluated wrappers
    //   that appear among the ei.
    // - If h has attribute Flat, then flatten out all nested expressions with head h.
    // - If h has attribute Listable, then thread through any ei that are lists.
    // - If h has attribute Orderless, then sort the ei into order.
    // - Unless h has attribute HoldAllComplete, use any applicable transformation rules associated
    //   with f that you have defined for objects of the form h[f[e1,…],…].
    // - Use any built‐in transformation rules associated with f for objects of the form h[f[e1,…],…].
    // - Use any applicable transformation rules that you have defined for h[f[e1,e2,…],…] or for h[…][…].
    // - Use any built‐in transformation rules for h[e1,e2,…] or for h[…][…].

    match expr {
        Expr::Symbol(symbol_id) => {
            let def = env.symbol_def(symbol_id);

            if let Some(val) = &def.value {
                return Ok(val.clone());
            }

            Ok(Expr::symbol(symbol_id))
        }

        Expr::Apply(head, args) => {
            // Evaluate the head
            let head_eval = eval_one(*head, env)?;

            // Determine the head attributes
            let head_attributes = match head_eval {
                Expr::Symbol(symbol_id) => env.symbol_def(symbol_id).attributes,
                _ => Attributes::empty(),
            };

            // Evaluate arguments respecting Hold
            let mut args_eval = Vec::with_capacity(args.len());

            for (i, arg) in args.into_iter().enumerate() {
                let hold = head_attributes.contains(Attributes::HOLD_ALL)
                    || (i == 0 && head_attributes.contains(Attributes::HOLD_FIRST))
                    || (i > 0 && head_attributes.contains(Attributes::HOLD_REST));

                if hold {
                    args_eval.push(arg);
                } else {
                    let arg_eval = eval_one(arg, env)?;
                    args_eval.push(arg_eval);
                }
            }

            // Apply Sequence splicing unless SequenceHold
            if !head_attributes.contains(Attributes::SEQUENCE_HOLD) {
                args_eval = splice_sequences(args_eval);
            }

            // Apply Listable
            if head_attributes.contains(Attributes::LISTABLE) {
                if let Some(threaded) = thread_listable(&head_eval, &args_eval) {
                    return Ok(threaded);
                }
            }

            // Apply Flat
            if head_attributes.contains(Attributes::FLAT) {
                args_eval = flatten_args(&head_eval, args_eval);
            }

            // Orderless normalization
            if head_attributes.contains(Attributes::ORDERLESS) {
                args_eval.sort_by(|a, b| expr_order(a, b, env));
            }

            // OneIdentity normalization
            if head_attributes.contains(Attributes::ONE_IDENTITY) && args_eval.len() == 1 {
                return Ok(args_eval[0].clone());
            }

            // Rebuild the expression ready for pattern matching
            let expr_eval = Expr::apply(head_eval.clone(), args_eval.clone());

            // Try to apply upvalues
            if let Some(rewritten) = try_upvalues(&expr_eval, env) {
                return Ok(rewritten);
            }

            // Try to apply downvalues
            if let Some(rewritten) = try_downvalues(&expr_eval, env) {
                return Ok(rewritten);
            }

            // Builtin application (only if head is symbol)
            if let Expr::Symbol(symbol_id) = head_eval {
                if let Some(f) = env.symbol_def(symbol_id).builtin {
                    return f(args_eval, env);
                }
            }

            // Otherwise make no changes to the expression
            Ok(expr_eval)
        }

        _ => Ok(expr),
    }
}

fn splice_sequences(args: Vec<Expr>) -> Vec<Expr> {
    let mut out = Vec::new();

    for arg in args {
        if let Some(seq_args) = arg.is_apply(SEQUENCE.symbol_id()) {
            out.extend(seq_args.iter().cloned());
        } else {
            out.push(arg);
        }
    }

    out
}

fn thread_listable(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let lists: Vec<&Vec<Expr>> = args
        .iter()
        .filter_map(|a| a.is_apply(LIST.symbol_id()))
        .collect();

    if lists.is_empty() {
        return None;
    }

    let len = lists[0].len();
    if !lists.iter().all(|l| l.len() == len) {
        return None;
    }

    let mut threaded = Vec::with_capacity(len);

    for i in 0..len {
        let mut new_args = Vec::with_capacity(args.len());

        for arg in args {
            if let Some(list) = arg.is_apply(LIST.symbol_id()) {
                new_args.push(list[i].clone());
            } else {
                new_args.push(arg.clone());
            }
        }

        threaded.push(Expr::apply(head.clone(), new_args));
    }

    Some(Expr::apply_symbol(LIST.symbol_id(), threaded))
}

fn flatten_args(head: &Expr, args: Vec<Expr>) -> Vec<Expr> {
    let mut out = Vec::with_capacity(args.len());

    for arg in args {
        match &arg {
            Expr::Apply(inner_head, inner_args) if **inner_head == *head => {
                let inner_flat = flatten_args(inner_head, inner_args.clone());
                out.extend(inner_flat);
            }
            _ => out.push(arg),
        }
    }

    out
}

fn try_upvalues(expr: &Expr, env: &mut Env) -> Option<Expr> {
    let args = match expr {
        Expr::Apply(_, args) => args,
        _ => return None,
    };

    for arg in args {
        if let Expr::Symbol(symbol_id) = arg {
            // TODO: Try and avoid this clone
            let upvalues = env.symbol_def(*symbol_id).upvalues.clone();

            for rule in &upvalues {
                let mut bindings = Bindings::new();

                if match_expr(&rule.pattern, expr, env, &mut bindings) {
                    return Some(bindings.apply(&rule.replacement));
                }
            }
        }
    }

    None
}

fn try_downvalues(expr: &Expr, env: &mut Env) -> Option<Expr> {
    let head = match expr {
        Expr::Apply(head, _) => &**head,
        _ => return None,
    };

    let symbol_id = match head {
        Expr::Symbol(symbol_id) => *symbol_id,
        _ => return None,
    };

    // TODO: Try and avoid this clone
    let downvalues = env.symbol_def(symbol_id).downvalues.clone();

    for rule in &downvalues {
        let mut bindings = Bindings::new();

        if match_expr(&rule.pattern, expr, env, &mut bindings) {
            return Some(bindings.apply(&rule.replacement));
        }
    }

    None
}

fn expr_order(a: &Expr, b: &Expr, env: &Env) -> Ordering {
    match (a, b) {
        (Expr::Integer(lhs), Expr::Integer(rhs)) => lhs.cmp(rhs),
        (Expr::Integer(_), _) => Ordering::Less,
        (_, Expr::Integer(_)) => Ordering::Greater,

        (Expr::Symbol(lhs), Expr::Symbol(rhs)) => {
            let lhs_name = env.symbol_def(*lhs).name();
            let rhs_name = env.symbol_def(*rhs).name();

            lhs_name.cmp(rhs_name)
        }
        (Expr::Symbol(_), _) => Ordering::Less,
        (_, Expr::Symbol(_)) => Ordering::Greater,

        (Expr::Apply(ha, aa), Expr::Apply(hb, ab)) => expr_order(ha, hb, env)
            .then_with(|| aa.len().cmp(&ab.len()))
            .then_with(|| {
                aa.iter()
                    .zip(ab.iter())
                    .map(|(x, y)| expr_order(x, y, env))
                    .find(|o| *o != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            }),

        _ => Ordering::Equal,
    }
}
