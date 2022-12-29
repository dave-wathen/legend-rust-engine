// Copyright 2022 Dave Wathen. All rights reserved.

use std::{borrow::Cow, marker::PhantomData};

use regex_syntax::hir::{ClassBytes, ClassUnicode, Hir, Literal};
use thiserror::Error;

use crate::CursorError;

use super::{CharCursor, CharToken, Span};

pub type RegexResult<T> = Result<T, RegexError>;

macro_rules! display_as_debug_for {
    ($name: ident) => {
        impl std::fmt::Display for $name
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
        }
    };
}

macro_rules! id_type {
    ($name: ident) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        struct $name(usize);

        impl From<usize> for $name
        {
            fn from(v: usize) -> $name { $name(v) }
        }

        impl From<$name> for usize
        {
            fn from(v: $name) -> usize { v.0 }
        }

        display_as_debug_for!($name);
    };
}

struct IndexedCollection<ID, ELEMENT>
where
    ID: From<usize> + Into<usize>,
    ELEMENT: Eq,
{
    data: Vec<ELEMENT>,
    phantom: PhantomData<ID>,
}

impl<ID, ELEMENT> IndexedCollection<ID, ELEMENT>
where
    ID: From<usize> + Into<usize>,
    ELEMENT: Eq,
{
    fn new() -> Self { Self { data: vec![], phantom: PhantomData } }

    fn last_id(&self) -> ID { self.relative_id(0) }

    fn next_id(&self) -> ID { self.relative_id(1) }

    fn relative_id(&self, offset: usize) -> ID { (self.data.len() + offset - 1).into() }

    fn push(&mut self, e: ELEMENT) -> ID
    {
        self.data.push(e);
        self.last_id()
    }

    fn add_if_missing(&mut self, v: ELEMENT) -> ID
    {
        match self.data.iter().position(|c| *c == v)
        {
            Some(index) => ID::from(index),
            None =>
            {
                self.data.push(v);
                ID::from(self.data.len() - 1)
            }
        }
    }
}

impl<ID, ELEMENT> std::fmt::Debug for IndexedCollection<ID, ELEMENT>
where
    ID: From<usize> + Into<usize> + std::fmt::Debug,
    ELEMENT: Eq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_map().entries(self.data.iter().enumerate().map(|(k, v)| (ID::from(k), v))).finish()
    }
}

impl<ID, ELEMENT> std::ops::Index<ID> for IndexedCollection<ID, ELEMENT>
where
    ID: From<usize> + Into<usize>,
    ELEMENT: Eq,
{
    type Output = ELEMENT;

    fn index(&self, index: ID) -> &Self::Output { &self.data[index.into()] }
}

impl<ID, ELEMENT> std::ops::IndexMut<ID> for IndexedCollection<ID, ELEMENT>
where
    ID: From<usize> + Into<usize>,
    ELEMENT: Eq,
{
    fn index_mut(&mut self, index: ID) -> &mut Self::Output { &mut self.data[index.into()] }
}

#[derive(Debug)]
pub struct Regex
{
    classes: Classes,
    alternatives: Alternatives,
    states: States,
}

impl Regex
{
    pub fn new(re: &str) -> RegexResult<Regex>
    {
        let parsed = regex_syntax::Parser::new().parse(re).map_err(|e| RegexError::Syntax { error: format!("{}", e) })?;
        compile(&parsed)
    }

    pub fn matches<'a, C>(&self, cursor_in: &C) -> RegexResult<Option<Match<'a, C>>>
    where
        C: CharCursor<'a> + std::fmt::Debug,
    {
        let cursor = cursor_in.clone();
        let mut stack = vec![(self.states[StateId(0)], cursor)];

        while !stack.is_empty()
        {
            let (state, mut cursor) = stack.pop().unwrap();

            match state
            {
                State::Alternation(id) =>
                {
                    let options = &self.alternatives[id];
                    options.iter().rev().for_each(|o| stack.push((self.states[*o], cursor.clone())))
                }
                State::Char(expected, next) =>
                {
                    if let CharToken::Char(ch) = cursor.token()?
                    {
                        if ch == expected
                        {
                            cursor.advance()?;
                            stack.push((self.states[next], cursor));
                        }
                    }
                }
                State::Class(id, next) =>
                {
                    if let CharToken::Char(ch) = cursor.token()?
                    {
                        if self.classes[id].includes(ch)
                        {
                            cursor.advance()?;
                            stack.push((self.states[next], cursor));
                        }
                    }
                }
                State::NoOp(next) => stack.push((self.states[next], cursor)),
                State::Terminal => return Ok(Some(Match { start_cursor: cursor_in.clone(), matched_cursor: cursor, phantom: PhantomData })),
            }
        }
        Ok(None)
    }
}

