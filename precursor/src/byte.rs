// Copyright 2022 Dave Wathen. All rights reserved.

use std::borrow::Cow;

use crate::{CursorError, CursorResult};

#[macro_use]
mod testing
{
    #[macro_export]
    macro_rules! byte_cursor_tests {
        ($factory:ident) => {
            use std::ops::Deref;

            #[test]
            fn empty_array_is_at_end_immediately() -> CursorResult<()>
            {
                let cursor = $factory(&[]);
                assert_eq!(ByteToken::EndOfData, cursor.token());
                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource() -> CursorResult<()>
            {
                let mut cursor = $factory(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
                // 00 01 02 03 04 05
                // ^^

                assert_eq!(0, cursor.index());
                assert_eq!(0x00, cursor.token().unwrap());

                cursor.advance()?;
                // 00 01 02 03 04 05
                //    ^^
                assert_eq!(1, cursor.index());
                assert_eq!(0x01, cursor.token().unwrap());

                cursor.advance_many(0)?;
                // 00 01 02 03 04 05
                //    ^^
                assert_eq!(1, cursor.index());
                assert_eq!(0x01, cursor.token().unwrap());

                cursor.advance_many(3)?;
                // 00 01 02 03 04 05
                //             ^^
                assert_eq!(4, cursor.index());
                assert_eq!(0x04, cursor.token().unwrap());

                cursor.advance_many(2)?;
                // 00 01 02 03 04 05
                //                   ^^
                assert_eq!(6, cursor.index());
                assert!(cursor.token().is_eod());
                Ok(())
            }

            #[test]
            fn advancing_many_returns_what_is_available() -> CursorResult<()>
            {
                let mut cursor = $factory(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
                // 00 01 02 03 04 05
                // ^^

                cursor.advance_many(10)?;
                // 00 01 02 03 04 05
                //                   ^^
                assert_eq!(6, cursor.index());
                assert!(cursor.token().is_eod());

                Ok(())
            }

            #[test]
            fn advancing_to_makes_cursors_equal() -> CursorResult<()>
            {
                let mut cursor1 = $factory(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
                let mut cursor2 = cursor1.clone();
                assert_eq!(cursor1, cursor2);

                cursor2.advance_many(3)?;
                assert_ne!(cursor1, cursor2);

                cursor1.advance_to(&cursor2)?;
                assert_eq!(cursor1, cursor2);
                assert_eq!(3, cursor1.index());
                assert_eq!(0x03, cursor1.token().unwrap());
                Ok(())
            }

            #[test]
            fn cannot_advance_to_a_cursor_of_a_different_resource() -> CursorResult<()>
            {
                let mut cursor1 = $factory(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
                let mut cursor2 = $factory(&[0x06, 0x07, 0x08]);

                assert_ne!(cursor1, cursor2);
                cursor2.advance()?;
                assert!(cursor1.advance_to(&cursor2).is_err());

                Ok(())
            }

            #[test]
            fn can_obtain_string_between_2_cursors() -> CursorResult<()>
            {
                let mut cursor1 = $factory(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
                let mut cursor2 = cursor1.clone();

                // 00 01 02 03 04 05
                // ^^
                // ^^
                assert_eq!([] as [u8; 0], cursor1.between(&cursor2)?.deref());
                assert_eq!([] as [u8; 0], cursor2.between(&cursor1)?.deref());

                cursor2.advance_many(3)?;
                // 00 01 02 03 04 05
                // ^^
                //          ^^
                assert_eq!([0x00, 0x01, 0x02], cursor1.between(&cursor2)?.deref());
                assert_eq!([0x00, 0x01, 0x02], cursor2.between(&cursor1)?.deref());

                cursor1.advance_many(6)?;
                // 00 01 02 03 04 05
                //                   ^^
                //          ^^
                assert_eq!([0x03, 0x04, 0x05], cursor1.between(&cursor2)?.deref());
                assert_eq!([0x03, 0x04, 0x05], cursor2.between(&cursor1)?.deref());

                Ok(())
            }

            #[test]
            fn cannot_obtain_between_2_cursors_of_differing_resources() -> CursorResult<()>
            {
                let cursor1 = $factory(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
                let cursor2 = $factory(&[0x06, 0x07, 0x08]);

                assert!(&cursor1.between(&cursor2).is_err());
                assert!(&cursor2.between(&cursor1).is_err());

                Ok(())
            }
        };
    }
}

mod array;

pub use array::ByteArrayCursor;

/// A `ByteCursor` represents a byte position in a some resource.
/// At creation the cursor points to the first byte (as defined by the implementation) or to the end of data if the resource is empty.
/// Advancing (`advance()` or `advance_many(1)`) will cause it to point at the next byte.
///
/// Once a cursor is created it can be cloned so that the resource can be explored in multiple paths subject to any constraints
/// imposed by the implementation.
pub trait ByteCursor<'data>: Clone + PartialOrd + Eq
{
    /// Advances the cursor forward by one byte. If the cursor is already at the end of data an error will be returned.
    fn advance(&mut self) -> CursorResult<()>;

    /// Advances the cursor forward by `n` bytes. If the cursor is already at the end of data an error will be returned.
    /// If the cursor is not at the end but there are fewer bytes than `n` remaining the cursor will advance to the
    /// end of data. The number of bytes actually advanced is returned.
    fn advance_many(&mut self, how_many: usize) -> CursorResult<usize>
    {
        if self.token().is_eod()
        {
            Err(CursorError::CannotAdvance)
        }
        else
        {
            let mut advanced = 0;
            for _ in 0..how_many
            {
                self.advance()?;
                advanced += 1;
                if self.token().is_eod()
                {
                    break;
                }
            }
            Ok(advanced)
        }
    }

    /// Advances this cursor forward to the `other` cursor's position.
    fn advance_to(&mut self, other: &Self) -> CursorResult<()>;

    /// Returns the token represented by this cursor.  
    fn token(&self) -> ByteToken;

    /// Returns the index (zero-based) of the position this cursor represents in the resource.  If the resource is at the end of data
    /// this will be the length of the resource in bytes.
    fn index(&self) -> usize;

    /// Returns the bytes that starts at the lower of the cursors and terminates immediately before the higher.
    /// The `other` cursor must be for the same resource (e.g. cloned from this cursor).
    fn between(&self, other: &Self) -> CursorResult<Cow<'data, [u8]>>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ByteToken
{
    Byte(u8),
    EndOfData,
}

impl ByteToken
{
    // `true` for `ByteToken::EndOfData`.  `false' otherwise.
    pub fn is_eod(&self) -> bool { *self == ByteToken::EndOfData }
    // `true` for `ByteToken::Byte`.  `false' otherwise.
    pub fn is_byte(&self) -> bool { matches!(*self, ByteToken::Byte(_)) }

    // Unwraps a byte from a `ByteToken::Byte`.  Panics if the token is `ByteToken::EndOfData`.
    pub fn unwrap(self) -> u8 { self.expect("called `ByteToken::unwrap` on an `EndOfData` value") }

    pub fn expect(self, msg: &'static str) -> u8
    {
        if let ByteToken::Byte(b) = self
        {
            b
        }
        else
        {
            panic!("{}", msg)
        }
    }
}
