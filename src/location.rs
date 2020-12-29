/*!

A `Location` is what is produced when a `Span` is resolved for human consumption. It is only
needed for reporting.

*/




#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{ColumnIndex, LineIndex};

/// A location in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Location {
  pub line: LineIndex,
  pub column: ColumnIndex,
}

impl Location {
  pub fn new(line: impl Into<LineIndex>, column: impl Into<ColumnIndex>) -> Location {
    Location {
      line: line.into(),
      column: column.into(),
    }
  }
}
