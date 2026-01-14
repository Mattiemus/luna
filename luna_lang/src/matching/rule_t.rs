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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MatchEquation, MatchRule, parse};
    use test_case::test_case;

    #[test_case("\"abc\"", "\"abc\"" ; "strings")]
    #[test_case("123", "123" ; "integers")]
    #[test_case("2.5", "2.5" ; "reals")]
    #[test_case("abc", "abc" ; "symbols")]
    #[test_case("f[a, b, c]", "f[a, b, c]" ; "expressions")]
    fn matches(pattern: &str, ground: &str) {
        let mut rule = RuleT::try_rule(&MatchEquation {
            pattern: parse(pattern).unwrap(),
            ground: parse(ground).unwrap(),
        })
        .unwrap();

        assert_eq!(rule.next(), Some(vec![]));
        assert_eq!(rule.next(), None);
    }

    #[test_case("\"abc\"", "\"def\"" ; "strings")]
    #[test_case("123", "456" ; "integers")]
    #[test_case("2.5", "3.5" ; "reals")]
    #[test_case("abc", "def" ; "symbols")]
    #[test_case("f[a, b, c]", "g[a, b, c]" ; "expressions with different heads")]
    #[test_case("f[a, b, c]", "f[c, b, a]" ; "expressions with different arguments")]
    fn does_not_match(pattern: &str, ground: &str) {
        let rule = RuleT::try_rule(&MatchEquation {
            pattern: parse(pattern).unwrap(),
            ground: parse(ground).unwrap(),
        });

        assert!(rule.is_none());
    }
}
