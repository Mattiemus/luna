use crate::Symbol;
use crate::matching::MatchRule;
use crate::matching::rule_df::RuleDF;
use crate::matching::rule_fve::RuleFVE;
use crate::matching::rule_ive::RuleIVE;
use crate::matching::rule_t::RuleT;
use crate::{
    Context, Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, SolutionSet,
    Substitution,
};
use std::collections::HashMap;

type BoxedMatchGenerator = Box<dyn MatchGenerator>;

/// Items that can be pushed onto the match stack.
enum MatchStack {
    /// The match generator responsible for the operations sitting immediately above it on
    /// the stack. Those operations are undone in order to get back to the  match generator
    /// to call `next()`.
    MatchGenerator(BoxedMatchGenerator),

    /// A variable or sequence variable substitution. We only need to record the name of the
    /// variable.
    Substitution(Symbol),

    /// An operation representing pushing matching equations onto the equation stack.
    ProducedMatchEquations(usize),
}

/// Holds the state of the in-process pattern matching attempt.
pub struct Matcher<'c> {
    context: &'c Context,

    /// The match_stack is where operations that change the state are recorded.
    /// Operations are pushed when they are done and popped when they are undone.
    match_stack: Vec<MatchStack>,

    /// The match equations that still need to be solved.
    equation_stack: Vec<MatchEquation>,

    /// The symbol table recording all variable/sequence variable bindings.
    substitutions: SolutionSet,
}

impl<'c> Matcher<'c> {
    pub fn new(pattern: impl Into<Expr>, subject: impl Into<Expr>, context: &'c Context) -> Self {
        Self {
            context,
            match_stack: Vec::new(),
            equation_stack: vec![MatchEquation {
                pattern: pattern.into(),
                ground: subject.into(),
            }],
            substitutions: HashMap::new(),
        }
    }

    /// Check which rule applies to the active match equation, creates the match generator for that
    /// rule, and pushes the match generator onto the match stack.
    fn select_rule(&mut self) -> Option<BoxedMatchGenerator> {
        // TODO: Substitute bound variables with their values. This probably should NOT happen here
        //  though as we would otherwise be rebuilding `match_equation` on every evaluation step.
        let match_equation = match self.equation_stack.pop() {
            Some(match_equation) => match_equation,
            None => return None,
        };

        if let Some(rule) = RuleT::try_rule(&match_equation) {
            return Some(Box::new(rule));
        }

        if let Some(rule) = RuleIVE::try_rule(&match_equation) {
            return Some(Box::new(rule));
        }

        if let Some(rule) = RuleFVE::try_rule(&match_equation) {
            return Some(Box::new(rule));
        }

        // At this point we next need to perform destructuring type operations. This requests both
        // `pattern` and `ground` to be expressions.
        if let (Some(p), Some(g)) = (
            match_equation.pattern.try_normal(),
            match_equation.ground.try_normal(),
        ) {
            // TODO: Need to be able to match `f[...][...]` to `g[...][...]`. Will need to add
            //  rules to destructure the head in this case. This should be recursive to support
            //  operations like `f[...][...][...]` etc.

            // Attempting to match `f[...]` with `g[...]' where `f` and `g` are symbols and match.
            if let (Some(phead), Some(ghead)) = (p.try_head_symbol(), g.try_head_symbol()) {
                if phead != ghead {
                    self.equation_stack.push(match_equation);
                    return None;
                }

                let ground_attributes = self.context.get_attributes(ghead);

                match (
                    ground_attributes.commutative(),
                    ground_attributes.associative(),
                ) {
                    // Rules for Free Functions (neither associative nor commutative).
                    (false, false) => {
                        if let Some(rule) = RuleDF::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        // if let Some(rule) = RuleSVEF::try_rule(&match_equation) {
                        //   return Some(Box::new(rule));
                        // }

                        self.equation_stack.push(match_equation);
                        return None;
                    }

                    // Rules for commutative functions.
                    (true, false) => {
                        // if let Some(rule) = RuleDecC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleSVEC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }

                        self.equation_stack.push(match_equation);
                        return None;
                    }

                    // Rules for associative functions.
                    (false, true) => {
                        // if let Some(rule) = RuleSVEA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleFVEA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleIVEA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleDecA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }

                        self.equation_stack.push(match_equation);
                        return None;
                    }

                    // Rules for associative-commutative symbols.
                    (true, true) => {
                        // if let Some(rule) = RuleSVEAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleFVEAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleIVEAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleDecAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }

                        self.equation_stack.push(match_equation);
                        return None;
                    }
                }
            }
        }

        self.equation_stack.push(match_equation);
        None
    }

