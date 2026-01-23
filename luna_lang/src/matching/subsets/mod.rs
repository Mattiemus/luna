use crate::BigInteger;

pub(crate) struct Subset {
    n: usize,
    value: BigInteger,
}

impl Subset {
    pub fn empty(n: usize) -> Self {
        Self {
            n,
            value: BigInteger::ZERO,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    pub fn count_ones(&self) -> usize {
        self.value.count_ones().unwrap_or(0) as usize
    }

    pub fn count_zeros(&self) -> usize {
        self.value.count_zeros().unwrap_or(0) as usize
    }

    pub fn get(&self, idx: usize) -> bool {
        (&self.value & (BigInteger::ONE.clone() << idx)) >= 1
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
        self.resize_next(self.n)
    }

    pub fn resize_next(&self, n: usize) -> Option<Self> {
        if n == 0 {
            return None;
        }

        if self.is_zero() {
            return Some(Self {
                n,
                value: BigInteger::ONE.clone(),
            });
        }

        let max = BigInteger::ONE.clone() << n;

        let c: BigInteger = self.value.clone() & (!self.value.clone() + 1);
        let r: BigInteger = self.value.clone() + &c;
        let next = ((r.clone() ^ &self.value) >> 2) / &c | &r;
        if next < max {
            return Some(Self { n, value: next });
        }

        let bits = self.count_ones();
        if bits < n {
            return Some(Self {
                n,
                value: (BigInteger::ONE.clone() << (bits + 1)) - 1,
            });
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn increments_in_order() {
        let mut subset = Subset::empty(4);
        assert_eq!(subset.value, BigInteger::from(0b0000));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0001));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0010));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0100));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1000));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0011));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0101));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0110));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1001));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1010));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1100));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b0111));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1011));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1101));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1110));

        subset = subset.next().unwrap();
        assert_eq!(subset.value, BigInteger::from(0b1111));

        assert!(subset.next().is_none());
    }
}
