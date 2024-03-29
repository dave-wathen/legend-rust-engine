// Copyright 2022 Dave Wathen. All rights reserved.

use thiserror::Error;

use crate::*;

#[derive(Error, Debug)]
pub enum PureExecutionError
{
    #[error("Cast exception: {from} cannot be cast to {to}")]
    IllegalCast
    {
        from: String, to: String
    },
    #[error("Unexpected value: expected {expected} but got {got}")]
    UnexpectedValue
    {
        expected: String, got: String
    },
    #[error("Invalid argument passed to arg {arg} of {func}: {cause}")]
    IllegalArgument
    {
        func: String, arg: usize, cause: String
    },
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
    #[error("Illegal value for type {pure_type}: {value}")]
    IllegalValue
    {
        pure_type: Type, value: String
    },
    #[error("Values ois of the wrong type: expected {expected}, found: {found}")]
    WrongType
    {
        expected: Type, found: Type
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
    #[error("DuplicateElementName: {name}")]
    DuplicateElementName
    {
        name: String
    },
    #[error("UnexpectedError: {problem}")]
    UnexpectedError
    {
        problem: &'static str
    },
    #[error("Infallible")]
    Infallible(#[from] std::convert::Infallible),
}
