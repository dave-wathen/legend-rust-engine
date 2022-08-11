// // Copyright 2022 Dave Wathen. All& rights reserved.

use crate::{data::*, *};

pub use meta_pure_functions_boolean_and_Boolean_1__Boolean_1__Boolean_1_ as and;
pub use meta_pure_functions_boolean_not_Boolean_1__Boolean_1_ as not;
pub use meta_pure_functions_boolean_or_Boolean_1__Boolean_1__Boolean_1_ as or;
pub use meta_pure_functions_collection_isEmpty_Any_MANY__Boolean_1_ as is_empty;
pub use meta_pure_functions_collection_isNotEmpty_Any_MANY__Boolean_1_ as is_not_empty;
pub use meta_pure_functions_math_plus_Float_MANY__Float_1__ as fplus;
pub use meta_pure_functions_math_plus_Integer_MANY__Integer_1__ as iplus;
pub use meta_pure_functions_math_plus_Number_MANY__Number_1__ as nplus;

#[allow(non_snake_case)]
pub fn meta_pure_functions_boolean_and_Boolean_1__Boolean_1__Boolean_1_(lcol: &Collection, rcol: &Collection) -> PureExecutionResult<Collection>
{
    const FUNC: &str = "meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_";

    // TODO Lazy eval
    let left = lcol.assume_boolean_one().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 0, cause: e.to_string() })?;
    let right = rcol.assume_boolean_one().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 1, cause: e.to_string() })?;
    Ok(boolean_one(left && right))
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_boolean_or_Boolean_1__Boolean_1__Boolean_1_(lcol: &Collection, rcol: &Collection) -> PureExecutionResult<Collection>
{
    const FUNC: &str = "meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_";

    // TODO Lazy eval
    let left = lcol.assume_boolean_one().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 0, cause: e.to_string() })?;
    let right = rcol.assume_boolean_one().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 1, cause: e.to_string() })?;
    Ok(boolean_one(left || right))
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_boolean_not_Boolean_1__Boolean_1_(col: &Collection) -> PureExecutionResult<Collection>
{
    const FUNC: &str = "meta::pure::functions::boolean::not_Boolean_1__Boolean_1_";

    let b = col.assume_boolean_one().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 0, cause: e.to_string() })?;
    Ok(boolean_one(!b))
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_math_plus_Integer_MANY__Integer_1__(col: &Collection) -> PureExecutionResult<Collection>
{
    const FUNC: &str = "meta::pure::functions::math::plus_Integer_MANY__Integer_1_";

    let iter = col.assume_integer_many().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 0, cause: e.to_string() })?;
    let mut error = None;
    let sum = iter.filter_map(|x| x.map_err(|e| error = Some(e)).ok()).sum();
    error.map_or_else(|| Ok(integer_one(sum)), Err)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_math_plus_Float_MANY__Float_1__(col: &Collection) -> PureExecutionResult<Collection>
{
    const FUNC: &str = "meta::pure::functions::math::plus_Float_MANY__Float_1_";

    let iter = col.assume_float_many().map_err(|e| PureExecutionError::IllegalArgument { func: FUNC, arg: 0, cause: e.to_string() })?;
    let mut error = None;
    let sum = iter.filter_map(|x| x.map_err(|e| error = Some(e)).ok()).sum();
    error.map_or_else(|| Ok(float_one(sum)), Err)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_math_plus_Number_MANY__Number_1__(col: &Collection) -> PureExecutionResult<Collection>
{
    const FUNC: &str = "meta::pure::functions::math::plus_Number_MANY__Number_1_";

    if !col.pure_type().is_number()
    {
        return Err(PureExecutionError::IllegalArgument {
            func: FUNC,
            arg: 0,
            cause: PureExecutionError::UnexpectedValue { expected: "Number[*]".into(), got: col.full_type_as_string() }.to_string(),
        })?;
    }

    let mut error = None;
    let sum = col.into_iter().fold(integer(0), |a, n| match (a, n)
    {
        (Value::Integer(l), Value::Integer(r)) => integer(l + r),
        (Value::Integer(l), Value::Float(r)) => float(l as f64 + r),
        (Value::Float(l), Value::Integer(r)) => float(l + *r as f64),
        (Value::Float(l), Value::Float(r)) => float(l + r),
        _ =>
        {
            error = Some(PureExecutionError::UnexpectedValue { expected: "Number".into(), got: n.pure_type().to_string() });
            integer(0)
        }
    });

    error.map_or_else(|| Ok(sum.to_collection()), Err)
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_collection_isEmpty_Any_MANY__Boolean_1_(c: &Collection) -> PureExecutionResult<Collection>
{
    Ok(boolean_one(c.size().as_i64()? == 0))
}

#[allow(non_snake_case)]
pub fn meta_pure_functions_collection_isNotEmpty_Any_MANY__Boolean_1_(c: &Collection) -> PureExecutionResult<Collection>
{
    meta_pure_functions_boolean_not_Boolean_1__Boolean_1_(&is_empty(c)?)
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn boolean_and() -> PureExecutionResult<()>
    {
        assert_eq!(boolean_one(true), and(&boolean_one(true), &boolean_one(true))?);
        assert_eq!(boolean_one(false), and(&boolean_one(false), &boolean_one(true))?);
        assert_eq!(boolean_one(false), and(&boolean_one(true), &boolean_one(false))?);
        assert_eq!(boolean_one(false), and(&boolean_one(false), &boolean_one(false))?);

        let bad = and(&integer_one(1), &boolean_one(false));
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            format!("{}", bad.err().unwrap())
        );

        let bad = and(&boolean_one(false), &integer_one(1));
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 1 of meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn boolean_or() -> PureExecutionResult<()>
    {
        assert_eq!(boolean_one(true), or(&boolean_one(true), &boolean_one(true))?);
        assert_eq!(boolean_one(true), or(&boolean_one(false), &boolean_one(true))?);
        assert_eq!(boolean_one(true), or(&boolean_one(true), &boolean_one(false))?);
        assert_eq!(boolean_one(false), or(&boolean_one(false), &boolean_one(false))?);

        let bad = or(&integer_one(1), &boolean_one(false));
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 0 of meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            format!("{}", bad.err().unwrap())
        );

        let bad = or(&boolean_one(false), &integer_one(1));
        assert!(bad.is_err());
        assert_eq!(
            "Invalid argument passed to arg 1 of meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_: Unexpected value: expected Boolean[1] but got Integer[1]",
            format!("{}", bad.err().unwrap())
        );
        Ok(())
    }

    #[test]
    fn boolean_not() -> PureExecutionResult<()>
    {
        assert_eq!(boolean_one(true), not(&boolean_one(false))?);
        assert_eq!(boolean_one(false), not(&boolean_one(true))?);

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
        assert_eq!(integer_one(0), iplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).build()?)?);
        assert_eq!(integer_one(1), iplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(integer(1))?.build()?)?);
        assert_eq!(
            integer_one(6),
            iplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(integer(1))?.push(integer(2))?.push(integer(3))?.build()?)?
        );

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
        assert_eq!(float_one(0.0), fplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).build()?)?);
        assert_eq!(float_one(1.1), fplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(float(1.1))?.build()?)?);
        assert_eq!(
            float_one(6.6),
            fplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(float(1.1))?.push(float(2.2))?.push(float(3.3))?.build()?)?
        );

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
        assert_eq!(integer_one(0), nplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).build()?)?);
        assert_eq!(integer_one(1), nplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(integer(1))?.build()?)?);
        assert_eq!(
            integer_one(6),
            nplus(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(integer(1))?.push(integer(2))?.push(integer(3))?.build()?)?
        );

        assert_eq!(integer_one(0), nplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).build()?)?);
        assert_eq!(float_one(1.1), nplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(float(1.1))?.build()?)?);
        assert_eq!(
            float_one(6.6),
            nplus(&CollectionBuilder::new(Type::Float, ZERO_MANY).push(float(1.1))?.push(float(2.2))?.push(float(3.3))?.build()?)?
        );

        assert_eq!(
            float_one(6.4),
            nplus(&CollectionBuilder::new(Type::Number, ZERO_MANY).push(float(1.1))?.push(integer(2))?.push(float(3.3))?.build()?)?
        );

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
        assert_eq!(boolean_one(true), is_empty(&ZERO_NIL)?);
        assert_eq!(boolean_one(false), is_empty(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(integer(1))?.build()?)?);

        Ok(())
    }

    #[test]
    fn collection_is_not_empty() -> PureExecutionResult<()>
    {
        assert_eq!(boolean_one(false), is_not_empty(&ZERO_NIL)?);
        assert_eq!(boolean_one(true), is_not_empty(&CollectionBuilder::new(Type::Integer, ZERO_MANY).push(integer(1))?.build()?)?);

        Ok(())
    }
}
