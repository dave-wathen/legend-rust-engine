// Copyright 2022 Dave Wathen. All rights reserved.

use std::{borrow::Cow, cmp::Ordering};

use crate::{CursorError, CursorResult};

use super::{ByteCursor, ByteToken};

/// A `ByteCursor` over a byte array.
#[derive(Debug, Clone, Eq)]
pub struct ByteArrayCursor<'data>
{
    bytes: &'data [u8],
    offset: usize,
}

impl ByteArrayCursor<'_>
{
    /// Creates a `ByteArrayCursor`.
    pub fn new(bytes: &[u8]) -> ByteArrayCursor { ByteArrayCursor { bytes, offset: 0 } }

    fn is_end_of_data(&self, index: usize) -> bool { index >= self.bytes.len() }
}

impl PartialEq for ByteArrayCursor<'_>
{
    fn eq(&self, other: &Self) -> bool { std::ptr::eq(self.bytes, other.bytes) && self.offset == other.offset }
}

impl PartialOrd for ByteArrayCursor<'_>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        if std::ptr::eq(self.bytes, other.bytes)
        {
            Some(self.index().cmp(&other.index()))
        }
        else
        {
            None
        }
    }
}

impl<'data> ByteCursor<'data> for ByteArrayCursor<'data>
{
    fn advance(&mut self) -> CursorResult<()>
    {
        if self.is_end_of_data(self.offset)
        {
            Err(CursorError::CannotAdvance)
        }
        else
        {
            self.offset += 1;
            Ok(())
        }
    }

    fn advance_to(&mut self, other: &Self) -> CursorResult<()>
    {
        match (*self).partial_cmp(other)
        {
            None => Err(CursorError::Incompatible),
            Some(Ordering::Equal) => Ok(()),
            Some(Ordering::Less) =>
            {
                self.offset = other.offset;
                Ok(())
            }
            Some(Ordering::Greater) => Err(CursorError::CannotAdvance),
        }
    }

    fn token(&self) -> ByteToken
    {
        if self.is_end_of_data(self.offset)
        {
            ByteToken::EndOfData
        }
        else
        {
            ByteToken::Byte(self.bytes[self.offset])
        }
    }

    fn index(&self) -> usize { self.offset }

    fn between(&self, other: &Self) -> CursorResult<Cow<'data, [u8]>>
    {
        match self.partial_cmp(other)
        {
            None => Err(CursorError::Incompatible),
            Some(Ordering::Equal) => Ok(Cow::Borrowed(&[])),
            Some(Ordering::Less) => Ok(Cow::Borrowed(&self.bytes[self.offset..other.offset])),
            Some(Ordering::Greater) => Ok(Cow::Borrowed(&self.bytes[other.offset..self.offset])),
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn new_cursor(bytes: &[u8]) -> ByteArrayCursor { ByteArrayCursor::new(bytes) }

    byte_cursor_tests!(new_cursor);
}
