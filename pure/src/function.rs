// Copyright 2022 Dave Wathen. All rights reserved.

use std::marker::PhantomData;

use crate::data::*;
use crate::pure_type::*;
use crate::*;

pub mod native;

pub trait FunctionArgument
{
    fn size(&self) -> PureExecutionResult<i64>;

    fn one<T>(&self, func: &str, arg: usize) -> PureExecutionResult<T>
    where
        T: 'static + TryFrom<Value, Error = PureExecutionError>;

    fn many<T>(&self, func: &str, arg: usize) -> PureExecutionResult<IterAdaptor<'_, T>>
    where
        T: 'static + TryFrom<Value, Error = PureExecutionError>;

    fn many_raw(&self, func: &str, arg: usize, pure_type: Type) -> PureExecutionResult<Iter>;
}

impl FunctionArgument for &Collection
{
    fn size(&self) -> PureExecutionResult<i64> { Collection::size(self).and_then(|v| v.try_into()) }

    fn one<T>(&self, func: &str, arg: usize) -> PureExecutionResult<T>
    where
        T: 'static + TryFrom<Value, Error = PureExecutionError>,
    {
        let result = if let CollectionContents::One(value) = &self.contents { T::try_from(value.clone()).ok() } else { None };

        result.ok_or_else(|| {
            let unexpected = PureExecutionError::UnexpectedValue { expected: format!("{}[1]", pure_type_of::<T>()), got: self.full_type_as_string() };
            PureExecutionError::IllegalArgument { func: func.to_string(), arg, cause: unexpected.to_string() }
        })
    }

    fn many<T>(&self, func: &str, arg: usize) -> PureExecutionResult<IterAdaptor<'_, T>>
    where
        T: 'static + TryFrom<Value, Error = PureExecutionError>,
    {
        if self.pure_type() == pure_type_of::<T>()
        {
            Ok(IterAdaptor { iter: self.into_iter(), phantom: PhantomData })
        }
        else
        {
            let unexpected = PureExecutionError::UnexpectedValue { expected: format!("{}[*]", pure_type_of::<T>()), got: self.full_type_as_string() };
            Err(PureExecutionError::IllegalArgument { func: func.to_string(), arg, cause: unexpected.to_string() })
        }
    }

    fn many_raw(&self, func: &str, arg: usize, pure_type: Type) -> PureExecutionResult<Iter>
    {
        if pure_type.is_assignable_from(&self.pure_type())
        {
            Ok(self.into_iter())
        }
        else
        {
            let unexpected = PureExecutionError::UnexpectedValue { expected: format!("{pure_type}[*]"), got: self.full_type_as_string() };
            Err(PureExecutionError::IllegalArgument { func: func.to_string(), arg, cause: unexpected.to_string() })
        }
    }
}

pub struct IterAdaptor<'a, T>
where
    T: TryFrom<Value, Error = PureExecutionError>,
{
    iter: data::Iter<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Iterator for IterAdaptor<'a, T>
where
    T: TryFrom<Value, Error = PureExecutionError>,
{
    type Item = PureExecutionResult<T>;

    fn next(&mut self) -> Option<Self::Item> { self.iter.next().map(|x| T::try_from(x.clone())) }
}