pub struct Match<'a, C>
where
    C: CharCursor<'a>,
{
    start_cursor: C,
    matched_cursor: C,
    phantom: PhantomData<&'a C>,
}

impl<'a, C> Match<'a, C>
where
    C: CharCursor<'a>,
{
    pub fn matched(&self) -> RegexResult<Cow<'a, str>> { Ok(self.start_cursor.between(&self.matched_cursor)?) }

    pub fn matched_span(&self) -> RegexResult<Option<Span>> { Ok(self.start_cursor.span_between(&self.matched_cursor)?) }

    pub fn into_end(self) -> C { self.matched_cursor }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State
{
    Alternation(AlternativeId),
    Char(char, StateId),
    Class(ClassId, StateId),
    NoOp(StateId),
    Terminal,
}

impl std::fmt::Debug for State
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            State::Terminal => write!(f, "END:")?,
            State::Alternation(id) =>
            {
                write!(f, "ALTERNATIVES: {:?}", id)?;
            }
            State::Char(expected, next) =>
            {
                write!(f, "CHAR: {} if '{}'", next, format_char(*expected))?;
            }
            State::Class(id, next) =>
            {
                write!(f, "CLASS: {} if in {:?} ", next, id)?;
            }
            Self::NoOp(next) => write!(f, "NO_OP: {}", next)?,
        };

        Ok(())
    }
}

id_type!(StateId);

impl std::fmt::Debug for StateId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:04}", self.0) }
}

type States = IndexedCollection<StateId, State>;

#[derive(PartialEq, Eq)]
struct Class(Vec<CharRange>);

impl Class
{
    fn includes(&self, ch: char) -> bool { self.0.iter().any(|r| r.0 <= ch && ch <= r.1) }
}

impl From<&ClassUnicode> for Class
{
    fn from(value: &ClassUnicode) -> Self { Self(value.iter().map(|ur| CharRange(ur.start(), ur.end())).collect()) }
}

impl TryFrom<&ClassBytes> for Class
{
    type Error = RegexError;

    fn try_from(value: &ClassBytes) -> Result<Self, Self::Error>
    {
        if value.iter().any(|br| br.start() > 0x7f || br.end() > 0x7f)
        {
            return Err(RegexError::Unsupported { error: "Only unicode matching is supported (illegal range)" });
        }
        Ok(Self(value.iter().map(|br| CharRange(br.start() as char, br.end() as char)).collect()))
    }
}

impl std::fmt::Debug for Class
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        for (tr_index, tr) in self.0.iter().enumerate()
        {
            match tr_index
            {
                0 => write!(f, "{}", tr)?,
                1..=15 => write!(f, ", {}", tr)?,
                16 => write!(f, ", ... {} more", self.0.len() - 16)?,
                _ => (),
            }
        }
        Ok(())
    }
}

id_type!(ClassId);

impl std::fmt::Debug for ClassId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Class-{}", self.0) }
}

type Classes = IndexedCollection<ClassId, Class>;

type Alternative = Vec<StateId>;

id_type!(AlternativeId);

impl std::fmt::Debug for AlternativeId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Alt-{}", self.0) }
}

type Alternatives = IndexedCollection<AlternativeId, Alternative>;

#[derive(PartialEq, Eq)]
struct CharRange(char, char);

impl std::fmt::Debug for CharRange
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "[{}-{}]", format_char(self.0), format_char(self.1)) }
}

display_as_debug_for!(CharRange);

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
    let mut regex = Regex { classes: Classes::new(), alternatives: Alternatives::new(), states: States::new() };
    add_states(hir, &mut regex)?;
    regex.states.push(State::Terminal);
    Ok(regex)
}

