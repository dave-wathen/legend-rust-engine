// // Copyright 2022 Dave Wathen. All rights reserved.

use crate::*;

// TODO Simplify function calling and composition

pub fn and(left: Value, right: Value) -> PureExecutionResult<Value>
{
    match (left, right)
    {
        // TODO Lazy eval
        (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l && r)),
        _ => Err(PureExecutionError::IllegalFunctionCall {
            func: "meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_",
            args: vec![left, right],
        }),
    }
}

pub fn or(left: Value, right: Value) -> PureExecutionResult<Value>
{
    match (left, right)
    {
        // TODO Lazy eval
        (Value::Boolean(l), Value::Boolean(r)) => Ok(Value::Boolean(l || r)),
        _ => Err(PureExecutionError::IllegalFunctionCall {
            func: "meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_",
            args: vec![left, right],
        }),
    }
}

pub fn not(b: Value) -> PureExecutionResult<Value>
{
    match b
    {
        // TODO Lazy eval
        Value::Boolean(b) => Ok(Value::Boolean(!b)),
        _ => Err(PureExecutionError::IllegalFunctionCall { func: "meta::pure::functions::boolean::not_Boolean_1__Boolean_1_", args: vec![b] }),
    }
}

pub fn is_empty(c: Collection) -> PureExecutionResult<Value> { Ok(Value::Boolean(c.size().as_i64()? == 0)) }

pub fn is_not_empty(c: Collection) -> PureExecutionResult<Value> { not(is_empty(c)?) }

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn boolean_and() -> PureExecutionResult<()>
    {
        assert_eq!(data::boolean(true), and(data::boolean(true), data::boolean(true))?);
        assert_eq!(data::boolean(false), and(data::boolean(false), data::boolean(true))?);
        assert_eq!(data::boolean(false), and(data::boolean(true), data::boolean(false))?);
        assert_eq!(data::boolean(false), and(data::boolean(false), data::boolean(false))?);

        let bad = and(data::integer(1), data::boolean(false));
        assert!(bad.is_err());
        assert_eq!(
            "Illegal call to function meta::pure::functions::boolean::and_Boolean_1__Boolean_1__Boolean_1_ with arguments [Integer(1), Boolean(false)]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn boolean_or() -> PureExecutionResult<()>
    {
        assert_eq!(data::boolean(true), or(data::boolean(true), data::boolean(true))?);
        assert_eq!(data::boolean(true), or(data::boolean(false), data::boolean(true))?);
        assert_eq!(data::boolean(true), or(data::boolean(true), data::boolean(false))?);
        assert_eq!(data::boolean(false), or(data::boolean(false), data::boolean(false))?);

        let bad = or(data::integer(1), data::boolean(false));
        assert!(bad.is_err());
        assert_eq!(
            "Illegal call to function meta::pure::functions::boolean::or_Boolean_1__Boolean_1__Boolean_1_ with arguments [Integer(1), Boolean(false)]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn boolean_not() -> PureExecutionResult<()>
    {
        assert_eq!(data::boolean(true), not(data::boolean(false))?);
        assert_eq!(data::boolean(false), not(data::boolean(true))?);

        let bad = not(data::integer(1));
        assert!(bad.is_err());
        assert_eq!(
            "Illegal call to function meta::pure::functions::boolean::not_Boolean_1__Boolean_1_ with arguments [Integer(1)]",
            format!("{}", bad.err().unwrap())
        );

        Ok(())
    }

    #[test]
    fn collection_is_empty() -> PureExecutionResult<()>
    {
        assert_eq!(data::boolean(true), is_empty(ZERO_NIL)?);
        assert_eq!(data::boolean(false), is_empty(data::CollectionBuilder::new(Type::Integer, ZERO_MANY).push(data::integer(1))?.build()?)?);

        Ok(())
    }

    #[test]
    fn collection_is_not_empty() -> PureExecutionResult<()>
    {
        assert_eq!(data::boolean(false), is_not_empty(ZERO_NIL)?);
        assert_eq!(data::boolean(true), is_not_empty(data::CollectionBuilder::new(Type::Integer, ZERO_MANY).push(data::integer(1))?.build()?)?);

        Ok(())
    }
}
