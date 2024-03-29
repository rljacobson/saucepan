/*!
Minimal error types for internal errors. Do not confuse these errors for errors the client
parsing code will generate for the user.
*/

// todo: combine these into a single enum.

use std::{
  error,
  fmt::{Debug, Display, Formatter},
};


#[cfg(feature = "reporting")]
use codespan_reporting::files::Error as CodespanError;

use crate::{ByteIndex, LineIndex, Source, Span};


type SourceID = usize;


#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
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


#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
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


#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum LocationError<'n, 't> {
  OutOfBounds { given: ByteIndex, source: &'t Source<'n, 't>},
  InvalidCharBoundary { given: ByteIndex },
}

impl error::Error for LocationError<'_, '_> {}

impl Display for LocationError<'_, '_>{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      &LocationError::OutOfBounds { given, source } => write!(
        f,
        "Byte index out of bounds - given: {}:{}",
        source.name(),
        given
      ),
      LocationError::InvalidCharBoundary { given } => {
        write!(f, "Byte index within character boundary - given: {}", given)
      }
    }
  }
}

#[cfg(feature = "reporting")]
impl From<LocationError<'_, '_>> for CodespanError{
    fn from(error: LocationError<'_, '_>) -> Self {
        match error {

          LocationError::OutOfBounds{given, source} => {
            CodespanError::IndexTooLarge{
              given: given.into(),
              max: source.last_line_index().into(),
            }
          },

          LocationError::InvalidCharBoundary{ given} => {
            CodespanError::InvalidCharBoundary{
              given: given.into()
            }
          }

        }
    }
}

// impl Debug for LocationError {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     Display::fmt(self, f)
//   }
// }


#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SpanOutOfBoundsError<'n, 't> {
  pub given: Span<'n, 't>,
  pub span: Span<'n, 't>,
}

impl error::Error for SpanOutOfBoundsError<'_, '_> {}

impl Display for SpanOutOfBoundsError<'_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Span out of bounds - given: {}, span: {}",
      self.given, self.span
    )
  }
}

impl Debug for SpanOutOfBoundsError<'_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}


#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct IncompatibleSourcesError<'n1, 't1, 'n2, 't2> {
  pub lhs: Span<'n1, 't1>,
  pub rhs: Span<'n2, 't2>,
}

impl error::Error for IncompatibleSourcesError<'_, '_, '_, '_> {}

impl Display for IncompatibleSourcesError<'_, '_, '_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Span {} and span {} have different sources.",
      self.lhs, self.rhs
    )
  }
}

impl Debug for IncompatibleSourcesError<'_, '_, '_, '_>  {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}
