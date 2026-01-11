use crate::{Atom, IString};

/// Matches an atom of the form `Sequence[a, b, ...]`, returning the children `a, b, ...`.
pub fn is_sequence(atom: &Atom) -> Option<&[Atom]> {
    if atom.has_symbol_head("Sequence") {
        return Some(&atom.parts()[1..]);
    }

    None
}

/// Matches an atom of the form `Pattern[symbol, pattern[h]]`, returning `symbol` and `h`.
pub fn is_pattern(atom: &Atom, pattern: impl Into<IString>) -> Option<(IString, Option<&Atom>)> {
    if atom.has_symbol_head("Pattern") {
        if let [_, Atom::Symbol(symbol), pat] = atom.parts() {
            if pat.has_symbol_head(pattern) {
                return match pat.len() {
                    1 | 2 => Some((*symbol, pat.parts().get(1))),
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
pub fn is_any_pattern(atom: &Atom) -> Option<(IString, Option<&Atom>)> {
    if atom.has_symbol_head("Pattern") {
        if let [_, Atom::Symbol(symbol), pat] = atom.parts() {
            if pat.has_symbol_head("Blank")
                || pat.head().has_symbol_head("BlankSequence")
                || pat.head().has_symbol_head("BlankNullSequence")
            {
                return match pat.len() {
                    1 | 2 => Some((*symbol, pat.parts().get(1))),
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
pub fn is_any_sequence_pattern(atom: &Atom) -> Option<(IString, Option<&Atom>)> {
    if atom.has_symbol_head("Pattern") {
        if let [_, Atom::Symbol(symbol), pat] = atom.parts() {
            if pat.has_symbol_head("BlankSequence") || pat.has_symbol_head("BlankNullSequence") {
                return match pat.len() {
                    1 | 2 => Some((*symbol, pat.parts().get(1))),
                    _ => None,
                };
            }
        }
    }

    None
}
