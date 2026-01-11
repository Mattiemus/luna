use crate::Atom;

// TODO: It would be useful to be able to parse at compile-time
#[macro_export]
macro_rules! parse {
    ($s:expr) => {
        parse($s).unwrap()
    };
}

// TODO
pub fn parse(s: &str) -> Result<Atom, ()> {
    Err(())
}
