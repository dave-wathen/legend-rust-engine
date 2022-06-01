// Copyright 2022 Dave Wathen. All rights reserved.

pub use crate::lines::delimited::DelimitedFormat;
pub use crate::lines::delimited::DelimitedFormatBuilder;
pub use crate::lines::delimited::DelimitedLine;
pub use crate::lines::delimited::DelimitedLineReader;
pub use crate::lines::simple::SimpleLine;
pub use crate::lines::simple::SimpleLineReader;
use crate::CharCursor;
use crate::FlatDataError;

type FlatDataResult<T> = Result<T, FlatDataError>;

enum MatchableSequence
{
    Char(char),
    MultiChar(Vec<char>),
}

impl MatchableSequence
{
    fn matches(&self, at: &CharCursor) -> FlatDataResult<bool>
    {
        match (at.current_char(), &self)
        {
            (None, _) => Ok(false),
            (Some(ch), MatchableSequence::Char(char)) => Ok(ch == *char),
            (Some(ch), MatchableSequence::MultiChar(chars)) =>
            {
                if ch == chars[0]
                {
                    let mut lookahead = at.clone();
                    for eol_char in chars.iter()
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
                    return Ok(true);
                }
                Ok(false)
            }
        }
    }

    fn consume(&self, cursor: &mut CharCursor) -> FlatDataResult<()>
    {
        match &self
        {
            MatchableSequence::Char(_) => cursor.advance()?,
            MatchableSequence::MultiChar(chars) => cursor.advance_many(chars.len() as u32)?,
        };
        Ok(())
    }
}

impl From<char> for MatchableSequence
{
    fn from(ch: char) -> MatchableSequence { MatchableSequence::Char(ch) }
}

impl From<&str> for MatchableSequence
{
    fn from(chars: &str) -> MatchableSequence
    {
        if chars.len() == 1
        {
            MatchableSequence::Char(chars.chars().next().unwrap())
        }
        else
        {
            MatchableSequence::MultiChar(chars.chars().collect())
        }
    }
}

pub struct EndOfLine(EndOfLineType);

enum EndOfLineType
{
    Smart,
    Exact(MatchableSequence),
}

impl EndOfLine
{
    pub fn smart() -> EndOfLine { EndOfLine(EndOfLineType::Smart) }

    fn is_end_of_line(&self, at: &CharCursor) -> FlatDataResult<bool>
    {
        match at.current_char()
        {
            None => Ok(true),
            Some(ch) => match &self.0
            {
                EndOfLineType::Smart => Ok(ch == '\n' || ch == '\r'),
                EndOfLineType::Exact(eol) => eol.matches(at),
            },
        }
    }

