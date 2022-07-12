// // Copyright 2022 Dave Wathen. All rights reserved.

use crate::{primitive, pure_type};

pub use self::empty::PureEmpty;

pub(super) mod empty;

pub trait PureCollection: pure_type::Typed + IntoIterator<Item = Self::PureItem>
{
    type PureItem: crate::PureValue;

    fn size(&self) -> primitive::PureInteger;
}

// impl pure_type::Typed for PureBoolean
// {
//     fn pure_type(&self) -> pure_type::Type { PURE_TYPE_BOOLEAN }
// }

// impl PureCollection for primitives::PureBoolean
// {
//     fn size(&self) -> PureInteger { PURE_INTEGER_1 }
// }

// pub struct PureMany<'pm>
// {
//     pure_type: pure_type::Type,
//     values: Vec<Box<dyn m4::CoreInstance<'pm>>>,
// }

// impl From<Vec<Box<dyn m4::CoreInstance>>> for PureMany
// {
//     // TODO Type
//     fn from(values: Vec<Box<dyn m4::CoreInstance>>) -> Self { Self { values, pure_type: PURE_TYPE_ANY } }
// }

// impl pure_type::Typed for PureMany
// {
//     fn pure_type(&self) -> pure_type::Type { self.pure_type }
// }

// impl PureCollection for PureMany
// {
//     fn size(&self) -> PureInteger { PureInteger::from(self.values.len()) }
// }

// pub struct PureBooleanMany(Vec<PureBoolean>);

// impl From<Vec<PureBoolean>> for PureBooleanMany
// {
//     fn from(values: Vec<PureBoolean>) -> Self { Self(values) }
// }

// impl pure_type::Typed for PureBooleanMany
// {
//     fn pure_type(&self) -> pure_type::Type { PURE_TYPE_BOOLEAN }
// }

// impl PureCollection for PureBooleanMany
// {
//     fn size(&self) -> PureInteger { PureInteger::from(self.0.len()) }
// }

// impl pure_type::Typed for [PureBoolean]
// {
//     fn pure_type(&self) -> pure_type::Type { PURE_TYPE_BOOLEAN }
// }

// impl PureCollection for [PureBoolean]
// {
//     fn size(&self) -> PureInteger { PureInteger::from(self.len()) }
// }

// // impl<T> PureCollection for Option<T>
// // where
// //     T: CoreInstance,
// // {
// //     fn size(&self) -> PureInteger
// //     {
// //         match self
// //         {
// //             Some(_) => PURE_INTEGER_1,
// //             None => PURE_INTEGER_0,
// //         }
// //     }
// // }

// impl pure_type::Typed for [PureInteger]
// {
//     fn pure_type(&self) -> pure_type::Type { PURE_TYPE_INTEGER }
// }

// impl PureCollection for [PureInteger]
// {
//     fn size(&self) -> PureInteger { PureInteger::from(self.len()) }
// }

#[cfg(test)]
mod tests
{
    // use super::*;
    // use crate::pure_type::Typed;
    //use crate::{PURE_FALSE, PURE_TRUE};

    // // TODO Restruct tests
    // // TODO Handle optionals
    // // TODO Types on CoreInstance and therefore on PureMany
    // #[test]
    // fn sizes()
    // {
    //     assert_eq!(PURE_INTEGER_1, PURE_FALSE.size());

    //     let values: Vec<Box<dyn m4::CoreInstance>> = vec![Box::new(PURE_FALSE), Box::new(PURE_TRUE), Box::new(PURE_INTEGER_0)];
    //     assert_eq!(3, *PureMany::from(values).size());

    //     assert_eq!(2, *PureBooleanMany::from(vec![PURE_TRUE, PURE_FALSE]).size());

    //     assert_eq!(2, *[PURE_TRUE, PURE_FALSE].size());
    //     // assert_eq!(1, *Some(PURE_TRUE).size());
    //     // assert_eq!(0, *Option::<PureBoolean>::None.size());

    //     assert_eq!(3, *[PURE_INTEGER_0, PURE_INTEGER_1, PureInteger::from(100)].size());
    //     // assert_eq!(1, *Some(PURE_INTEGER_0).size());
    //     // assert_eq!(0, *Option::<PureInteger>::None.size());

    //     // assert_eq!(1, *Some(PURE_INTEGER_0).size());
    //     //assert_eq!(0, *Option::<Box<dyn CoreInstance>>::None.size());
    // }
}
