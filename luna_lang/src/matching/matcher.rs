use crate::matching::MatchRule;
use crate::matching::rule_fve::RuleFVE;
use crate::matching::rule_ive::RuleIVE;
use crate::matching::rule_t::RuleT;
use crate::{
    Atom, Context, MatchEquation, MatchGenerator, MatchResult, MatchResultList, SolutionSet,
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

    /// A variable or sequence variable substitution. We only need to record the key
    /// (the expression) of the `SolutionSet` hashmap.
    Substitution(Atom),

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
    pub fn new(pattern: impl Into<Atom>, subject: impl Into<Atom>, context: &'c Context) -> Self {
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
    fn select_rule(&mut self) -> Result<BoxedMatchGenerator, ()> {
        // Nothing to select.
        if self.equation_stack.is_empty() {
            return Err(());
        }

        // TODO: Substitute bound variables with their values.
        let match_equation = self.equation_stack.pop().unwrap();

        if let Some(rule) = RuleT::try_rule(&match_equation) {
            return Ok(Box::new(rule));
        }

        if let Some(rule) = RuleIVE::try_rule(&match_equation) {
            return Ok(Box::new(rule));
        }

        if let Some(rule) = RuleFVE::try_rule(&match_equation) {
            return Ok(Box::new(rule));
        }

        // In order to begin destructuring both `pattern` and `ground` must be S-Expressions.
        if !match_equation.pattern.is_sexpr() || !match_equation.ground.is_sexpr() {
            self.equation_stack.push(match_equation);
            return Err(());
        }

        // The heads of both `pattern` and `ground` must match for us to perform any sensible
        // destructuring.
        if match_equation.pattern.head() != match_equation.ground.head() {
            self.equation_stack.push(match_equation);
            return Err(());
        }

        let ground_attributes = self
            .context
            .get_attributes(match_equation.ground.name().unwrap());

        match (
            ground_attributes.commutative(),
            ground_attributes.associative(),
        ) {
            // Rules for Free Functions (neither associative nor commutative)
            (false, false) => {
                // if let Some(rule) = RuleDecF::try_rule(&match_equation) {
                //   return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleSVEF::try_rule(&match_equation) {
                //   return Ok(Box::new(rule));
                // }

                self.equation_stack.push(match_equation);
                Err(())
            }

            // Rules for commutative functions
            (true, false) => {
                // if let Some(rule) = RuleDecC::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleSVEC::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }

                self.equation_stack.push(match_equation);
                Err(())
            }

            // Rules for associative functions
            (false, true) => {
                // if let Some(rule) = RuleSVEA::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleFVEA::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleIVEA::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleDecA::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }

                self.equation_stack.push(match_equation);
                Err(())
            }

            // Rules for associative-commutative symbols.
            (true, true) => {
                // if let Some(rule) = RuleSVEAC::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleFVEAC::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleIVEAC::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }
                //
                // if let Some(rule) = RuleDecAC::try_rule(&match_equation) {
                //     return Ok(Box::new(rule));
                // }

                self.equation_stack.push(match_equation);
                Err(())
            }
        }
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
                Ok(match_generator) => {
                    // Push the `MatchGenerator` onto the match stack.
                    // It is now the active `MatchGenerator`.
                    self.push_rule(match_generator);
                }

                // No rule applies.
                Err(_) => {
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
                    // If there is nothing left on the match stack then fail.
                    None => {
                        return None;
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