    // Returns Ok(true) if an EOL was consumed and Ok(false) if the end of the input was reached
    fn consume_end_of_line(&mut self, at: &mut CharCursor) -> FlatDataResult<bool>
    {
        match at.current_char()
        {
            Some(c1) =>
            {
                match &mut self.0
                {
                    EndOfLineType::Smart =>
                    {
                        at.advance()?;
                        if c1 == '\r' && at.current_char() == Some('\n')
                        {
                            at.advance()?;
                        }
                    }
                    EndOfLineType::Exact(eol) => eol.consume(at)?,
                };
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

impl From<char> for EndOfLine
{
    fn from(ch: char) -> EndOfLine { EndOfLine(EndOfLineType::Exact(MatchableSequence::from(ch))) }
}

impl From<&str> for EndOfLine
{
    fn from(chars: &str) -> EndOfLine { EndOfLine(EndOfLineType::Exact(MatchableSequence::from(chars))) }
}

struct LineState<'a>
{
    eol: EndOfLine,
    line_number: &'a mut Box<u64>,
    last_consumed_eol: bool,
}

impl<'a> LineState<'a>
{
    fn next_line_number(&mut self) -> u64
    {
        *self.line_number.as_mut() += 1;
        **self.line_number
    }

    fn is_end_of_line(&self, at: &CharCursor) -> FlatDataResult<bool> { self.eol.is_end_of_line(at) }

    fn consume_end_of_line(&mut self, at: &mut CharCursor) -> FlatDataResult<()>
    {
        self.last_consumed_eol = self.eol.consume_end_of_line(at)?;
        Ok(())
    }

    pub fn line_at_end<T, G>(&mut self, empty_line_generator: G) -> FlatDataResult<Option<T>>
    where
        G: FnOnce(u64) -> T,
    {
        if self.last_consumed_eol
        {
            self.last_consumed_eol = false;
            Ok(Some(empty_line_generator(self.next_line_number())))
        }
        else
        {
            Ok(None)
        }
    }
}

fn is_whitespace(at: &CharCursor) -> bool
{
    match at.current_char()
    {
        None => false,
        Some(ch) => ch.is_whitespace(),
    }
}

mod simple
{
    use super::EndOfLine;
    use super::LineState;
    use crate::CharCursor;
    use crate::FlatDataError;

    type FlatDataResult<T> = Result<T, FlatDataError>;
    pub struct SimpleLineReader<'a>
    {
        cursor: CharCursor,
        line_state: LineState<'a>,
    }

    pub struct SimpleLine
    {
        pub text: String,
        pub line_number: u64,
    }

    impl<'a> SimpleLineReader<'a>
    {
        pub fn new(cursor: CharCursor, eol: EndOfLine, line_number: &'a mut Box<u64>) -> SimpleLineReader<'a>
        {
            SimpleLineReader { cursor, line_state: LineState { eol, line_number, last_consumed_eol: false } }
        }

        pub fn read_line(&mut self) -> FlatDataResult<Option<SimpleLine>>
        {
            if self.cursor.is_end_of_data()
            {
                return self.line_state.line_at_end(|lno| SimpleLine { text: String::new(), line_number: lno });
            };

            let mut ahead = self.cursor.clone();
            while !self.line_state.is_end_of_line(&ahead)?
            {
                ahead.advance()?;
            }
            let line = self.cursor.between(&ahead)?;
            self.cursor.advance_to(&ahead)?;
            self.line_state.consume_end_of_line(&mut self.cursor)?;
            Ok(Some(SimpleLine { text: line, line_number: self.line_state.next_line_number() }))
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
            can_read_lines(Box::new("Line1\nLine2\nLine3".as_bytes()), EndOfLine::smart(), vec!["Line1", "Line2", "Line3"])
        }

        #[test]
        fn can_read_smart_lines_with_crs() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("Line1\rLine2\rLine3".as_bytes()), EndOfLine::smart(), vec!["Line1", "Line2", "Line3"])
        }

        #[test]
        fn can_read_smart_lines_with_crlfs() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("Line1\r\nLine2\r\nLine3".as_bytes()), EndOfLine::smart(), vec!["Line1", "Line2", "Line3"])
        }

        #[test]
        fn can_read_exact_lines() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("Line1\nLine2\rLine3".as_bytes()), EndOfLine::from('\n'), vec!["Line1", "Line2\rLine3"])
        }

        #[test]
        fn can_read_exact_multi_lines() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("Line1~@~Line2~@~Line~3".as_bytes()), EndOfLine::from("~@~"), vec!["Line1", "Line2", "Line~3"])
        }

        #[test]
        fn can_read_initial_blank_line() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("\nLine2\nLine3".as_bytes()), EndOfLine::smart(), vec!["", "Line2", "Line3"])
        }

        #[test]
        fn can_read_intermediate_blank_line() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("Line1\n\nLine3".as_bytes()), EndOfLine::smart(), vec!["Line1", "", "Line3"])
        }

        #[test]
        fn can_read_final_blank_line() -> FlatDataResult<()>
        {
            can_read_lines(Box::new("Line1\nLine2\n".as_bytes()), EndOfLine::smart(), vec!["Line1", "Line2", ""])
        }

        fn can_read_lines(reader: Box<dyn io::Read>, eol: EndOfLine, expected: Vec<&str>) -> FlatDataResult<()>
        {
            let cursor = CharCursor::open(10, 20, reader)?;
            let mut lno = Box::new(0);
            let mut lines = SimpleLineReader::new(cursor, eol, &mut lno);

            let mut expected_num = 1;
            for expected_line in expected.iter()
            {
                let line = lines.read_line()?.unwrap();
                assert_eq!(*expected_line, line.text);
                assert_eq!(expected_num, line.line_number);
                expected_num += 1;
            }
            assert!(lines.read_line()?.is_none());

            // Ensure borrowed line number has been updated
            assert_eq!(expected.len() as u64, *lno);

            Ok(())
        }
    }
}

mod delimited
{
    use super::is_whitespace;
    use super::EndOfLine;
    use super::LineState;
    use super::MatchableSequence;
    use crate::CharCursor;
    use crate::FlatDataError;

    type FlatDataResult<T> = Result<T, FlatDataError>;
    pub struct DelimitedLineReader<'a>
    {
        cursor: CharCursor,
        format: DelimitedFormat,
        line_state: LineState<'a>,
    }

