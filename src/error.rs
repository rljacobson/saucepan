

use std::{error, fmt};

use crate::{ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span};


#[derive(Debug, PartialEq)]
pub struct LineIndexOutOfBoundsError {
  pub given: LineIndex,
  pub max: LineIndex,
}

impl error::Error for LineIndexOutOfBoundsError {}

impl fmt::Display for LineIndexOutOfBoundsError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Line index out of bounds - given: {}, max: {}",
      self.given, self.max
    )
  }
}

#[derive(Debug, PartialEq)]
pub enum LocationError {
  OutOfBounds { given: ByteIndex, span: Span },
  InvalidCharBoundary { given: ByteIndex },
}

impl error::Error for LocationError {}

impl fmt::Display for LocationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      LocationError::OutOfBounds { given, span } => write!(
        f,
        "Byte index out of bounds - given: {}, span: {}",
        given, span
      ),
      LocationError::InvalidCharBoundary { given } => {
        write!(f, "Byte index within character boundary - given: {}", given)
      }
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct SpanOutOfBoundsError {
  pub given: Span,
  pub span: Span,
}

impl error::Error for SpanOutOfBoundsError {}

impl fmt::Display for SpanOutOfBoundsError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Span out of bounds - given: {}, span: {}",
      self.given, self.span
    )
  }
}
