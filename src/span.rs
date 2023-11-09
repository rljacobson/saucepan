/*!
The `Span` struct is the fundamental type of Saucepan that represents a location within a source
text.


```
//use nom::{bytes::complete::};

use saucepan::Span;

fn main()  {


}




```

*/



use std::{
  ops::{RangeBounds, Bound},
  cmp::{min, max}
};

pub use std::{
  fmt::{
    Display,
    Formatter,
    Result as FmtResult
  },
  num::NonZeroUsize,
  ops::{Range, RangeFrom, RangeFull, RangeTo},
  slice,
  slice::{Iter},
  str::{FromStr, CharIndices, Chars},
  iter::{Enumerate, Map},
  convert::Into,
};


#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};


use crate::{
  ByteIndex,
  RawIndex,
  ByteOffset,
  Source,
  LineNumber,
  Slice,
  LocationError,
  Location,
  ColumnNumber
};


/**
A `Span` holds the start, length, and reference to the source of a piece of source code. A `Span`
should not be created directly. Rather, the `Span` should be obtained from the `Source` or `Sources`
struct that owns the text, or through a method on an exiting span.
*/
#[derive(Debug, Copy, Clone, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Span<'n, 't> {
  start     : ByteIndex,
  length    : ByteOffset,
  pub source: &'t Source<'n, 't>
}

impl<'n, 't> Span<'n, 't> {
  pub fn len(&self) -> usize {
    self.length.into()
  }


  /// The start represents the position of the fragment relatively to
  /// the input of the parser. It starts at start 0.
  pub fn location_offset(&self) -> ByteIndex {
    self.start
  }

  /// Combine two spans by taking the start of the earlier span
  /// and the end of the later span.
  ///
  /// Note: this will work even if the two spans are disjoint.
  /// If this doesn't make sense in your application, you should handle it yourself.
  /// In that case, you can use `Span::disjoint` as a convenience function.
  pub fn merge(mut self, other: Span<'n, 't>)
    -> Result<Span<'n, 't>, IncompatibleSourcesError<'n, 't, 'n, 't>>
  {
    if  self.source != other.source {
      return Err(
        IncompatibleSourcesError{
          lhs: self,
          rhs: other
        }
      );
    }

    let start  = min(self.start, other.start);
    let end    = max(self.end(), other.end());
    let length = end - start;

    let mut new_span = Self {
      start,
      length,
      source: self.source
    };

    std::mem::swap( & mut self, & mut new_span);
    Ok(self)
  }

  /// A helper function to tell whether two spans do not overlap.
  pub fn disjoint(self, other: Span) -> bool {
    if self.source != other.source {
      return true;
    }


    let (first, last) = if self.start < other.start {
      (self, other)
    } else {
      (other, self)
    };
    first.end() <= last.start
  }


  pub fn start(self) -> ByteIndex {
    self.start
  }


  pub fn end(self) -> ByteIndex {
    self.start + self.len().into()
  }


  // Create a new span from a start and fragment.
  pub fn new<S: Into<ByteIndex>, L: Into<ByteOffset>>(
    start : S,
    length: L,
    source: &'t Source<'n, 't>
    // todo: Consider adding `extra` as in `Span`.
  ) -> Span<'n, 't>
  {
    let start  = start.into();
    let length = length.into();

    Span {
      start,
      length,
      source
    }
  }


  pub fn fragment(&self) -> &'t str {
    self.source.fragment(self)
  }

  /// The line number of the start of the fragment in the source file. Lines
  /// start at line 1. You probably want to use `self.location(..)` instead.
  pub fn location_line(&self) -> Result<LineNumber, LocationError> {
    Ok((self.source.line_index(self.start)?).number())
  }

  /// The line number of the start of the fragment in the source file. Lines
  /// start at line 1. You probably want to use `self.location(..)` instead.
  pub fn row(&self) -> Result<LineNumber, LocationError> {
    self.location_line()
  }

  /// Gives the column number (counting UTF-8 characters) of the start of the fragment. Columns
  /// start at 1. You almost certainly want to use `self.location(..)` instead of this function.
  pub fn column(&self) -> Result<ColumnNumber, LocationError> {
    let location = self.location()?;
    Ok(location.column_index.number())
  }

  /// Provides the (row_index, column_index) location of the start of the span. The row/column
  /// indices start at 0.
  pub fn location(&self) -> Result<Location, LocationError> {
    self.source.location_utf8(self.start)
  }

}


