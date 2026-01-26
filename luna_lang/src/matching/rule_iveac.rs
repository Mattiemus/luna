use crate::matching::rule_dc::RuleDC;
use crate::matching::rule_svec::RuleSVEC;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_individual_variable, try_sequence,
};

/// Individual variable elimination under an associative-commutative head.
///
/// Matches a pattern `f[x_, ...]` against a value `g[y, ...]` where `f` is an
/// associative-commutative function.
///
/// For example `f[x_, ...]` and `g[a, b, c]` will result in the matches:
///     - `x` => `a`            (+ match equation for `f[...]` against `g[b, c]`)
///     - `x` => `f[a]`         (+ match equation for `f[...]` against `g[b, c]`)
///     - `x` => `f[a]`         (+ match equation for `f[...]` against `g[c, b]`)
///     - `x` => `f[a, b]`      (+ match equation for `f[...]` against `g[c]`)
///     - `x` => `f[b, a]`      (+ match equation for `f[...]` against `g[c]`)
///     - `x` => `f[a, b, c]`, `f[b, c, a]`, `f[c, b, a]`, ...
///
/// Internally this makes use of `RuleSVEC` and `RuleDC` to determine potential matches. This is
/// due to the implementations of this and `RuleSVEC` being effectively identical, albeit with
/// differing wrapping rules for the various possible `Sequence[...]` substitutions. Usage of
/// `RuleDC` facilitates matching of singleton values (i.e. `x` => `a` in the above example).
///
/// Assumptions:
/// - `f` is an associative-commutative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleIVEAC {
    pattern: Normal,
    ground: Normal,
    rule_dc: RuleDC,
    rule_svec: RuleSVEC,
}

impl RuleIVEAC {
    pub(crate) fn new(pattern: Normal, ground: Normal, variable: Option<Symbol>) -> Self {
        Self {
            pattern: pattern.clone(),
            ground: ground.clone(),
            rule_dc: RuleDC::new(pattern.clone(), ground.clone()),
            rule_svec: RuleSVEC::new(pattern.clone(), ground.clone(), variable.clone(), false),
        }
    }
}

impl MatchRule for RuleIVEAC {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let (p_elem0, _) = (p.element(0)?, g.element(0)?);
        let (variable, _) = parse_individual_variable(p_elem0)?;

        // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

        Some(Self::new(p.clone(), g.clone(), variable.cloned()))
    }
}

impl MatchGenerator for RuleIVEAC {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleIVEAC {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        // Attempt decomposition first.
        // This ensures we put the singleton results ahead of the function application results.
        if let Some(result) = self.rule_dc.next() {
            return Some(result);
        }

        // Next attempt to get the next result from `RuleSVEC`.
        let result = self.rule_svec.next()?;

        // Transform the results list into a consuming iterator and transform substitutions
        // into function application(s) instead of sequence values.
        let transformed_results = result
            .into_iter()
            .map(|match_result| match match_result {
                MatchResult::Substitution(Substitution { variable, ground }) => {
                    let sequence_elements = try_sequence(&ground)
                        .expect("RuleSVEC should only produce Sequence[...] results");

                    MatchResult::Substitution(Substitution {
                        variable,
                        ground: Expr::from(Normal::new(
                            self.ground.head().clone(),
                            sequence_elements,
                        )),
                    })
                }
                _ => match_result,
            })
            .collect::<Vec<_>>();

        // Return the transformed set of results.
        Some(transformed_results)
    }
}
