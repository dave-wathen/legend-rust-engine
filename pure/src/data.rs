// // Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt::Display;

use crate::{pure_type::Typed, *};

pub mod model;

pub const ZERO_NIL: Collection = Collection { pure_type: Type::Nil, multiplicity: PURE_ZERO, contents: CollectionContents::Zero };

#[derive(Debug, PartialEq, Clone)]
pub enum Value
{
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Value
{
    pub fn to_collection(self) -> Collection
    {
        Collection { pure_type: self.pure_type(), multiplicity: PURE_ONE, contents: CollectionContents::One(self) }
    }
}

macro_rules! impl_from {
    ($source:ty, $pure_type:ident, $as:ty) => {
        impl From<$source> for Value
        {
            fn from(v: $source) -> Self { Value::$pure_type(v as $as) }
        }
    };
}
macro_rules! impl_try_from {
    ($source:ty, $pure_type:ident, $as:ty) => {
        impl TryFrom<$source> for Value
        where
            $source: ToString,
        {
            type Error = PureExecutionError;

            fn try_from(v: $source) -> PureExecutionResult<Self>
            {
                match <$as as TryFrom<$source>>::try_from(v)
                {
                    Ok(x) => Ok(Value::$pure_type(x)),
                    Err(_) => Err(PureExecutionError::IllegalValue { pure_type: Type::$pure_type, value: v.to_string() }),
                }
            }
        }
    };
}

impl_from!(bool, Boolean, bool);
impl_from!(i8, Integer, i64);
impl_from!(u8, Integer, i64);
impl_from!(i16, Integer, i64);
impl_from!(u16, Integer, i64);
impl_from!(i32, Integer, i64);
impl_from!(u32, Integer, i64);
impl_from!(i64, Integer, i64);
impl_try_from!(u64, Integer, i64);
impl_try_from!(usize, Integer, i64);
impl_try_from!(isize, Integer, i64);
impl_try_from!(i128, Integer, i64);
impl_try_from!(u128, Integer, i64);
impl_from!(f32, Float, f64);
impl_from!(f64, Float, f64);

macro_rules! impl_try_from_value {
    ($for:ty, $pure_type:ident) => {
        impl TryFrom<Value> for $for
        where
            $for: ToString,
        {
            type Error = PureExecutionError;

            fn try_from(value: Value) -> Result<Self, Self::Error>
            {
                match value
                {
                    Value::$pure_type(x) => Ok(x),
                    _ => Err(PureExecutionError::WrongType { expected: Type::$pure_type, found: value.pure_type() }),
                }
            }
        }
    };
}

impl_try_from_value!(bool, Boolean);
impl_try_from_value!(i64, Integer);

impl TryFrom<Value> for f64
{
    type Error = PureExecutionError;

    fn try_from(value: Value) -> Result<Self, Self::Error>
    {
        match value
        {
            Value::Float(f) => Ok(f),
            Value::Integer(i) => Ok(i as f64),
            _ => Err(PureExecutionError::WrongType { expected: Type::Number, found: value.pure_type() }),
        }
    }
}

impl Display for Value
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Value::Boolean(v) => write!(f, "{v}"),
            Value::Integer(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
        }
    }
}

