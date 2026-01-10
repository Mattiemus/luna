use crate::matching::MatchRule;
use crate::matching::rule_trivial::RuleTrivial;
use crate::{
    Atom, Context, MatchEquation, MatchGenerator, MatchResult, MatchResultList, SolutionSet,
    Substitution,
};
use std::collections::HashMap;

type BoxedMatchGenerator = Box<dyn MatchGenerator>;

/// Items that can be pushed onto the match stack.
enum MatchStack {
    /// The match generator responsible for the operations sitting immediately above it on
    /// the stack. Those operations are undone in order to get back to the
    /// match generator to call `next()`.
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
    /// Create a new `MatchGenerator` for the match equation `pattern ≪ subject`.
    pub fn new(pattern: Atom, subject: Atom, context: &'c Context) -> Self {
        Self {
            context,
            match_stack: Vec::new(),
            equation_stack: vec![MatchEquation {
                pattern,
                ground: subject,
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
        let me = self.equation_stack.pop().unwrap();

        if let Some(rule) = RuleTrivial::try_rule(&me) {
            return Ok(Box::new(rule));
        }

        //     if let Some(rule) = RuleIVE::try_rule(&me) {
        //       self.push_rule(Box::new(rule));
        //       return Ok(());
        //     }

        //     if let Some(rule) = RuleFVE::try_rule(&me) {
        //       self.push_rule(Box::new(rule));
        //       return Ok(());
        //     }

        //     // A prerequisite for destructuring f/g and good place to bail early.
        //     if me.pattern.kind() != AtomKind::SExpression
        //         || me.ground.kind() != AtomKind::SExpression
        //     {
        //       // Return match equation.
        //       self.equation_stack.push(me);
        //       return Err(());
        //     }
        //
        //     // Another opportunity to bail early. This indicates program state that should be impossible.
        //     // Note: The case of `x_ << ƒ[…]` should have been taken care of in IVE above.
        //     if me.pattern.head() != me.ground.head() {
        //       log(Channel::Debug, 5, "Both sides not functions.".to_string().as_str());
        //       // Return match equation.
        //       self.equation_stack.push(me);
        //       return Err(());
        //     }
        //
        //     let ground_attributes: Attributes = context.get_attributes(me.ground.name().unwrap());
        //
        //     match (ground_attributes.commutative(), ground_attributes.associative()) {
        //
        //       // Rules for Free Functions (neither associative nor commutative)
        //       (false, false) => {
        //         // Dec-F
        //         if let Some(rule) = RuleDecF::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating Dec-F for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // SVE-F
        //         else if let Some(rule) = RuleSVEF::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating DVE-F for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         } else {
        //           // Return match equation.
        //           self.equation_stack.push(me);
        //           log(Channel::Debug, 5, format!("No applicable (free) rule found.").as_str());
        //           return Err(());
        //         }
        //         Ok(())
        //       }
        //
        //       // Rules for commutative functions
        //       (true, false) => {
        //
        //         // Dec-C
        //         if let Some(rule) = RuleDecC::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating Dec-C for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // SVE-C
        //         else if let Some(rule) = RuleSVEC::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating SVE-C for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         } else {
        //           // Return match equation.
        //           self.equation_stack.push(me);
        //           log(Channel::Debug, 5, format!("No applicable (commutative) rule found.").as_str());
        //           return Err(());
        //         }
        //         Ok(())
        //       }
        //
        //       // Rules for associative functions
        //       (false, true) => {
        //
        //         // SVE-A
        //         if let Some(rule) = RuleSVEA::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating SVE-A for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // FVE-A
        //         else if let Some(rule) = RuleFVEA::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating FVE-A for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // IVE-A
        //         else if let Some(rule) = RuleIVEA::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating IVE-A for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // Dec-A
        //         else if let Some(rule) = RuleDecA::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating Dec-A for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         } else {
        //           log(Channel::Debug, 5, format!("No applicable (associative) rule found.").as_str());
        //           // Return match equation.
        //           self.equation_stack.push(me);
        //           return Err(());
        //         }
        //         Ok(())
        //       }
        //
        //       // Rules for associative-commutative symbols.
        //       (true, true) => {
        //
        //         // SVE-AC
        //         if let Some(rule) = RuleSVEAC::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating SVE-AC for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // FVE-AC
        //         else if let Some(rule) = RuleFVEAC::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating FVE-AC for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // IVE-AC
        //         else if let Some(rule) = RuleIVEAC::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating IVE-AC for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         }
        //         // Dec-AC
        //         else if let Some(rule) = RuleDecAC::try_rule(&me) {
        //           log(Channel::Debug, 5, format!("Creating Dec-AC for {}", me).as_str());
        //           self.push_rule(Box::new(rule));
        //         } else {
        //           log(Channel::Debug, 5, format!("No applicable (associative-commutative) rule found.").as_str());
        //           // Return match equation.
        //           self.equation_stack.push(me);
        //           return Err(());
        //         }
        //         Ok(())
        //       }
        //     } // end match on (commutative, associative)

        // TODO
        Err(())
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
                    // Push the `MatchGenerator` onto the match stack. It is not the active
                    // `MatchGenerator`.
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
                    None => {
                        return None;
                    }

                    Some(MatchStack::MatchGenerator(match_generator)) => {
                        match match_generator.next() {
                            // Found a potential match.
                            Some(results) => {
                                self.process_match_list(results);

                                // If the equation stack is empty then we have successfully found a match.
                                if self.equation_stack.is_empty() {
                                    return Some(self.substitutions.clone());
                                }

                                continue 'step1;
                            }

                            // No matches found.
                            // Backtrack to the previous matcher.
                            None => {
                                self.backtrack();

                                // Fail if theres no previous matcher to backtrack to.
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
