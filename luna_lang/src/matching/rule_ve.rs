use crate::{
    MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    parse_individual_variable,
};
use crate::{Symbol, parse_any_sequence_variable, try_sequence};

/// Variable (both individual and sequence) elimination.
///
/// Matches a pattern `x_` against any value.
/// Matches a pattern `x__` against a non-empty `Sequence[...]` value.
/// Matches a pattern `x___` against any `Sequence[...]` value.
pub(crate) struct RuleVE {
    match_equation: MatchEquation,
    variable: Option<Symbol>,
    exhausted: bool,
}

impl MatchRule for RuleVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        // Match `x_` against any value.
        if let Some((variable, _)) = parse_individual_variable(&match_equation.pattern) {
            // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

            return Some(Self {
                match_equation: match_equation.clone(),
                variable: variable.cloned(),
                exhausted: false,
            });
        }

        // Match `x__` and `x___` against sequence values.
        if let Some((matches_empty, variable, _)) =
            parse_any_sequence_variable(&match_equation.pattern)
        {
            if let Some(gelements) = try_sequence(&match_equation.ground) {
                if !matches_empty && gelements.is_empty() {
                    return None;
                }

                // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

                return Some(Self {
                    match_equation: match_equation.clone(),
                    variable: variable.cloned(),
                    exhausted: false,
                });
            }
        }

        None
    }
}

impl MatchGenerator for RuleVE {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleVE {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;

        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: self.match_equation.ground.clone(),
            });

            return Some(vec![substitution]);
        }

        Some(vec![])
    }
}
