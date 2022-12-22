// Copyright 2022 Dave Wathen. All rights reserved.

use std::{borrow::Cow, cmp::Ordering, marker::PhantomData};

use crate::{
    byte::{ByteCursor, ByteToken},
    CursorError, CursorResult,
};

use super::{CharCursor, CharToken, EndOfLine, LineEndings, Location};

const INVALID_UTF8: CursorError = CursorError::InvalidData("Invalid UTF8 encoding");
const INVALID_UTF8_EOD: CursorError = CursorError::InvalidData("Invalid UTF8 encoding (unexpected end of data)");

/// A `CharCursor` that interprets bytes as UTF-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8CharCursor<'data, BC: ByteCursor<'data>>
{
    bytes: BC,
    line_endings: LineEndings,
    char_offset: usize,
    line_number: usize,
    line_start_char_offset: usize,
    phantom: PhantomData<&'data usize>,
}

impl<'data, BC: ByteCursor<'data>> Utf8CharCursor<'data, BC>
{
    /// Creates a `Utf8CharCursor`.
    pub fn new(bytes: BC, line_endings: LineEndings) -> Utf8CharCursor<'data, BC>
    {
        Utf8CharCursor { bytes, line_endings, char_offset: 0, line_number: 1, line_start_char_offset: 0, phantom: PhantomData }
    }

    fn char_token(&self) -> CursorResult<(CharToken, BC)>
    {
        let (token, mut bc) = Utf8CharCursor::char_at(&self.bytes)?;

        match token
        {
            CharToken::Char(ch) => match self.line_endings
            {
                LineEndings::None => Ok((token, bc)),
                LineEndings::Smart =>
                {
                    if ch == '\n'
                    {
                        Ok((CharToken::EndOfLine(EndOfLine::LF), bc))
                    }
                    else if ch == '\r'
                    {
                        let mut ending = EndOfLine::CR;
                        if let ByteToken::Byte(next_byte) = bc.token()
                        {
                            if next_byte == 10_u8
                            {
                                bc.advance()?;
                                ending = EndOfLine::CRLF;
                            }
                        };

                        Ok((CharToken::EndOfLine(ending), bc))
                    }
                    else
                    {
                        Ok((token, bc))
                    }
                }
                LineEndings::Char(eol_ch) =>
                {
                    if eol_ch == ch
                    {
                        let ending = if eol_ch == '\n'
                        {
                            EndOfLine::LF
                        }
                        else if eol_ch == '\r'
                        {
                            EndOfLine::CR
                        }
                        else
                        {
                            EndOfLine::Other
                        };
                        Ok((CharToken::EndOfLine(ending), bc))
                    }
                    else
                    {
                        Ok((token, bc))
                    }
                }
                LineEndings::TwoChar(eol_ch1, eol_ch2) =>
                {
                    if eol_ch1 == ch
                    {
                        let (next_token, next_bc) = Utf8CharCursor::char_at(&bc)?;
                        if let CharToken::Char(ch2) = next_token
                        {
                            if ch2 == eol_ch2
                            {
                                let ending = if eol_ch1 == '\r' && eol_ch2 == '\n' { EndOfLine::CRLF } else { EndOfLine::Other };
                                return Ok((CharToken::EndOfLine(ending), next_bc));
                            }
                        }
                    }
                    Ok((token, bc))
                }
            },
            _ => Ok((token, bc)),
        }
    }

    fn char_at(start: &BC) -> CursorResult<(CharToken, BC)>
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
}

impl<'data, BC: ByteCursor<'data>> PartialOrd for Utf8CharCursor<'data, BC>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.bytes.partial_cmp(&other.bytes) }
}

impl<'data, BC: ByteCursor<'data>> CharCursor<'data> for Utf8CharCursor<'data, BC>
{
    fn advance(&mut self) -> crate::CursorResult<()>
    {
        let (token, end) = self.char_token()?;
        match token
        {
            CharToken::Char(_) => self.char_offset += 1,
            CharToken::EndOfLine(eol) =>
            {
                self.char_offset += match eol
                {
                    EndOfLine::LF => 1,
                    EndOfLine::CRLF => 2,
                    EndOfLine::CR => 1,
                    EndOfLine::Other => match self.line_endings
                    {
                        LineEndings::Char(_) => 1,
                        LineEndings::TwoChar(_, _) => 2,
                        _ => unreachable!(),
                    },
                };
                self.line_number += 1;
                self.line_start_char_offset = self.char_offset;
            }
            CharToken::EndOfData => return Err(CursorError::CannotAdvance),
        }
        self.bytes.advance_to(&end)?;
        Ok(())
    }

    fn advance_to(&mut self, other: &Self) -> crate::CursorResult<()>
    {
        match (*self).partial_cmp(other)
        {
            None => Err(CursorError::Incompatible),
            Some(Ordering::Equal) => Ok(()),
            Some(Ordering::Less) =>
            {
                self.bytes.advance_to(&other.bytes)?;
                self.char_offset = other.char_offset;
                self.line_number = other.line_number;
                self.line_start_char_offset = other.line_start_char_offset;
                Ok(())
            }
            Some(Ordering::Greater) => Err(CursorError::CannotAdvance),
        }
    }

    fn location(&self) -> Location { Location::new(self.char_offset, self.line_number, self.char_offset - self.line_start_char_offset + 1) }

    fn token(&self) -> CursorResult<CharToken>
    {
        match self.bytes.token()
        {
            ByteToken::EndOfData => Ok(CharToken::EndOfData),
            ByteToken::Byte(_) => Ok(self.char_token()?.0),
        }
    }

    fn token_bytes(&self) -> CursorResult<Cow<[u8]>>
    {
        let (_, end) = self.char_token()?;
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
    use crate::{byte::ByteArrayCursor, char::LineEndings};

    use super::*;

    fn new_cursor(s: &str, eols: LineEndings) -> Utf8CharCursor<ByteArrayCursor>
    {
        let bytes = ByteArrayCursor::new(s.as_bytes());
        Utf8CharCursor::new(bytes, eols)
    }

    char_cursor_tests!(new_cursor);

    #[test]
    fn read_utf8_chars() -> CursorResult<()>
    {
        let mut cursor = new_cursor("$£€\u{10348}", LineEndings::Smart);

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
        let mut cursor = Utf8CharCursor::new(bytes, LineEndings::Smart);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid prohibited (UTF-16 surrogate pairs)
        let bytes = ByteArrayCursor::new(b"\xD8\x00");
        let mut cursor = Utf8CharCursor::new(bytes, LineEndings::Smart);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid first byte (out of valid range for 110xxxxx)
        let bytes = ByteArrayCursor::new(b"\xC0\x80");
        let mut cursor = Utf8CharCursor::new(bytes, LineEndings::Smart);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid subsequent byte (not 10xxxxxx)
        let bytes = ByteArrayCursor::new(b"\xE0\xA0\x00");
        let mut cursor = Utf8CharCursor::new(bytes, LineEndings::Smart);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        // Invalid missing third byte
        let bytes = ByteArrayCursor::new(b"\xE0\xA0");
        let mut cursor = Utf8CharCursor::new(bytes, LineEndings::Smart);
        assert!(cursor.token().is_err());
        assert!(cursor.token_bytes().is_err());
        assert!(cursor.advance().is_err());

        Ok(())
    }
}