fn add_states(hir: &Hir, regex: &mut Regex) -> RegexResult<()>
{
    match hir.kind()
    {
        regex_syntax::hir::HirKind::Empty =>
        {
            regex.states.push(State::NoOp(regex.states.relative_id(2)));
        }
        regex_syntax::hir::HirKind::Literal(lit) =>
        {
            let char = match *lit
            {
                Literal::Unicode(c) => c,
                Literal::Byte(b) if b <= 0x7F => b as char,
                Literal::Byte(_) => return Err(RegexError::Unsupported { error: "Only unicode matching is supported (illegal char)" }),
            };
            regex.states.push(State::Char(char, regex.states.relative_id(2)));
        }
        regex_syntax::hir::HirKind::Class(class) => match class
        {
            regex_syntax::hir::Class::Unicode(uc) =>
            {
                let id = regex.classes.add_if_missing(uc.into());
                regex.states.push(State::Class(id, regex.states.relative_id(2)));
            }
            regex_syntax::hir::Class::Bytes(cb) =>
            {
                let id = regex.classes.add_if_missing(cb.try_into()?);
                regex.states.push(State::Class(id, regex.states.relative_id(2)));
            }
        },
        regex_syntax::hir::HirKind::Anchor(_) =>
        {
            return Err(RegexError::Unsupported { error: "anchors not supported" });
        }
        regex_syntax::hir::HirKind::WordBoundary(_) =>
        {
            return Err(RegexError::Unsupported { error: "word boundaries not supported" });
        }
        regex_syntax::hir::HirKind::Repetition(repeat) =>
        {
            let (min, max) = match &repeat.kind
            {
                regex_syntax::hir::RepetitionKind::ZeroOrOne => (0, Some(1)),
                regex_syntax::hir::RepetitionKind::ZeroOrMore => (0, None),
                regex_syntax::hir::RepetitionKind::OneOrMore => (1, None),
                regex_syntax::hir::RepetitionKind::Range(range) => match range
                {
                    regex_syntax::hir::RepetitionRange::Exactly(n) => (*n as usize, Some(*n as usize)),
                    regex_syntax::hir::RepetitionRange::AtLeast(n) => (*n as usize, None),
                    regex_syntax::hir::RepetitionRange::Bounded(m, n) => (*m as usize, Some(*n as usize)),
                },
            };

            for _ in 0..min
            {
                add_states(repeat.hir.as_ref(), regex)?;
            }

            if let Some(mx) = max
            {
                for _ in (min + 1)..=mx
                {
                    let id = regex.alternatives.push(Vec::with_capacity(2));
                    regex.states.push(State::Alternation(id));

                    let next = regex.states.next_id();
                    add_states(repeat.hir.as_ref(), regex)?;
                    let skip = regex.states.next_id();

                    if repeat.greedy
                    {
                        regex.alternatives[id].push(next);
                        regex.alternatives[id].push(skip);
                    }
                    else
                    {
                        regex.alternatives[id].push(skip);
                        regex.alternatives[id].push(next);
                    }
                }
            }
            else
            {
                let id = regex.alternatives.push(Vec::with_capacity(2));
                let alt_id = regex.states.push(State::Alternation(id));

                let next = regex.states.next_id();
                add_states(repeat.hir.as_ref(), regex)?;
                regex.states.push(State::NoOp(alt_id));
                let skip = regex.states.next_id();

                if repeat.greedy
                {
                    regex.alternatives[id].push(next);
                    regex.alternatives[id].push(skip);
                }
                else
                {
                    regex.alternatives[id].push(skip);
                    regex.alternatives[id].push(next);
                }
            }
        }
        regex_syntax::hir::HirKind::Group(group) =>
        {
            add_states(&group.hir, regex)?;
        }
        regex_syntax::hir::HirKind::Concat(hirs) =>
        {
            for hir in hirs
            {
                add_states(hir, regex)?;
            }
        }
        regex_syntax::hir::HirKind::Alternation(hirs) =>
        {
            let mut ends = Vec::with_capacity(hirs.len());

            let id = regex.alternatives.push(Vec::with_capacity(hirs.len()));
            regex.states.push(State::Alternation(id));

            // Generate each alternative and note the last state of each
            for hir in hirs
            {
                regex.alternatives[id].push(regex.states.next_id());
                add_states(hir, regex)?;
                ends.push(regex.states.last_id());
            }

            // Adjust the ends to point beyond the alternation
            let next = regex.states.next_id();
            for end in ends.iter()
            {
                regex.states[*end] = adjust_state_transitions(&regex.states[*end], StateId(next.0));
            }
        }
    };
    Ok(())
}

