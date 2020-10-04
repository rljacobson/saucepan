/*!
Saucepan, a special input type for source spans

The source code is available on [Github](https://github.com/rljacobson/saucepan)

Saucepan is a mash-up of codespan and nom_locate. You can use saucepan independent of nom by disabling the `"nom"` feature.

## Features

This crate exposes two cargo feature flags, `generic-simd` and `runtime-dispatch-simd`.
These correspond to the features exposed by [bytecount](https://github.com/llogiq/bytecount).

## How to use it
The explanations are given in the [README](https://github.com/rljacobson/saucepan/blob/master/README.md) of the Github repository. You may also consult the [FAQ](https://github.com/rljacobson/saucepan/blob/master/FAQ.md).

```
#[macro_use]
extern crate nom;
#[macro_use]
extern crate saucepan;

use saucepan::Span;
type Span<'s> = Span<&'s str>;

struct Token<'s> {
    pub position: Span<'s>,
    pub foo: String,
    pub bar: String,
}

# #[cfg(feature = "alloc")]
named!(parse_foobar( Span ) -> Token, do_parse!(
    take_until!("foo") >>
    position: position!() >>
    foo: tag!("foo") >>
    bar: tag!("bar") >>
    (Token {
        position: position,
        foo: foo.to_string(),
        bar: bar.to_string()
    })
));

# #[cfg(feature = "alloc")]
fn main () {
    let input = Span::new("Lorem ipsum \n foobar");
    let output = parse_foobar(input);
    let position = output.unwrap().1.position;
    assert_eq!(position.location_offset(), 14);
    assert_eq!(position.location_line(), 2);
    assert_eq!(position.fragment(), &"");
    assert_eq!(position.get_column(), 2);
}
# #[cfg(not(feature = "alloc"))]
fn main() {}
```

## Extra information
You can also add arbitrary extra information using the extra property of `Span`.
This property is not used when comparing two `Span`s.

``̀`
use saucepan::Span;
type Span<'s> = Span<&'s str, String>;

let input = Span::new("Lorem ipsum \n foobar", "filename");
let output = parse_foobar(input);
let extra = output.unwrap().1.extra;
``̀`
*/



use std::{
  ops::{RangeBounds, Bound},
  cmp::{min, max},
  str::from_utf8_unchecked
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


// use bytecount::{naive_num_chars, num_chars};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};


use crate::{ByteIndex, RawIndex, ByteOffset, Source, LineNumber, ColumnIndex, RawOffset, Slice, LineIndex, AsBytes};


/**
A `Span` holds the start, length, and reference to the source of a piece of source code. A `Span`
should not be created directly. Rather, the `Span` should be obtained from the `Source` or `Sources`
struct that owns the text, or it should be created through a method on an exiting span.
*/
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Span<'s, SourceType: 's> {
  start: ByteIndex,
  length: ByteOffset,
  pub source: &'s Source<SourceType>
}

impl<'s, SourceType> Span<'s, SourceType> {
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
  pub fn merge(mut self, other: Span<'s, SourceType>) -> Span<'s, SourceType> {
    let start = min(self.start, other.start);
    let length = max(self.length, other.length);
    let mut new_span = Self {
      start,
      length,
      source: self.source
    };

    std::mem::swap( & mut self, & mut new_span);
    self
  }

  /// A helper function to tell whether two spans do not overlap.
  pub fn disjoint(self, other: Span<'s, SourceType>) -> bool {
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
}

impl<'s, SourceType> Span<'s, SourceType>
// todo: Relax these constraints
  where SourceType: 's + Copy + AsBytes
{
  // Create a new span from a start and fragment.
  pub fn new<S: Into<ByteIndex>, L: Into<ByteOffset>>(
    start: S,
    length: L,
    source: &'s Source<SourceType>
    // todo: Consider adding `extra` as in `Span`.
  ) -> Span<'s, SourceType>
  {
    let start = start.into();
    let length = length.into();

    Span {
      start,
      length,
      source
    }
  }


  pub fn fragment(&self) -> SourceType {
    self.source.fragment(self)
  }


  /// The line number of the fragment relative to the input of the
  /// parser. It starts at line 1.
  pub fn location_line(&self) -> LineNumber {
    self.source.line_index(self.start).number()
  }

  pub fn row(&self) -> LineNumber {
    self.location_line()
  }


  pub fn column(&self) -> ColumnIndex {
    self.source.column(self)
  }
}

impl<'s, SourceType> Span<'s, SourceType>
  where SourceType: 's + AsRef<str> + Slice<Range<usize>> + Copy + AsBytes
{
  /// If `source` is the original span, then `from_start_end(start, end)==source[start..end]`.
  //noinspection RsSelfConvention
  pub fn from_start_end(&self, start: usize, end: usize) -> Self {
    self.source.slice(start..end)
  }
}

impl<'s, SourceType: 's> Display for Span<'s, SourceType>
  where SourceType: 's + Display + Copy + AsBytes + Slice<Range<usize>>
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.len() > 9 {
      let end = self.len() - 4;

      write!(
        f,
        "Span<{}, {}>(`{}…{}`)",
        self.row(),
        self.column(),
        self.fragment().slice(0..4),
        self.fragment().slice(end..self.len())
      )
    } else {
      write!(
        f,
        "Span<{}, {}>(`{}`)",
        self.row(),
        self.column(),
        self.fragment()
      )
    }
  }
}

// The following are needed for `nom` integration but are also useful in themselves.


impl<'s, SourceType: 's> From<Span<'s, SourceType>> for Range<usize> {
  fn from(span: Span<'s, SourceType>) -> Range<usize> {
    span.start.into()..span.end().into()
  }
}

