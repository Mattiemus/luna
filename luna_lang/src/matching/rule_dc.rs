use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    is_any_sequence_variable,
};

/// Decomposition under a commutative head.
///
/// Matches a pattern `f[x, ...]` against a value `g[y, ...]` where `f` is a commutative function.
/// The value of `x` must not be a sequence variable.
///
/// Assumptions:
/// - `f` is a commutative or associative-commutative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleDC {
    pattern: Normal,
    ground: Normal,

    /// Which child of the ground function we are matching on.
    term_idx: usize,
}

impl RuleDC {
    pub(crate) fn new(pattern: Normal, ground: Normal) -> Self {
        Self {
            pattern,
            ground,
            term_idx: 0,
        }
    }
}

impl MatchRule for RuleDC {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let (p0, _) = (p.part(0)?, g.part(0)?);

        if is_any_sequence_variable(p0) {
            return None;
        }

        Some(Self::new(p.clone(), g.clone()))
    }
}

impl MatchGenerator for RuleDC {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleDC {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.term_idx == self.ground.len() {
            return None;
        }

        // Match equation to attempt to match `x` against any of the terms within `g`.
        let result_variable_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: self.pattern.elements()[0].clone(),
            ground: self.ground.elements()[self.term_idx].clone(),
        });

        // Match equation to attempt to match the rest of the function parameters, i.e. `f[...]`
        // and `g[...]`.
        //
        // Note this has remove `x` from `g[x, ...]`.
        let result_function_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(
                self.ground.head().clone(),
                self.ground
                    .elements()
                    .iter()
                    .enumerate()
                    .filter_map(|(k, v)| {
                        if k != self.term_idx {
                            Some(v.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            )),
        });

        self.term_idx += 1;
        Some(vec![result_variable_equation, result_function_equation])
    }
}
