mod afa_generator;
mod afac_generator;

use crate::{Expr, Normal};

pub use afa_generator::AFAGenerator;
pub use afac_generator::AFACGenerator;

pub trait FunctionApplicationGenerator: Iterator<Item = Vec<Expr>> {
    fn new(function: Normal) -> Self;
}
