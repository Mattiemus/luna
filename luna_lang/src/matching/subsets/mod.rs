use std::cmp::min;
use bit_vec::BitVec;

pub(crate) struct Subset(BitVec);

impl Subset {
    pub fn empty(n: usize) -> Self {
        Self(BitVec::from_elem(n, false))
    }

    pub fn is_zero(&self) -> bool {
        self.0.none()
    }

    pub fn count_ones(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn count_zeros(&self) -> usize {
        self.0.count_zeros() as usize
    }

    pub fn get(&self, idx: usize) -> bool {
        self.0.get(idx).unwrap_or(false)
    }

    pub fn extract<T: Clone>(&self, values: &[T]) -> (Vec<T>, Vec<T>) {
        let mut subset = Vec::with_capacity(self.count_ones());
        let mut complement = Vec::with_capacity(self.count_zeros());

        for (k, c) in values.iter().enumerate() {
            if self.get(k) {
                subset.push(c.clone());
            } else {
                complement.push(c.clone());
            }
        }

        (subset, complement)
    }

    pub fn next(&self) -> Option<Self> {
        self.resize_next(self.0.len())
    }

    pub fn resize_next(&self, n: usize) -> Option<Self> {
        if n == 0 || self.count_ones() >= n {
            return None;
        }

        let mut bits = BitVec::from_elem(n, false);
        for i in 0..min(n, self.0.len()) {
            bits.set(i, self.0[i]);
        }

        if self.is_zero() {
            bits.set(0, true);
            return Some(Self(bits));
        }

        // Find first "10" pattern
        let mut pivot = None;
        for i in (0..n - 1).rev() {
            if bits[i] && !bits[i + 1] {
                pivot = Some(i);
                break;
            }
        }

        // If found, flip 10 → 01 and compact
        if let Some(i) = pivot {
            bits.set(i, false);
            bits.set(i + 1, true);

            // Count ones to the left of pivot
            let mut ones = 0;
            for j in 0..i {
                if bits[j] {
                    ones += 1;
                }
                bits.set(j, false);
            }

            // Push them all the way left
            for j in 0..ones {
                bits.set(j, true);
            }

            return Some(Subset(bits));
        }

        let ones = self.count_ones();
        let mut bits = BitVec::from_elem(n, false);
        for i in 0..=ones {
            bits.set(i, true);
        }

        Some(Subset(bits))
    }
}
