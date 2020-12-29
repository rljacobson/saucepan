#![feature(layout_for_ptr)]
#![feature(const_fn)]
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


mod range_impl;

/**
If the `nom-parsing` feature is disabled, we include `AsBytes` and `AsSlice` from the `shims`
module instead.
*/
#[cfg(feature = "nom-parsing")]
pub use nom::{
  AsBytes,
  Slice
};


#[cfg(not(feature = "nom-parsing"))]
mod shims;

#[cfg(not(feature = "nom-parsing"))]
pub use shims::{
  AsBytes,
  Slice
};


mod source;
mod sources;
mod index;
mod location;
mod error;
mod span;
// #[cfg(test)]
// mod tests;


pub use crate::{
  error::{
    LineIndexOutOfBoundsError,
    LocationError,
    SpanOutOfBoundsError
  },
  source::Source,
  sources::Sources,
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

