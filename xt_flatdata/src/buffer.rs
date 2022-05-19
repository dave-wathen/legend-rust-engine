// Copyright 2022 Dave Wathen. All rights reserved.

use crate::FlatDataError;
use std::cell::RefCell;
use std::cmp::{min, Ordering};
use std::fmt;
use std::io::Read;
use std::rc::Rc;

type FlatDataResult<T> = Result<T, FlatDataError>;

/// A `CharCursor` represents a character position in a UTF-8 encoded resource.
/// It is initially created using CharCursor::open.  At creation the cursor points to the first character
/// or to the end point if the resource is empty. Advancing (`advance()` or `advance_many(1)`) will cause it
/// to point at the next character.
///
/// Once a cursor is obtained it can be cloned to allow the file to be explored through multiple code paths.
///
/// # Example
/// ```
/// use legend_xt_flatdata::CharCursor;
///
/// let reader = Box::new("Life in the model world".as_bytes());
/// let mut cursor = CharCursor::open(10, 30, reader).unwrap();
/// let mut cursor2 = cursor.clone();
///
/// cursor.advance_many(12);
/// cursor2.advance_many(17);
/// assert_eq!('m', cursor.current_char().unwrap());
/// assert_eq!("model", cursor.between(&cursor2).unwrap());
/// ```
pub struct CharCursor
{
    reader: Rc<RefCell<BufferedReader>>,
    state: CharCursorState,
}

impl CharCursor
{
    /// Creates a `CharCursor`
    ///
    /// # Arguments
    /// * `block_size` - the size of blocks to be read from the reader.
    /// * `capacity` - the maximum number of bytes that can be held in memory at a point in time.  This should be a multiple of the `block_size`.
    ///   The data required to be in memory is defined by the lowest and highest cursor positions with respect to the blocks they reference.
    /// * `reader` - the UTF-8 resource from which bytes are consumed.
    pub fn open(block_size: usize, capacity: u64, reader: Box<dyn Read>) -> FlatDataResult<CharCursor>
    {
        let mut buffered_reader = BufferedReader {
            block_size,
            capacity,
            reader,
            blocks: vec![],
            end_index: None,
        };
        let maybe_char = buffered_reader.ensure_char_from(0)?;
        let state = CharCursorState::from(maybe_char, 0);

        buffered_reader.add_cursor(state);

        let wrapped = Rc::new(RefCell::new(buffered_reader));

        let mut cursor = CharCursor {
            reader: Rc::clone(&wrapped),
            state,
        };

        if let Some(char) = maybe_char
        {
            if char == '\u{FEFF}'
            {
                cursor.advance()?;
            }
        }

        Ok(cursor)
    }

    pub fn advance(&mut self) -> FlatDataResult<()>
    {
        match self.state
        {
            CharCursorState::CharAtByteIndex(char, index) =>
            {
                let pre_state = self.state;
                let width = char.len_utf8() as u64;
                let new_index = index + width;
                let maybe_char = self.reader.borrow_mut().ensure_char_from(new_index)?;
                self.state = CharCursorState::from(maybe_char, new_index);
                self.reader.borrow_mut().move_cursor(pre_state, self.state);
                Ok(())
            }
            CharCursorState::End => Err(FlatDataError::CannotAdvanceBeyondEndOfData),
        }
    }

    pub fn advance_many(&mut self, how_many: u32) -> FlatDataResult<()>
    {
        for _ in 0..how_many
        {
            self.advance()?;
            if self.is_end_of_data()
            {
                break;
            }
        }
        Ok(())
    }

    pub fn advance_to(&mut self, other: &CharCursor) -> FlatDataResult<()>
    {
        match (&*self).partial_cmp(other)
        {
            None => Err(FlatDataError::CursorResourcesDiffer),
            Some(Ordering::Equal) => Ok(()),
            Some(Ordering::Less) =>
            {
                let pre_state = self.state;
                self.state = other.state;
                self.reader.borrow_mut().move_cursor(pre_state, self.state);
                Ok(())
            }
            Some(Ordering::Greater) => Err(FlatDataError::CursorResourcesDiffer),
        }
    }