    pub struct DelimitedFormat
    {
        delimiter: MatchableSequence,
        quote: Option<MatchableSequence>,
        escape: Option<MatchableSequence>,
    }

    impl DelimitedFormat
    {
        fn is_delimiter(&self, at: &CharCursor) -> FlatDataResult<bool> { self.delimiter.matches(at) }

        fn consume_delimiter(&self, at: &mut CharCursor) -> FlatDataResult<()> { self.delimiter.consume(at) }

        fn is_quote(&self, at: &CharCursor) -> FlatDataResult<bool>
        {
            match &self.quote
            {
                None => Ok(false),
                Some(quote) => quote.matches(at),
            }
        }

        fn consume_quote(&self, at: &mut CharCursor) -> FlatDataResult<()>
        {
            self.quote.as_ref().expect("Quote not defined so cannot be consumed").consume(at)
        }
    }

    pub struct DelimitedFormatBuilder(DelimitedFormat);

    impl DelimitedFormatBuilder
    {
        pub fn with_quote(mut self, ch: char) -> DelimitedFormatBuilder
        {
            self.0.quote = Some(MatchableSequence::from(ch));
            self
        }

        pub fn with_escape(mut self, ch: char) -> DelimitedFormatBuilder
        {
            self.0.escape = Some(MatchableSequence::from(ch));
            self
        }

        pub fn build(self) -> DelimitedFormat { self.0 }
    }

    impl From<char> for DelimitedFormatBuilder
    {
        fn from(delimiter: char) -> DelimitedFormatBuilder
        {
            DelimitedFormatBuilder(DelimitedFormat { delimiter: MatchableSequence::from(delimiter), quote: None, escape: None })
        }
    }

    impl From<&str> for DelimitedFormatBuilder
    {
        fn from(delimiter: &str) -> DelimitedFormatBuilder
        {
            DelimitedFormatBuilder(DelimitedFormat { delimiter: MatchableSequence::from(delimiter), quote: None, escape: None })
        }
    }

    pub struct DelimitedLine
    {
        pub text: String,
        pub line_number: u64,
        pub values: Vec<String>,
        // TODO Defects
    }

    struct DelimitedValues
    {
        current_value: Option<String>,
        values: Vec<String>,
    }

    impl DelimitedValues
    {
        fn finish_value(&mut self)
        {
            match self.current_value.take()
            {
                None => self.values.push(String::new()),
                Some(v) => self.values.push(v),
            };
            self.current_value = Some(String::new());
        }

        fn consume_to_current_value(&mut self, at: &mut CharCursor) -> FlatDataResult<()>
        {
            let ch = at.current_char().expect("Character expected otherwise should be end of line");
            self.current_value.get_or_insert_with(String::new).push(ch);
            at.advance()
        }
    }

    #[derive(Debug)]
    enum Stage
    {
        BeforeValue,
        WhitespaceStartOfValue,
        WhitespaceAfterQuoted,
        InUnquoted,
        InQuoted,
        AfterQuoted,
        Finished,
    }

    #[derive(Debug)]
    enum CharType
    {
        EndOfLine,
        Delimiter,
        Quote,
        Whitespace,
        Other,
    }

