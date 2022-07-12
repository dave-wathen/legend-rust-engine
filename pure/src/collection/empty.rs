// // Copyright 2022 Dave Wathen. All rights reserved.

use crate::collection;
use crate::{primitive, pure_type};

pub struct PureEmpty
{
    pure_type: pure_type::Type,
}

impl PureEmpty
{
    pub fn new() -> Self { Self { pure_type: pure_type::Type::Nil } }
}

impl pure_type::Typed for PureEmpty
{
    fn pure_type(&self) -> pure_type::Type { self.pure_type }
}

impl collection::PureCollection for PureEmpty
{
    type PureItem = crate::Nil;

    fn size(&self) -> primitive::PureInteger { crate::PURE_INTEGER_0 }
}

impl IntoIterator for PureEmpty
{
    type Item = crate::Nil;

    type IntoIter = EmptyIter;

    fn into_iter(self) -> Self::IntoIter { EmptyIter {} }
}

pub struct EmptyIter {}

impl Iterator for EmptyIter
{
    type Item = crate::Nil;

    fn next(&mut self) -> Option<Self::Item> { None }
}

#[derive(PartialEq, Debug)]
struct NoValue {}
impl crate::PureValue for NoValue {}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{collection::PureCollection, pure_type::Typed};

    #[test]
    fn empty()
    {
        let empty: PureEmpty = PureEmpty::new();
        assert_eq!(0, *empty.size());
        assert_eq!(pure_type::Type::Nil, empty.pure_type());
        assert_eq!(None, empty.into_iter().next());
    }
}
