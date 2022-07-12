// // Copyright 2022 Dave Wathen. All rights reserved.

use std::iter;

use crate::{primitive, pure_type};

use super::PureCollection;

#[derive(Debug, PartialEq)]
pub enum PureValue
{
    Boolean(primitive::PureBoolean),
    Integer(primitive::PureInteger),
    Float(primitive::PureFloat),
}

impl pure_type::Typed for PureValue
{
    fn pure_type(&self) -> crate::Type
    {
        match self
        {
            Self::Boolean(_) => pure_type::Type::Boolean,
            Self::Integer(_) => pure_type::Type::Integer,
            Self::Float(_) => pure_type::Type::Float,
        }
    }
}

impl PureCollection for PureValue
{
    fn size(&self) -> primitive::PureInteger { crate::PURE_INTEGER_1 }
}

impl IntoIterator for PureValue
{
    type Item = PureValue;

    type IntoIter = iter::Once<PureValue>;

    fn into_iter(self) -> Self::IntoIter { iter::once(self) }
}

#[cfg(test)]
mod tests
{
    use crate::pure_type::Typed;

    use super::*;

    #[test]
    fn value_as_collection()
    {
        // TODO - Do we need double wrapping for Booleans/Integers/..
        // TODO - size should return a PureValue, can the type system express that it must be an Integer?
        let b: PureValue = PureValue::Boolean(primitive::PureBoolean(true));
        assert_eq!(1, *b.size());
        assert_eq!(pure_type::Type::Boolean, b.pure_type());
        let mut iter = b.into_iter();
        assert_eq!(Some(PureValue::Boolean(primitive::PureBoolean(true))), iter.next());
        assert_eq!(None, iter.next());
    }
}
