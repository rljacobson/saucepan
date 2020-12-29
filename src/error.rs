use std::{
  error,
  fmt::{Debug, Display, Formatter},
};

use crate::{ByteIndex, LineIndex, Span, };


type SourceID = usize;

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
pub enum LocationError<SpanType>{
  OutOfBounds { given: ByteIndex, span: SpanType},
  InvalidCharBoundary { given: ByteIndex },
}

impl<SpanType: Debug + Display> error::Error for LocationError<SpanType> {}

impl<SpanType: Debug + Display> Display for LocationError<SpanType>{
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
  pub given: Span<'s>,
  pub span: Span<'s>,
}

impl<'s> error::Error for SpanOutOfBoundsError<'s> {}

impl<'s> Display for SpanOutOfBoundsError<'s> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Span out of bounds - given: {}, span: {}",
      self.given, self.span
    )
  }
}


impl<'s> Debug for SpanOutOfBoundsError<'s> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}
