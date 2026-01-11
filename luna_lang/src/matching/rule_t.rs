use crate::{MatchEquation, MatchGenerator, MatchResultList, MatchRule};

/// Trivial elimination.
///
/// Rule for when the `pattern` and `ground` match exactly.
pub(crate) struct RuleT {
    match_equation: MatchEquation,
    exhausted: bool,
}

impl RuleT {
    fn new(match_equation: MatchEquation) -> Self {
        Self {
            match_equation,
            exhausted: false,
        }
    }
}

impl MatchRule for RuleT {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if match_equation.pattern == match_equation.ground {
            Some(Self::new(match_equation.clone()))
        } else {
            None
        }
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
    use crate::{Atom, MatchEquation, MatchRule};

    #[test]
    fn accepts_equal_strings() {
        let mut rule = RuleT::try_rule(&MatchEquation {
            pattern: Atom::string("abc"),
            ground: Atom::string("abc"),
        })
        .unwrap();

        assert_eq!(rule.next(), Some(vec![]));
        assert_eq!(rule.next(), None);
    }

    #[test]
    fn accepts_equal_symbols() {
        let mut rule = RuleT::try_rule(&MatchEquation {
            pattern: Atom::symbol("x"),
            ground: Atom::symbol("x"),
        })
        .unwrap();

        assert_eq!(rule.next(), Some(vec![]));
        assert_eq!(rule.next(), None);
    }

    #[test]
    fn rejects_different_strings() {
        let rule = RuleT::try_rule(&MatchEquation {
            pattern: Atom::string("abc"),
            ground: Atom::string("def"),
        });

        assert!(rule.is_none());
    }

    #[test]
    fn rejects_different_symbols() {
        let rule = RuleT::try_rule(&MatchEquation {
            pattern: Atom::symbol("x"),
            ground: Atom::symbol("y"),
        });

        assert!(rule.is_none());
    }
}
