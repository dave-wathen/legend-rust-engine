// Copyright 2022 Dave Wathen. All rights reserved.

use regex_syntax::hir::{Hir, Literal};
use thiserror::Error;

use crate::CursorError;

use super::{CharCursor, CharToken};

pub type RegexResult<T> = Result<T, RegexError>;

pub struct Regex
{
    states: Vec<State>,
}

type CharRange = (char, char);

#[derive(Debug)]
enum State
{
    Alternation(Vec<usize>),
    Char(char, usize),
    Class(Vec<CharRange>, usize),
    Terminal,
}

impl Regex
{
    pub fn new(re: &str) -> RegexResult<Regex>
    {
        let parsed = regex_syntax::Parser::new().parse(re).map_err(|e| RegexError::Syntax { error: format!("{}", e) })?;
        dbg!(&parsed);
        compile(&parsed)
    }

    pub fn matches<'a, C>(&self, cursor: &C) -> RegexResult<bool>
    where
        C: CharCursor<'a> + std::fmt::Debug,
    {
        let cursor = cursor.clone();
        let mut stack = vec![(&self.states[0], cursor)];

        while !stack.is_empty()
        {
            let (state, mut cursor) = stack.pop().unwrap();
            dbg!(state);
            dbg!(&cursor);

            match state
            {
                State::Alternation(options) => options.iter().rev().for_each(|o| stack.push((&self.states[*o], cursor.clone()))),
                State::Char(expected, next) =>
                {
                    if let CharToken::Char(ch) = cursor.token()?
                    {
                        if ch == *expected
                        {
                            cursor.advance()?;
                            stack.push((&self.states[*next], cursor));
                        }
                    }
                }
                State::Class(ranges, next) =>
                {
                    if let CharToken::Char(ch) = cursor.token()?
                    {
                        if ranges.iter().any(|r| r.0 <= ch && ch <= r.1)
                        {
                            cursor.advance()?;
                            stack.push((&self.states[*next], cursor));
                        }
                    }
                }
                State::Terminal => return Ok(true),
            }
        }
        Ok(false)
    }
}

