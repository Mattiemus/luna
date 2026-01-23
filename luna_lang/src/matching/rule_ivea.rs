use crate::matching::rule_dnc::RuleDNC;
use crate::matching::rule_svef::RuleSVEF;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_individual_variable, try_sequence,
};

/// Individual variable elimination under an associative head.
///
/// Matches a pattern `f[x_, ...]` against a value `g[y, ...]` where `f` is an associative function.
///
/// For example `f[x_, ...]` and `g[a, b, c]` will result in the matches:
///     - `x` => `a`            (+ match equation for `f[...]` against `g[b, c]`)
///     - `x` => `f[a]`         (+ match equation for `f[...]` against `g[b, c]`)
///     - `x` => `f[a, b]`      (+ match equation for `f[...]` against `g[c]`)
///     - `x` => `f[a, b, c]`
///
/// Internally this makes use of `RuleSVEF` and `RuleDNC` to determine potential matches. This is
/// due to the implementations of this and `RuleSVEF` being effectively identical, albeit with
/// differing wrapping rules for the various possible `Sequence[...]` substitutions. Usage of
/// `RuleDNC` facilitates matching of singleton values (i.e. `x` => `a` in the above example).
///
/// Assumptions:
/// - `f` is an associative function.
/// - `f` and `g` are equal.
pub struct RuleIVEA {
    pattern: Normal,
    ground: Normal,
    rule_dnc: RuleDNC,
    rule_svef: RuleSVEF,
}

impl RuleIVEA {
    pub(crate) fn new(pattern: Normal, ground: Normal, variable: Option<Symbol>) -> Self {
        Self {
            pattern: pattern.clone(),
            ground: ground.clone(),
            rule_dnc: RuleDNC::new(pattern.clone(), ground.clone()),
            rule_svef: RuleSVEF::new(pattern.clone(), ground.clone(), variable.clone(), false),
        }
    }
}

impl MatchRule for RuleIVEA {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let (p0, _) = (p.part(0)?, g.part(0)?);
        let (variable, _) = parse_individual_variable(p0)?;

        // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

        Some(Self::new(p.clone(), g.clone(), variable.cloned()))
    }
}

impl MatchGenerator for RuleIVEA {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleIVEA {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        // Attempt decomposition first.
        // This ensures we put the singleton results ahead of the function application results.
        if let Some(result) = self.rule_dnc.next() {
            return Some(result);
        }

        // Next attempt to get the next result from `RuleSVEF`.
        let result = self.rule_svef.next()?;

        // Transform the results list into a consuming iterator and transform substitutions
        // into function application(s) instead of sequence values.
        let transformed_results = result
            .into_iter()
            .map(|match_result| match match_result {
                MatchResult::Substitution(Substitution { variable, ground }) => {
                    let sequence_elements = try_sequence(&ground)
                        .expect("RuleSVEA should only produce Sequence[...] results");

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
