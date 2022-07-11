// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

pub const PURE_TRUE: PureBoolean = PureBoolean(true);
pub const PURE_FALSE: PureBoolean = PureBoolean(false);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PureBoolean(bool);

impl crate::PureValue for PureBoolean {}

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
}
