// Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

pub const PRIMITIVES: [Type; 12] = [
    Type::String,
    Type::Binary,
    Type::Boolean,
    Type::Number,
    Type::Integer,
    Type::Float,
    Type::Decimal,
    Type::Date,
    Type::StrictTime,
    Type::StrictDate,
    Type::DateTime,
    Type::LatestDate,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Type
{
    Nil,
    Any,

    // Primitive Types
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
    // TODO Pure Enums
    // TODO Pure Classes
    // TODO Pure Measures
}

const REL_ANY: TypeRelation = TypeRelation::Type(Type::Any);

impl Type
{
    pub fn is_primitive(&self) -> bool { PRIMITIVES.iter().any(|prim| prim == self) }

    pub fn generalizations(&self) -> &[TypeRelation]
    {
        match self
        {
            Type::Any => &[],
            Type::Nil => &[TypeRelation::AllPrimitives, TypeRelation::AllClasses, TypeRelation::AllEnums, TypeRelation::AllMeasures],
            Type::String | Type::Binary | Type::Boolean | Type::Number | Type::Date => &[REL_ANY],
            Type::Integer | Type::Float | Type::Decimal => &[REL_ANY, TypeRelation::Type(Type::Number)],
            Type::StrictTime | Type::StrictDate | Type::DateTime | Type::LatestDate => &[REL_ANY, TypeRelation::Type(Type::Date)],
        }
    }

    pub fn specializations(&self) -> &[TypeRelation]
    {
        match self
        {
            Type::Any => &[TypeRelation::AllPrimitives, TypeRelation::AllClasses, TypeRelation::AllEnums, TypeRelation::AllMeasures],
            Type::Number => &[TypeRelation::Type(Type::Integer), TypeRelation::Type(Type::Float), TypeRelation::Type(Type::Decimal)],
            Type::Date => &[
                TypeRelation::Type(Type::StrictTime),
                TypeRelation::Type(Type::StrictDate),
                TypeRelation::Type(Type::DateTime),
                TypeRelation::Type(Type::LatestDate),
            ],
            _ => &[],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeRelation
{
    AllPrimitives,
    AllClasses,
    AllEnums,
    AllMeasures,
    Type(Type),
}

impl fmt::Display for Type
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:?}", self) }
}

pub trait Typed
{
    fn pure_type(&self) -> Type;
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn generalizations_match_specializations()
    {
        let types: Vec<&Type> = [Type::Any, Type::Nil].iter().chain(super::PRIMITIVES.iter()).collect();

        for ty in types
        {
            for gen in ty.generalizations()
            {
                if let TypeRelation::Type(gen_ty) = gen
                {
                    let reverse_exists = gen_ty.specializations().iter().any(|spc| *spc == TypeRelation::Type(*ty));
                    let is_via_all_primitives = ty.is_primitive() && gen_ty.specializations().iter().any(|spc| *spc == TypeRelation::AllPrimitives);
                    assert!(
                        reverse_exists || is_via_all_primitives,
                        "For type {:?} the generalization {:?} missing in {:?} specializations: {:?}",
                        ty,
                        gen,
                        gen_ty,
                        gen_ty.specializations()
                    );
                }
            }
        }
    }

    #[test]
    fn specializations_match_generalizations()
    {
        let types: Vec<&Type> = [Type::Any, Type::Nil].iter().chain(super::PRIMITIVES.iter()).collect();

        for ty in types
        {
            for spc in ty.specializations()
            {
                if let TypeRelation::Type(spc_ty) = spc
                {
                    let reverse_exists = spc_ty.generalizations().iter().any(|gen| *gen == TypeRelation::Type(*ty));
                    assert!(
                        reverse_exists,
                        "For type {:?} the specialization {:?} is missing in {:?} generalizations: {:?}",
                        ty,
                        spc,
                        spc_ty,
                        spc_ty.generalizations(),
                    );
                }
            }
        }
    }
}
