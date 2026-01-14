use crate::symbol::Symbol;
use crate::{
    MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    is_any_blank_sequence, is_any_sequence_pattern, is_sequence, parse_blank_pattern,
};

/// Individual variable elimination.
///
/// Matches a pattern `x_` against any value so long as it is not a sequence or sequence variable
/// (i.e. `__`, `___`, `x__`, `x___`, or `Sequence[...]`).
pub(crate) struct RuleIVE {
    match_equation: MatchEquation,
    variable: Option<Symbol>,
    exhausted: bool,
}

impl MatchRule for RuleIVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if is_any_blank_sequence(&match_equation.ground)
            || is_any_sequence_pattern(&match_equation.ground)
            || is_sequence(&match_equation.ground)
        {
            return None;
        }

        if let Some((variable, _)) = parse_blank_pattern(&match_equation.pattern) {
            // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

            return Some(Self {
                match_equation: match_equation.clone(),
                variable: variable.cloned(),
                exhausted: false,
            });
        }

        None
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

        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                pattern: self.match_equation.pattern.clone(),
                ground: self.match_equation.ground.clone(),
            });

            return Some(vec![substitution]);
        }

        Some(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MatchEquation, MatchRule, parse};
    use test_case::test_case;

    #[test_case("\"abc\"" ; "string")]
    #[test_case("123" ; "integer")]
    #[test_case("1.25" ; "real")]
    #[test_case("abc" ; "symbol")]
    #[test_case("f[a, b, c]" ; "expression")]
    fn matches_named_variable(ground: &str) {
        let mut rule = RuleIVE::try_rule(&MatchEquation {
            pattern: parse("x_").unwrap(),
            ground: parse(ground).unwrap(),
        })
        .unwrap();

        assert_eq!(
            rule.next(),
            Some(vec![MatchResult::Substitution(Substitution {
                variable: Symbol::new("x"),
                pattern: parse("x").unwrap(),
                ground: parse(ground).unwrap(),
            })])
        );

        assert_eq!(rule.next(), None);
    }

    #[test_case("\"abc\"" ; "string")]
    #[test_case("123" ; "integer")]
    #[test_case("1.25" ; "real")]
    #[test_case("abc" ; "symbol")]
    #[test_case("f[a, b, c]" ; "expression")]
    fn matches_unnamed_variable(ground: &str) {
        let mut rule = RuleIVE::try_rule(&MatchEquation {
            pattern: parse("_").unwrap(),
            ground: parse(ground).unwrap(),
        })
        .unwrap();

        assert_eq!(rule.next(), Some(vec![]));
        assert_eq!(rule.next(), None);
    }

    #[test_case("abc", "abc" ; "pattern is not variable")]
    #[test_case("_", "Sequence[a, b, c]" ; "ground is a sequence")]
    #[test_case("_", "__" ; "ground is blank sequence")]
    #[test_case("_", "___" ; "ground is blank null sequence")]
    #[test_case("_", "x__" ; "ground is blank sequence pattern")]
    #[test_case("_", "x___" ; "ground is blank null sequence pattern")]
    fn does_not_match(pattern: &str, ground: &str) {
        let rule = RuleIVE::try_rule(&MatchEquation {
            pattern: parse(pattern).unwrap(),
            ground: parse(ground).unwrap(),
        });

        assert!(rule.is_none());
    }
}
