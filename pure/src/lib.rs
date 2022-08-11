// Copyright 2022 Dave Wathen. All rights reserved.

pub use crate::data::Collection;
pub use crate::data::Value;
pub use crate::data::ZERO_NIL;
pub use crate::error::PureExecutionError;
pub use crate::multiplicity::Multiplicitied;
pub use crate::multiplicity::Multiplicity;
pub use crate::multiplicity::PURE_ONE;
pub use crate::multiplicity::PURE_ZERO;
pub use crate::multiplicity::ZERO_MANY;
pub use crate::multiplicity::ZERO_ONE;
pub use crate::pure_type::Type;
pub use crate::pure_type::Typed;

pub mod data;
pub mod error;
#[allow(non_camel_case_types)]
pub mod multiplicity;
pub mod natives;
pub mod pure_type;

pub type PureExecutionResult<T> = Result<T, crate::PureExecutionError>;

trait TypedWithMultiplicity: Typed + Multiplicitied
{
    fn full_type_as_string(&self) -> String { format!("{}{}", self.pure_type(), self.multiplicity()) }
}

#[macro_export]
macro_rules! pure {
    ([ $l:literal .. $u:literal ]) => {
        $crate::Multiplicity::from($l..=$u)
    };
    ([ $m:literal ]) => {
        $crate::Multiplicity::from($m)
    };
    ([ * ]) => {
        $crate::ZERO_MANY
    };
    ([ $l:literal .. * ]) => {
        $crate::Multiplicity::from($l..)
    };
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::multiplicity::Multiplicity;

    #[test]
    fn macro_constructs()
    {
        let m = pure!([4]);
        assert_eq!(Multiplicity::from(4), m);

        let m = pure!([1..4]);
        assert_eq!(Multiplicity::from(1..=4), m);

        let m = pure!([*]);
        assert_eq!(ZERO_MANY, m);

        let m = pure!([1..*]);
        assert_eq!(Multiplicity::from(1..), m);
    }
}