impl<'s, SourceType: 's> From<Span<'s, SourceType>> for Range<RawIndex> {
  fn from(span: Span<'s, SourceType>) -> Range<RawIndex> {
    span.start.0..span.end().0
  }
}


impl<'s, SourceType, RangeType> Slice<RangeType> for Span<'s, SourceType>
  where SourceType: 's + Copy + AsRef<str> + AsBytes,
        RangeType: RangeBounds<usize>
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


impl<'s, SourceType> PartialEq for Span<'s, SourceType>
  where SourceType: 's + Eq + PartialEq
{
  fn eq(&self, other: &Self) -> bool {
    *self.source == *other.source &&
        self.start == other.start &&
        self.length == other.length
  }
}

impl<'s, SourceType> Eq for Span<'s, SourceType>
  where SourceType: 's + Eq + PartialEq
{}


// endregion


#[cfg(feature = "nom-parsing")]
pub use nom_impls::*;
// A module defined below.
#[cfg(feature = "nom-parsing")]
use codespan_reporting::files::Files;

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
    FindToken,
    IResult,
    InputIter,
    InputLength,
    InputTake,
    InputTakeAtPosition,
    Offset,
    ParseTo,
    Slice,
  };
  use nom_locate::LocatedSpan;

  type LSpan<'s> = LocatedSpan<&'s str, &'s str>;


  // region Macros


  impl<'s, SourceType: 's> AsBytes for Span<'s, SourceType>
    where SourceType: 's + AsBytes
  {
    fn as_bytes(&self) -> &[u8] {
      self.source.as_bytes()
    }
  }

  impl<'s, SourceType: 's> InputLength for Span<'s, SourceType> {
    fn input_len(&self) -> usize {
      self.source.len()
    }
  }

  impl<'s, SourceType: 's> InputTake for Span<'s, SourceType>
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

  impl<'s, SourceType> InputTakeAtPosition for Span<'s, SourceType>
    where SourceType: 's + InputTakeAtPosition + InputLength + InputIter +
        Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Copy + AsBytes
  {
    type Item = <SourceType as InputIter>::Item;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
      where
          P: Fn(Self::Item) -> bool,
    {
      match self.source.text().position(predicate) {
        Some(n) => Ok(self.take_split(n)),
        None => Err(Err::Incomplete(nom::Needed::Size(NonZeroUsize::new(1).unwrap()))),
      }
    }

    fn split_at_position1<P, E: ParseError<&'s Self>>(
      &self,
      predicate: P,
      e: ErrorKind,
    ) -> IResult<Self, Self, E>
      where P: Fn(Self::Item) -> bool,
    {
      match self.source.text().position(predicate) {
        Some(0) => Err(Err::Error(E::from_error_kind(self, e))),
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
      match self.source.position(predicate) {
        Some(0) => Err(Err::Error(E::from_error_kind(self.copy(), e))),
        Some(n) => Ok(self.take_split(n)),
        None => {
          if self.source.input_len() == 0 {
            Err(Err::Error(E::from_error_kind(self.copy(), e)))
          } else {
            Ok(self.take_split(self.input_len()))
          }
        }
      }
    }
  }



  /// Implement nom::Compare for a specific fragment type.
  ///
  /// # Parameters
  /// * `$fragment_type` - The Span's `fragment` type
  /// * `$compare_to_type` - The type to be comparable to `Span<'s, $fragment_type>`
  ///
  /// # Example of use
  ///
  /// NB: This example is an extract from the nom_locate source code.
  ///
  /// ````ignore
  /// #[macro_use]
  /// extern crate nom_locate;
  /// impl_compare!(&'b str, &'s str);
  /// impl_compare!(&'b [u8], &'s [u8]);
  /// impl_compare!(&'b [u8], &'s str);
  /// ````
  #[macro_export]
  macro_rules! impl_compare {
    ( $fragment_type:ty, $compare_to_type:ty ) => {
        impl<'a, 'b> Compare<$compare_to_type> for Span<'a, $fragment_type> {
            #[inline(always)]
            fn compare(&self, t: $compare_to_type) -> CompareResult {
                self.source.compare(t)
            }

            #[inline(always)]
            fn compare_no_case(&self, t: $compare_to_type) -> CompareResult {
                self.source.compare_no_case(t)
            }
        }
    };
}
  impl_compare!(&'b str, &'a str);
  impl_compare!(&'b [u8], &'a [u8]);
  impl_compare!(&'b [u8], &'a str);
  impl<'s, A: Compare<B> + 's, B: 's> Compare<Span<'s, B>> for Span<'s, A> {
    #[inline(always)]
    fn compare(&self, t: Span<'s, B>) -> CompareResult {
      self.source.compare(t.source)
    }

    #[inline(always)]
    fn compare_no_case(&self, t: Span<'s, B>) -> CompareResult {
      self.source.compare_no_case(t.source)
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
  impl<'s, Fragment: FindToken<Token> + 's, Token> FindToken<Token> for Span<'s, Fragment> {
    fn find_token(&self, token: Token) -> bool {
      self.source.find_token(token)
    }
  }

  impl<'s, SourceType: 's> FindSubstring<&'s str> for Span<'s, SourceType>
    where
        SourceType: FindSubstring<&'s str>,
  {
    #[inline]
    fn find_substring(&self, substr: &'s str) -> Option<usize> {
      self.source.find_substring(substr)
    }
  }

  impl<'s, SourceType: 's, R: FromStr> ParseTo<R> for Span<'s, SourceType>
    where SourceType: ParseTo<R>,
  {
    #[inline]
    fn parse_to(&self) -> Option<R> {
      self.source.parse_to()
    }
  }

  impl<'s, SourceType: 's> Offset for Span<'s, SourceType> {
    fn offset(&self, second: &Self) -> usize {
      let fst = self.start;
      let snd = second.start;

      (snd - fst).into()
    }
  }


  /// Implement nom::ExtendInto for a specific fragment type.
  ///
  /// # Parameters
  /// * `$fragment_type` - The Span's `fragment` type
  /// * `$item` - The type of the item being iterated (a reference for fragments of type `&[SourceType]`).
  /// * `$extender` - The type of the Extended.
  ///
  /// # Example of use
  ///
  /// NB: This example is an extract from the nom_locate source code.
  ///
  /// ````ignore
  /// #[macro_use]
  /// extern crate nom_locate;
  ///
  /// impl_extend_into!(&'s str, char, String);
  /// impl_extend_into!(&'s [u8], u8, Vec<u8>);
  /// ````
  #[macro_export]
  macro_rules! impl_extend_into {
    ($fragment_type:ty, $item:ty, $extender:ty) => {
        impl<'s> ExtendInto for Span<'s, $fragment_type> {
            type Item = $item;
            type Extender = $extender;

            #[inline]
            fn new_builder(&self) -> Self::Extender {
                self.source.new_builder()
            }

            #[inline]
            fn extend_into(&self, acc: &mut Self::Extender) {
                self.source.extend_into(acc)
            }
        }
    };
  }

  impl_extend_into!(&'s str, char, String);
  impl_extend_into!(&'s [u8], u8, Vec<u8>);
  #[macro_export]
  macro_rules! impl_hex_display {
    ($fragment_type:ty) => {
        #[cfg(feature = "alloc")]
        impl<'s> nom::HexDisplay for Span<'s, $fragment_type> {
            fn to_hex(&self, chunk_size: usize) -> String {
                self.source.to_hex(chunk_size)
            }

            fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
                self.source.to_hex_from(chunk_size, from)
            }
        }
    };
}
  impl_hex_display!(&'s str);
  impl_hex_display!(&'s [u8]);

  /// Capture the position of the current fragment
  #[macro_export]
  macro_rules! position {
    ($input:expr,) => {
        tag!($input, "")
    };
  }

  /// Capture the position of the current fragment
  pub fn position<'s, SourceType: 's, E>(s: SourceType) -> IResult<SourceType, SourceType, E>
    where
        E: ParseError<SourceType>,
        SourceType: InputIter + InputTake,
  {
    nom::bytes::complete::take(0usize)(s)
  }
}

// endregion
