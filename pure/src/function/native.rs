// // Copyright 2022 Dave Wathen. All& rights reserved.

use crate::function::*;

pub use meta_pure_functions_boolean_and_Boolean_1__Boolean_1__Boolean_1_ as and;
pub use meta_pure_functions_boolean_not_Boolean_1__Boolean_1_ as not;
pub use meta_pure_functions_boolean_or_Boolean_1__Boolean_1__Boolean_1_ as or;
pub use meta_pure_functions_collection_isEmpty_Any_MANY__Boolean_1_ as is_empty;
pub use meta_pure_functions_collection_isNotEmpty_Any_MANY__Boolean_1_ as is_not_empty;
pub use meta_pure_functions_math_plus_Float_MANY__Float_1__ as fplus;
pub use meta_pure_functions_math_plus_Integer_MANY__Integer_1__ as iplus;
pub use meta_pure_functions_math_plus_Number_MANY__Number_1__ as nplus;

#[allow(non_snake_case)]
pub fn meta_pure_functions_boolean_and_Boolean_1__Boolean_1__Boolean_1_<L, R>(left: L, right: R) -> PureExecutionResult<Collection>
where
    L: FunctionArgument,
    R: FunctionArgument,
{
    const FUNC: &str = "meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_";

    Collection::one(left.one(FUNC, 0)? && right.one(FUNC, 1)?)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_boolean_or_Boolean_1__Boolean_1__Boolean_1_<L, R>(left: L, right: R) -> PureExecutionResult<Collection>
where
    L: FunctionArgument,
    R: FunctionArgument,
{
    const FUNC: &str = "meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_";

    Collection::one(left.one(FUNC, 0)? || right.one(FUNC, 1)?)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_boolean_not_Boolean_1__Boolean_1_<C>(col: C) -> PureExecutionResult<Collection>
where
    C: FunctionArgument,
{
    const FUNC: &str = "meta::pure::functions::boolean::not_Boolean_1__Boolean_1_";

    Collection::one(!col.one::<bool>(FUNC, 0)?)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_math_plus_Integer_MANY__Integer_1__<C>(col: C) -> PureExecutionResult<Collection>
where
    C: FunctionArgument,
{
    const FUNC: &str = "meta::pure::functions::math::plus_Integer_MANY__Integer_1_";

    let iter = col.many::<i64>(FUNC, 0)?;
    let mut error = None;
    let sum: i64 = iter.filter_map(|x| x.map_err(|e| error = Some(e)).ok()).sum();
    error.map_or_else(|| Collection::one(sum), Err)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_math_plus_Float_MANY__Float_1__<C>(col: C) -> PureExecutionResult<Collection>
where
    C: FunctionArgument,
{
    const FUNC: &str = "meta::pure::functions::math::plus_Float_MANY__Float_1_";

    let iter = col.many::<f64>(FUNC, 0)?;
    let mut error = None;
    let sum: f64 = iter.filter_map(|x| x.map_err(|e| error = Some(e)).ok()).sum();
    error.map_or_else(|| Collection::one(sum), Err)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_math_plus_Number_MANY__Number_1__<C>(col: C) -> PureExecutionResult<Collection>
where
    C: FunctionArgument,
{
    const FUNC: &str = "meta::pure::functions::math::plus_Number_MANY__Number_1_";

    let mut error = None;
    let sum = col.many_raw(FUNC, 0, Type::Number)?.fold(0.into(), |a, n| match (a, n)
    {
        (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
        (Value::Integer(l), Value::Float(r)) => Value::Float(l as f64 + r),
        (Value::Float(l), Value::Integer(r)) => Value::Float(l + *r as f64),
        (Value::Float(l), Value::Float(r)) => Value::Float(l + r),
        _ =>
        {
            error = Some(PureExecutionError::UnexpectedValue { expected: "Number".into(), got: n.pure_type().to_string() });
            0.into()
        }
    });

    error.map_or_else(|| Ok(sum.to_collection()), Err)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_collection_isEmpty_Any_MANY__Boolean_1_<C>(c: C) -> PureExecutionResult<Collection>
where
    C: FunctionArgument,
{
    Collection::one(c.size()? == 0)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_collection_isNotEmpty_Any_MANY__Boolean_1_<C>(c: C) -> PureExecutionResult<Collection>
where
    C: FunctionArgument,
{
    not(&is_empty(c)?)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn boolean_and() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(true)?, and(&Collection::one(true)?, &Collection::one(true)?)?);
        assert_eq!(Collection::one(false)?, and(&Collection::one(false)?, &Collection::one(true)?)?);
        assert_eq!(Collection::one(false)?, and(&Collection::one(true)?, &Collection::one(false)?)?);
        assert_eq!(Collection::one(false)?, and(&Collection::one(false)?, &Collection::one(false)?)?);

        let bad = and(&Collection::one(1)?, &Collection::one(false)?);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            bad.err().unwrap().to_string()
        );

        let bad = and(&Collection::one(true)?, &Collection::zero(Type::Integer));
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 1 of meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[0]",
            bad.err().unwrap().to_string()
        );

        // Error undetected due to lazy eval
        // TODO Lazy eval for function as argument
        assert_eq!(Collection::one(false)?, and(&Collection::one(false)?, &Collection::zero(Type::Integer))?);

        Ok(())
    }

    #[test]
    fn boolean_or() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(true)?, or(&Collection::one(true)?, &Collection::one(true)?)?);
        assert_eq!(Collection::one(true)?, or(&Collection::one(false)?, &Collection::one(true)?)?);
        assert_eq!(Collection::one(true)?, or(&Collection::one(true)?, &Collection::one(false)?)?);
        assert_eq!(Collection::one(false)?, or(&Collection::one(false)?, &Collection::one(false)?)?);

        let bad = or(&Collection::one(1)?, &Collection::one(false)?);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            format!("{}", bad.err().unwrap())
        );

        let bad = or(&Collection::one(false)?, &Collection::one(1)?);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 1 of meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            format!("{}", bad.err().unwrap())
        );

        // Error undetected due to lazy eval
        // TODO Lazy eval for function as argument
        assert_eq!(Collection::one(true)?, or(&Collection::one(true)?, &Collection::zero(Type::Integer))?);

        Ok(())
    }

    #[test]
    fn boolean_not() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(true)?, not(&Collection::one(false)?)?);
        assert_eq!(Collection::one(false)?, not(&Collection::one(true)?)?);

        let bad = not(&ZERO_NIL);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::boolean::not_Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Nil[0]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn integer_plus() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(0)?, iplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).build()?)?);
        assert_eq!(Collection::one(1)?, iplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(1)?.build()?)?);
        assert_eq!(Collection::one(6)?, iplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(1)?.push(2)?.push(3)?.build()?)?);

        let bad = iplus(&ZERO_NIL);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::math::plus_Integer_MANY__Integer_1_: Unexpected value: expected Integer[*] but got Nil[0]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn float_plus() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(0.0)?, fplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).build()?)?);
        assert_eq!(Collection::one(1.1)?, fplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(1.1)?.build()?)?);
        assert_eq!(Collection::one(6.6)?, fplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(1.1)?.push(2.2)?.push(3.3)?.build()?)?);

        let bad = fplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).build()?);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::math::plus_Float_MANY__Float_1_: Unexpected value: expected Float[*] but got Integer[*]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn number_plus() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(0)?, nplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).build()?)?);
        assert_eq!(Collection::one(1)?, nplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(1)?.build()?)?);
        assert_eq!(Collection::one(6)?, nplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(1)?.push(2)?.push(3)?.build()?)?);

        assert_eq!(Collection::one(0)?, nplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).build()?)?);
        assert_eq!(Collection::one(1.1)?, nplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(1.1)?.build()?)?);
        assert_eq!(Collection::one(6.6)?, nplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(1.1)?.push(2.2)?.push(3.3)?.build()?)?);

        assert_eq!(Collection::one(6.4)?, nplus(&CollectionBuilder::new(Type::Number, ZERO_MANY).push(1.1)?.push(2)?.push(3.3)?.build()?)?);

        let bad = nplus(&CollectionBuilder::new(Type::Boolean, ZERO_MANY).build()?);
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::math::plus_Number_MANY__Number_1_: Unexpected value: expected Number[*] but got Boolean[*]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn collection_is_empty() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(true)?, is_empty(&ZERO_NIL)?);
        assert_eq!(Collection::one(false)?, is_empty(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(1)?.build()?)?);

        Ok(())
    }

    #[test]
    fn collection_is_not_empty() -> PureExecutionResult<()>
    {
        assert_eq!(Collection::one(false)?, is_not_empty(&ZERO_NIL)?);
        assert_eq!(Collection::one(true)?, is_not_empty(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(2)?.build()?)?);

        Ok(())
    }
}
