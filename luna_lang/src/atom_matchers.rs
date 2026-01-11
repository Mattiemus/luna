use crate::{Atom, Symbol};

/// Matches an atom of the form `Sequence[a, b, ...]`, returning the children `a, b, ...`.
pub fn is_sequence(atom: &Atom) -> Option<&[Atom]> {
    if atom.has_head(Symbol::new("Sequence")) {
        return Some(&atom.parts()[1..]);
    }

    None
}

/// Matches an atom of the form `Pattern[symbol, pattern[h]]`, returning `symbol` and `h`.
pub fn is_pattern(atom: &Atom, pattern: impl Into<Symbol>) -> Option<(&Symbol, Option<&Atom>)> {
    if let Some(sexpr) = atom.as_sexpr_with_head(Symbol::new("Pattern")) {
        if let [_, Atom::Symbol(symbol), pat] = sexpr.parts() {
            if let Some(pat_sexpr) = pat.as_sexpr_with_head(pattern.into()) {
                return match pat_sexpr.len() {
                    1 | 2 => Some((symbol, pat_sexpr.part(1))),
                    _ => None,
                };
            }
        }
    }

    None
}

/// Matches an atom of any the following forms:
///
/// - `Pattern[symbol, Blank[h]]`
/// - `Pattern[symbol, BlankSequence[h]]`
/// - `Pattern[symbol, BlankNullSequence[h]]`
///
/// Returning `symbol` and `h`.
pub fn is_any_pattern(atom: &Atom) -> Option<(&Symbol, Option<&Atom>)> {
    if atom.has_head(Symbol::new("Pattern")) {
        if let [_, Atom::Symbol(symbol), pat] = atom.parts() {
            if pat.has_head(Symbol::new("Blank"))
                || pat.head().has_head(Symbol::new("BlankSequence"))
                || pat.head().has_head(Symbol::new("BlankNullSequence"))
            {
                return match pat.len() {
                    1 | 2 => Some((symbol, pat.part(1))),
                    _ => None,
                };
            }
        }
    }

    None
}

/// Matches an atom of any the following forms:
///
/// - `Pattern[symbol, BlankSequence[h]]`
/// - `Pattern[symbol, BlankNullSequence[h]]`
///
/// Returning `symbol` and `h`.
pub fn is_any_sequence_pattern(atom: &Atom) -> Option<(&Symbol, Option<&Atom>)> {
    if let Some(sexpr) = atom.as_sexpr_with_head(Symbol::new("Pattern")) {
        if let [_, Atom::Symbol(symbol), pat] = sexpr.parts() {
            if pat.has_head(Symbol::new("BlankSequence"))
                || pat.has_head(Symbol::new("BlankNullSequence"))
            {
                return match pat.len() {
                    1 | 2 => Some((symbol, pat.part(1))),
                    _ => None,
                };
            }
        }
    }

    None
}