    pub fn current_char(&self) -> Option<char>
    {
        match self.state
        {
            CharCursorState::CharAtByteIndex(char, _) => Some(char),
            _ => None,
        }
    }

    pub fn current_byte_index(&self) -> Option<u64>
    {
        match self.state
        {
            CharCursorState::CharAtByteIndex(_, index) => Some(index),
            _ => None,
        }
    }

    pub fn is_end_of_data(&self) -> bool
    {
        matches!(self.state, CharCursorState::End)
    }

    pub fn between(&self, other: &CharCursor) -> FlatDataResult<String>
    {
        match self.partial_cmp(other)
        {
            None => Err(FlatDataError::CursorResourcesDiffer),
            Some(Ordering::Equal) => Ok(String::from("")),
            Some(Ordering::Less) => self.reader.borrow().between(self.state, other.state),
            Some(Ordering::Greater) => self.reader.borrow().between(other.state, self.state),
        }
    }
}

impl PartialOrd for CharCursor
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    {
        if Rc::ptr_eq(&self.reader, &other.reader)
        {
            match (self.state, other.state)
            {
                (CharCursorState::CharAtByteIndex(_, self_index), CharCursorState::CharAtByteIndex(_, other_index)) =>
                {
                    Some(self_index.cmp(&other_index))
                }
                (CharCursorState::CharAtByteIndex(_, _), CharCursorState::End) => Some(Ordering::Less),
                (CharCursorState::End, CharCursorState::End) => Some(Ordering::Equal),
                (CharCursorState::End, _) => Some(Ordering::Greater),
            }
        }
        else
        {
            None
        }
    }
}

impl PartialEq for CharCursor
{
    fn eq(&self, other: &Self) -> bool
    {
        Rc::ptr_eq(&self.reader, &other.reader)
            && match (self.state, other.state)
            {
                (CharCursorState::CharAtByteIndex(_, self_index), CharCursorState::CharAtByteIndex(_, other_index)) => self_index == other_index,
                (CharCursorState::End, CharCursorState::End) => true,
                _ => false,
            }
    }
}

impl Clone for CharCursor
{
    fn clone(&self) -> Self
    {
        self.reader.borrow_mut().add_cursor(self.state);

        CharCursor {
            reader: Rc::clone(&self.reader),
            state: self.state,
        }
    }
}

impl Drop for CharCursor
{
    fn drop(&mut self)
    {
        self.reader.borrow_mut().remove_cursor(self.state);
    }
}

// BufferedReader holds the on heap data for a resource that is being read by one or more CharCursors.
struct BufferedReader
{
    block_size: usize,
    capacity: u64,
    reader: Box<dyn Read>,
    blocks: Vec<Block>,
    end_index: Option<u64>,
}

impl fmt::Debug for BufferedReader
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(
            f,
            "BufferedReader {{ block_size: {:?}, capacity: {:?}, blocks: {:?}, end_index: {:?} }}",
            self.block_size, self.capacity, self.blocks, self.end_index
        )
    }
}

impl BufferedReader
{
    fn between(&self, state_1: CharCursorState, state_2: CharCursorState) -> FlatDataResult<String>
    {
        let start_index = match state_1
        {
            CharCursorState::CharAtByteIndex(_, index) => index,
            CharCursorState::End => self.end_index.unwrap(),
        };

        let end_index = match state_2
        {
            CharCursorState::CharAtByteIndex(char, index) => index + char.len_utf8() as u64 - 1,
            CharCursorState::End => self.end_index.unwrap(),
        };

        let mut bytes = Vec::with_capacity((end_index - start_index) as usize);
        for block in self.blocks.iter()
        {
            if start_index >= block.start_index && start_index < block.end_index
            {
                let start = (start_index - block.start_index) as usize;
                let end = (min(end_index, block.end_index) - block.start_index) as usize;
                bytes.extend_from_slice(&block.data[start..end]);
            }
            else if start_index < block.start_index && end_index > block.start_index
            {
                let start = 0;
                let end = (min(end_index, block.end_index) - block.start_index) as usize;
                bytes.extend_from_slice(&block.data[start..end]);
            };
        }

        String::from_utf8(bytes).map_err(|_| FlatDataError::Utf8Error)
    }

