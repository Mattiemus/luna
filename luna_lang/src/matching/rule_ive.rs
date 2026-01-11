use crate::{
    MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    is_any_pattern, is_any_sequence_pattern, is_sequence,
};

/// Individual variable elimination.
///
/// Matches a pattern `x_` against any value so long as it is not a sequence or sequence variable
/// (i.e. `x__`, `x___`, or `Sequence[...]`).
pub(crate) struct RuleIVE {
    match_equation: MatchEquation,
    exhausted: bool,
}

impl RuleIVE {
    fn new(match_equation: MatchEquation) -> Self {
        Self {
            match_equation,
            exhausted: false,
        }
    }
}

impl MatchRule for RuleIVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        // Pattern: `x_`
        // Ground: Not a sequence or sequence variable.

        if is_any_pattern(&match_equation.pattern).is_some()
            && is_sequence(&match_equation.ground).is_none()
            && is_any_sequence_pattern(&match_equation.ground).is_none()
        {
            Some(RuleIVE::new(match_equation.clone()))
        } else {
            None
        }
    }
}

impl MatchGenerator for RuleIVE {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleIVE {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;
        Some(vec![MatchResult::Substitution(Substitution {
            variable: self.match_equation.pattern.clone(),
            ground: self.match_equation.ground.clone(),
        })])
    }
}
