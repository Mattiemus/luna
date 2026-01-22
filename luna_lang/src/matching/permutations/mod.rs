use bit_index::BitIndex32;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SinglePermutation32 {
    elems: BitIndex32,
    next_mod: u128,
    current_idx: u128,
}

impl SinglePermutation32 {
    pub(crate) fn new(nb_elems: u8, nb_perms: u128, idx: u128) -> Option<Self> {
        if idx >= nb_perms {
            None
        } else {
            Some(Self {
                elems: BitIndex32::new(nb_elems).unwrap(),
                next_mod: nb_perms / (nb_elems as u128),
                current_idx: idx,
            })
        }
    }
}

impl Iterator for SinglePermutation32 {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elems.nb_elements() == 0 {
            return None;
        }
        let bit_nb = self.current_idx / self.next_mod;
        self.current_idx -= bit_nb * self.next_mod;
        self.next_mod /= (self.elems.nb_elements() as u128).saturating_sub(2) + 1;
        self.elems.pop(bit_nb as u8).map(|idx| idx as usize)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PermutationGenerator32 {
    nb_elems: u8,
    nb_perms: u128,
    next_idx: u128,
}

impl PermutationGenerator32 {
    pub fn new(nb_elems: u8) -> Self {
        Self {
            next_idx: 0,
            nb_perms: factorial128(nb_elems),
            nb_elems,
        }
    }
}

impl Iterator for PermutationGenerator32 {
    type Item = SinglePermutation32;

    fn next(&mut self) -> Option<Self::Item> {
        let step_result = self.next_idx.saturating_add(0);
        let res = SinglePermutation32::new(self.nb_elems, self.nb_perms, step_result);
        self.next_idx = step_result + 1;
        res
    }
}

#[inline]
pub(crate) fn factorial128(nb_elems: u8) -> u128 {
    match nb_elems {
        0 | 1 | 2 => nb_elems as u128,
        _ => (1..=nb_elems).map(|i| i as u128).product(),
    }
}
