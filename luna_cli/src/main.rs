use std::collections::btree_set::{BTreeSet, IntoIter};

enum UniquePermutations<T: Clone + PartialEq + Ord> {
    Leaf {
        elements: Option<Vec<T>>,
    },
    Stem {
        elements: Vec<T>,
        unique_elements: IntoIter<T>,
        first_element: T,
        inner: Box<Self>,
    },
}

impl<T: Clone + PartialEq + Ord> UniquePermutations<T> {
    fn new(elements: Vec<T>) -> Self {
        if elements.len() == 0 || elements.len() == 1 {
            let elements = Some(elements);
            return Self::Leaf { elements };
        }

        let mut unique_elements = elements
            .clone()
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter();

        let (first_element, inner) = Self::next_level(&mut unique_elements, elements.clone())
            .expect("must have at least one item");

        Self::Stem {
            elements,
            unique_elements,
            first_element,
            inner,
        }
    }

    fn next_level(
        mut unique_elements: impl Iterator<Item = T>,
        elements: Vec<T>,
    ) -> Option<(T, Box<Self>)> {
        let first_element = unique_elements.next()?;
        let mut remaining_elements = elements;

        if let Some(idx) = remaining_elements.iter().position(|i| i == &first_element) {
            remaining_elements.remove(idx);
        }

        let inner = Box::new(Self::new(remaining_elements));

        Some((first_element, inner))
    }
}

impl<T: Clone + PartialEq + Ord> Iterator for UniquePermutations<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Leaf { elements } => elements.take(),
            Self::Stem {
                elements,
                unique_elements,
                first_element,
                inner,
            } => loop {
                match inner.next() {
                    Some(mut v) => {
                        v.insert(0, first_element.clone());
                        return Some(v);
                    }
                    None => {
                        let (next_fe, next_i) =
                            Self::next_level(&mut *unique_elements, elements.clone())?;

                        *first_element = next_fe;
                        *inner = next_i;
                    }
                }
            },
        }
    }
}

fn main() {
    let items = vec![0, 0, 1, 2];
    for perm in UniquePermutations::new(items) {
        println!("{:?}", perm);
    }
}
