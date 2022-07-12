// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

pub const PURE_INTEGER_0: PureInteger = PureInteger(0);
pub const PURE_INTEGER_1: PureInteger = PureInteger(1);

// TODO Consider a scaling integer type (along with Decimal perhaps)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PureInteger(pub i64);

impl fmt::Display for PureInteger
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

macro_rules! impl_from_for_pure_integer {
    ($T:ident) => {
        impl From<$T> for PureInteger
        {
            fn from(i: $T) -> Self { (i as i64).into() }
        }
    };
}

impl_from_for_pure_integer!(i8);
impl_from_for_pure_integer!(u8);
impl_from_for_pure_integer!(i16);
impl_from_for_pure_integer!(u16);
impl_from_for_pure_integer!(i32);
impl_from_for_pure_integer!(u32);
// TODO These are falable while we use i64
impl_from_for_pure_integer!(u64);
impl_from_for_pure_integer!(usize);
impl_from_for_pure_integer!(isize);
impl_from_for_pure_integer!(i128);
impl_from_for_pure_integer!(u128);

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

impl std::ops::Add for PureInteger
{
    type Output = PureInteger;

    fn add(self, rhs: Self) -> Self::Output { PureInteger(*self + *rhs) }
}

impl std::ops::Sub for PureInteger
{
    type Output = PureInteger;

    fn sub(self, rhs: Self) -> Self::Output { PureInteger(*self - *rhs) }
}

impl std::ops::Neg for PureInteger
{
    type Output = PureInteger;

    fn neg(self) -> Self::Output { PureInteger(-*self) }
}

impl std::ops::Mul for PureInteger
{
    type Output = PureInteger;

    fn mul(self, rhs: Self) -> Self::Output { PureInteger(*self * *rhs) }
}

impl std::ops::Div for PureInteger
{
    type Output = PureInteger;

    fn div(self, rhs: Self) -> Self::Output { PureInteger(*self / *rhs) }
}

impl std::ops::Rem for PureInteger
{
    type Output = PureInteger;

    fn rem(self, rhs: Self) -> Self::Output { PureInteger(*self % *rhs) }
}

impl std::ops::AddAssign for PureInteger
{
    fn add_assign(&mut self, rhs: Self) { *self = PureInteger(**self + *rhs) }
}

impl std::ops::SubAssign for PureInteger
{
    fn sub_assign(&mut self, rhs: Self) { *self = PureInteger(**self - *rhs) }
}

impl std::ops::MulAssign for PureInteger
{
    fn mul_assign(&mut self, rhs: Self) { *self = PureInteger(**self * *rhs) }
}

impl std::ops::DivAssign for PureInteger
{
    fn div_assign(&mut self, rhs: Self) { *self = PureInteger(**self / *rhs) }
}

impl std::ops::RemAssign for PureInteger
{
    fn rem_assign(&mut self, rhs: Self) { *self = PureInteger(**self % *rhs) }
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

    #[test]
    fn arithmetic()
    {
        assert_eq!(PureInteger(4), PureInteger(1) + PureInteger(3));
        assert_eq!(PureInteger(1), PureInteger(4) - PureInteger(3));
        assert_eq!(PureInteger(-4), -PureInteger(4));
        assert_eq!(PureInteger(12), PureInteger(4) * PureInteger(3));
        assert_eq!(PureInteger(4), PureInteger(12) / PureInteger(3));
        assert_eq!(PureInteger(4), PureInteger(13) / PureInteger(3));
        assert_eq!(PureInteger(1), PureInteger(13) % PureInteger(3));

        let mut n = PureInteger(2);
        n += PureInteger(3);
        assert_eq!(PureInteger(5), n);

        n -= PureInteger(1);
        assert_eq!(PureInteger(4), n);

        n *= PureInteger(3);
        assert_eq!(PureInteger(12), n);

        n /= PureInteger(4);
        assert_eq!(PureInteger(3), n);

        n %= PureInteger(2);
        assert_eq!(PureInteger(1), n);
    }
}
