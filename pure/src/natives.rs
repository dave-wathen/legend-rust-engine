// Copyright 2022 Dave Wathen. All rights reserved.

use crate::{collection::PureCollection, primitives::PureBoolean};

pub fn and(left: PureBoolean, right: PureBoolean) -> PureBoolean { PureBoolean::from(*left && *right) }

pub fn or(left: PureBoolean, right: PureBoolean) -> PureBoolean { PureBoolean::from(*left || *right) }

pub fn not(b: PureBoolean) -> PureBoolean { PureBoolean::from(!*b) }

pub fn is_empty(c: impl PureCollection) -> PureBoolean { (*c.size() == 0).into() }

pub fn is_not_empty(c: impl PureCollection) -> PureBoolean { not(is_empty(c)) }

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{collection::PureEmpty, PURE_FALSE, PURE_TRUE};

    #[test]
    fn boolean_and()
    {
        assert_eq!(PURE_TRUE, and(PURE_TRUE, PURE_TRUE));
        assert_eq!(PURE_FALSE, and(PURE_FALSE, PURE_TRUE));
        assert_eq!(PURE_FALSE, and(PURE_TRUE, PURE_FALSE));
        assert_eq!(PURE_FALSE, and(PURE_FALSE, PURE_FALSE));
    }

    #[test]
    fn boolean_or()
    {
        assert_eq!(PURE_TRUE, or(PURE_TRUE, PURE_TRUE));
        assert_eq!(PURE_TRUE, or(PURE_FALSE, PURE_TRUE));
        assert_eq!(PURE_TRUE, or(PURE_TRUE, PURE_FALSE));
        assert_eq!(PURE_FALSE, or(PURE_FALSE, PURE_FALSE));
    }

    #[test]
    fn boolean_not()
    {
        assert_eq!(PURE_TRUE, not(PURE_FALSE));
        assert_eq!(PURE_FALSE, not(PURE_TRUE));
    }

    #[test]
    fn collection_is_empty()
    {
        assert_eq!(PURE_TRUE, is_empty(PureEmpty));
        assert_eq!(PURE_FALSE, is_empty(PURE_TRUE));
    }

    #[test]
    fn collection_is_not_empty()
    {
        assert_eq!(PURE_FALSE, is_not_empty(PureEmpty));
        assert_eq!(PURE_TRUE, is_not_empty(PURE_TRUE));
    }
}