    /// Undoes the effects of the last call to `next()`, removes the match generator, and
    /// restores any match equation corresponding to the match generator, if any.
    fn backtrack(&mut self) {
        let match_generator = self.undo();
        self.equation_stack.push(match_generator.match_equation());
    }

    /// Undoes the effects of the last call to `next()` and, for convenience, returns the
    /// responsible match generator.
    /// Upon return the `MatchGenerator` will be active, i.e. on top of the match stack.
    fn undo(&mut self) -> BoxedMatchGenerator {
        loop {
            match self.match_stack.pop().unwrap() {
                // Leave the match generator on top of the match stack.
                // Don't restore its match equation.
                MatchStack::MatchGenerator(match_generator) => {
                    return match_generator;
                }

                // Remove the substitution
                MatchStack::Substitution(expression) => {
                    self.substitutions.remove(&expression);
                }

                // Remove the produced equations
                MatchStack::ProducedMatchEquations(added) => {
                    let new_length = self.equation_stack.len() - added;
                    self.equation_stack.truncate(new_length);
                }
            }
        }
    }

    fn process_match_list(&mut self, mut results: MatchResultList) {
        let mut equation_count: usize = 0;

        for result in results.drain(..) {
            match result {
                MatchResult::Substitution(substitution) => {
                    self.push_substitution(substitution);
                }

                MatchResult::MatchEquation(match_equation) => {
                    self.equation_stack.push(match_equation);
                    equation_count += 1;
                }
            }
        }

        if equation_count > 0 {
            self.push_match_equations(equation_count);
        }
    }

    fn push_rule(&mut self, generator: BoxedMatchGenerator) {
        self.match_stack.push(MatchStack::MatchGenerator(generator));
    }

    fn push_substitution(&mut self, Substitution { variable, ground }: Substitution) {
        self.substitutions.insert(variable.clone(), ground.clone());

        self.match_stack
            .push(MatchStack::Substitution(variable.clone()));
    }

    fn push_match_equations(&mut self, equation_count: usize) {
        self.match_stack
            .push(MatchStack::ProducedMatchEquations(equation_count));
    }
}

impl<'c> Iterator for Matcher<'c> {
    type Item = SolutionSet;

