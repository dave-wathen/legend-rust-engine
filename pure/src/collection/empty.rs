// // Copyright 2022 Dave Wathen. All rights reserved.

use crate::{collection, PureValue};
use crate::{primitive, pure_type};

pub struct PureEmpty<T: PureValue>
{
    pure_type: pure_type::Type,
    phantom: std::marker::PhantomData<T>,
}

impl<T: PureValue> PureEmpty<T>
{
    pub fn new() -> Self { Self { pure_type: pure_type::Type::Nil, phantom: std::marker::PhantomData } }
}

impl<T: PureValue> pure_type::Typed for PureEmpty<T>
{
    fn pure_type(&self) -> pure_type::Type { self.pure_type }
}

impl<T: PureValue> collection::PureCollection<T> for PureEmpty<T>
{
    fn size(&self) -> primitive::PureInteger { crate::PURE_INTEGER_0 }
}

impl<T: PureValue> IntoIterator for PureEmpty<T>
{
    type Item = T;

    type IntoIter = EmptyIter<T>;

    fn into_iter(self) -> Self::IntoIter { EmptyIter { phantom: std::marker::PhantomData } }
}

pub struct EmptyIter<T>
{
    phantom: std::marker::PhantomData<T>,
}

impl<T> Iterator for EmptyIter<T>
{
    type Item = T;

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
        // TODO try to get rid of the need for NoValue to be explicitly used
        let empty: PureEmpty<NoValue> = PureEmpty::new();
        assert_eq!(0, *empty.size());
        assert_eq!(pure_type::Type::Nil, empty.pure_type());
        assert_eq!(None, empty.into_iter().next());
    }
}
