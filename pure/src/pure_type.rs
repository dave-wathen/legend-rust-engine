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
    pub fn is_class(&self) -> bool { false } // TODO
    pub fn is_enum(&self) -> bool { false } // TODO
    pub fn is_measure(&self) -> bool { false } // TODO

    pub fn generalizations(&self) -> &[TypeRelation]
    {
        match self
        {
            Type::Any => &[],
            Type::Nil => &[REL_ANY, TypeRelation::AllPrimitives, TypeRelation::AllClasses, TypeRelation::AllEnums, TypeRelation::AllMeasures],
            Type::String | Type::Binary | Type::Boolean | Type::Number | Type::Date => &[REL_ANY],
            Type::Integer | Type::Float | Type::Decimal => &[REL_ANY, TypeRelation::Type(Type::Number)],
            Type::StrictTime | Type::StrictDate | Type::DateTime | Type::LatestDate => &[REL_ANY, TypeRelation::Type(Type::Date)],
        }
    }

    pub fn specializations(&self) -> &[TypeRelation]
    {
        match self
        {
            Type::Any => &[
                TypeRelation::Type(Type::Nil),
                TypeRelation::AllPrimitives,
                TypeRelation::AllClasses,
                TypeRelation::AllEnums,
                TypeRelation::AllMeasures,
            ],
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

    pub fn is_assignable_from(&self, other: &Type) -> bool
    {
        if self == other
        {
            true
        }
        else
        {
            self.is_generalization_of(other)
        }
    }

    pub fn is_generalization_of(&self, other: &Type) -> bool
    {
        for rel in other.generalizations()
        {
            match rel
            {
                TypeRelation::AllPrimitives =>
                {
                    if self.is_primitive()
                    {
                        return true;
                    }
                }
                TypeRelation::AllClasses =>
                {
                    if self.is_class()
                    {
                        return true;
                    }
                }
                TypeRelation::AllEnums =>
                {
                    if self.is_enum()
                    {
                        return true;
                    }
                }
                TypeRelation::AllMeasures =>
                {
                    if self.is_measure()
                    {
                        return true;
                    }
                }
                TypeRelation::Type(t) =>
                {
                    if t == self || self.is_generalization_of(t)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_specialization_of(&self, other: &Type) -> bool { other.is_generalization_of(self) }
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

    #[test]
    fn primitive_generalizations()
    {
        assert!(Type::Any.is_generalization_of(&Type::String));
        assert!(Type::Any.is_generalization_of(&Type::Binary));
        assert!(Type::Any.is_generalization_of(&Type::Boolean));
        assert!(Type::Any.is_generalization_of(&Type::Number));
        assert!(Type::Any.is_generalization_of(&Type::Integer));
        assert!(Type::Any.is_generalization_of(&Type::Float));
        assert!(Type::Any.is_generalization_of(&Type::Decimal));
        assert!(Type::Any.is_generalization_of(&Type::Date));
        assert!(Type::Any.is_generalization_of(&Type::StrictTime));
        assert!(Type::Any.is_generalization_of(&Type::StrictDate));
        assert!(Type::Any.is_generalization_of(&Type::DateTime));
        assert!(Type::Any.is_generalization_of(&Type::LatestDate));

        assert!(Type::Number.is_generalization_of(&Type::Integer));
        assert!(Type::Number.is_generalization_of(&Type::Float));
        assert!(Type::Number.is_generalization_of(&Type::Decimal));

        assert!(Type::Date.is_generalization_of(&Type::StrictTime));
        assert!(Type::Date.is_generalization_of(&Type::StrictDate));
        assert!(Type::Date.is_generalization_of(&Type::DateTime));
        assert!(Type::Date.is_generalization_of(&Type::LatestDate));
    }

    #[test]
    fn nil_generalizations()
    {
        assert!(Type::Any.is_generalization_of(&Type::Nil));

        assert!(Type::String.is_generalization_of(&Type::Nil));
        assert!(Type::Binary.is_generalization_of(&Type::Nil));
        assert!(Type::Boolean.is_generalization_of(&Type::Nil));
        assert!(Type::Number.is_generalization_of(&Type::Nil));
        assert!(Type::Integer.is_generalization_of(&Type::Nil));
        assert!(Type::Float.is_generalization_of(&Type::Nil));
        assert!(Type::Decimal.is_generalization_of(&Type::Nil));
        assert!(Type::Date.is_generalization_of(&Type::Nil));
        assert!(Type::StrictTime.is_generalization_of(&Type::Nil));
        assert!(Type::StrictDate.is_generalization_of(&Type::Nil));
        assert!(Type::DateTime.is_generalization_of(&Type::Nil));
        assert!(Type::LatestDate.is_generalization_of(&Type::Nil));
    }

    #[test]
    fn primitive_specializations()
    {
        assert!(Type::String.is_specialization_of(&Type::Any));
        assert!(Type::Binary.is_specialization_of(&Type::Any));
        assert!(Type::Boolean.is_specialization_of(&Type::Any));
        assert!(Type::Number.is_specialization_of(&Type::Any));
        assert!(Type::Integer.is_specialization_of(&Type::Any));
        assert!(Type::Float.is_specialization_of(&Type::Any));
        assert!(Type::Decimal.is_specialization_of(&Type::Any));
        assert!(Type::Date.is_specialization_of(&Type::Any));
        assert!(Type::StrictTime.is_specialization_of(&Type::Any));
        assert!(Type::StrictDate.is_specialization_of(&Type::Any));
        assert!(Type::DateTime.is_specialization_of(&Type::Any));
        assert!(Type::LatestDate.is_specialization_of(&Type::Any));

        assert!(Type::Integer.is_specialization_of(&Type::Number));
        assert!(Type::Float.is_specialization_of(&Type::Number));
        assert!(Type::Decimal.is_specialization_of(&Type::Number));

        assert!(Type::StrictTime.is_specialization_of(&Type::Date));
        assert!(Type::StrictDate.is_specialization_of(&Type::Date));
        assert!(Type::DateTime.is_specialization_of(&Type::Date));
        assert!(Type::LatestDate.is_specialization_of(&Type::Date));
    }

    #[test]
    fn nil_specializations()
    {
        assert!(Type::Nil.is_specialization_of(&Type::Any));

        assert!(Type::Nil.is_specialization_of(&Type::String));
        assert!(Type::Nil.is_specialization_of(&Type::Binary));
        assert!(Type::Nil.is_specialization_of(&Type::Boolean));
        assert!(Type::Nil.is_specialization_of(&Type::Number));
        assert!(Type::Nil.is_specialization_of(&Type::Integer));
        assert!(Type::Nil.is_specialization_of(&Type::Float));
        assert!(Type::Nil.is_specialization_of(&Type::Decimal));
        assert!(Type::Nil.is_specialization_of(&Type::Date));
        assert!(Type::Nil.is_specialization_of(&Type::StrictTime));
        assert!(Type::Nil.is_specialization_of(&Type::StrictDate));
        assert!(Type::Nil.is_specialization_of(&Type::DateTime));
        assert!(Type::Nil.is_specialization_of(&Type::LatestDate));
    }
}
