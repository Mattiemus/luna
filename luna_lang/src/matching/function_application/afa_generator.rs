use crate::matching::function_application::FunctionApplicationGenerator;
use crate::matching::subsets::Subset;
use crate::{Expr, Normal};

/// Associative Function Application Generator.
pub struct AFAGenerator {
    function: Normal,
    exhausted: bool,

    /// Encodes which terms are outside any function application.
    singleton_state: Subset,

    /// Encodes how terms are grouped into function applications.
    application_state: Subset,
}

impl FunctionApplicationGenerator for AFAGenerator {
    fn new(function: Normal) -> Self {
        if function.is_empty() {
            return Self {
                function,
                exhausted: true,
                singleton_state: Subset::empty(0),
                application_state: Subset::empty(0),
            };
        }

        let initial_application_state = Subset::empty(function.len() - 1);

        Self {
            function,
            exhausted: false,
            singleton_state: Subset::empty(0),
            application_state: initial_application_state,
        }
    }
}

impl Iterator for AFAGenerator {
    type Item = Vec<Expr>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        let mut last_boundary_position = 0;
        let mut singleton_count = 0;

        let mut result_sequence = Vec::with_capacity(self.function.len());

        for position in 1..=self.function.len() {
            if position == self.function.len() || !self.application_state.get(position - 1) {
                if position - last_boundary_position > 1 {
                    let new_function = Expr::from(Normal::new(
                        self.function.head().clone(),
                        &self.function.elements()[last_boundary_position..position],
                    ));

                    result_sequence.push(new_function);
                } else {
                    if self.singleton_state.get(singleton_count) {
                        let new_function = Expr::from(Normal::new(
                            self.function.head().clone(),
                            [self.function.element(position - 1).clone()],
                        ));

                        result_sequence.push(new_function);
                    } else {
                        result_sequence.push(self.function.element(position - 1).clone());
                    }

                    singleton_count += 1;
                }

                last_boundary_position = position;
            }
        }

        if let Some(next_singleton_state) = self.singleton_state.resize_next(singleton_count) {
            self.singleton_state = next_singleton_state;
        } else {
            if let Some(next_application_state) = self.application_state.next() {
                self.singleton_state = Subset::empty(0);
                self.application_state = next_application_state;
            } else {
                self.exhausted = true;
            }
        }

        Some(result_sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn mk(exprs: &[&str]) -> Option<Vec<Expr>> {
        Some(exprs.iter().map(|e| parse(e).unwrap()).collect::<Vec<_>>())
    }

    #[test]
    fn generates_all_afa_applications() {
        let f = parse("f[a, b, c]").unwrap();
        let f_normal = f.try_normal().unwrap().clone();

        let mut afa_gen = AFAGenerator::new(f_normal);

        assert_eq!(afa_gen.next(), mk(&["a", "b", "c"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "b", "c"]));
        assert_eq!(afa_gen.next(), mk(&["a", "f[b]", "c"]));
        assert_eq!(afa_gen.next(), mk(&["a", "b", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "f[b]", "c"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "b", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["a", "f[b]", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "f[b]", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a, b]", "c"]));
        assert_eq!(afa_gen.next(), mk(&["f[a, b]", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["a", "f[b, c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "f[b, c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a, b, c]"]));
        assert_eq!(afa_gen.next(), None);
    }
}