    impl<'a> DelimitedLineReader<'a>
    {
        pub fn new(cursor: CharCursor, eol: EndOfLine, format: DelimitedFormat, line_number: &'a mut Box<u64>) -> DelimitedLineReader<'a>
        {
            DelimitedLineReader { cursor, line_state: LineState { eol, line_number, last_consumed_eol: false }, format }
        }

        pub fn read_line(&mut self) -> FlatDataResult<Option<DelimitedLine>>
        {
            if self.cursor.is_end_of_data()
            {
                return self.line_state.line_at_end(|lno| DelimitedLine { text: String::new(), line_number: lno, values: vec![] });
            };

            let mut ahead = self.cursor.clone();
            let mut stage = Stage::BeforeValue;
            let mut state = DelimitedValues { current_value: None, values: vec![] };

            while !matches!(stage, Stage::Finished)
            {
                let ch_type = if self.is_end_of_line(&ahead)?
                {
                    CharType::EndOfLine
                }
                else if self.is_delimiter(&ahead)?
                {
                    CharType::Delimiter
                }
                else if self.is_quote(&ahead)?
                {
                    CharType::Quote
                }
                else if is_whitespace(&ahead)
                {
                    CharType::Whitespace
                }
                else
                {
                    CharType::Other
                };

                match (&stage, &ch_type)
                {
                    (Stage::BeforeValue, CharType::Quote) | (Stage::WhitespaceStartOfValue, CharType::Quote) =>
                    {
                        self.consume_quote(&mut ahead)?;
                        state.current_value = Some(String::new());
                        stage = Stage::InQuoted;
                    }
                    (Stage::BeforeValue, CharType::Whitespace) =>
                    {
                        state.consume_to_current_value(&mut ahead)?;
                        stage = Stage::WhitespaceStartOfValue;
                    }
                    (Stage::BeforeValue, CharType::Other) | (Stage::WhitespaceStartOfValue, CharType::Other) =>
                    {
                        state.consume_to_current_value(&mut ahead)?;
                        stage = Stage::InUnquoted;
                    }
                    (Stage::BeforeValue, CharType::EndOfLine) => stage = Stage::Finished,

                    (Stage::InQuoted, CharType::Quote) =>
                    {
                        self.consume_quote(&mut ahead)?;
                        stage = Stage::AfterQuoted;
                    }

                    (Stage::AfterQuoted, CharType::Whitespace) =>
                    {
                        ahead.advance()?;
                        stage = Stage::WhitespaceAfterQuoted
                    }
                    (Stage::AfterQuoted, CharType::Quote) =>
                    {
                        state.consume_to_current_value(&mut ahead)?;
                        stage = Stage::InQuoted
                    }

                    (Stage::WhitespaceAfterQuoted, CharType::Whitespace) => ahead.advance()?,

                    (Stage::InQuoted, _) | (Stage::InUnquoted, CharType::Quote) | (_, CharType::Whitespace) | (_, CharType::Other) =>
                    {
                        state.consume_to_current_value(&mut ahead)?;
                    }

                    (_, CharType::EndOfLine) =>
                    {
                        state.finish_value();
                        stage = Stage::Finished;
                    }
                    (_, CharType::Delimiter) =>
                    {
                        state.finish_value();
                        self.consume_delimiter(&mut ahead)?;
                        stage = Stage::BeforeValue;
                    }
                    (_, CharType::Quote) =>
                    {
                        self.consume_quote(&mut ahead)?;
                        state.current_value = Some(String::new());
                    }
                }
            }

            let line = self.cursor.between(&ahead)?;
            self.cursor.advance_to(&ahead)?;
            self.line_state.consume_end_of_line(&mut self.cursor)?;
            Ok(Some(DelimitedLine { text: line, line_number: self.line_state.next_line_number(), values: state.values }))
        }

        fn is_end_of_line(&self, at: &CharCursor) -> FlatDataResult<bool> { self.line_state.is_end_of_line(at) }

        fn is_delimiter(&self, at: &CharCursor) -> FlatDataResult<bool> { self.format.is_delimiter(at) }

        fn is_quote(&self, at: &CharCursor) -> FlatDataResult<bool> { self.format.is_quote(at) }

        fn consume_delimiter(&self, at: &mut CharCursor) -> FlatDataResult<()> { self.format.consume_delimiter(at) }

        fn consume_quote(&self, at: &mut CharCursor) -> FlatDataResult<()> { self.format.consume_quote(at) }
    }

    #[cfg(test)]
    mod test
    {
        use super::*;
        use std::io;

        struct ExpectedLine
        {
            text: &'static str,
            values: Vec<&'static str>,
        }

        #[test]
        fn can_read_simple_delimited_lines() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("A,B,C\nD,E,F\nGee,Aitch,I".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').build(),
                vec![
                    ExpectedLine { text: "A,B,C", values: vec!["A", "B", "C"] },
                    ExpectedLine { text: "D,E,F", values: vec!["D", "E", "F"] },
                    ExpectedLine { text: "Gee,Aitch,I", values: vec!["Gee", "Aitch", "I"] },
                ],
            )
        }

        #[test]
        fn can_read_final_blank_line() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("A,B,C\nD,E,F\n".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').build(),
                vec![
                    ExpectedLine { text: "A,B,C", values: vec!["A", "B", "C"] },
                    ExpectedLine { text: "D,E,F", values: vec!["D", "E", "F"] },
                    ExpectedLine { text: "", values: vec![] },
                ],
            )
        }

