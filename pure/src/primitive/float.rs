// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

pub const PURE_FLOAT_0: PureFloat = PureFloat(0.0);

// TODO Pure Eq in terms of PartialEq?
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PureFloat(pub f64);

impl fmt::Display for PureFloat
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if self.0.fract() == 0.0
        {
            write!(f, "{}.0", self.0)
        }
        else
        {
            write!(f, "{:.}", self.0)
        }
    }
}

macro_rules! impl_from_for_pure_float {
    ($T:ident) => {
        impl From<$T> for PureFloat
        {
            fn from(i: $T) -> Self { (i as f64).into() }
        }
    };
}

impl_from_for_pure_float!(f32);

impl_from_for_pure_float!(i8);
impl_from_for_pure_float!(u8);
impl_from_for_pure_float!(i16);
impl_from_for_pure_float!(u16);
impl_from_for_pure_float!(i32);
// TODO These are falable while we use f64
impl_from_for_pure_float!(u32);
impl_from_for_pure_float!(u64);
impl_from_for_pure_float!(usize);
impl_from_for_pure_float!(isize);
impl_from_for_pure_float!(i128);
impl_from_for_pure_float!(u128);

impl From<f64> for PureFloat
{
    fn from(f: f64) -> Self
    {
        if f == 0.0
        {
            PURE_FLOAT_0
        }
        else
        {
            PureFloat(f)
        }
    }
}

impl std::ops::Add for PureFloat
{
    type Output = PureFloat;

    fn add(self, rhs: Self) -> Self::Output { PureFloat(*self + *rhs) }
}

impl std::ops::Sub for PureFloat
{
    type Output = PureFloat;

    fn sub(self, rhs: Self) -> Self::Output { PureFloat(*self - *rhs) }
}

impl std::ops::Neg for PureFloat
{
    type Output = PureFloat;

    fn neg(self) -> Self::Output { PureFloat(-*self) }
}

impl std::ops::Mul for PureFloat
{
    type Output = PureFloat;

    fn mul(self, rhs: Self) -> Self::Output { PureFloat(*self * *rhs) }
}

impl std::ops::Div for PureFloat
{
    type Output = PureFloat;

    fn div(self, rhs: Self) -> Self::Output { PureFloat(*self / *rhs) }
}

impl std::ops::AddAssign for PureFloat
{
    fn add_assign(&mut self, rhs: Self) { *self = PureFloat(**self + *rhs) }
}

impl std::ops::SubAssign for PureFloat
{
    fn sub_assign(&mut self, rhs: Self) { *self = PureFloat(**self - *rhs) }
}

impl std::ops::MulAssign for PureFloat
{
    fn mul_assign(&mut self, rhs: Self) { *self = PureFloat(**self * *rhs) }
}

impl std::ops::DivAssign for PureFloat
{
    fn div_assign(&mut self, rhs: Self) { *self = PureFloat(**self / *rhs) }
}

impl std::ops::RemAssign for PureFloat
{
    fn rem_assign(&mut self, rhs: Self) { *self = PureFloat(**self % *rhs) }
}

impl std::ops::Deref for PureFloat
{
    type Target = f64;

    fn deref(&self) -> &f64 { &self.0 }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn float_display()
    {
        assert_eq!("0.0", format!("{}", PURE_FLOAT_0));
        assert_eq!("1.24", format!("{}", PureFloat(1.24)));
    }

    #[test]
    fn float_from()
    {
        assert_eq!(PureFloat(0.0), PureFloat::from(0.0));
        assert_eq!(PureFloat(1.2), PureFloat::from(1.2));
        assert_eq!(PureFloat(1.5), PureFloat::from(1.5_f32));
        assert_eq!(PureFloat(-1.0), (-1).into());
        assert_eq!(PureFloat(1.0), 1_u32.into());
        assert_eq!(PureFloat(3.0), 3_i32.into());
        assert_eq!(PureFloat(4.0), 4_u16.into());
        assert_eq!(PureFloat(5.0), 5_i16.into());
        assert_eq!(PureFloat(6.0), 6_u8.into());
        assert_eq!(PureFloat(7.0), 7_i8.into());
    }

    macro_rules! assert_pure_float_eq {
        ($left:expr, $right:expr) => {
            assert_pure_float_eq!($left, $right, 0.00001);
        };
        ($left:expr, $right:expr, $diff:expr) => {
            if (($left - $right).abs() > $diff)
            {
                panic!("Difference greater than {}\n left: {}\nright: {}", $diff, $left, $right);
            }
        };
    }

    #[test]
    fn arithmetic()
    {
        assert_pure_float_eq!(PureFloat(3.5), PureFloat(1.2) + PureFloat(2.3));
        assert_pure_float_eq!(PureFloat(1.2), PureFloat(3.5) - PureFloat(2.3));
        assert_pure_float_eq!(PureFloat(-4.0), -PureFloat(4.0));
        assert_pure_float_eq!(PureFloat(5.04), PureFloat(1.4) * PureFloat(3.6));
        assert_pure_float_eq!(PureFloat(1.4), PureFloat(5.04) / PureFloat(3.6));

        let mut n = PureFloat(2.1);
        n += PureFloat(3.2);
        assert_pure_float_eq!(PureFloat(5.3), n);

        n -= PureFloat(1.7);
        assert_pure_float_eq!(PureFloat(3.6), n);

        n *= PureFloat(3.9);
        assert_pure_float_eq!(PureFloat(14.04), n);

        n /= PureFloat(4.75);
        assert_pure_float_eq!(PureFloat(2.95578947), n, 0.00000001);
    }
}
