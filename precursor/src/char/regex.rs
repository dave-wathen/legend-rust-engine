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
                //                ch.escape_unicode().to_string()
            }
        }

        writeln!(f)?;
        writeln!(f, "Regex(")?;
        for (state_index, state) in self.states.iter().enumerate()
        {
            match state
            {
                State::Terminal => writeln!(f, "    {:04}:         end", state_index)?,
                _ =>
                {
                    let mut first = true;
                    for t in state
                    {
                        let id = if first { format!("{:04}", state_index) } else { "    ".to_owned() };

                        match t
                        {
                            Transition::Char(ch, next) => writeln!(f, "    {}: -> {:04} if '{}'", id, next, format_char(*ch))?,
                            Transition::Class(ranges, next) =>
                            {
                                write!(f, "    {}: -> {:04} if in ", id, next)?;
                                for (tr_index, tr) in ranges.iter().enumerate()
                                {
                                    if tr_index > 0 && tr_index % 8 == 0
                                    {
                                        writeln!(f)?;
                                        write!(f, "                        ")?;
                                    }
                                    let range_text = format!("{}-{}", format_char(tr.0), format_char(tr.1));
                                    write!(f, "{:18}", range_text)?;
                                }

                                writeln!(f)?
                            }
                            Transition::Epsilon(next) => writeln!(f, "    {}: -> {:04} always", id, next)?,
                        };

                        first = false;
                    }
                }
            };
        }
        writeln!(f, ")")?;
        Ok(())
    }
}

#[derive(Debug)]
enum State
{
    Terminal,
    OneTransition(Transition),
    MultiTransition(Vec<Transition>),
}

impl State
{
    fn is_terminal(&self) -> bool { matches!(self, State::Terminal) }
}

impl<'a> std::iter::IntoIterator for &'a State
{
    type Item = &'a Transition;
    type IntoIter = StateIter<'a>;

    fn into_iter(self) -> Self::IntoIter
    {
        match &self
        {
            State::Terminal => StateIter::None,
            State::OneTransition(one) => StateIter::Single(std::iter::once(one)),
            State::MultiTransition(many) => StateIter::Many(many.iter()),
        }
    }
}

pub enum StateIter<'a>
{
    None,
    Single(std::iter::Once<&'a Transition>),
    Many(std::slice::Iter<'a, Transition>),
}

impl<'a> Iterator for StateIter<'a>
{
    type Item = &'a Transition;

    fn next(&mut self) -> Option<Self::Item>
    {
        match self
        {
            StateIter::None => None,
            StateIter::Single(iter) => iter.next(),
            StateIter::Many(iter) => iter.next(),
        }
    }
}

type CharRange = (char, char);

#[derive(Debug)]
pub enum Transition
{
    Char(char, usize),
    Class(Vec<CharRange>, usize),
    Epsilon(usize),
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
        C: CharCursor<'a>,
    {
        let mut stack = vec![&self.states[0]];
        let mut cursor = cursor.clone();

        while !stack.is_empty()
        {
            let state = stack.pop().unwrap();

            if state.is_terminal()
            {
                return Ok(true);
            }

            for t in state
            {
                match t
                {
                    Transition::Char(expected, next) =>
                    {
                        if let CharToken::Char(ch) = cursor.token()?
                        {
                            if ch == *expected
                            {
                                stack.push(&self.states[*next]);
                                cursor.advance()?;
                            }
                        }
                    }
                    Transition::Class(ranges, next) =>
                    {
                        if let CharToken::Char(ch) = cursor.token()?
                        {
                            if ranges.iter().any(|r| r.0 <= ch && ch <= r.1)
                            {
                                stack.push(&self.states[*next]);
                                cursor.advance()?;
                            }
                        }
                    }
                    Transition::Epsilon(next) => stack.push(&self.states[*next]),
                };
            }
        }
        Ok(false)
    }
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
            regex.states.push(State::OneTransition(Transition::Char(char, regex.states.len() + 1)));
        }
        regex_syntax::hir::HirKind::Class(class) => match class
        {
            regex_syntax::hir::Class::Unicode(uc) =>
            {
                let ranges: Vec<CharRange> = uc.iter().map(|ur| (ur.start(), ur.end())).collect();
                regex.states.push(State::OneTransition(Transition::Class(ranges, regex.states.len() + 1)));
            }
            regex_syntax::hir::Class::Bytes(cb) =>
            {
                if cb.iter().any(|br| br.start() > 0x7f || br.end() > 0x7f)
                {
                    return Err(RegexError::Unsupported { error: "Only unicode matching is supported (illegal range)" });
                }
                let ranges: Vec<CharRange> = cb.iter().map(|br| (br.start() as char, br.end() as char)).collect();
                regex.states.push(State::OneTransition(Transition::Class(ranges, regex.states.len() + 1)));
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
                root_trans.push(Transition::Epsilon(regex.states.len()));
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
            regex.states[root_placeholder] = State::MultiTransition(root_trans);
        }
    };
    Ok(())
}

fn adjust_state_transitions(state: &State, to: usize) -> State
{
    let map_transition = |t: &Transition| match t
    {
        Transition::Char(char, _) => Transition::Char(*char, to),
        Transition::Class(ranges, _) => Transition::Class(ranges.clone(), to),
        Transition::Epsilon(_) => Transition::Epsilon(to),
    };

    match state
    {
        State::Terminal => State::Terminal,
        State::OneTransition(t) => State::OneTransition(map_transition(t)),
        State::MultiTransition(ts) => State::MultiTransition(ts.iter().map(map_transition).collect()),
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
}
