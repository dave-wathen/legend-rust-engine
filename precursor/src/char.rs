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
                let cursor = $factory("", LineEndings::Smart);
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());
                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_with_smart_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn", LineEndings::Smart);
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                // ^
                assert_eq!(CharToken::Char('H'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance()?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //  ^
                assert_eq!(CharToken::Char('e'), cursor.token()?);
                assert_eq!(Location::new(1, 1, 2), cursor.location());

                cursor.advance_many(0)?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //  ^
                assert_eq!(CharToken::Char('e'), cursor.token()?);
                assert_eq!(Location::new(1, 1, 2), cursor.location());

                cursor.advance_many(6)?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //        ^
                assert_eq!(CharToken::Char('W'), cursor.token()?);
                assert_eq!(Location::new(7, 1, 8), cursor.location());

                cursor.advance_many(5)?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //             ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::LF), cursor.token()?);
                assert_eq!(Location::new(12, 1, 13), cursor.location());
                assert_eq!(b"\n", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //               ^
                assert_eq!(CharToken::Char('H'), cursor.token()?);
                assert_eq!(Location::new(13, 2, 1), cursor.location());

                cursor.advance_many(11)?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //                          ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::CRLF), cursor.token()?);
                assert_eq!(Location::new(24, 2, 12), cursor.location());
                assert_eq!(b"\r\n", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //                              ^
                assert_eq!(CharToken::Char('H'), cursor.token()?);
                assert_eq!(Location::new(26, 3, 1), cursor.location());

                cursor.advance_many(14)?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //                                            ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::CR), cursor.token()?);
                assert_eq!(Location::new(40, 3, 15), cursor.location());
                assert_eq!(b"\r", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //                                              ^
                assert_eq!(CharToken::Char('H'), cursor.token()?);
                assert_eq!(Location::new(41, 4, 1), cursor.location());

                cursor.advance_many(99)?;
                // Hello, World\nHello, Mars\r\nHello, Jupiter\rHello, Saturn
                //                                                           ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(54, 4, 14), cursor.location());

                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_with_lf_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("aaa\nbbb\n", LineEndings::LF);
                // aaa\nbbb\n
                // ^
                assert_eq!(CharToken::Char('a'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\nbbb\n
                //    ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::LF), cursor.token()?);
                assert_eq!(Location::new(3, 1, 4), cursor.location());
                assert_eq!(b"\n", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa\nbbb\n
                //      ^
                assert_eq!(CharToken::Char('b'), cursor.token()?);
                assert_eq!(Location::new(4, 2, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\nbbb\n
                //         ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::LF), cursor.token()?);
                assert_eq!(Location::new(7, 2, 4), cursor.location());
                assert_eq!(b"\n", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa\nbbb\n
                //           ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(8, 3, 1), cursor.location());

                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_with_cr_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("aaa\rbbb\r", LineEndings::CR);
                // aaa\rbbb\r
                // ^
                assert_eq!(CharToken::Char('a'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\rbbb\r
                //    ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::CR), cursor.token()?);
                assert_eq!(Location::new(3, 1, 4), cursor.location());
                assert_eq!(b"\r", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa\rbbb\r
                //      ^
                assert_eq!(CharToken::Char('b'), cursor.token()?);
                assert_eq!(Location::new(4, 2, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\rbbb\r
                //         ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::CR), cursor.token()?);
                assert_eq!(Location::new(7, 2, 4), cursor.location());
                assert_eq!(b"\r", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa\rbbb\r
                //           ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(8, 3, 1), cursor.location());

                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_with_crlf_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("aaa\r\nbbb\r\n", LineEndings::CRLF);
                // aaa\r\nbbb\r\n
                // ^
                assert_eq!(CharToken::Char('a'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\r\nbbb\r\n
                //    ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::CRLF), cursor.token()?);
                assert_eq!(Location::new(3, 1, 4), cursor.location());
                assert_eq!(b"\r\n", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa\r\nbbb\r\n
                //        ^
                assert_eq!(CharToken::Char('b'), cursor.token()?);
                assert_eq!(Location::new(5, 2, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\r\nbbb\r\n
                //           ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::CRLF), cursor.token()?);
                assert_eq!(Location::new(8, 2, 4), cursor.location());
                assert_eq!(b"\r\n", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa\r\nbbb\r\n
                //               ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(10, 3, 1), cursor.location());

                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_with_custon_one_char_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("aaa+bbb+", LineEndings::Char('+'));
                // aaa+bbb+
                // ^
                assert_eq!(CharToken::Char('a'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa+bbb+
                //    ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::Other), cursor.token()?);
                assert_eq!(Location::new(3, 1, 4), cursor.location());
                assert_eq!(b"+", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa+bbb+
                //     ^
                assert_eq!(CharToken::Char('b'), cursor.token()?);
                assert_eq!(Location::new(4, 2, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa+bbb+
                //        ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::Other), cursor.token()?);
                assert_eq!(Location::new(7, 2, 4), cursor.location());
                assert_eq!(b"+", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa+bbb+
                //         ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(8, 3, 1), cursor.location());

                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_with_custon_two_char_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("aaa+@bbb+@", LineEndings::TwoChar('+', '@'));
                // aaa+@bbb+@
                // ^
                assert_eq!(CharToken::Char('a'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa+@bbb+@
                //    ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::Other), cursor.token()?);
                assert_eq!(Location::new(3, 1, 4), cursor.location());
                assert_eq!(b"+@", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa+@bbb+@
                //      ^
                assert_eq!(CharToken::Char('b'), cursor.token()?);
                assert_eq!(Location::new(5, 2, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa+@bbb+@
                //         ^
                assert_eq!(CharToken::EndOfLine(EndOfLine::Other), cursor.token()?);
                assert_eq!(Location::new(8, 2, 4), cursor.location());
                assert_eq!(b"+@", cursor.token_bytes()?.as_ref());

                cursor.advance()?;
                // aaa+@bbb+@
                //           ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(10, 3, 1), cursor.location());

                Ok(())
            }

            #[test]
            fn can_advance_through_a_resource_without_line_endings() -> CursorResult<()>
            {
                let mut cursor = $factory("aaa\nbbb\n", LineEndings::None);
                // aaa\nbbb\n
                // ^
                assert_eq!(CharToken::Char('a'), cursor.token()?);
                assert_eq!(Location::new(0, 1, 1), cursor.location());

                cursor.advance_many(3)?;
                // aaa\nbbb\n
                //    ^
                assert_eq!(CharToken::Char('\n'), cursor.token()?);
                assert_eq!(Location::new(3, 1, 4), cursor.location());

                cursor.advance()?;
                // aaa\nbbb\n
                //      ^
                assert_eq!(CharToken::Char('b'), cursor.token()?);
                assert_eq!(Location::new(4, 1, 5), cursor.location());

                cursor.advance_many(3)?;
                // aaa\nbbb\n
                //         ^
                assert_eq!(CharToken::Char('\n'), cursor.token()?);
                assert_eq!(Location::new(7, 1, 8), cursor.location());

                cursor.advance()?;
                // aaa\nbbb\n
                //           ^
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(8, 1, 9), cursor.location());

                Ok(())
            }

            #[test]
            fn advancing_many_returns_what_is_available() -> CursorResult<()>
            {
                let mut cursor = $factory("Hello, World", LineEndings::Smart);

                let advanced = cursor.advance_many(7)?;
                assert_eq!(7, advanced);
                assert_eq!(CharToken::Char('W'), cursor.token()?);
                assert_eq!(Location::new(7, 1, 8), cursor.location());

                let advanced = cursor.advance_many(10)?;
                assert_eq!(5, advanced);
                assert_eq!(CharToken::EndOfData, cursor.token()?);
                assert_eq!(Location::new(12, 1, 13), cursor.location());

                Ok(())
            }

            #[test]
            fn advancing_to_makes_cursors_equal() -> CursorResult<()>
            {
                let mut cursor = $factory("Hello, World", LineEndings::Smart);
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
                assert_eq!(Location::new(0, 1, 1), cursor.location());
                assert_eq!(Location::new(7, 1, 8), cursor2.location());

                cursor.advance_to(&cursor2)?;
                // Hello, World
                //        ^
                //        ^
                assert_eq!(CharToken::Char('W'), cursor.token()?);
                assert_eq!(Location::new(7, 1, 8), cursor.location());
                assert_eq!(Location::new(7, 1, 8), cursor2.location());

                Ok(())
            }

            #[test]
            fn cannot_advance_to_a_cursor_of_a_different_resource() -> CursorResult<()>
            {
                let mut cursor1 = $factory("Hello, Earth", LineEndings::Smart);
                let mut cursor2 = $factory("Hello, Mars", LineEndings::Smart);

                assert_ne!(cursor1, cursor2);
                cursor2.advance_many(7)?;
                assert!(cursor1.advance_to(&cursor2).is_err());

                Ok(())
            }

            #[test]
            fn can_obtain_string_between_2_cursors() -> CursorResult<()>
            {
                let mut cursor1 = $factory("Hello, World", LineEndings::Smart);
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
                let cursor1 = $factory("Hello, Earth", LineEndings::Smart);
                let cursor2 = $factory("Hello, Mars", LineEndings::Smart);

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

/// The line endings within the resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndings
{
    /// Line endings are not recognized (the resource is viewed as consisting of a single line)
    None,
    /// Recognizes any of the common endings (LF, CRLF, CR).  
    /// This is recommended unless you really need to only accept a specific line ending.  It is particularly useful when resources are transferred between
    /// systems on differing operating systems.
    Smart,
    /// Recognizes a specific 1 char line ending
    Char(char),
    /// Recognizes a specific 2 char line ending
    TwoChar(char, char),
}

impl LineEndings
{
    pub const LF: Self = Self::Char('\n');
    pub const CRLF: Self = Self::TwoChar('\r', '\n');
    pub const CR: Self = Self::Char('\r');
}

impl From<char> for LineEndings
{
    fn from(ch: char) -> Self { LineEndings::Char(ch) }
}

/// Describes the location within a character resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location
{
    /// The zero-based offset of the character within the resource
    char_offset: usize,
    /// The one-based line number within the resource
    line_number: usize,
    /// The one-based column number within the line of the resource
    column_number: usize,
}

impl Location
{
    pub fn new(char_offset: usize, line_number: usize, column_number: usize) -> Self { Location { char_offset, line_number, column_number } }

    pub fn char_offset(&self) -> usize { self.char_offset }
    pub fn line_number(&self) -> usize { self.line_number }
    pub fn column_number(&self) -> usize { self.column_number }
}

impl std::fmt::Display for Location
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "[{}:{}]", self.line_number, self.column_number) }
}

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
    fn advance_to(&mut self, other: &Self) -> CursorResult<()>;

    /// Returns the location represented by this cursor.  
    /// If the cursor is at the location returned is the location the next character would occupy if omne were appended to the resource.
    fn location(&self) -> Location;

    /// Returns the token represented by this cursor.  
    /// If the current position does not represent a valid token (for example a bad character encoding) an error is returned.
    fn token(&self) -> CursorResult<CharToken>;

    /// Returns the bytes that represent the token currently pointed at by this cursor.  
    /// If the current position does not represent a valid token (for example a bad character encoding) an error is returned.
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
    EndOfLine(EndOfLine),
    EndOfData,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EndOfLine
{
    LF,
    CRLF,
    CR,
    Other,
}
