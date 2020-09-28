use std::{
  error,
  fmt::{Debug, Display, Formatter},
};

use crate::{ByteIndex, LineIndex, Span, SourceID};


type SourceType<'s> = &'s str;


#[derive(Debug, PartialEq, Eq)]
pub struct LineIndexOutOfBoundsError {
  pub given: LineIndex,
  pub max: LineIndex,
}

impl error::Error for LineIndexOutOfBoundsError {}

impl Display for LineIndexOutOfBoundsError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Line index out of bounds - given: {}, max: {}",
      self.given, self.max
    )
  }
}


#[derive(Debug, PartialEq, Eq)]
pub struct NotASourceError {
  pub given: SourceID,
  pub max: SourceID,
}

impl error::Error for NotASourceError {}

impl Display for NotASourceError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Source ID not found - given: {}, max: {}",
      self.given, self.max
    )
  }
}


#[derive(Debug, PartialEq, Eq)]
pub enum LocationError<'s>{
  OutOfBounds { given: ByteIndex, span: Span<SourceType<'s>> },
  InvalidCharBoundary { given: ByteIndex },

}

impl error::Error for LocationError<'_> {}

impl Display for LocationError<'_>{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

// impl Debug for LocationError {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     Display::fmt(self, f)
//   }
// }


#[derive(PartialEq, Eq)]
pub struct SpanOutOfBoundsError<'s> {
  pub given: Span<SourceType<'s>>,
  pub span: Span<SourceType<'s>>,
}

impl error::Error for SpanOutOfBoundsError<'_> {}

impl Display for SpanOutOfBoundsError<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Span out of bounds - given: {}, span: {}",
      self.given, self.span
    )
  }
}


impl Debug for SpanOutOfBoundsError<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}