impl<'n, 't> Display for Span<'n, 't> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let location = match self.location() {
      Ok(loc) => loc,
      Err(_) => {
        return write!(
          f,
          "Span<{}:«invalid location»>()",
          self.source.name()
        );
      }
    };

    if self.len() > 9 {
      let end = self.len() - 4;

      write!(
        f,
        "Span<{}:{}:{}>(`{}…{}`)",
        self.source.name(),
        location.line_index.number(),
        location.column_index.number(),
        self.fragment().slice(0..4),
        self.fragment().slice(end..self.len())
      )
    } else {
      write!(
        f,
        "Span<{}:{}:{}>(`{}`)",
        self.source.name(),
        location.line_index.number(),
        location.column_index.number(),
        self.fragment()
      )
    }
  }
}

// The following are needed for `nom` integration but are also useful in themselves.

impl<'n, 't> From<Span<'n, 't>> for Range<usize> {
  fn from(span: Span<'n, 't>) -> Range<usize> {
    span.start.into()..span.end().into()
  }
}

impl<'n, 't> From<Span<'n, 't>> for Range<RawIndex> {
  fn from(span: Span<'n, 't>) -> Range<RawIndex> {
    span.start.0..span.end().0
  }
}


impl<'n, 't, RangeType> Slice<RangeType> for Span<'n, 't>
  where RangeType: RangeBounds<usize>
{
  fn slice(&self, range: RangeType) -> Self {
    let range_start =
        match range.start_bound() {
          Bound::Included(s) => { *s }
          Bound::Excluded(s) => { s + 1 }
          Bound::Unbounded => { 0 }
        };
    let range_end =
        match range.end_bound() {
          Bound::Included(s) => { *s }
          Bound::Excluded(s) => { s - 1 }
          Bound::Unbounded => { self.len() }
        };

    let start: ByteIndex = max(self.start().into(), range_start).into();
    let length: ByteOffset = max(0, min(self.len().into(), range_end - range_start)).into();

    Span::new(start, length, self.source)
  }
}


impl<'n, 't> PartialEq for Span<'n, 't>

{
  fn eq(&self, other: &Self) -> bool {
    *self.source == *other.source &&
        self.start == other.start &&
        self.length == other.length
  }
}

impl<'n, 't> Eq for Span<'n, 't>{}


// endregion


#[cfg(feature = "nom-parsing")]
pub use nom_impls::*;
use crate::error::IncompatibleSourcesError; // A module defined below.

#[cfg(feature = "nom-parsing")]
mod nom_impls {
  use super::*;
  use nom::{
    ExtendInto,
    error::{ErrorKind, ParseError},
    AsBytes,
    Compare,
    CompareResult,
    Err,
    FindSubstring,
    IResult,
    InputIter,
    InputLength,
    InputTake,
    InputTakeAtPosition,
    Offset,
    ParseTo,
    Slice,
  };

  // use nom_locate::LocatedSpan;
  // type LSpan<'s> = LocatedSpan<&'s str, &'s str>;


  // region Macros


  impl<'n, 't> AsBytes for Span<'n, 't> {
    fn as_bytes(&self) -> &[u8] {
      self.fragment().as_bytes()
    }
  }

  impl<'n, 't> InputLength for Span<'n, 't>{
    fn input_len(&self) -> usize {
      self.fragment().as_bytes().len()
    }
  }

  impl<'n, 't> InputTake for Span<'n, 't>
    where
        Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>>,
  {
    fn take(&self, count: usize) -> Self {
      self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
      (self.slice(count..), self.slice(..count))
    }
  }

