// Copyright 2022 Dave Wathen. All rights reserved.

use crate::{
    primitives::{self, PureInteger},
    PURE_INTEGER_0, PURE_INTEGER_1,
};

pub trait PureCollection
{
    fn size(&self) -> PureInteger;
}

pub struct PureEmpty;

impl PureCollection for PureEmpty
{
    fn size(&self) -> PureInteger { PURE_INTEGER_0 }
}

impl PureCollection for primitives::PureBoolean
{
    fn size(&self) -> PureInteger { PURE_INTEGER_1 }
}

#[cfg(test)]
mod tests
{
    use crate::PURE_FALSE;

    use super::*;

    #[test]
    fn sizes()
    {
        assert_eq!(PURE_INTEGER_0, PureEmpty.size());
        assert_eq!(PURE_INTEGER_1, PURE_FALSE.size());
    }
}
