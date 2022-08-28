/*!

  A `Location` consisting of a (line, column) pair, is what is produced when a `Span` is resolved
  for human consumption. It is only needed for reporting. Both lines and columns start at 1.

*/


#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{ColumnIndex, LineIndex};

/// A location, a (line, column) pair, in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Location {
  pub line: LineIndex,
  pub column: ColumnIndex,
}

impl Location {
  pub const BEGINNING_OF_FILE: Self = Location{ line: LineIndex(1), column: ColumnIndex(1)};
  pub const BOF: Self = Location::BEGINNING_OF_FILE;

  pub fn new(line: impl Into<LineIndex>, column: impl Into<ColumnIndex>) -> Location {
    Location {
      line: line.into(),
      column: column.into(),
    }
  }
}
