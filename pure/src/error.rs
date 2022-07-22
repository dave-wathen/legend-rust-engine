// Copyright 2022 Dave Wathen. All rights reserved.

use thiserror::Error;

use crate::*;

#[derive(Error, Debug)]
pub enum PureExecutionError
{
    #[error("Illegal assignment: {from} value cannot be assigned to {to}")]
    IllegalAssignment
    {
        from: Type, to: Type
    },
    #[error("Illegal multiplicity: size of {size} cannot be assigned to {mult}")]
    IllegalMultiplicity
    {
        size: i64, mult: Multiplicity
    },
    #[error("Expected a Boolean value but got a {found}")]
    NotABoolean
    {
        found: Type
    },
    #[error("Expected an Integer value but got a {found}")]
    NotAnInteger
    {
        found: Type
    },
    #[error("Expected a Float value but got a {found}")]
    NotAFloat
    {
        found: Type
    },
    #[error("Illegal call to function {func} with arguments {args:?}")]
    IllegalFunctionCall
    {
        func: &'static str, args: Vec<Value>
    },
}
