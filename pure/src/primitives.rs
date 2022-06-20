// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

pub const PURE_TRUE: PureBoolean = PureBoolean(true);
pub const PURE_FALSE: PureBoolean = PureBoolean(false);

pub const PURE_INTEGER_0: PureInteger = PureInteger(0);
pub const PURE_INTEGER_1: PureInteger = PureInteger(1);

macro_rules! impl_debug_as_display {
    ($T:ident) => {
        impl fmt::Debug for $T
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self) }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PureBoolean(bool);
impl_debug_as_display!(PureBoolean);

impl fmt::Display for PureBoolean
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<bool> for PureBoolean
{
    fn from(b: bool) -> Self
    {
        if b
        {
            PURE_TRUE
        }
        else
        {
            PURE_FALSE
        }
    }
}

impl std::ops::Deref for PureBoolean
{
    type Target = bool;

    fn deref(&self) -> &bool { &self.0 }
}

// TODO Consider a scaling integer type (along with Decimal perhaps)
// TODO Handle usize/u64/u128/i128 conversions
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PureInteger(i64);
impl_debug_as_display!(PureInteger);

impl fmt::Display for PureInteger
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<i8> for PureInteger
{
    fn from(i: i8) -> Self { (i as i64).into() }
}

impl From<u8> for PureInteger
{
    fn from(i: u8) -> Self { (i as i64).into() }
}
impl From<i16> for PureInteger
{
    fn from(i: i16) -> Self { (i as i64).into() }
}

impl From<u16> for PureInteger
{
    fn from(i: u16) -> Self { (i as i64).into() }
}

impl From<i32> for PureInteger
{
    fn from(i: i32) -> Self { (i as i64).into() }
}

impl From<u32> for PureInteger
{
    fn from(i: u32) -> Self { (i as i64).into() }
}

impl From<i64> for PureInteger
{
    fn from(i: i64) -> Self
    {
        if i == 0
        {
            PURE_INTEGER_0
        }
        else if i == 1
        {
            PURE_INTEGER_1
        }
        else
        {
            PureInteger(i)
        }
    }
}

impl std::ops::Deref for PureInteger
{
    type Target = i64;

    fn deref(&self) -> &i64 { &self.0 }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn boolean_display()
    {
        assert_eq!("true", format!("{}", PURE_TRUE));
        assert_eq!("false", format!("{}", PURE_FALSE));
    }

    #[test]
    fn boolean_from()
    {
        assert_eq!(PURE_TRUE, PureBoolean::from(true));
        assert_eq!(PURE_FALSE, PureBoolean::from(false));
    }

    #[test]
    fn integer_display()
    {
        assert_eq!("0", format!("{}", PURE_INTEGER_0));
    }
    #[test]
    fn integer_from()
    {
        assert_eq!(PureInteger(0), PureInteger::from(0));
        assert_eq!(PureInteger(-1), (-1).into());
        assert_eq!(PureInteger(1), 1_u32.into());
        assert_eq!(PureInteger(3), 3_i32.into());
        assert_eq!(PureInteger(4), 4_u16.into());
        assert_eq!(PureInteger(5), 5_i16.into());
        assert_eq!(PureInteger(6), 6_u8.into());
        assert_eq!(PureInteger(7), 7_i8.into());
    }
}