impl pure_type::Typed for Value
{
    fn pure_type(&self) -> Type
    {
        match self
        {
            Self::Boolean(_) => Type::Boolean,
            Self::Integer(_) => Type::Integer,
            Self::Float(_) => Type::Float,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Collection
{
    pure_type: pure_type::Type,
    multiplicity: multiplicity::Multiplicity,
    pub contents: CollectionContents,
}

impl Collection
{
    pub fn zero(pure_type: Type) -> Collection { Collection { pure_type, multiplicity: PURE_ZERO, contents: CollectionContents::Zero } }

    pub fn one<T: TryInto<Value>>(valuable: T) -> PureExecutionResult<Collection>
    where
        error::PureExecutionError: std::convert::From<<T as std::convert::TryInto<data::Value>>::Error>,
    {
        let value: Value = valuable.try_into()?;
        Ok(Collection { pure_type: value.pure_type(), multiplicity: PURE_ONE, contents: CollectionContents::One(value) })
    }

    pub fn size(&self) -> PureExecutionResult<Value>
    {
        let result = match &self.contents
        {
            CollectionContents::Zero => 0.into(),
            CollectionContents::One(_) => 1.into(),
            CollectionContents::Many(many) => many.len().try_into()?,
        };
        Ok(result)
    }
}

#[derive(Debug, PartialEq)]
pub enum CollectionContents
{
    Zero,
    One(Value),
    Many(Vec<Value>),
}

impl TypedWithMultiplicity for Collection {}

impl pure_type::Typed for Collection
{
    fn pure_type(&self) -> Type { self.pure_type }
}

impl multiplicity::Multiplicitied for Collection
{
    fn multiplicity(&self) -> Multiplicity { self.multiplicity }
}

impl<'a> std::iter::IntoIterator for &'a Collection
{
    type Item = &'a Value;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter
    {
        match &self.contents
        {
            CollectionContents::Zero => Iter::Zero,
            CollectionContents::One(one) => Iter::One(std::iter::once(one)),
            CollectionContents::Many(many) => Iter::Many(many.iter()),
        }
    }
}

pub enum Iter<'a>
{
    Zero,
    One(std::iter::Once<&'a Value>),
    Many(std::slice::Iter<'a, Value>),
}

impl<'a> Iterator for Iter<'a>
{
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item>
    {
        match self
        {
            Iter::Zero => None,
            Iter::One(iter) => iter.next(),
            Iter::Many(iter) => iter.next(),
        }
    }
}

pub struct CollectionBuilder
{
    collection: Collection,
}

impl CollectionBuilder
{
    pub fn new(pure_type: Type, multiplicity: Multiplicity) -> Self
    {
        Self { collection: Collection { pure_type, multiplicity, contents: CollectionContents::Zero } }
    }

    pub fn push<T: TryInto<Value>>(mut self, valuable: T) -> PureExecutionResult<Self>
    where
        error::PureExecutionError: std::convert::From<<T as std::convert::TryInto<data::Value>>::Error>,
    {
        let value = valuable.try_into()?;

        let col_type = self.collection.pure_type();
        if !col_type.is_assignable_from(&value.pure_type())
        {
            return Err(PureExecutionError::IllegalAssignment { from: value.pure_type(), to: col_type });
        }

        if let Some(col_max) = self.collection.multiplicity.upper_bound
        {
            let size: i64 = self.collection.size().and_then(|v| v.try_into())?;
            if size == col_max
            {
                return Err(PureExecutionError::IllegalMultiplicity { size: col_max + 1, mult: self.collection.multiplicity });
            }
        }

        match self.collection.contents
        {
            CollectionContents::Zero => self.collection.contents = CollectionContents::One(value),
            CollectionContents::One(one) => self.collection.contents = CollectionContents::Many(vec![one, value]),
            CollectionContents::Many(ref mut many) => many.push(value),
        }

        Ok(self)
    }

    pub fn build(self) -> PureExecutionResult<Collection>
    {
        let size: i64 = self.collection.size().and_then(|v| v.try_into())?;
        if size < self.collection.multiplicity.lower_bound
        {
            return Err(PureExecutionError::IllegalMultiplicity { size, mult: self.collection.multiplicity });
        }
        Ok(self.collection)
    }
}

#[cfg(test)]
mod tests
{
    use crate::pure_type::Typed;

    use super::*;

    #[test]
    fn boolean_value() -> PureExecutionResult<()>
    {
        let b: Value = true.into();
        assert_eq!(pure_type::Type::Boolean, b.pure_type());
        assert!(b.try_into()?);
        Ok(())
    }

    #[test]
    fn integer_value() -> PureExecutionResult<()>
    {
        let i: Value = 42.into();
        assert_eq!(pure_type::Type::Integer, i.pure_type());
        assert_eq!(42_i64, i.clone().try_into()?);
        assert_eq!(42.0, <Value as TryInto<f64>>::try_into(i)?);
        Ok(())
    }

    #[test]
    fn float_value() -> PureExecutionResult<()>
    {
        let f: Value = 4.2.into();
        assert_eq!(pure_type::Type::Float, f.pure_type());
        assert_eq!(4.2, <Value as TryInto<f64>>::try_into(f)?);
        Ok(())
    }

    #[test]
    fn build_empty_nil() -> PureExecutionResult<()>
    {
        let empty = CollectionBuilder::new(Type::Nil, PURE_ZERO).build()?;
        let size: i64 = empty.size().and_then(|v| v.try_into())?;
        assert_eq!(0, size);
        assert_eq!(pure_type::Type::Nil, empty.pure_type());
        assert_eq!(None, (&empty).into_iter().next());

        Ok(())
    }

    #[test]
    fn build_empty_integer() -> PureExecutionResult<()>
    {
        let empty = CollectionBuilder::new(Type::Integer, PURE_ZERO).build()?;
        let size: i64 = empty.size().and_then(|v| v.try_into())?;
        assert_eq!(0, size);
        assert_eq!(pure_type::Type::Integer, empty.pure_type());
        assert_eq!(None, (&empty).into_iter().next());

        Ok(())
    }

    #[test]
    fn build_one_integer() -> PureExecutionResult<()>
    {
        let one = CollectionBuilder::new(Type::Integer, PURE_ONE).push(2)?.build()?;
        let size: i64 = one.size().and_then(|v| v.try_into())?;
        assert_eq!(1, size);
        assert_eq!(pure_type::Type::Integer, one.pure_type());

        let mut iter = (&one).into_iter();
        assert_eq!(Some(&2.into()), iter.next());
        assert_eq!(None, iter.next());

        Ok(())
    }

    #[test]
    fn build_many_integer() -> PureExecutionResult<()>
    {
        let mut builder = CollectionBuilder::new(Type::Integer, ZERO_MANY);
        for i in 0..50
        {
            builder = builder.push(i)?;
        }
        let many = builder.build()?;

        let size: i64 = many.size().and_then(|v| v.try_into())?;
        assert_eq!(50, size);
        assert_eq!(pure_type::Type::Integer, many.pure_type());

        let mut iter = (&many).into_iter();
        for i in 0..50
        {
            assert_eq!(Some(&i.into()), iter.next());
        }
        assert_eq!(None, iter.next());

        Ok(())
    }

    #[test]
    fn build_integers_and_float() -> PureExecutionResult<()>
    {
        let numbers = CollectionBuilder::new(Type::Number, ZERO_MANY).push(1_usize)?.push(2.0)?.build()?;

        let size: i64 = numbers.size().and_then(|v| v.try_into())?;
        assert_eq!(2, size);
        assert_eq!(pure_type::Type::Number, numbers.pure_type());

        let mut iter = (&numbers).into_iter();
        assert_eq!(Some(&1.into()), iter.next());
        assert_eq!(Some(&2.0.into()), iter.next());
        assert_eq!(None, iter.next());

        Ok(())
    }

    #[test]
    fn invalid_type_assignment() -> PureExecutionResult<()>
    {
        let bad = CollectionBuilder::new(Type::Integer, ZERO_ONE).push(2.0);

        assert!(bad.is_err());
        assert_eq!("Illegal assignment: Float value cannot be assigned to Integer", format!("{}", bad.err().unwrap()));

        Ok(())
    }

    #[test]
    fn too_many_values() -> PureExecutionResult<()>
    {
        let bad = CollectionBuilder::new(Type::Integer, PURE_ONE).push(1)?.push(1);

        assert!(bad.is_err());
        assert_eq!("Illegal multiplicity: size of 2 cannot be assigned to [1]", format!("{}", bad.err().unwrap()));

        Ok(())
    }

    #[test]
    fn too_few_values() -> PureExecutionResult<()>
    {
        let bad = CollectionBuilder::new(Type::Integer, 2.into()).push(1)?.build();

        assert!(bad.is_err());
        assert_eq!("Illegal multiplicity: size of 1 cannot be assigned to [2]", format!("{}", bad.err().unwrap()));

        Ok(())
    }
}
