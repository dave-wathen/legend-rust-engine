// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

// TODO treat as a class?
#[derive(PartialEq, Eq)]
pub struct Multiplicity
{
    lower_bound: i64,
    upper_bound: Option<i64>,
}

macro_rules! pure_multiplicity {
    ([ $l:literal .. $u:literal ]) => {
        $crate::Multiplicity { lower_bound: $l, upper_bound: Some($u) }
    };
    ([ $m:literal ]) => {
        $crate::Multiplicity { lower_bound: $m, upper_bound: Some($m) }
    };
    ([ * ]) => {
        $crate::Multiplicity { lower_bound: 0, upper_bound: None }
    };
    ([ $m:literal .. * ]) => {
        $crate::Multiplicity { lower_bound: $m, upper_bound: None }
    };
}

impl Multiplicity
{
    pub const ZERO_ONE: Multiplicity = pure_multiplicity!([0..1]);
    pub const ZERO_MANY: Multiplicity = pure_multiplicity!([*]);
    pub const PURE_ONE: Multiplicity = pure_multiplicity!([1]);
}

impl From<i64> for Multiplicity
{
    fn from(n: i64) -> Self { Multiplicity { lower_bound: n, upper_bound: Some(n) } }
}

impl From<std::ops::Range<i64>> for Multiplicity
{
    fn from(r: std::ops::Range<i64>) -> Self { Multiplicity { lower_bound: r.start, upper_bound: Some(r.end - 1) } }
}

impl From<std::ops::RangeInclusive<i64>> for Multiplicity
{
    fn from(r: std::ops::RangeInclusive<i64>) -> Self { Multiplicity { lower_bound: *r.start(), upper_bound: Some(*r.end()) } }
}

impl From<std::ops::RangeFrom<i64>> for Multiplicity
{
    fn from(r: std::ops::RangeFrom<i64>) -> Self { Multiplicity { lower_bound: r.start, upper_bound: None } }
}

impl From<std::ops::RangeTo<i64>> for Multiplicity
{
    fn from(r: std::ops::RangeTo<i64>) -> Self { Multiplicity { lower_bound: 0, upper_bound: Some(r.end - 1) } }
}

impl From<std::ops::RangeToInclusive<i64>> for Multiplicity
{
    fn from(r: std::ops::RangeToInclusive<i64>) -> Self { Multiplicity { lower_bound: 0, upper_bound: Some(r.end) } }
}

impl From<std::ops::RangeFull> for Multiplicity
{
    fn from(_: std::ops::RangeFull) -> Self { Multiplicity::ZERO_MANY }
}

impl fmt::Display for Multiplicity
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error>
    {
        if let Some(ubound) = self.upper_bound
        {
            if ubound == self.lower_bound
            {
                write!(f, "[{}]", self.lower_bound)
            }
            else
            {
                write!(f, "[{}..{}]", self.lower_bound, ubound)
            }
        }
        else if self.lower_bound == 0
        {
            write!(f, "[*]")
        }
        else
        {
            write!(f, "[{}..*]", self.lower_bound)
        }
    }
}

impl fmt::Debug for Multiplicity
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { write!(f, "{}", self) }
}

#[cfg(test)]
mod tests
{
    use crate::multiplicity::Multiplicity;

    #[test]
    fn standard_multiplicities()
    {
        assert_eq!(0, Multiplicity::ZERO_ONE.lower_bound);
        assert_eq!(Some(1), Multiplicity::ZERO_ONE.upper_bound);
        assert_eq!("[0..1]", format!("{}", Multiplicity::ZERO_ONE));
        assert_eq!("[0..1]", format!("{:?}", Multiplicity::ZERO_ONE));

        assert_eq!(0, Multiplicity::ZERO_MANY.lower_bound);
        assert_eq!(None, Multiplicity::ZERO_MANY.upper_bound);
        assert_eq!("[*]", format!("{}", Multiplicity::ZERO_MANY));
        assert_eq!("[*]", format!("{:?}", Multiplicity::ZERO_MANY));

        assert_eq!(1, Multiplicity::PURE_ONE.lower_bound);
        assert_eq!(Some(1), Multiplicity::PURE_ONE.upper_bound);
        assert_eq!("[1]", format!("{}", Multiplicity::PURE_ONE));
        assert_eq!("[1]", format!("{:?}", Multiplicity::PURE_ONE));
    }

    #[test]
    fn specific_multiplicities()
    {
        let m = Multiplicity::from(4);
        assert_eq!(4, m.lower_bound);
        assert_eq!(Some(4), m.upper_bound);
        assert_eq!("[4]", format!("{}", m));

        // Range end is exclusive
        let m = Multiplicity::from(1..5);
        assert_eq!(1, m.lower_bound);
        assert_eq!(Some(4), m.upper_bound);
        assert_eq!("[1..4]", format!("{}", m));

        let m = Multiplicity::from(1..=4);
        assert_eq!(1, m.lower_bound);
        assert_eq!(Some(4), m.upper_bound);
        assert_eq!("[1..4]", format!("{}", m));

        let m = Multiplicity::from(2..);
        assert_eq!(2, m.lower_bound);
        assert_eq!(None, m.upper_bound);
        assert_eq!("[2..*]", format!("{}", m));

        let m = Multiplicity::from(..5);
        assert_eq!(0, m.lower_bound);
        assert_eq!(Some(4), m.upper_bound);
        assert_eq!("[0..4]", format!("{}", m));

        let m = Multiplicity::from(..);
        assert_eq!(Multiplicity::ZERO_MANY, m);
        assert_eq!("[*]", format!("{}", m));

        let m = Multiplicity::from(0..);
        assert_eq!(Multiplicity::ZERO_MANY, m);
        assert_eq!("[*]", format!("{}", m));

        let m = Multiplicity::from(0..=1);
        assert_eq!(Multiplicity::ZERO_ONE, m);
        assert_eq!("[0..1]", format!("{}", m));

        let m = Multiplicity::from(..=1);
        assert_eq!(Multiplicity::ZERO_ONE, m);
        assert_eq!("[0..1]", format!("{}", m));

        let m = Multiplicity::from(1);
        assert_eq!(Multiplicity::PURE_ONE, m);
        assert_eq!("[1]", format!("{}", m));
    }

    #[test]
    fn specific_macro_constructs()
    {
        let m = pure_multiplicity!([4]);
        assert_eq!(4, m.lower_bound);
        assert_eq!(Some(4), m.upper_bound);
        assert_eq!("[4]", format!("{}", m));

        let m = pure_multiplicity!([1..4]);
        assert_eq!(1, m.lower_bound);
        assert_eq!(Some(4), m.upper_bound);
        assert_eq!("[1..4]", format!("{}", m));

        let m = pure_multiplicity!([*]);
        assert_eq!(0, m.lower_bound);
        assert_eq!(None, m.upper_bound);
        assert_eq!("[*]", format!("{}", m));

        let m = pure_multiplicity!([1..*]);
        assert_eq!(1, m.lower_bound);
        assert_eq!(None, m.upper_bound);
        assert_eq!("[1..*]", format!("{}", m));
    }
}
