pub(crate) struct Subset(u32);

impl Subset {
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn initial() -> Self {
        Self(1)
    }
}


pub(crate) fn next_subset(n: u32, subset: u32) -> Option<u32> {
    let max = 1 << n;

    if subset == 0 {
        return if n != 0 { Some(1) } else { None };
    }

    let c = subset & (!subset + 1);
    let r = subset + c;
    let next = ((r ^ subset) >> 2) / c | r;
    if next < max {
        return Some(next);
    }

    let bits = subset.count_ones();
    if bits < n {
        return Some((1 << (bits + 1)) - 1);
    }

    None
}
