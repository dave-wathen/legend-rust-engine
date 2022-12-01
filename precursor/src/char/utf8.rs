// Copyright 2022 Dave Wathen. All rights reserved.

use std::{borrow::Cow, cmp::Ordering, marker::PhantomData};

use crate::{
    byte::{ByteCursor, ByteToken},
    CursorError, CursorResult,
};

use super::{CharCursor, CharToken};

const INVALID_UTF8: CursorError = CursorError::InvalidData("Invalid UTF8 encoding");
const INVALID_UTF8_EOD: CursorError = CursorError::InvalidData("Invalid UTF8 encoding (unexpected end of data)");

/// A `CharCursor` that interprets bytes as UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8CharCursor<'data, BC: ByteCursor<'data>>
{
    bytes: BC,
    phantom: PhantomData<&'data usize>,
}

impl<'data, BC: ByteCursor<'data>> Utf8CharCursor<'data, BC>
{
    /// Creates a `Utf8CharCursor`.
    pub fn new(bytes: BC) -> Utf8CharCursor<'data, BC> { Utf8CharCursor { bytes, phantom: PhantomData } }
}

impl<'data, BC: ByteCursor<'data>> PartialOrd for Utf8CharCursor<'data, BC>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.bytes.partial_cmp(&other.bytes) }
}

impl<'data, BC: ByteCursor<'data>> CharCursor<'data> for Utf8CharCursor<'data, BC>
{
    fn advance(&mut self) -> crate::CursorResult<()>
    {
        if self.bytes.token().is_eod()
        {
            Err(CursorError::CannotAdvance)
        }
        else
        {
            let (_, end) = char_token(&self.bytes)?;
            self.bytes.advance_to(&end)?;
            Ok(())
        }
    }

    fn advance_to(&mut self, other: Self) -> crate::CursorResult<()>
    {
        match (*self).partial_cmp(&other)
        {
            None => Err(CursorError::Incompatible),
            Some(Ordering::Equal) => Ok(()),
            Some(Ordering::Less) =>
            {
                self.bytes.advance_to(&other.bytes)?;
                Ok(())
            }
            Some(Ordering::Greater) => Err(CursorError::CannotAdvance),
        }
    }

    fn token(&self) -> CursorResult<CharToken>
    {
        match self.bytes.token()
        {
            ByteToken::EndOfData => Ok(CharToken::EndOfData),
            ByteToken::Byte(_) => Ok(char_token(&self.bytes)?.0),
        }
    }

    fn token_bytes(&self) -> CursorResult<Cow<[u8]>>
    {
        let (_, end) = char_token(&self.bytes)?;
        self.bytes.between(&end)
    }

    fn byte_index(&self) -> usize { self.bytes.index() }

    fn between(&self, other: &Self) -> CursorResult<Cow<'data, str>>
    {
        match self.partial_cmp(other)
        {
            None => Err(CursorError::Incompatible),
            _ => match self.bytes.between(&other.bytes)?
            {
                Cow::Borrowed(bytes) => match std::str::from_utf8(bytes)
                {
                    Ok(s) => Ok(s.into()),
                    Err(_) => Err(INVALID_UTF8),
                },
                Cow::Owned(v) => match String::from_utf8(v)
                {
                    Ok(s) => Ok(s.into()),
                    Err(_) => Err(INVALID_UTF8),
                },
            },
        }
    }

    // fn matches(&self, seq: &crate::MatchableSequence) -> ScanResult<bool>
    // {
    //     if seq.is_empty()
    //     {
    //         Ok(self.token_at(self.offset)? != (Token::EndOfData, 0))
    //     }
    //     else if let MatchableSequence::Char(expected) = seq
    //     {
    //         match self.token_at(self.offset)
    //         {
    //             Ok((Token::Char(actual), _)) => Ok(actual == *expected),
    //             Err(e) => Err(e),
    //             _ => Ok(false),
    //         }
    //     }
    //     else if let MatchableSequence::MultiChar(multi) = seq
    //     {
    //         let mut pos = self.offset;
    //         for expected in multi
    //         {
    //             let (actual, width) = self.token_at(pos)?;
    //             if Token::Char(*expected) != actual
    //             {
    //                 return Ok(false);
    //             }
    //             pos += width;
    //         }
    //         Ok(true)
    //     }
    //     else
    //     {
    //         // Should not reach here as MatchableSequence::Empty should always return true for is_empty() above
    //         Ok(false)
    //     }
    // }
}

