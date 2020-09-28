//! Utilities for working with source code and printing nicely formatted
//! diagnostic information like warnings and errors.
//!
//! # Optional Features
//!
//! Extra functionality is accessible by enabling feature flags. The features
//! currently available are:
//!
//! - **serialization** - Adds `Serialize` and `Deserialize` implementations
//!   for use with `serde`

mod source;
mod index;
mod location;
mod error;
mod span;
#[cfg(test)]
mod tests;

pub use crate::{
  error::{LineIndexOutOfBoundsError, LocationError, SpanOutOfBoundsError},
  source::{SourceID, Sources, Source},
  index::{
    ColumnIndex,
    ByteIndex,
    ByteOffset,
    ColumnNumber,
    ColumnOffset,
    Index,
    Offset,
    LineIndex,
    LineNumber,
    LineOffset,
    RawIndex,
    RawOffset
  },
  location::Location,
  span::Span
};