fn adjust_state_transitions(state: &State, to: StateId) -> State
{
    match state
    {
        State::Char(expected, _) => State::Char(*expected, to),
        State::Class(class_id, _) => State::Class(*class_id, to),
        State::NoOp(_) => State::NoOp(to),
        _ => panic!("State should not be adjusted: {:?}", state),
    }
}

fn format_char(ch: char) -> String
{
    if ch <= 0x7f as char
    {
        format!("{}", ch.escape_debug())
    }
    else
    {
        format!("u{:x}", ch as i32)
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
        match_ok(&re, "a", "a")?;
        match_fails(&re, "xa")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn multichar_literal_match() -> RegexResult<()>
    {
        let re = Regex::new(r"abc")?;
        match_ok(&re, "abc", "abc")?;
        match_fails(&re, "xabc")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn alternative_match() -> RegexResult<()>
    {
        let re = Regex::new(r"a|b")?;
        match_ok(&re, "a", "a")?;
        match_ok(&re, "b", "b")?;
        match_fails(&re, "xa")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn custom_character_class_match() -> RegexResult<()>
    {
        let re = Regex::new(r"[abcxyz]")?;
        match_ok(&re, "a", "a")?;
        match_ok(&re, "b", "b")?;
        match_ok(&re, "c", "c")?;
        match_ok(&re, "x", "x")?;
        match_ok(&re, "y", "y")?;
        match_ok(&re, "z", "z")?;
        match_fails(&re, "m")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn alternative_causing_backtracking() -> RegexResult<()>
    {
        let re = Regex::new(r"aa|ab")?;
        match_ok(&re, "ab", "ab")?;
        Ok(())
    }

    #[test]
    fn greedy_star_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a*")?;
        match_ok(&re, "a", "a")?;
        match_ok(&re, "aa", "aa")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaaaaab", "aaaaaaaaaaaaaaaaaaaaa")?;
        match_ok(&re, "x", "")?;
        match_ok(&re, "", "")?;

        let re = Regex::new(r"a*aaaaa")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaaaaa", "aaaaaaaaaaaaaaaaaaaaa")?;
        Ok(())
    }

    #[test]
    fn lazy_star_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a*?b")?;
        match_ok(&re, "ab", "ab")?;
        match_ok(&re, "aaaaab", "aaaaab")?;
        match_ok(&re, "b", "b")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn greedy_plus_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a+")?;
        match_ok(&re, "a", "a")?;
        match_ok(&re, "aa", "aa")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaaaaa", "aaaaaaaaaaaaaaaaaaaaa")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;

        let re = Regex::new(r"a+aaaaa")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaaaaa", "aaaaaaaaaaaaaaaaaaaaa")?;
        match_ok(&re, "aaaaaa", "aaaaaa")?;
        match_fails(&re, "aaaaa")?;
        Ok(())
    }

    #[test]
    fn lazy_plus_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a+?")?;
        match_ok(&re, "aa", "a")?;

        let re = Regex::new(r"a+?b")?;
        match_ok(&re, "ab", "ab")?;
        match_ok(&re, "aaaaab", "aaaaab")?;
        match_fails(&re, "b")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn greedy_optional() -> RegexResult<()>
    {
        let re = Regex::new(r"a?b")?;
        match_ok(&re, "ab", "ab")?;
        match_ok(&re, "b", "b")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn lazy_optional() -> RegexResult<()>
    {
        let re = Regex::new(r"a??")?;
        match_ok(&re, "a", "")?;

        let re = Regex::new(r"a??b")?;
        match_ok(&re, "ab", "ab")?;
        match_ok(&re, "b", "b")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn exact_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a{3}b")?;
        match_ok(&re, "aaab", "aaab")?;
        match_fails(&re, "aaaab")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn at_least_greedy_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a{3,}b")?;
        match_ok(&re, "aaab", "aaab")?;
        match_ok(&re, "aaaaaaab", "aaaaaaab")?;
        match_fails(&re, "aaa")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn at_least_lazy_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a{3,}?a")?;
        match_ok(&re, "aaaaaa", "aaaa")?;

        let re = Regex::new(r"a{3,}?b")?;
        match_ok(&re, "aaab", "aaab")?;
        match_ok(&re, "aaaaaaab", "aaaaaaab")?;
        match_fails(&re, "aaa")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn bounded_greedy_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a{3,6}b")?;
        match_ok(&re, "aaab", "aaab")?;
        match_ok(&re, "aaaab", "aaaab")?;
        match_ok(&re, "aaaaab", "aaaaab")?;
        match_ok(&re, "aaaaaab", "aaaaaab")?;
        match_fails(&re, "aaaaaaab")?;
        match_fails(&re, "aaa")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn bounded_lazy_repetition() -> RegexResult<()>
    {
        let re = Regex::new(r"a{3,6}?b")?;
        match_ok(&re, "aaab", "aaab")?;
        match_ok(&re, "aaaab", "aaaab")?;
        match_ok(&re, "aaaaab", "aaaaab")?;
        match_ok(&re, "aaaaaab", "aaaaaab")?;
        match_fails(&re, "aaaaaaab")?;
        match_fails(&re, "aaa")?;
        match_fails(&re, "x")?;
        match_fails(&re, "")?;
        Ok(())
    }

    #[test]
    fn combined_repetitions_1() -> RegexResult<()>
    {
        let re = Regex::new(r"a*b+")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaaaab", "aaaaaaaaaaaaaaaaaaaab")?;
        match_ok(&re, "bbbbb", "bbbbb")?;
        match_ok(&re, "aaaabbbbb", "aaaabbbbb")?;
        match_fails(&re, "aaaa")?;
        Ok(())
    }

    #[test]
    fn combined_repetitions_2() -> RegexResult<()>
    {
        let re = Regex::new(r"(a|b)*b{3,6}")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaabbb", "aaaaaaaaaaaaaaaaaabbb")?;
        match_ok(&re, "bbbbbbbbbbbbabbb", "bbbbbbbbbbbbabbb")?;
        match_ok(&re, "bbbbbbbbbbbbabbbbbb", "bbbbbbbbbbbbabbbbbb")?;
        match_ok(&re, "bbb", "bbb")?;
        match_fails(&re, "aabbaa")?;
        match_fails(&re, "aaaaaaaaaaaaaaaaaabb")?;
        Ok(())
    }

    #[test]
    fn nested_repetitions() -> RegexResult<()>
    {
        let re = Regex::new(r"(((a|b)*c){3,6}d){2}")?;
        match_ok(&re, "aaaaaaaaaaaaaaaaaabbbcabacacdcccccd", "aaaaaaaaaaaaaaaaaabbbcabacacdcccccd")?;
        Ok(())
    }

    fn match_ok(re: &Regex, data: &str, expected: &str) -> RegexResult<()>
    {
        let bytes = ByteArrayCursor::new(data.as_bytes());
        let cursor = Utf8CharCursor::new(bytes, crate::char::LineEndings::Smart);
        let maybe_match = re.matches(&cursor)?;
        assert!(maybe_match.is_some());

        let re_match = maybe_match.unwrap();
        let expected_len = expected.chars().count();

        if expected.is_empty()
        {
            assert_eq!(None, re_match.matched_span()?);
        }
        else if expected_len == 1
        {
            assert_eq!("[1:1]", format!("{}", re_match.matched_span()?.unwrap()));
        }
        else
        {
            assert_eq!(format!("[1:1-{}]", expected_len), format!("{}", re_match.matched_span()?.unwrap()));
        }

        assert_eq!(expected, format!("{}", re_match.matched()?));
        let end_cursor = re_match.into_end();
        assert_eq!(expected, cursor.between(&end_cursor)?);

        Ok(())
    }

    fn match_fails(re: &Regex, data: &str) -> RegexResult<()>
    {
        let bytes = ByteArrayCursor::new(data.as_bytes());
        let cursor = Utf8CharCursor::new(bytes, crate::char::LineEndings::Smart);
        assert!(re.matches(&cursor)?.is_none());
        Ok(())
    }
}
