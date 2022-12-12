// Copyright 2022 Dave Wathen. All rights reserved.

use std::borrow::Cow;

use crate::{CursorError, CursorResult};

#[macro_use]
mod testing
{
    #[macro_export]
    macro_rules! char_cursor_tests {
        ($factory:ident) => {
            #[test]
            fn empty_array_is_at_end_immediately() -> CursorResult<()>
            {
                let cursor = $factory("");
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource() -> CursorResult<()>
            {
                let mut cursor = $factory("Hello, World");
                // Hello, World
                // ^
                assert_eq!(CharToken::Char('H'), cursor.token()?);

                cursor.advance()?;
                // Hello, World
                //  ^
                assert_eq!(CharToken::Char('e'), cursor.token()?);

                cursor.advance_many(0)?;
                // Hello, World
                //  ^
                assert_eq!(CharToken::Char('e'), cursor.token()?);

                cursor.advance_many(6)?;
                // Hello, World
                //        ^
                assert_eq!(CharToken::Char('W'), cursor.token()?);

                cursor.advance_many(5)?;
                // Hello, World
                //             ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                Ok(())
            }

            #[test]
            fn advancing_many_returns_what_is_available() -> CursorResult<()>
            {
                let mut cursor = $factory("Hello, World");

                let advanced = cursor.advance_many(7)?;
                assert_eq!(7, advanced);
                assert_eq!(CharToken::Char('W'), cursor.token()?);
                let advanced = cursor.advance_many(10)?;
                assert_eq!(5, advanced);
                assert_eq!(CharToken::EndOfData, cursor.token()?);

                Ok(())
            }

            #[test]
            fn advancing_to_makes_cursors_equal() -> CursorResult<()>
            {
                let mut cursor = $factory("Hello, World");
                let mut cursor2 = cursor.clone();
                // Hello, World
                // ^
                // ^
                assert_eq!(cursor, cursor2);

                cursor2.advance_many(7)?;
                // Hello, World
                // ^
                //        ^
                assert_ne!(cursor, cursor2);

                cursor.advance_to(cursor2)?;
                // Hello, World
                //        ^
                //        ^
                assert_eq!(CharToken::Char('W'), cursor.token()?);
                Ok(())
            }

            #[test]
            fn cannot_advance_to_a_cursor_of_a_different_resource() -> CursorResult<()>
            {
                let mut cursor1 = $factory("Hello, Earth");
                let mut cursor2 = $factory("Hello, Mars");

                assert_ne!(cursor1, cursor2);
                cursor2.advance_many(7)?;
                assert!(cursor1.advance_to(cursor2).is_err());

                Ok(())
            }

            #[test]
            fn can_obtain_string_between_2_cursors() -> CursorResult<()>
            {
                let mut cursor1 = $factory("Hello, World");
                let mut cursor2 = cursor1.clone();

                // Hello, World
                // ^
                // ^
                assert_eq!("", &cursor1.between(&cursor2).unwrap());
                assert_eq!("", &cursor2.between(&cursor1).unwrap());

                cursor2.advance_many(5)?;
                // Hello, World
                // ^
                //      ^
                assert_eq!("Hello", &cursor1.between(&cursor2).unwrap());
                assert_eq!("Hello", &cursor2.between(&cursor1).unwrap());

                cursor2.advance_many(2)?;
                // Hello, World
                // ^
                //        ^
                assert_eq!("Hello, ", &cursor1.between(&cursor2).unwrap());
                assert_eq!("Hello, ", &cursor2.between(&cursor1).unwrap());

                cursor1.advance_many(12)?;
                // Hello, World
                //             ^
                //        ^
                assert_eq!("World", &cursor1.between(&cursor2).unwrap());
                assert_eq!("World", &cursor2.between(&cursor1).unwrap());

                Ok(())
            }

            #[test]
            fn cannot_obtain_between_2_cursors_of_differing_resources() -> CursorResult<()>
            {
                let cursor1 = $factory("Hello, Earth");
                let cursor2 = $factory("Hello, Mars");

                assert!(&cursor1.between(&cursor2).is_err());
                assert!(&cursor2.between(&cursor1).is_err());

                Ok(())
            }
        };
    }
}

mod regex;
mod utf8;

pub use regex::{Regex, RegexError, RegexResult};
pub use utf8::Utf8CharCursor;

/// A `CharCursor` represents a character position in a some resource (represented by a [precursor::byte::ByteCursor]).
pub trait CharCursor<'data>: Clone + PartialOrd
{
    /// Advances the cursor forward by one character. If the cursor is already at the end of data an error will be returned.
    ///
    /// An error may also be returned if the resource contains invalid data.
    fn advance(&mut self) -> CursorResult<()>;

    /// Advances the cursor forward by `n` characters. If the cursor is already at the end of data an error will be returned.
    /// If the cursor is not at the end but there are fewer characters than `n` remaining the cursor will advance to the
    /// end of data. The number of characters actually advanced is returned.
    ///
    /// An error may also be returned if the resource contains invalid data.
    fn advance_many(&mut self, how_many: usize) -> CursorResult<u64>
    {
        if self.token()? == CharToken::EndOfData
        {
            Err(CursorError::CannotAdvance)
        }
        else
        {
            let mut advanced = 0_u64;
            for _ in 0..how_many
            {
                self.advance()?;
                advanced += 1;
                if self.token()? == CharToken::EndOfData
                {
                    break;
                }
            }
            Ok(advanced)
        }
    }

    /// Advances this cursor forward to the `other` cursor's position.
    fn advance_to(&mut self, other: Self) -> CursorResult<()>;

    /// Returns the token represented by this cursor.  
    /// If the current position does not represent a valid tken (for example a bad character encoding) an error is returned.
    fn token(&self) -> CursorResult<CharToken>;

    /// Returns the bytes that represent the token currently pointed at by this cursor.  
    /// If the current position does not represent a valid tken (for example a bad character encoding) an error is returned.
    fn token_bytes(&self) -> CursorResult<Cow<[u8]>>;

    /// Returns the index (zero-based) of the position this cursor represents in the resource.  If the resource is at the end of data
    /// this will be the length of the resource in bytes.
    fn byte_index(&self) -> usize;

    /// Returns the string that starts at the lower of the cursors and terminates immediately before the higher.
    /// The `other` cursor must be for the same resource (e.g. cloned from this cursor).
    fn between(&self, other: &Self) -> CursorResult<Cow<'data, str>>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum CharToken
{
    Char(char),
    EndOfData,
}
