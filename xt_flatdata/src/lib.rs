// Copyright 2022 Dave Wathen. All rights reserved.

pub use crate::buffer::CharCursor;
pub use crate::error::FlatDataError;
pub use crate::lines::DelimitedFormat;
pub use crate::lines::DelimitedFormatBuilder;
pub use crate::lines::DelimitedLine;
pub use crate::lines::DelimitedLineReader;
pub use crate::lines::SimpleLine;
pub use crate::lines::SimpleLineReader;

mod buffer;
mod error;
mod lines;
