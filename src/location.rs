/*!

  A `Location` consisting of a (line index, column index) pair, is the range of a `Span` It is
  typically only used for reporting. Note they are indices, not "numbers", and thus are zero-based
  instead of one-based.

*/


use std::fmt::Display;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{ColumnIndex, ColumnNumber, LineIndex, LineNumber};

/// A location, a (line, column) pair, in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Location {
  pub line_index:   LineIndex,
  pub column_index: ColumnIndex,
}

impl Location {
  pub const BEGINNING_OF_FILE: Self = Location{ line_index: LineIndex(0), column_index: ColumnIndex(0)};
  pub const BOF:               Self = Location::BEGINNING_OF_FILE;

  #[inline(always)]
  pub fn new(line_index: impl Into<LineIndex>, column_index: impl Into<ColumnIndex>) -> Location {
    Location {
      line_index:   line_index.into(),
      column_index: column_index.into(),
    }
  }

  /// Gives a human-readable line number (which start at 1).
  #[inline(always)]
  pub fn line_number(&self) -> LineNumber {
    self.line_index.number()
  }

  /// Gives a human-readable column number (which start at 1).
  #[inline(always)]
  pub fn column_number(&self) -> ColumnNumber {
    self.column_index.number()
  }
}

impl Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Location(line={}, column={})", self.line_number(), self.column_number())
  }
}