  impl<'n, 't> InputTakeAtPosition for Span<'n, 't>
    //
    //     Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Copy + AsBytes
    where Self: InputTake
  {
    type Item = <&'t str as InputIter>::Item;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
      where
          P: Fn(Self::Item) -> bool,
    {
      match self.fragment().position(predicate) {
        Some(n) => Ok(self.take_split(n)),
        None => Err(Err::Incomplete(nom::Needed::Size(NonZeroUsize::new(1).unwrap()))),
      }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
      &self,
      predicate: P,
      e: ErrorKind,
    ) -> IResult<Self, Self, E>
      where P: Fn(Self::Item) -> bool,
    {
      match self.fragment().position(predicate) {
        Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
        Some(n) => Ok(self.take_split(n)),
        None => Err(Err::Incomplete(nom::Needed::Size(NonZeroUsize::new(1).unwrap()))),
      }
    }

    fn split_at_position_complete<P, E: ParseError<Self>>(
      &self,
      predicate: P,
    ) -> IResult<Self, Self, E>
      where P: Fn(Self::Item) -> bool,
    {
      match self.split_at_position(predicate) {
        Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
        res => res,
      }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
      &self,
      predicate: P,
      e: ErrorKind,
    ) -> IResult<Self, Self, E>
      where P: Fn(Self::Item) -> bool,
    {
      match self.fragment().position(predicate) {
        Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
        Some(n) => Ok(self.take_split(n)),
        None => {
          if self.input_len() == 0 {
            Err(Err::Error(E::from_error_kind(self.clone(), e)))
          } else {
            Ok(self.take_split(self.input_len()))
          }
        }
      }
    }
  }


  impl<'n, 't> Compare<Span<'n, 't>> for Span<'n, 't> {
    #[inline(always)]
    fn compare(&self, t: Span<'n, 't>) -> CompareResult {
      self.fragment().compare(t.fragment())
    }

    #[inline(always)]
    fn compare_no_case(&self, t: Span<'n, 't>) -> CompareResult {
      self.fragment().compare_no_case(t.fragment())
    }
  }

  impl<'n, 't> Compare<&str> for Span<'n, 't> {
    #[inline(always)]
    fn compare(&self, t: &str) -> CompareResult {
      self.fragment().compare(t)
    }

    #[inline(always)]
    fn compare_no_case(&self, t: &str) -> CompareResult {
      self.fragment().compare_no_case(t)
    }
  }

  // TODO(future): replace impl_compare! with below default specialization?
  // default impl<A: Compare<B>, B> Compare<B> for Span<'s, A> {
  //     #[inline(always)]
  //     fn compare(&self, t: B) -> CompareResult {
  //         self.source.compare(t)
  //     }
  //
  //     #[inline(always)]
  //     fn compare_no_case(&self, t: B) -> CompareResult {
  //         self.source.compare_no_case(t)
  //     }
  // }


  // todo: Do we need FindToken?

  // impl<'s, Token> FindToken<Token> for Span<'n, 't>
  // {
  //   fn find_token(&self, token: Token) -> bool {
  //     self.fragment().find_token(token)
  //   }
  // }

  impl<'n, 't> FindSubstring<&'t str> for Span<'n, 't> {
    #[inline]
    fn find_substring(&self, substr: &str) -> Option<usize> {
      self.fragment().find_substring(substr)
    }
  }

  impl<'n, 't, R: FromStr> ParseTo<R> for Span<'n, 't> {
    #[inline]
    fn parse_to(&self) -> Option<R> {
      self.fragment().parse_to()
    }
  }

  impl<'n, 't> Offset for Span<'n, 't> {
    fn offset(&self, second: &Self) -> usize {
      let fst = self.start;
      let snd = second.start;

      (snd - fst).into()
    }
  }


  impl<'n, 't> ExtendInto for Span<'n, 't> {
    type Item = char;
    type Extender = String;

    #[inline]
    fn new_builder(&self) -> Self::Extender {
      self.fragment().new_builder()
    }

    #[inline]
    fn extend_into(&self, acc: &mut Self::Extender) {
      self.fragment().extend_into(acc)
    }
  }


  impl<'n, 't> nom::HexDisplay for Span<'n, 't> {
    fn to_hex(&self, chunk_size: usize) -> String {
      self.fragment().to_hex(chunk_size)
    }

    fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
      self.fragment().to_hex_from(chunk_size, from)
    }
  }

  /// Capture the position of the current fragment
  #[macro_export]
  macro_rules! position {
    ($input:expr,) => {
        tag!($input, "")
    };
  }

  /// Capture the position of the current fragment
  #[allow(unused)]
  pub fn position<'s, E>(text: &'s str) -> IResult<&str, &str, E>
    where
        E: ParseError<&'s str>
  {
    nom::bytes::complete::take(0usize)(text)
  }
// endregion


}

