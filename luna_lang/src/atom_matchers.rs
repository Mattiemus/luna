use crate::Atom;

/// Matches an atom of the form `Sequence[___]`.
pub fn is_sequence(atom: &Atom) -> bool {
    atom.has_symbol_head("Sequence")
}

/// Matches an atom of any the following forms:
///
/// - `Pattern[s_Symbol, Blank[]]`
/// - `Pattern[s_Symbol, Blank[h]]`
pub fn is_blank_pattern(atom: &Atom) -> bool {
    if let Some([_, Atom::Symbol(_), pat]) = atom.try_as_apply_symbol("Pattern") {
        if pat.has_symbol_head("Blank") {
            return pat.len() == 1 || pat.len() == 2;
        }
    }

    false
}

/// Matches an atom of any the following forms:
///
/// - `Pattern[s_Symbol, BlankSequence[]]`
/// - `Pattern[s_Symbol, BlankSequence[h]]`
/// - `Pattern[s_Symbol, BlankNullSequence[]]`
/// - `Pattern[s_Symbol, BlankNullSequence[h]]`
pub fn is_any_sequence_pattern(atom: &Atom) -> bool {
    if let Some([_, Atom::Symbol(_), pat]) = atom.try_as_apply_symbol("Pattern") {
        if pat.has_symbol_head("BlankSequence") || pat.has_symbol_head("BlankNullSequence") {
            return pat.len() == 1 || pat.len() == 2;
        }
    }

    false
}