    fn move_cursor(&mut self, from: CharCursorState, to: CharCursorState)
    {
        self.remove_cursor(from);
        self.add_cursor(to);
    }

    fn add_cursor(&mut self, state: CharCursorState)
    {
        if let CharCursorState::CharAtByteIndex(_, index) = state
        {
            for block in self.blocks.iter_mut()
            {
                if index >= block.start_index && index < block.end_index
                {
                    block.cursor_count += 1;
                    return;
                }
            }
            panic!("No block for index {}", index);
        }
    }

    fn remove_cursor(&mut self, state: CharCursorState)
    {
        if let CharCursorState::CharAtByteIndex(_, index) = state
        {
            for block in self.blocks.iter_mut()
            {
                if index >= block.start_index && index < block.end_index
                {
                    block.cursor_count -= 1;
                    return;
                }
            }
            panic!("No block for index {}", index);
        }
    }

    fn ensure_char_from(&mut self, index: u64) -> FlatDataResult<Option<char>>
    {
        match self.ensure_byte(index)?
        {
            None => Ok(None),
            Some(byte) =>
            {
                let width = utf8_char_width(byte)?;
                if width == 1
                {
                    Ok(Some(char::from(byte)))
                }
                else
                {
                    let mut bytes: [u8; 4] = [0; 4];
                    bytes[0] = byte;
                    #[allow(clippy::needless_range_loop)]
                    for i in 1..width
                    {
                        bytes[i] = self.byte_at(index + i as u64)?;
                    }

                    let s = std::str::from_utf8(&bytes[0..width]).map_err(|_| FlatDataError::Utf8Error)?;
                    s.chars().next().unwrap();
                    Ok(Some(s.chars().next().unwrap()))
                }
            }
        }
    }

    fn ensure_byte(&mut self, index: u64) -> FlatDataResult<Option<u8>>
    {
        while !self.index_has_been_read(index) && self.end_index.is_none()
        {
            self.read_block()?;
        }

        let result = match self.end_index
        {
            Some(i) if index >= i => None,
            _ => Some(self.byte_at(index)?),
        };
        Ok(result)
    }

    fn index_has_been_read(&mut self, index: u64) -> bool
    {
        match self.blocks.last()
        {
            Some(block) => index < block.end_index,
            None => false,
        }
    }

    fn read_block(&mut self) -> FlatDataResult<()>
    {
        let previously_read = match self.blocks.last()
        {
            Some(block) => block.end_index,
            None => 0,
        };

        let mut can_delete = true;
        self.blocks.retain(|block| {
            can_delete = can_delete && block.cursor_count == 0;
            !can_delete
        });

        let capacity_used: u64 = self.blocks.iter().map(|block| block.end_index - block.start_index).sum();
        if capacity_used >= self.capacity
        {
            return Err(FlatDataError::CapacityUsed(self.capacity));
        }

        let block_size = min(self.block_size, (self.capacity - capacity_used) as usize);
        let mut buffer = vec![0; block_size];
        let read = self.reader.as_mut().read(&mut buffer)?;

        if read == 0
        {
            self.end_index = Some(previously_read)
        }
        else
        {
            let block = Block {
                start_index: previously_read,
                end_index: previously_read + read as u64,
                data: buffer,
                cursor_count: 0,
            };

            self.blocks.push(block);
        }
        Ok(())
    }

    fn byte_at(&self, index: u64) -> FlatDataResult<u8>
    {
        for block in &self.blocks
        {
            if block.start_index <= index && index < block.end_index
            {
                let i: usize = (index - block.start_index) as usize;
                return Ok(block.data[i]);
            }
        }
        Err(FlatDataError::ByteIndexUnavailable(index))
    }
}

