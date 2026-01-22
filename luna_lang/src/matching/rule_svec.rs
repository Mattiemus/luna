use crate::matching::permutations::PermutationGenerator32;
use crate::matching::subsets::Subset;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};

/// Sequence variable elimination under a commutative head.
///
/// Matches a pattern `f[x__, ...]` or `f[x___, ...]` against a value `g[...]` where `f` is a
/// commutative function.
///
/// Assumptions:
/// - `f` is a commutative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleSVEC {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,

    /// Have we produced the empty sequence as the first result yet?
    empty_produced: bool,

    /// Current subset of the grounds arguments which are being matched against.
    subset: Subset,

    /// Generator for all permutations of the current subset of elements.
    permutations: PermutationGenerator32,
}

impl MatchRule for RuleSVEC {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if let (Some(p), Some(g)) = (
            match_equation.pattern.try_normal(),
            match_equation.ground.try_normal(),
        ) {
            if let Some(p0) = p.part(0) {
                if let Some((matches_empty, variable, _)) = parse_any_sequence_variable(p0) {
                    // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

                    return Some(Self {
                        pattern: p.clone(),
                        ground: g.clone(),
                        variable: variable.cloned(),
                        empty_produced: !matches_empty,
                        subset: Subset::empty(g.len()),
                        permutations: PermutationGenerator32::new(1),
                    });
                }
            }
        }

        None
    }
}

impl MatchGenerator for RuleSVEC {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleSVEC {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        // If the current subset is empty, and we have not yet produced the empty sequence we should
        // do that now.
        if self.subset.is_zero() && !self.empty_produced {
            self.empty_produced = true;

            // Attempt to continue to match `f[...]` against `g[...]`.
            let match_equation = MatchResult::MatchEquation(MatchEquation {
                pattern: Expr::from(Normal::new(
                    self.pattern.head().clone(),
                    &self.pattern.elements()[1..],
                )),
                ground: Expr::from(self.ground.clone()),
            });

            // Create the substitution so long as the pattern was named.
            if let Some(variable) = &self.variable {
                let substitution = MatchResult::Substitution(Substitution {
                    variable: variable.clone(),
                    ground: Expr::from(Normal::new(Symbol::new("Sequence"), vec![])),
                });

                return Some(vec![match_equation, substitution]);
            }

            return Some(vec![match_equation]);
        }

        // If the current subset is empty we should try and increment it.
        // It is possible to bail early here if ground is `f[]` (i.e. it is empty).
        if self.subset.is_zero() {
            self.subset = self.subset.next()?;
        }

        // Try and get the next permutation for `ground`s elements.
        let permutation = match self.permutations.next() {
            Some(permutation) => permutation,
            None => {
                self.subset = self.subset.next()?;
                self.permutations = PermutationGenerator32::new(self.subset.count_ones() as u8);
                self.permutations.next()?
            }
        };

        // Extract the subset and complement from the current subset
        let (subset, complement) = self.subset.extract(self.ground.elements());

        // Continue matching `f[...]` against the pattern `g[...]` where the contents of `g` is the
        // complement of the current subset.
        let match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(self.ground.head().clone(), complement)),
        });

        // The current subset ordered into the current permutation represents the substitution
        // for `x`.
        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(Normal::new(
                    Symbol::new("Sequence"),
                    permutation
                        .map(|idx| subset[idx].clone())
                        .collect::<Vec<_>>(),
                )),
            });

            return Some(vec![match_equation, substitution]);
        }

        Some(vec![match_equation])
    }
}