fn char_token<'data, BC: ByteCursor<'data>>(start: &BC) -> CursorResult<(CharToken, BC)>
{
    let mut bc = start.clone();

    match start.token()
    {
        ByteToken::EndOfData => Ok((CharToken::EndOfData, bc)),
        ByteToken::Byte(byte) =>
        {
            let width = utf8_char_width(byte)?;
            let advanced = bc.advance_many(width)?;
            if advanced != width
            {
                Err(INVALID_UTF8_EOD)
            }
            else if width == 1
            {
                Ok((CharToken::Char(char::from(byte)), bc))
            }
            else
            {
                let bytes = start.between(&bc)?;
                match std::str::from_utf8(&bytes)
                {
                    Ok(s) => Ok((CharToken::Char(s.chars().next().unwrap()), bc)),
                    Err(_) => Err(INVALID_UTF8),
                }
            }
        }
    }
}

fn utf8_char_width(byte: u8) -> CursorResult<usize>
{
    if byte < 128
    {
        Ok(1)
    }
    else if byte & b'\xE0' == b'\xC0'
    // 110x xxxx
    {
        Ok(2)
    }
    else if byte & b'\xF0' == b'\xE0'
    // 1110 xxxx
    {
        Ok(3)
    }
    else if byte & b'\xF8' == b'\xF0'
    // 1111 0xxx
    {
        Ok(4)
    }
    else
    {
        Err(INVALID_UTF8)
    }
}

#[cfg(test)]
mod tests
{
    use crate::byte::ByteArrayCursor;

    use super::*;

    fn new_cursor(s: &str) -> Utf8CharCursor<ByteArrayCursor>
    {
        let bytes = ByteArrayCursor::new(s.as_bytes());
        Utf8CharCursor::new(bytes)
    }

    char_cursor_tests!(new_cursor);

    #[test]
    fn read_utf8_chars() -> CursorResult<()>
    {
        let mut cursor = new_cursor("$£€\u{10348}");

        // $£€\u{10348}
        // ^
        assert_eq!(CharToken::Char('$'), cursor.token()?);
        assert_eq!([b'\x24'], cursor.token_bytes()?.as_ref());
        assert_eq!(0, cursor.byte_index());
        cursor.advance()?;
        // $£€\u{10348}
        //  ^
        assert_eq!(CharToken::Char('£'), cursor.token()?);
        assert_eq!([b'\xC2', b'\xA3'], cursor.token_bytes()?.as_ref());
        assert_eq!(1, cursor.byte_index());
        cursor.advance()?;
        // $£€\u{10348}
        //   ^
        assert_eq!(CharToken::Char('€'), cursor.token()?);
        assert_eq!([b'\xE2', b'\x82', b'\xAC'], cursor.token_bytes()?.as_ref());
        assert_eq!(3, cursor.byte_index());
        cursor.advance()?;
        // $£€\u{10348}
        //    ^
        assert_eq!(CharToken::Char('\u{10348}'), cursor.token()?);
        assert_eq!([b'\xF0', b'\x90', b'\x8D', b'\x88'], cursor.token_bytes()?.as_ref());
        assert_eq!(6, cursor.byte_index());
        cursor.advance()?;
        // $£€\u{10348}
        //             ^
        assert_eq!(CharToken::EndOfData, cursor.token()?);
        assert_eq!([] as [u8; 0], cursor.token_bytes()?.as_ref());
        assert_eq!(10, cursor.byte_index());
        Ok(())
    }

    #[test]
    fn fails_on_invalid_utf8_chars() -> CursorResult<()>
    {
        // Invalid first byte (does not match any of 0xxxxxxx, 110xxxxx, 1110xxxx, 11110xxx)
        let bytes = ByteArrayCursor::new(b"\xFF");
        let mut cursor = Utf8CharCursor::new(bytes);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid prohibited (UTF-16 surrogate pairs)
        let bytes = ByteArrayCursor::new(b"\xD8\x00");
        let mut cursor = Utf8CharCursor::new(bytes);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid first byte (out of valid range for 110xxxxx)
        let bytes = ByteArrayCursor::new(b"\xC0\x80");
        let mut cursor = Utf8CharCursor::new(bytes);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid subsequent byte (not 10xxxxxx)
        let bytes = ByteArrayCursor::new(b"\xE0\xA0\x00");
        let mut cursor = Utf8CharCursor::new(bytes);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid missing third byte
        let bytes = ByteArrayCursor::new(b"\xE0\xA0");
        let mut cursor = Utf8CharCursor::new(bytes);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        Ok(())
    }
}
