// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Type
{
    Primitive(PrimitiveType),
}

impl fmt::Debug for Type
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Type::Primitive(pt) => write!(f, "{}", pt),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum PrimitiveType
{
    String,
    Binary,
    Boolean,
    Number,
    Integer,
    Float,
    Decimal,
    Date,
    StrictTime,
    StrictDate,
    DateTime,
    LatestDate,
}

impl fmt::Display for PrimitiveType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) }
}

#[cfg(test)]
mod tests
{
    use crate::pure_type::PrimitiveType;

    #[test]
    fn primitives()
    {
        assert_eq!("String", format!("{}", PrimitiveType::String));
        assert_eq!("String", format!("{:?}", PrimitiveType::String));
        assert_eq!("Binary", format!("{}", PrimitiveType::Binary));
        assert_eq!("Binary", format!("{:?}", PrimitiveType::Binary));
        assert_eq!("Boolean", format!("{}", PrimitiveType::Boolean));
        assert_eq!("Boolean", format!("{:?}", PrimitiveType::Boolean));
        assert_eq!("Number", format!("{}", PrimitiveType::Number));
        assert_eq!("Number", format!("{:?}", PrimitiveType::Number));
        assert_eq!("Integer", format!("{}", PrimitiveType::Integer));
        assert_eq!("Integer", format!("{:?}", PrimitiveType::Integer));
        assert_eq!("Float", format!("{}", PrimitiveType::Float));
        assert_eq!("Float", format!("{:?}", PrimitiveType::Float));
        assert_eq!("Decimal", format!("{}", PrimitiveType::Decimal));
        assert_eq!("Decimal", format!("{:?}", PrimitiveType::Decimal));
        assert_eq!("Date", format!("{}", PrimitiveType::Date));
        assert_eq!("Date", format!("{:?}", PrimitiveType::Date));
        assert_eq!("StrictTime", format!("{}", PrimitiveType::StrictTime));
        assert_eq!("StrictTime", format!("{:?}", PrimitiveType::StrictTime));
        assert_eq!("StrictDate", format!("{}", PrimitiveType::StrictDate));
        assert_eq!("StrictDate", format!("{:?}", PrimitiveType::StrictDate));
        assert_eq!("DateTime", format!("{}", PrimitiveType::DateTime));
        assert_eq!("DateTime", format!("{:?}", PrimitiveType::DateTime));
        assert_eq!("LatestDate", format!("{}", PrimitiveType::LatestDate));
        assert_eq!("LatestDate", format!("{:?}", PrimitiveType::LatestDate));
    }
}
