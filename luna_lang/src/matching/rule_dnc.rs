use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    is_any_sequence_variable,
};

/// Decomposition under a non-commutative head.
///
/// Matches a pattern `f[x, ...]` against a value `g[y, ...]` where `f` is a free function.
/// The value of `x` must not be a sequence variable.
///
/// Assumptions:
/// - `f` is a non-commutative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleDNC {
    pattern: Normal,
    ground: Normal,
    exhausted: bool,
}

impl MatchRule for RuleDNC {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if let (Some(p), Some(g)) = (
            match_equation.pattern.try_normal(),
            match_equation.ground.try_normal(),
        ) {
            if let (Some(p0), Some(_)) = (p.part(0), g.part(0)) {
                if is_any_sequence_variable(p0) {
                    return None;
                }

                return Some(Self {
                    pattern: p.clone(),
                    ground: g.clone(),
                    exhausted: false,
                });
            }
        }

        None
    }
}

impl MatchGenerator for RuleDNC {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleDNC {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;

        // Match equation to attempt to match `x` and `y`.
        let result_variable_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: self.pattern.elements()[0].clone(),
            ground: self.ground.elements()[0].clone(),
        });

        // Match equation to attempt to match the rest of the function parameters, i.e. `f[...]`
        // and `g[...]`.
        let result_function_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(
                self.ground.head().clone(),
                &self.ground.elements()[1..],
            )),
        });

        Some(vec![result_variable_equation, result_function_equation])
    }
}
