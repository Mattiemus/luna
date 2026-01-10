use crate::{MatchEquation, MatchGenerator, MatchResultList, MatchRule};

/// Trivial elimination.
/// Rule for when the `pattern` and `ground` match exactly.
///
/// `s ≪ᴱ s ⇝ᵩ ∅`
pub(crate) struct RuleTrivial {
    exhausted: bool,
    match_equation: MatchEquation,
}

impl RuleTrivial {
    pub(crate) fn new(match_equation: MatchEquation) -> Self {
        Self {
            exhausted: false,
            match_equation,
        }
    }
}

impl MatchRule for RuleTrivial {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if match_equation.pattern == match_equation.ground {
            Some(Self::new(match_equation.clone()))
        } else {
            None
        }
    }
}

impl MatchGenerator for RuleTrivial {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleTrivial {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;
        Some(vec![])
    }
}