        #[test]
        fn can_read_with_mulicharacter_delimiter() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("A~!@B~!@C\nD~!@E~!@F\nGee~!@Aitch~!@I".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from("~!@").build(),
                vec![
                    ExpectedLine { text: "A~!@B~!@C", values: vec!["A", "B", "C"] },
                    ExpectedLine { text: "D~!@E~!@F", values: vec!["D", "E", "F"] },
                    ExpectedLine { text: "Gee~!@Aitch~!@I", values: vec!["Gee", "Aitch", "I"] },
                ],
            )
        }

        #[test]
        fn can_read_with_quoting() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("'Hello','The','World'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello','The','World'", values: vec!["Hello", "The", "World"] }],
            )
        }

        #[test]
        fn if_quotes_are_used_there_can_be_whitespace_around_delimiters() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("'Hello'\t , 'The' ,  'World'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello'\t , 'The' ,  'World'", values: vec!["Hello", "The", "World"] }],
            )
        }

        #[test]
        fn if_quotes_are_not_used_whitespace_around_delimiters_is_part_of_value() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("Hello\t, The ,  World".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "Hello\t, The ,  World", values: vec!["Hello\t", " The ", "  World"] }],
            )
        }

        #[test]
        fn if_quotes_are_not_used_and_only_whitespace_is_between_delimiters_the_whitespace_is_the_value() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("Hello,  ,World".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "Hello,  ,World", values: vec!["Hello", "  ", "World"] }],
            )
        }

        #[test]
        fn quote_inside_value_is_just_a_char() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("Hello 'World','and','Bye'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "Hello 'World','and','Bye'", values: vec!["Hello 'World'", "and", "Bye"] }],
            )
        }

        #[test]
        fn delimiter_in_quotes_is_part_of_value() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("'Hello, World','and','Bye'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello, World','and','Bye'", values: vec!["Hello, World", "and", "Bye"] }],
            )
        }

        #[test]
        fn eol_in_quotes_is_part_of_value() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("'Hello\r\nWorld','and','Bye'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello\r\nWorld','and','Bye'", values: vec!["Hello\r\nWorld", "and", "Bye"] }],
            )?;
            can_read_lines(
                Box::new("'Hello\nWorld','and','Bye'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello\nWorld','and','Bye'", values: vec!["Hello\nWorld", "and", "Bye"] }],
            )?;
            can_read_lines(
                Box::new("'Hello\rWorld','and','Bye'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello\rWorld','and','Bye'", values: vec!["Hello\rWorld", "and", "Bye"] }],
            )
        }

        #[test]
        fn two_quotes_is_an_escaped_quote() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("'''','X','Y'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'''','X','Y'", values: vec!["'", "X", "Y"] }],
            )?;
            can_read_lines(
                Box::new("'''''','X','Y'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'''''','X','Y'", values: vec!["''", "X", "Y"] }],
            )?;
            can_read_lines(
                Box::new("'Hello, ''World''','and','Bye'".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![ExpectedLine { text: "'Hello, ''World''','and','Bye'", values: vec!["Hello, 'World'", "and", "Bye"] }],
            )
        }

        #[test]
        fn can_include_blank_lines() -> FlatDataResult<()>
        {
            can_read_lines(
                Box::new("Hello,World\n\nGoodbye,World".as_bytes()),
                EndOfLine::smart(),
                DelimitedFormatBuilder::from(',').with_quote('\'').build(),
                vec![
                    ExpectedLine { text: "Hello,World", values: vec!["Hello", "World"] },
                    ExpectedLine { text: "", values: vec![] },
                    ExpectedLine { text: "Goodbye,World", values: vec!["Goodbye", "World"] },
                ],
            )
        }

        /* TODO Tests for defects. From Java version:
            @Test
            public void quotedFieldShouldBeFollowedByDelimiter() {
                runTestInvalid("Unexpected text after closing quote in value 1 at line 2", "'Hello' World,'Bye'");
            }

            @Test
            public void quotesShouldBeTerminated() {
                runTestInvalid("Unclosed quotes in value 2 at line 2", "'Hello','W");
            }
        */

        fn can_read_lines(reader: Box<dyn io::Read>, eol: EndOfLine, format: DelimitedFormat, expected: Vec<ExpectedLine>) -> FlatDataResult<()>
        {
            let cursor = CharCursor::open(100, 1000, reader)?;
            let mut lno = Box::new(0);
            let mut lines = DelimitedLineReader::new(cursor, eol, format, &mut lno);

            let mut expected_num = 1;
            for expected_line in expected.iter()
            {
                let line = lines.read_line()?.unwrap();
                assert_eq!(*expected_line.text, line.text);
                assert_eq!(*expected_line.values, line.values);
                assert_eq!(expected_num, line.line_number);
                expected_num += 1;
            }
            assert!(lines.read_line()?.is_none());

            // Ensure borrowed line number has been updated
            assert_eq!(expected.len() as u64, *lno);

            Ok(())
        }
    }
}
