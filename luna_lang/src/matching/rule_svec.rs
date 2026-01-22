use crate::matching::permutations::PermutationGenerator32;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};
use crate::matching::subsets::next_subset;

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

    /// Bit positions indicate which subset of the ground's arguments are currently being matched
    /// against.
    subset: u32,

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
                        subset: if matches_empty { 0 } else { 1 },
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
        if self.subset == 0 {
            self.subset = 1;

            let match_equation = MatchResult::MatchEquation(MatchEquation {
                pattern: Expr::from(Normal::new(
                    self.pattern.head().clone(),
                    &self.pattern.elements()[1..],
                )),
                ground: Expr::from(self.ground.clone()),
            });

            if let Some(variable) = &self.variable {
                let substitution = MatchResult::Substitution(Substitution {
                    variable: variable.clone(),
                    ground: Expr::from(Normal::new(Symbol::new("Sequence"), vec![])),
                });

                return Some(vec![match_equation, substitution]);
            }

            return Some(vec![match_equation]);
        }

        if self.subset == 1 && self.ground.is_empty() {
            return None;
        }

        let permutation = match self.permutations.next() {
            Some(permutation) => permutation,
            None => {
                if let Some(next_subset) = next_subset(self.ground.len() as u32, self.subset) {
                    self.subset = next_subset;
                    self.permutations = PermutationGenerator32::new(self.subset.count_ones() as u8);
                    self.permutations.next()?
                } else {
                    return None;
                }
            }
        };

        let mut subset = Vec::with_capacity(self.subset.count_ones() as usize);
        let mut complement = Vec::with_capacity(self.subset.count_zeros() as usize);

        for (k, c) in self.ground.elements().iter().enumerate() {
            if ((1 << k) as u32 & self.subset) != 0 {
                subset.push(c);
            } else {
                complement.push(c.clone());
            }
        }

        let match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(self.ground.head().clone(), complement)),
        });

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
