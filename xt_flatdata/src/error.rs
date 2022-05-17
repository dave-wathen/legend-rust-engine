// Copyright 2022 Dave Wathen. All rights reserved.

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlatDataError
{
    #[error("Cannot advance when cursor is at end of data")]
    CannotAdvanceBeyondEndOfData,
    #[error("Byte index not available: {0}")]
    ByteIndexUnavailable(u64),
    #[error("Buffer capacity ({0}) has been fully used.  This is often caused by the wrong line ending being specified and hence trying to fit all data in memory at once.")]
    CapacityUsed(u64),
    #[error("Invalid UTF-8 data: first byte {0}")]
    InvalidUtf8(u8),
    #[error("Cursors are for different resources")]
    CursorResourcesDiffer,
    #[error("Cursors are not in the exprected order")]
    CursorsOutOfOrder,
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("UTF-8 Error")]
    Utf8Error,
}
