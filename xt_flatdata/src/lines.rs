// Copyright 2022 Dave Wathen. All rights reserved.

use crate::CharCursor;
use crate::FlatDataError;

type FlatDataResult<T> = Result<T, FlatDataError>;

pub struct SimpleLineReader
{
    cursor: CharCursor,
    eol: EndOfLine,
    line_number_generator: Box<dyn Iterator<Item = u64>>,
    last_consumed_eol: bool,
}

pub struct SimpleLine
{
    text: String,
    line_number: u64,
}

pub enum EndOfLine
{
    Smart,
    Exact(char),
    ExactMulti(Vec<char>),
}

impl SimpleLineReader
{
    pub fn new(cursor: CharCursor, eol: EndOfLine, line_number_generator: Box<dyn Iterator<Item = u64>>) -> SimpleLineReader
    {
        SimpleLineReader {
            cursor,
            eol,
            line_number_generator,
            last_consumed_eol: false,
        }
    }

    pub fn read_line(&mut self) -> FlatDataResult<Option<SimpleLine>>
    {
        if self.cursor.is_end_of_data()
        {
            if self.last_consumed_eol
            {
                self.last_consumed_eol = false;
                return Ok(Some(SimpleLine {
                    text: String::new(),
                    line_number: self.line_number_generator.next().unwrap(),
                }));
            }
            else
            {
                return Ok(None);
            }
        }

        let mut ahead = self.cursor.clone();
        while !self.is_end_of_line(&ahead)?
        {
            ahead.advance()?;
        }
        let line = self.cursor.between(&ahead)?;
        self.cursor.advance_to(&ahead)?;
        self.consume_end_of_line()?;
        Ok(Some(SimpleLine {
            text: line,
            line_number: self.line_number_generator.next().unwrap(),
        }))
    }

    fn is_end_of_line(&self, at: &CharCursor) -> FlatDataResult<bool>
    {
        match at.current_char()
        {
            None => Ok(true),
            Some(ch) => match &self.eol
            {
                EndOfLine::Smart => Ok(ch == '\n' || ch == '\r'),
                EndOfLine::Exact(eol_char) => Ok(ch == *eol_char),
                EndOfLine::ExactMulti(eol_chars) =>
                {
                    if ch == eol_chars[0]
                    {
                        let mut lookahead = at.clone();
                        for eol_char in eol_chars.iter()
                        {
                            let matches = match lookahead.current_char()
                            {
                                Some(ch) => ch == *eol_char,
                                None => false,
                            };
                            if matches
                            {
                                lookahead.advance()?
                            }
                            else
                            {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    else
                    {
                        Ok(false)
                    }
                }
            },
        }
    }

    fn consume_end_of_line(&mut self) -> FlatDataResult<()>
    {
        match self.cursor.current_char()
        {
            Some(c1) =>
            {
                match &self.eol
                {
                    EndOfLine::Smart =>
                    {
                        self.cursor.advance()?;
                        if c1 == '\r' && self.cursor.current_char() == Some('\n')
                        {
                            self.cursor.advance()?;
                        }
                    }
                    EndOfLine::Exact(_) => self.cursor.advance()?,
                    EndOfLine::ExactMulti(eol_chars) => self.cursor.advance_many(eol_chars.len() as u32)?,
                };
                self.last_consumed_eol = true;
            }
            None => self.last_consumed_eol = false,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use std::io;

    #[test]
    fn can_read_smart_lines_with_lfs() -> FlatDataResult<()>
    {
        can_read_lines(
            Box::new("Line1\nLine2\nLine3".as_bytes()),
            EndOfLine::Smart,
            vec!["Line1", "Line2", "Line3"],
        )
    }

    #[test]
    fn can_read_smart_lines_with_crs() -> FlatDataResult<()>
    {
        can_read_lines(
            Box::new("Line1\rLine2\rLine3".as_bytes()),
            EndOfLine::Smart,
            vec!["Line1", "Line2", "Line3"],
        )
    }

    #[test]
    fn can_read_smart_lines_with_crlfs() -> FlatDataResult<()>
    {
        can_read_lines(
            Box::new("Line1\r\nLine2\r\nLine3".as_bytes()),
            EndOfLine::Smart,
            vec!["Line1", "Line2", "Line3"],
        )
    }

    #[test]
    fn can_read_exact_lines() -> FlatDataResult<()>
    {
        can_read_lines(
            Box::new("Line1\nLine2\rLine3".as_bytes()),
            EndOfLine::Exact('\n'),
            vec!["Line1", "Line2\rLine3"],
        )
    }

    #[test]
    fn can_read_exact_multi_lines() -> FlatDataResult<()>
    {
        can_read_lines(
            Box::new("Line1~@~Line2~@~Line~3".as_bytes()),
            EndOfLine::ExactMulti(vec!['~', '@', '~']),
            vec!["Line1", "Line2", "Line~3"],
        )
    }

    #[test]
    fn can_read_initial_blank_line() -> FlatDataResult<()>
    {
        can_read_lines(Box::new("\nLine2\nLine3".as_bytes()), EndOfLine::Smart, vec!["", "Line2", "Line3"])
    }

    #[test]
    fn can_read_intermediate_blank_line() -> FlatDataResult<()>
    {
        can_read_lines(Box::new("Line1\n\nLine3".as_bytes()), EndOfLine::Smart, vec!["Line1", "", "Line3"])
    }

    #[test]
    fn can_read_final_blank_line() -> FlatDataResult<()>
    {
        can_read_lines(Box::new("Line1\nLine2\n".as_bytes()), EndOfLine::Smart, vec!["Line1", "Line2", ""])
    }

    fn can_read_lines(reader: Box<dyn io::Read>, eol: EndOfLine, expected: Vec<&str>) -> FlatDataResult<()>
    {
        let cursor = CharCursor::open(10, 20, reader)?;
        let mut lines = SimpleLineReader::new(cursor, eol, Box::new(1..));

        let mut expected_num = 1;
        for expected_line in expected.iter()
        {
            let line = lines.read_line()?.unwrap();
            assert_eq!(*expected_line, line.text);
            assert_eq!(expected_num, line.line_number);
            expected_num += 1;
        }
        assert!(lines.read_line()?.is_none());

        Ok(())
    }
}