    fn next(&mut self) -> Option<Self::Item> {
        // If the last match was successful, the equation stack will be empty. But there could be
        // more solutions possible, in which case backtracking will put equations back on the stack.
        if self.equation_stack.is_empty() && self.match_stack.is_empty() {
            return None;
        }

        'step1: loop {
            // Attempt to select a rule to apply
            match self.select_rule() {
                // We have found a rule to apply.
                Some(match_generator) => {
                    // Push the `MatchGenerator` onto the match stack.
                    // It is now the active `MatchGenerator`.
                    self.push_rule(match_generator);
                }

                // No rule applies.
                None => {
                    // If the match stack is empty, halt with failure.
                    if self.match_stack.is_empty() {
                        return None;
                    }

                    // If there is an active match generator on top of the matcher stack, undo the
                    // actions of the last match generated from this match generator.
                    let match_generator = self.undo();

                    // But retain the active match generator.
                    self.push_rule(match_generator);
                }
            }

            // Apply the rule.
            'step2: loop {
                match self.match_stack.last_mut() {
                    None => {
                        panic!("Expected value on the match stack. Found nothing.")
                    }

                    Some(MatchStack::MatchGenerator(match_generator)) => {
                        match match_generator.next() {
                            // Found a match.
                            Some(results) => {
                                self.process_match_list(results);

                                // Succeed if the equation stack is empty.
                                if self.equation_stack.is_empty() {
                                    return Some(self.substitutions.clone());
                                }

                                continue 'step1;
                            }

                            // No matches found.
                            // Backtrack to the previous matcher.
                            None => {
                                self.backtrack();

                                // Fail if there is no previous matcher to backtrack to.
                                if self.match_stack.is_empty() {
                                    return None;
                                }

                                let match_generator = self.undo();
                                self.push_rule(match_generator);

                                continue 'step2;
                            }
                        }
                    }

                    Some(MatchStack::Substitution(substitution)) => {
                        panic!(
                            "Expected a MatchGenerator. Found substitution: '{}'.",
                            substitution
                        )
                    }

                    Some(MatchStack::ProducedMatchEquations(me)) => {
                        panic!(
                            "Expected a MatchGenerator. Found {} produced match equations.",
                            me
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use test_case::test_case;

    #[test_case("\"abc\"", "\"def\"" ; "mismatched strings")]
    #[test_case("123", "456" ; "mismatched integers")]
    #[test_case("2.5", "8.75" ; "mismatched reals")]
    #[test_case("abc", "def" ; "mismatched symbols")]
    #[test_case("f[a, b, c]", "g[a, b, c]" ; "mismatched expression heads")]
    fn unmatchable_expressions_gives_no_solutions(pattern: &str, ground: &str) {
        let context = Context::new();

        let mut matcher = Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

        assert_eq!(matcher.next(), None);
    }

    #[test_case("\"abc\"" ; "strings")]
    #[test_case("123" ; "integers")]
    #[test_case("2.5" ; "reals")]
    #[test_case("abc" ; "symbols")]
    #[test_case("f[a, b, c]" ; "expressions")]
    fn exact_matches_gives_single_empty_solution(pattern: &str) {
        let context = Context::new();

        let mut matcher = Matcher::new(parse(pattern).unwrap(), parse(pattern).unwrap(), &context);

        assert_eq!(matcher.next(), Some(HashMap::new()));
        assert_eq!(matcher.next(), None);
    }

    mod free_functions {
        use super::*;
        use test_case::test_case;

        #[test_case("f[a, b, c]", "g[a, b, c]" ; "mismatched expression heads")]
        #[test_case("f[a, b, c]", "f[d, e, f]" ; "mismatched expression elements")]
        fn unmatchable_expressions_gives_no_solutions(pattern: &str, ground: &str) {
            let context = Context::new();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_eq!(matcher.next(), None);
        }

        #[test_case("f[_, b, c]", "f[a, b, c]" ; "in first element")]
        #[test_case("f[a, _, c]", "f[a, b, c]" ; "in second element")]
        #[test_case("f[a, b, _]", "f[a, b, c]" ; "in third element")]
        #[test_case("f[_, _, c]", "f[a, b, c]" ; "in first and second elements")]
        #[test_case("f[a, _, _]", "f[a, b, c]" ; "in second and third elements")]
        #[test_case("f[_, b, _]", "f[a, b, c]" ; "in first and third elements")]
        #[test_case("f[_, _, _]", "f[a, b, c]" ; "in all elements")]
        fn handles__blank_unnamed_variables(pattern: &str, ground: &str) {
            let context = Context::new();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_eq!(matcher.next(), Some(HashMap::new()));
            assert_eq!(matcher.next(), None);
        }

        #[test_case("f[x_, b, c]", "f[a, b, c]", "a" ; "in first element")]
        #[test_case("f[a, x_, c]", "f[a, b, c]", "b" ; "in second element")]
        #[test_case("f[a, b, x_]", "f[a, b, c]", "c" ; "in third element")]
        fn handles_single_blank_named_variable(pattern: &str, ground: &str, x: &str) {
            let context = Context::new();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_eq!(
                matcher.next(),
                Some(HashMap::from([(Symbol::new("x"), parse(x).unwrap())]))
            );
            assert_eq!(matcher.next(), None);
        }

        #[test_case("f[x_, y_, c]", "f[a, b, c]", "a", "b" ; "in first and second elements")]
        #[test_case("f[x_, b, y_]", "f[a, b, c]", "a", "c" ; "in first and third elements")]
        #[test_case("f[a, y_, x_]", "f[a, b, c]", "c", "b" ; "in second and third elements")]
        fn handles_two_blank_named_variables(pattern: &str, ground: &str, x: &str, y: &str) {
            let context = Context::new();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_eq!(
                matcher.next(),
                Some(HashMap::from([
                    (Symbol::new("x"), parse(x).unwrap()),
                    (Symbol::new("y"), parse(y).unwrap()),
                ]))
            );
            assert_eq!(matcher.next(), None);
        }

        // TODO: f[__]
        // TODO: f[___]

        // TODO: f[x_, x_, c]

        // TODO: f_[a, b, c]
        // TODO: f_[x_ b, c]
    }
}
