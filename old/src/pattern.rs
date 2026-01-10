use crate::ast::{Expr, is_blank, is_blank_null_sequence, is_blank_sequence, is_pattern};
use crate::builtins::SEQUENCE;
use crate::env::Env;
use crate::symbol::SymbolId;
use std::collections::HashMap;
use std::iter;

#[derive(Clone, Debug)]
pub struct Bindings(HashMap<SymbolId, Expr>);

impl Bindings {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    fn bind(&mut self, symbol_id: SymbolId, expr: Expr) -> bool {
        match self.0.get(&symbol_id) {
            Some(bound) => *bound == expr,
            None => {
                self.0.insert(symbol_id, expr);
                true
            }
        }
    }

    pub fn apply(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Symbol(symbol_id) => self
                .0
                .get(symbol_id)
                .cloned()
                .unwrap_or_else(|| Expr::symbol(*symbol_id)),
            Expr::Apply(head, args) => Expr::apply(
                self.apply(&**head),
                args.iter().map(|a| self.apply(a)).collect::<Vec<_>>(),
            ),
            _ => expr.clone(),
        }
    }
}

// ----

#[derive(Clone, Debug)]
pub struct PatternMatch {
    bindings: Bindings,
}

impl PatternMatch {
    pub fn new(bindings: Bindings) -> Self {
        Self { bindings }
    }

    pub fn empty() -> Self {
        Self {
            bindings: Bindings::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Pattern(Expr);

impl Pattern {
    pub fn new(expr: Expr) -> Self {
        Self(expr)
    }

    pub fn matches(&self, expr: &Expr, env: &Env) -> Vec<PatternMatch> {
        let scope = Bindings::new();
        Self::matches_with_scope(&self.0, expr, env, scope).collect()
    }

    fn matches_with_scope<'a>(
        pattern: &'a Expr,
        expr: &'a Expr,
        env: &'a Env,
        scope: Bindings,
    ) -> Box<dyn Iterator<Item = PatternMatch> + 'a> {
        // Pattern[x, subpat]
        if let Some((symbol_id, subpattern)) = is_pattern(pattern) {
            return Box::new(
                Self::matches_with_scope(subpattern, expr, env, scope.clone()).filter_map(
                    move |m| {
                        let mut bindings = m.bindings.clone();

                        if bindings.bind(symbol_id, expr.clone()) {
                            Some(PatternMatch::new(bindings))
                        } else {
                            None
                        }
                    },
                ),
            );
        }

        // Blank[...]
        if let Some(constraint) = is_blank(pattern) {
            return match constraint {
                // Blank[]
                None => Box::new(iter::once(PatternMatch::new(scope.clone()))),
                // Blank[h]
                Some(h) if *h == expr.head() => {
                    Box::new(iter::once(PatternMatch::new(scope.clone())))
                }
                Some(_) => Box::new(iter::empty()),
            };
        }

        // If expr is a Sequence[...] we can attempt our sequence matchers
        if let Some(sequence) = expr.is_apply(SEQUENCE.symbol_id()) {
            // BlankSequence[...]
            if let Some(constraint) = is_blank_sequence(pattern) {
                if sequence.is_empty() {
                    return Box::new(iter::empty());
                }

                return match constraint {
                    // BlankSequence[]
                    None => Box::new(iter::once(PatternMatch::new(scope.clone()))),
                    // BlankSequence[h]
                    Some(h) if sequence.iter().all(|e| e.head() == *h) => {
                        Box::new(iter::once(PatternMatch::new(scope.clone())))
                    }
                    Some(_) => Box::new(iter::empty()),
                };
            }

            // BlankNullSequence[...]
            if let Some(constraint) = is_blank_null_sequence(pattern) {
                return match constraint {
                    // BlankSequence[]
                    None => Box::new(iter::once(PatternMatch::new(scope.clone()))),
                    // BlankSequence[h]
                    Some(h) if sequence.iter().all(|e| e.head() == *h) => {
                        Box::new(iter::once(PatternMatch::new(scope.clone())))
                    }
                    Some(_) => Box::new(iter::empty()),
                };
            }
        }

        // Recursively match
        if let (Expr::Apply(phead, pargs), Expr::Apply(ehead, eargs)) = (pattern, expr) {
            return Box::new(
                Self::matches_with_scope(phead, ehead, env, scope)
                    .flat_map(|m| Self::match_apply_args(pargs, eargs, env, m.bindings)),
            );
        }

        // Exact match
        if pattern == expr {
            return Box::new(iter::once(PatternMatch::new(scope.clone())));
        }

        // No match
        Box::new(iter::empty())
    }
    fn match_apply_args<'a>(
        patterns: &'a [Expr],
        exprs: &'a [Expr],
        env: &'a Env,
        scope: Bindings,
    ) -> Box<dyn Iterator<Item = PatternMatch> + 'a> {
        if patterns.is_empty() {
            return if exprs.is_empty() {
                Box::new(iter::once(PatternMatch::new(scope.clone())))
            } else {
                Box::new(iter::empty())
            };
        }

