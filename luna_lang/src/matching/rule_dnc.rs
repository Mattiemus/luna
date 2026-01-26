use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    is_any_sequence_variable,
};

/// Decomposition under a non-commutative (i.e free or associative) head.
///
/// Matches a pattern `f[x, ...]` against a value `g[y, ...]` where `f` is a free or associative
/// function and the value of `x` is not a sequence variable.
///
/// Assumptions:
/// - `f` is a free or associative (i.e. non-commutative) function.
/// - `f` and `g` are equal.
pub(crate) struct RuleDNC {
    pattern: Normal,
    ground: Normal,
    exhausted: bool,
}

impl RuleDNC {
    pub(crate) fn new(pattern: Normal, ground: Normal) -> Self {
        Self {
            pattern,
            ground,
            exhausted: false,
        }
    }
}

impl MatchRule for RuleDNC {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let (p_elem0, _) = (p.element(0)?, g.element(0)?);

        if is_any_sequence_variable(p_elem0) {
            return None;
        }

        Some(Self::new(p.clone(), g.clone()))
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

        self.exhausted = true;
        Some(vec![result_variable_equation, result_function_equation])
    }
}
