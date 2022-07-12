// Copyright 2022 Dave Wathen. All rights reserved.

pub use crate::multiplicity::Multiplicity;
pub use crate::primitive::boolean::PURE_FALSE;
pub use crate::primitive::boolean::PURE_TRUE;
pub use crate::primitive::integer::PURE_INTEGER_0;
pub use crate::primitive::integer::PURE_INTEGER_1;
pub use crate::pure_type::Type;

mod collection;
#[allow(non_camel_case_types)]
mod multiplicity;
pub mod natives;
mod primitive;
mod pure_type;

pub trait PureValue {}

#[derive(PartialEq, Eq, Debug)]
pub enum Nil {}

impl PureValue for Nil {}

#[macro_export]
macro_rules! pure {
    ([ $l:literal .. $u:literal ]) => {
        $crate::Multiplicity::from($l..=$u)
    };
    ([ $m:literal ]) => {
        $crate::Multiplicity::from($m)
    };
    ([ * ]) => {
        $crate::Multiplicity::ZERO_MANY
    };
    ([ $l:literal .. * ]) => {
        $crate::Multiplicity::from($l..)
    };
}

#[cfg(test)]
mod tests
{
    use crate::multiplicity::Multiplicity;

    #[test]
    fn macro_constructs()
    {
        let m = pure!([4]);
        assert_eq!(Multiplicity::from(4), m);

        let m = pure!([1..4]);
        assert_eq!(Multiplicity::from(1..=4), m);

        let m = pure!([*]);
        assert_eq!(Multiplicity::ZERO_MANY, m);

        let m = pure!([1..*]);
        assert_eq!(Multiplicity::from(1..), m);
    }
}