impl std::fmt::Debug for Regex
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        fn format_char(ch: char) -> String
        {
            if ch <= 0x7f as char
            {
                format!("'{}'", ch.escape_debug())
            }
            else
            {
                format!("u{:x}", ch as i32)
            }
        }

        writeln!(f)?;
        writeln!(f, "Regex(")?;
        for (state_index, state) in self.states.iter().enumerate()
        {
            match state
            {
                State::Terminal => writeln!(f, "    {:04} END:", state_index)?,
                State::Alternation(options) =>
                {
                    writeln!(f, "    {:04} ALT: -> {:04}", state_index, options[0])?;
                    for o in &options[1..]
                    {
                        writeln!(f, "            : -> {:04}", o)?;
                    }
                }
                State::Char(expected, next) =>
                {
                    writeln!(f, "    {:04} CHR: -> {:04} if {}", state_index, next, format_char(*expected))?;
                }
                State::Class(ranges, next) =>
                {
                    write!(f, "    {:04} CLS: -> {:04} if in ", state_index, next)?;
                    for (tr_index, tr) in ranges.iter().enumerate()
                    {
                        if tr_index > 0 && tr_index % 8 == 0
                        {
                            writeln!(f)?;
                            write!(f, "                            ")?;
                        }
                        let range_text = format!("{}-{}", format_char(tr.0), format_char(tr.1));
                        write!(f, "{:18}", range_text)?;
                    }

                    writeln!(f)?
                }
            };
        }
        writeln!(f, ")")?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum RegexError
{
    #[error("CursorError {0}")]
    Cursor(#[from] CursorError),
    #[error("Unsupported] {error}")]
    Unsupported
    {
        error: &'static str
    },
    #[error("Syntax {error}")]
    Syntax
    {
        error: String
    },
}

fn compile(hir: &Hir) -> RegexResult<Regex>
{
    let mut regex = Regex { states: vec![] };
    add_states(hir, &mut regex)?;
    regex.states.push(State::Terminal);
    dbg!(&regex);
    Ok(regex)
}

fn add_states(hir: &Hir, regex: &mut Regex) -> RegexResult<()>
{
    match hir.kind()
    {
        regex_syntax::hir::HirKind::Empty => todo!(),
        regex_syntax::hir::HirKind::Literal(lit) =>
        {
            let char = match *lit
            {
                Literal::Unicode(c) => c,
                Literal::Byte(b) if b <= 0x7F => b as char,
                Literal::Byte(_) => return Err(RegexError::Unsupported { error: "Only unicode matching is supported (illegal char)" }),
            };
            regex.states.push(State::Char(char, regex.states.len() + 1));
        }
        regex_syntax::hir::HirKind::Class(class) => match class
        {
            regex_syntax::hir::Class::Unicode(uc) =>
            {
                let ranges: Vec<CharRange> = uc.iter().map(|ur| (ur.start(), ur.end())).collect();
                regex.states.push(State::Class(ranges, regex.states.len() + 1));
            }
            regex_syntax::hir::Class::Bytes(cb) =>
            {
                if cb.iter().any(|br| br.start() > 0x7f || br.end() > 0x7f)
                {
                    return Err(RegexError::Unsupported { error: "Only unicode matching is supported (illegal range)" });
                }
                let ranges: Vec<CharRange> = cb.iter().map(|br| (br.start() as char, br.end() as char)).collect();
                regex.states.push(State::Class(ranges, regex.states.len() + 1));
            }
        },
        regex_syntax::hir::HirKind::Anchor(_) => todo!(),
        regex_syntax::hir::HirKind::WordBoundary(_) => todo!(),
        regex_syntax::hir::HirKind::Repetition(_) => todo!(),
        regex_syntax::hir::HirKind::Group(_) => todo!(),
        regex_syntax::hir::HirKind::Concat(hirs) =>
        {
            for hir in hirs
            {
                add_states(hir, regex)?;
            }
        }
        regex_syntax::hir::HirKind::Alternation(hirs) =>
        {
            // Insert placeholder for alternation transitions
            let root_placeholder = regex.states.len();
            regex.states.push(State::Terminal);
            let mut root_trans = Vec::with_capacity(hirs.len());

            // Generate each alternative and note the last state of each
            let mut ends = Vec::with_capacity(hirs.len());
            for hir in hirs
            {
                root_trans.push(regex.states.len());
                add_states(hir, regex)?;
                ends.push(regex.states.len() - 1);
            }

            // Adjust the ends to point beyond the alternation
            let next = regex.states.len();
            for end in ends.iter_mut()
            {
                regex.states[*end] = adjust_state_transitions(&regex.states[*end], next);
            }

            // Replace the paceholder
            regex.states[root_placeholder] = State::Alternation(root_trans);
        }
    };
    Ok(())
}

fn adjust_state_transitions(state: &State, to: usize) -> State
{
    match state
    {
        State::Terminal => panic!("Terminal should not be adjusted"),
        State::Alternation(_) => panic!("Alternation should not be adjusted"),
        State::Char(expected, _) => State::Char(*expected, to),
        State::Class(ranges, _) => State::Class(ranges.clone(), to),
    }
}

#[cfg(test)]
mod tests
{
    use crate::{byte::ByteArrayCursor, char::Utf8CharCursor};

    use super::*;

    #[test]
    fn bad_pattern_fails_to_create() -> RegexResult<()>
    {
        assert!(Regex::new(r"a[bc").is_err());
        Ok(())
    }

    #[test]
    fn single_literal_match() -> RegexResult<()>
    {
        let re = Regex::new(r"a")?;

        let bytes = ByteArrayCursor::new(b"xa");
        let mut cursor = Utf8CharCursor::new(bytes);
        // xa
        // ^
        assert!(!re.matches(&cursor)?);

        cursor.advance()?;
        // xa
        //  ^
        assert!(re.matches(&cursor)?);

        cursor.advance()?;
        // xa
        //   ^
        assert!(!re.matches(&cursor)?);

        Ok(())
    }

    #[test]
    fn multichar_literal_match() -> RegexResult<()>
    {
        let re = Regex::new(r"abc")?;

        let bytes = ByteArrayCursor::new(b"xabc");
        let mut cursor = Utf8CharCursor::new(bytes);
        // xabc
        // ^
        assert!(!re.matches(&cursor)?);

        cursor.advance()?;
        // xabc
        //  ^
        assert!(re.matches(&cursor)?);

        cursor.advance_many(3)?;
        // xabc
        //     ^
        assert!(!re.matches(&cursor)?);

        Ok(())
    }

    #[test]
    fn alternative_match() -> RegexResult<()>
    {
        let re = Regex::new(r"a|b")?;

        let bytes = ByteArrayCursor::new(b"xab");
        let mut cursor = Utf8CharCursor::new(bytes);
        // xab
        // ^
        assert!(!re.matches(&cursor)?);

        cursor.advance()?;
        // xab
        //  ^
        assert!(re.matches(&cursor)?);

        cursor.advance()?;
        // xab
        //   ^
        assert!(re.matches(&cursor)?);

        cursor.advance()?;
        // xab
        //    ^
        assert!(!re.matches(&cursor)?);

        Ok(())
    }

    #[test]
    fn custom_character_class_match() -> RegexResult<()>
    {
        let re = Regex::new(r"[abcxyz]")?;

        let bytes = ByteArrayCursor::new(b"-aby");
        let mut cursor = Utf8CharCursor::new(bytes);
        // -aby
        // ^
        assert!(!re.matches(&cursor)?);

        cursor.advance()?;
        // -aby
        //  ^
        assert!(re.matches(&cursor)?);

        cursor.advance()?;
        // -aby
        //   ^
        assert!(re.matches(&cursor)?);

        cursor.advance()?;
        // -aby
        //    ^
        assert!(re.matches(&cursor)?);

        cursor.advance()?;
        // -aby
        //     ^
        assert!(!re.matches(&cursor)?);

        Ok(())
    }

    #[test]
    fn alternative_causing_backtracking() -> RegexResult<()>
    {
        let re = Regex::new(r"aa|ab")?;

        let bytes = ByteArrayCursor::new(b"ab");
        let cursor = Utf8CharCursor::new(bytes);
        // ab
        // ^
        assert!(re.matches(&cursor)?);

        Ok(())
    }
}
