use crate::{MatchEquation, MatchGenerator, MatchResultList, MatchRule};

/// Trivial elimination.
///
/// Rule for when `pattern` and `ground` match exactly.
pub(crate) struct RuleT {
    match_equation: MatchEquation,
    exhausted: bool,
}

impl MatchRule for RuleT {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if match_equation.pattern == match_equation.ground {
            return Some(Self {
                match_equation: match_equation.clone(),
                exhausted: false,
            });
        }

        None
    }
}

impl MatchGenerator for RuleT {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleT {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;
        Some(vec![])
    }
}