        Box::new(
            (0..=exprs.len())
                .map(move |i| (&exprs[..i], &exprs[i..]))
                .flat_map(move |(prefix, rest)| {
                    let expr = if prefix.len() == 1 {
                        prefix[0].clone()
                    } else {
                        Expr::apply_symbol(SEQUENCE.symbol_id(), prefix)
                    };

                    Self::matches_with_scope(&patterns[0], &expr, env, scope).flat_map(move |m| {
                        Self::match_apply_args(&patterns[1..], rest, env, m.bindings)
                    })
                }),
        )
    }
}

// ----

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rule {
    pub pattern: Expr,
    pub replacement: Expr,
}

pub fn match_expr(pattern: &Expr, expr: &Expr, env: &mut Env, bindings: &mut Bindings) -> bool {
    // Pattern[x, subpat]
    if let Some((symbol_id, subpat)) = is_pattern(pattern) {
        if !match_expr(subpat, expr, env, bindings) {
            return false;
        }

        return bindings.bind(symbol_id, expr.clone());
    }

    // Blank[...]
    if let Some(constraint) = is_blank(pattern) {
        return match constraint {
            // Blank[]
            None => true,
            // Blank[h]
            Some(h) => *h == expr.head(),
        };
    }

    // Exact match
    match (pattern, expr) {
        (Expr::Apply(phead, pargs), Expr::Apply(ehead, eargs)) => {
            match_expr(phead, ehead, env, bindings) && match_args(pargs, eargs, env, bindings)
        }
        (lhs, rhs) => lhs == rhs,
    }
}

fn match_args(patterns: &[Expr], exprs: &[Expr], env: &mut Env, bindings: &mut Bindings) -> bool {
    if patterns.is_empty() {
        return exprs.is_empty();
    }

    // Unwrap Pattern[x, subpat] if present
    let (symbol_id, subpat) = if let Some((sid, sp)) = is_pattern(&patterns[0]) {
        (Some(sid), sp)
    } else {
        (None, &patterns[0])
    };

    // BlankSequence[...]
    if let Some(seq_constraint) = is_blank_sequence(subpat) {
        return match_sequence(1, symbol_id, seq_constraint, patterns, exprs, env, bindings);
    }

    // BlankNullSequence[...]
    if let Some(seq_constraint) = is_blank_null_sequence(subpat) {
        return match_sequence(0, symbol_id, seq_constraint, patterns, exprs, env, bindings);
    }

    // Normal case
    if exprs.is_empty() {
        return false;
    }

    // Attempt to match a single pattern
    if !match_expr(&patterns[0], &exprs[0], env, bindings) {
        return false;
    }

    // Now match further arguments recursively
    match_args(&patterns[1..], &exprs[1..], env, bindings)
}

fn match_sequence(
    start: usize,
    symbol_id: Option<SymbolId>,
    seq_constraint: Option<&Expr>,
    pats: &[Expr],
    exprs: &[Expr],
    env: &mut Env,
    bindings: &mut Bindings,
) -> bool {
    for split in start..=exprs.len() {
        let slice = &exprs[..split];

        // Enforce constraint
        if let Some(h) = seq_constraint {
            if !slice.iter().all(|e| e.head() == *h) {
                continue;
            }
        }

        let mut local = bindings.clone();

        if let Some(symbol_id) = symbol_id {
            let expr_sequence = Expr::apply_symbol(SEQUENCE.symbol_id(), slice);

            if !local.bind(symbol_id, expr_sequence) {
                continue;
            }
        }

        if match_args(&pats[1..], &exprs[split..], env, &mut local) {
            *bindings = local;
            return true;
        }
    }

    false
}
