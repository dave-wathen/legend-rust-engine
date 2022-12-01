// Copyright 2022 Dave Wathen. All rights reserved.

use std::io;

use thiserror::Error;

pub mod byte;
pub mod char;

pub type CursorResult<T> = Result<T, CursorError>;

#[derive(Error, Debug)]
pub enum CursorError
{
    #[error("Cannot advance: cursor is at end of data")]
    CannotAdvance,
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
    #[error("The supplied cursors is not compatible (e.g. a child of) this cursor")]
    Incompatible,
    #[error("IO Error")]
    IOError(#[from] io::Error),
}
