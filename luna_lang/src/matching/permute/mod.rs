use bit_index::BitIndex32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    #[inline]
    fn nb_remaining(&self) -> usize {
        self.elems.nb_elements() as usize
    }
}

impl Iterator for SinglePermutation32 {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elems.nb_elements() == 0 {
            return None;
        }
        let bit_nb = self.current_idx / self.next_mod;
        self.current_idx -= bit_nb * self.next_mod;
        self.next_mod /= (self.elems.nb_elements() as u128).saturating_sub(2) + 1;
        self.elems.pop(bit_nb as u8)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let nb_remaining = self.nb_remaining();
        (nb_remaining, Some(nb_remaining))
    }

    fn count(self) -> usize {
        self.nb_remaining()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PermutationGenerator32 {
    nb_elems: u8,
    nb_perms: u128,
    next_idx: u128,
}

impl PermutationGenerator32 {
    const MAX_ELEMENTS: u8 = 32;

    pub fn new(nb_elems: u8) -> Self {
        if nb_elems > Self::MAX_ELEMENTS {
            panic!("too many elements");
        }

        Self {
            next_idx: 0,
            nb_perms: factorial128(nb_elems),
            nb_elems,
        }
    }

    pub fn next_permutation(&mut self) -> Option<SinglePermutation32> {
        self.nth(0)
    }

    pub fn nth_absolute(nb_elems: u8, idx: u128) -> Option<SinglePermutation32> {
        if nb_elems > Self::MAX_ELEMENTS {
            panic!("too many elements");
        }

        SinglePermutation32::new(nb_elems, factorial128(nb_elems), idx)
    }

    pub fn nth(&mut self, step: u128) -> Option<SinglePermutation32> {
        let step_result = self.next_idx.saturating_add(step);
        let res = SinglePermutation32::new(self.nb_elems, self.nb_perms, step_result);
        self.next_idx = step_result + 1;
        res
    }

    /// Panics on nb_elems > 20
    pub fn nb_remaining(&self) -> usize {
        match (self.nb_perms - self.next_idx).try_into() {
            Ok(nb) => nb,
            Err(_) => panic!("The size of the iterator owerflowed usize"),
        }
    }
}

impl Iterator for PermutationGenerator32 {
    type Item = SinglePermutation32;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_permutation()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let nb_remaining = self.nb_remaining();
        (nb_remaining, Some(nb_remaining))
    }

    fn count(self) -> usize {
        self.nb_remaining()
    }
}

#[inline]
pub(crate) fn factorial128(nb_elems: u8) -> u128 {
    match nb_elems {
        0 | 1 | 2 => nb_elems as u128,
        _ => (1..=nb_elems).map(|i| i as u128).product(),
    }
}
