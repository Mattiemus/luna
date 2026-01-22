use crate::matching::function_application::{AFAGenerator, FunctionApplicationGenerator};
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};

pub(crate) struct RuleSVEA {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,
    ground_sequence: Vec<Expr>,
    afa_generator: Option<Box<AFAGenerator>>,
}

impl RuleSVEA {
    fn make_next(&self, ordered_sequence: Vec<Expr>) -> MatchResultList {
        let result_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(
                self.ground.head().clone(),
                &self.ground.elements()[self.ground_sequence.len()..],
            )),
        });

        if let Some(variable) = &self.variable {
            let result_substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(Normal::new(Symbol::new("Sequence"), ordered_sequence)),
            });

            return vec![result_equation, result_substitution];
        }

        vec![result_equation]
    }
}

impl MatchRule for RuleSVEA {
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
                        ground_sequence: Vec::new(),
                        afa_generator: if matches_empty {
                            None
                        } else {
                            Some(Box::new(AFAGenerator::new(Normal::new(
                                g.head().clone(),
                                vec![],
                            ))))
                        },
                    });
                }
            }
        }

        None
    }
}

impl MatchGenerator for RuleSVEA {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleSVEA {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.afa_generator {
            None => {
                self.afa_generator = Some(Box::new(AFAGenerator::new(Normal::new(
                    self.ground.head().clone(),
                    vec![],
                ))));

                Some(self.make_next(Vec::new()))
            }

            Some(afa_generator) => {
                let ordered_sequence = match afa_generator.next() {
                    None => {
                        if let Some(next_element) = self.ground.part(self.ground_sequence.len()) {
                            self.ground_sequence.push(next_element.clone());
                        } else {
                            return None;
                        }

                        let mut new_afa_generator = AFAGenerator::new(Normal::new(
                            self.ground.head().clone(),
                            self.ground_sequence.clone(),
                        ));

                        let next_result = new_afa_generator.next().unwrap();
                        self.afa_generator = Some(Box::new(new_afa_generator));

                        next_result
                    }

                    Some(next_result) => next_result,
                };

                Some(self.make_next(ordered_sequence))
            }
        }
    }
}