fn utf8_char_width(byte: u8) -> FlatDataResult<usize>
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
        Err(FlatDataError::InvalidUtf8(byte))
    }
}

#[derive(Debug)]
struct Block
{
    start_index: u64, // Inclusive
    end_index: u64,   // Exclusive
    data: Vec<u8>,
    cursor_count: u32,
}

#[derive(Copy, Clone, Debug)]
enum CharCursorState
{
    CharAtByteIndex(char, u64),
    End,
}

impl CharCursorState
{
    pub fn from(maybe_char: Option<char>, index: u64) -> CharCursorState
    {
        match maybe_char
        {
            Some(char) => CharCursorState::CharAtByteIndex(char, index),
            None => CharCursorState::End,
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn can_open_a_cursor_and_read_ascii_data() -> FlatDataResult<()>
    {
        let reader = Box::new("Life in the model world".as_bytes());

        let mut cursor = CharCursor::open(10, 20, reader)?;
        assert!(!cursor.is_end_of_data());
        assert_eq!('L', cursor.current_char().unwrap());
        cursor.advance()?;
        assert_eq!('i', cursor.current_char().unwrap());
        cursor.advance()?;
        assert_eq!('f', cursor.current_char().unwrap());
        cursor.advance()?;
        assert_eq!('e', cursor.current_char().unwrap());
        cursor.advance_many(6)?;
        assert_eq!('h', cursor.current_char().unwrap());

        // 10 bytes (which is also 10 ASCII chars in UTF-8) have been consumed so next advance causes next block
        cursor.advance()?;
        assert_eq!('e', cursor.current_char().unwrap());

        // Advance to last char
        cursor.advance_many(12)?;
        assert_eq!('d', cursor.current_char().unwrap());

        // Next advance gives end of data
        cursor.advance()?;
        assert!(cursor.is_end_of_data());
        assert!(cursor.current_char().is_none());

        Ok(())
    }

    #[test]
    fn empty_resource_is_immediately_at_end() -> FlatDataResult<()>
    {
        let reader = Box::new("".as_bytes());

        let cursor = CharCursor::open(10, 20, reader)?;
        assert!(cursor.is_end_of_data());

        Ok(())
    }

    #[test]
    fn advancing_beyond_eof_returns_what_is_available() -> FlatDataResult<()>
    {
        let reader = Box::new("Life in the model world".as_bytes());

        let mut cursor = CharCursor::open(10, 20, reader)?;
        cursor.advance_many(17)?;
        assert_eq!(' ', cursor.current_char().unwrap());
        cursor.advance_many(10)?;
        assert!(cursor.is_end_of_data());
        assert!(cursor.current_char().is_none());

        Ok(())
    }

    #[test]
    fn capacity_exceeded_if_consuming_too_far_on_only_one_cursor() -> FlatDataResult<()>
    {
        let reader = Box::new("Life in the model world".as_bytes());

        let cursor = CharCursor::open(5, 15, reader)?;
        #[allow(clippy::redundant_clone)]
        let mut cursor2 = cursor.clone();
        assert!(cursor2.advance_many(15).is_err());

        Ok(())
    }

    #[test]
    fn cursors_can_advance_if_they_stay_within_capacity() -> FlatDataResult<()>
    {
        let reader = Box::new("Life in the model world".as_bytes());

        let mut cursor = CharCursor::open(5, 15, reader)?;
        let mut cursor2 = cursor.clone();

        cursor.advance_many(13)?;
        assert_eq!('o', cursor.current_char().unwrap());
        cursor2.advance_many(12)?;
        assert_eq!('m', cursor2.current_char().unwrap());

        cursor.advance_many(5)?;
        assert_eq!('w', cursor.current_char().unwrap());
        cursor2.advance_many(5)?;
        assert_eq!(' ', cursor2.current_char().unwrap());

        Ok(())
    }

    #[test]
    fn can_obtain_str_between_2_cursors() -> FlatDataResult<()>
    {
        let reader = Box::new("Life in the model world".as_bytes());

        let mut cursor = CharCursor::open(5, 50, reader)?;
        let mut cursor2 = cursor.clone();

        assert_eq!("", &cursor.between(&cursor2).unwrap());
        assert_eq!("", &cursor2.between(&cursor).unwrap());

        cursor2.advance_many(4)?;
        assert_eq!("Life", &cursor.between(&cursor2).unwrap());
        assert_eq!("Life", &cursor2.between(&cursor).unwrap());

        cursor2.advance_many(4)?;
        assert_eq!("Life in ", &cursor.between(&cursor2).unwrap());
        assert_eq!("Life in ", &cursor2.between(&cursor).unwrap());

        cursor2.advance_many(9)?;
        assert_eq!("Life in the model", &cursor.between(&cursor2).unwrap());
        assert_eq!("Life in the model", &cursor2.between(&cursor).unwrap());

        cursor2.advance_many(20)?;
        assert_eq!("Life in the model world", &cursor.between(&cursor2).unwrap());
        assert_eq!("Life in the model world", &cursor2.between(&cursor).unwrap());

        cursor.advance_many(4)?;
        assert_eq!(" in the model world", &cursor2.between(&cursor).unwrap());
        assert_eq!(" in the model world", &cursor.between(&cursor2).unwrap());

        cursor.advance_many(3)?;
        assert_eq!(" the model world", &cursor2.between(&cursor).unwrap());
        assert_eq!(" the model world", &cursor.between(&cursor2).unwrap());

        cursor.advance_many(4)?;
        assert_eq!(" model world", &cursor2.between(&cursor).unwrap());
        assert_eq!(" model world", &cursor.between(&cursor2).unwrap());

        cursor.advance_many(7)?;
        assert_eq!("world", &cursor2.between(&cursor).unwrap());
        assert_eq!("world", &cursor.between(&cursor2).unwrap());

        cursor.advance_many(20)?;
        assert_eq!("", &cursor2.between(&cursor).unwrap());
        assert_eq!("", &cursor.between(&cursor2).unwrap());

        Ok(())
    }

    #[test]
    fn a_dropped_cursor_does_not_hold_capacity() -> FlatDataResult<()>
    {
        let reader = Box::new("Life in the model world".as_bytes());

        let cursor = CharCursor::open(5, 15, reader)?;
        let mut cursor2 = cursor.clone();

        drop(cursor);

        cursor2.advance_many(15)?;
        assert_eq!('e', cursor2.current_char().unwrap());

        Ok(())
    }

    #[test]
    fn can_handle_all_utf_8_encoding_lengths() -> FlatDataResult<()>
    {
        let reader: Box<&[u8]> = Box::new(&[b'\x24', b'\xC2', b'\xA3', b'\xE2', b'\x82', b'\xAC', b'\xF0', b'\x90', b'\x8D', b'\x88']);

        let mut cursor = CharCursor::open(10, 20, reader)?;
        assert_eq!('$', cursor.current_char().unwrap());
        cursor.advance()?;
        assert_eq!('£', cursor.current_char().unwrap());
        cursor.advance()?;
        assert_eq!('€', cursor.current_char().unwrap());
        cursor.advance()?;
        assert_eq!('\u{10348}', cursor.current_char().unwrap());
        cursor.advance()?;
        assert!(cursor.is_end_of_data());
        assert!(cursor.current_char().is_none());

        Ok(())
    }

    #[test]
    fn the_bom_is_ignored_if_present_at_start_of_data() -> FlatDataResult<()>
    {
        let reader = Box::new("\u{FEFF}Life in the model world".as_bytes());

        let mut cursor = CharCursor::open(10, 20, reader)?;
        assert!(!cursor.is_end_of_data());
        assert_eq!('L', cursor.current_char().unwrap());
        // Advance to last char
        cursor.advance_many(22)?;
        assert_eq!('d', cursor.current_char().unwrap());
        Ok(())
    }
}
