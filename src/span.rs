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
type Span<'a> = Span<&'a str>;

struct Token<'a> {
    pub position: Span<'a>,
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
type Span<'a> = Span<&'a str, String>;

let input = Span::new("Lorem ipsum \n foobar", "filename");
let output = parse_foobar(input);
let extra = output.unwrap().1.extra;
``̀`
*/

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
  convert::Into
};


use bytecount::{naive_num_chars, num_chars};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "nom-parsing")]
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
  ToUsize,
};
#[cfg(feature = "nom-parsing")]
use nom_locate::LocatedSpan;

use crate::{ByteIndex, RawIndex, SourceID, ByteOffset};

use std::str::from_utf8_unchecked;

// todo: SourceType is a type def until we figure out the trait constraints.
pub type SourceType<'s> = &'s str;


#[cfg(feature = "nom-parsing")]
type LSpan<'s> = LocatedSpan<&'s str, SourceID>;


/**
A `Span` holds the start, end, and source ID of a piece of source code. A `Span` should not be
created directly. Rather, the `Span` should be obtained from the `Source` of `Sources` struct
that owns the text.
*/
#[derive(Debug, Copy, Clone)] //, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Span<Source>{
    // where SourceType: AsBytes{
  /// This `Span` begins at byte `start` in the original "Source"
  start: ByteIndex,
  fragment: Source,
  // todo: source_id can be recovered by comparing fragment to sources in Sources.
  pub(crate) source_id: SourceID
}

impl<'s> Span<SourceType<'s>>
    // todo: Find alternatives to AsBytes to make more generic. (AsBytes is only for &str.)
    // where SourceType:
{
  /**
  Create a span for a particular input with default `start` which starts at 0, `line` starts at
  1, and `column` starts at 1.

  Do not use this constructor in parser functions; `nom` and `nom_locate` assume span offsets are
  relative to the beginning of the same input. In these cases, you probably want to use the
  `nom::traits::Slice` trait instead.

  # Example of use

  ```
  # extern crate nom_locate;
  use nom_locate::Span;

  # fn main() {
  let span = Span::new_extra(b"foobar", "extra");

  assert_eq!(span.location_offset(), 0);
  assert_eq!(span.location_line(),   1);
  assert_eq!(span.get_column(),      1);
  assert_eq!(span.fragment(),        &&b"foobar"[..]);
  assert_eq!(span.extra,             "extra");
  # }
  ```
  */
  pub fn new(program: SourceType) -> Span<SourceType> {
    Span {
      start: ByteIndex::default(),
      fragment: program,
      source_id: SourceID::new(0)
    }
  }

  pub fn len(&self) -> usize {
    self.fragment.len()
  }


  /// The start represents the position of the fragment relatively to
  /// the input of the parser. It starts at start 0.
  pub fn location_offset(&self) -> ByteIndex {
    self.start
  }

  /// The line number of the fragment relative to the input of the
  /// parser. It starts at line 1.
  // pub fn location_line(&self) -> u32 {
  //     let consumed = self.fragment.slice(..consumed_len);
  //     let next_offset = self.start + consumed_len;
  //
  //     let consumed_as_bytes = consumed.as_bytes();
  //     let iter = Memchr::new(b'\n', consumed_as_bytes);
  //     let number_of_lines = iter.count() as u32;
  //     let next_line = self.line + number_of_lines;
  // }


  /**
  Create a new span from a start and fragment. Allows overriding start and line. This is unsafe,
  because giving an start too large may result in undefined behavior, as some methods move back
  along the fragment assuming any negative index within the start is valid.
  */
  pub fn with_start(
    start: impl Into<ByteIndex>,
    fragment: SourceType,
    source_id: SourceID
    // todo: Consider adding `extra` as in `Span`.
  ) -> Span<SourceType>
  {
    let start = start.into();

    Span { start, fragment, source_id }
  }

  /// Gives an empty span at the start of a source.
  // pub const fn initial() -> Span<SourceType>
  //   where SourceType: AsRef<str> {
  //   Span {
  //     start: ByteIndex::default(),
  //     source_id: SourceID::new(0),
  //     fragment: &""
  //   }
  // }

  /// Measure the span of a string.
  ///
  /// ```rust
  /// use codespan::{ByteIndex, Span};
  ///
  /// let span = Span::from_str("hello");
  ///
  /// assert_eq!(span, Span::new(0, 5));
  /// ```
  pub fn from_str(text: SourceType, source_id: SourceID) -> Span<SourceType> {
    Span{
      start: ByteIndex::default(),
      fragment: text,
      source_id
    }
  }

  /// If `source` is the original span, then `from_start_end(start, end)==source[start..end]`.
  /// This method is very unsafe. You should only use it if you really know what you are doing.
  //noinspection RsSelfConvention
  pub fn from_start_end(&self, start: usize, end: usize) -> Self {
    // let self_bytes = self.fragment.as_bytes();
    let self_ptr = self.fragment.as_ptr();
    let new_slice = unsafe {
      assert!(
        self.start.to_usize() <= isize::max_value() as usize,
        "offset is too big"
      );
      let orig_input_ptr = self_ptr.offset((start as isize - self.start.0 as isize) as isize);
      from_utf8_unchecked(slice::from_raw_parts(orig_input_ptr, end.into()))
    };

    Span::with_start(ByteIndex(start as u32), new_slice, self.source_id)
  }

  /// Combine two spans by taking the start of the earlier span
  /// and the end of the later span.
  ///
  /// Note: this will work even if the two spans are disjoint.
  /// If this doesn't make sense in your application, you should handle it yourself.
  /// In that case, you can use `Span::disjoint` as a convenience function.
  ///
  /// ```rust
  /// use codespan::Span;
  ///
  /// let span1 = Span::new(0, 4);
  /// let span2 = Span::new(10, 16);
  ///
  /// assert_eq!(Span::merge(span1, span2), Span::new(0, 16));
  /// ```
  pub fn merge(mut self, other: Span<SourceType<'s>>) -> Span<SourceType<'s>> {
    use std::cmp::{max, min};

    let start = min(self.start, other.start);
    let end = max(self.end(), other.end());
    let mut new_span = self.from_start_end(usize::from(start), usize::from(end));

    std::mem::swap(&mut self, &mut new_span);
    self
  }

  /// A helper function to tell whether two spans do not overlap.
  ///
  /// ```
  /// use codespan::Span;
  /// let span1 = Span::new(0, 4);
  /// let span2 = Span::new(10, 16);
  /// assert!(span1.disjoint(span2));
  /// ```
  pub fn disjoint(self, other: Span<SourceType>) -> bool {
    let (first, last) = if self.end() < other.end() {
      (self, other)
    } else {
      (other, self)
    };
    first.end() <= last.start
  }

  /// Get the starting byte index.
  ///
  /// ```rust
  /// use codespan::{ByteIndex, Span};
  ///
  /// let span = Span::new(0, 4);
  ///
  /// assert_eq!(span.start(), ByteIndex::from(0));
  /// ```
  pub fn start(self) -> ByteIndex {
    self.start
  }

  /// Get the ending byte index.
  ///
  /// ```rust
  /// use codespan::{ByteIndex, Span};
  ///
  /// let span = Span::new(0, 4);
  ///
  /// assert_eq!(span.end(), ByteIndex::from(4));
  /// ```
  pub fn end(self) -> ByteIndex {
    self.start + ByteOffset(self.len() as i64)
  }



  /// The fragment that is spanned.
  pub fn fragment(&self) -> SourceType {
    self.fragment.clone()
  }

  fn get_columns_and_bytes_before(&self) -> (usize, &[u8]) {
    let self_bytes = self.fragment.as_bytes();
    let self_ptr = self_bytes.as_ptr();
    let before_self = unsafe {
      assert!(
        self.start.to_usize() <= isize::max_value() as usize,
        "offset is too big"
      );
      let orig_input_ptr = self_ptr.offset(-(self.start.to_usize() as isize));
      slice::from_raw_parts(orig_input_ptr, self.start.into())
    };

    let column = match memchr::memrchr(b'\n', before_self) {
      None => self.start.to_usize() + 1,
      Some(pos) => self.start.to_usize() - pos,
    };

    (column, &before_self[self.start.to_usize() - (column - 1)..])
  }

  /// Return the column index, assuming 1 byte = 1 column.
  ///
  /// Use it for ascii text, or use get_utf8_column for UTF8.
  ///
  /// # Example of use
  /// ```
  ///
  /// # extern crate nom_locate;
  /// # extern crate nom;
  /// # use nom_locate::Span;
  /// # use nom::Slice;
  /// #
  /// # fn main() {
  /// let span = Span::new("foobar");
  ///
  /// assert_eq!(span.slice(3..).get_column(), 4);
  /// # }
  /// ```
  pub fn get_column(&self) -> usize {
    self.get_columns_and_bytes_before().0
  }

  /// Return the column index for UTF8 text. Return value is unspecified for non-utf8 text.
  ///
  /// This version uses bytecount's hyper algorithm to count characters. This is much faster
  /// for long lines, but is non-negligibly slower for short slices (below around 100 bytes).
  /// This is also sped up significantly more depending on architecture and enabling the simd
  /// feature gates. If you expect primarily short lines, you may get a noticeable speedup in
  /// parsing by using `naive_get_utf8_column` instead. Benchmark your specific use case!
  ///
  /// # Example of use
  /// ```
  ///
  /// # extern crate nom_locate;
  /// # extern crate nom;
  /// # use nom_locate::Span;
  /// # use nom::{Slice, FindSubstring};
  /// #
  /// # fn main() {
  /// let span = Span::new("メカジキ");
  /// let indexOf3dKanji = span.find_substring("ジ").unwrap();
  ///
  /// assert_eq!(span.slice(indexOf3dKanji..).get_column(), 7);
  /// assert_eq!(span.slice(indexOf3dKanji..).get_utf8_column(), 3);
  /// # }
  /// ```
  pub fn get_utf8_column(&self) -> usize {
    let before_self = self.get_columns_and_bytes_before().1;
    num_chars(before_self) + 1
  }

  /// Return the column index for UTF8 text. Return value is unspecified for non-utf8 text.
  ///
  /// A simpler implementation of `get_utf8_column` that may be faster on shorter lines.
  /// If benchmarking shows that this is faster, you can use it instead of `get_utf8_column`.
  /// Prefer defaulting to `get_utf8_column` unless this legitimately is a performance bottleneck.
  ///
  /// # Example of use
  /// ```
  ///
  /// # extern crate nom_locate;
  /// # extern crate nom;
  /// # use nom_locate::Span;
  /// # use nom::{Slice, FindSubstring};
  /// #
  /// # fn main() {
  /// let span = Span::new("メカジキ");
  /// let indexOf3dKanji = span.find_substring("ジ").unwrap();
  ///
  /// assert_eq!(span.slice(indexOf3dKanji..).get_column(), 7);
  /// assert_eq!(span.slice(indexOf3dKanji..).naive_get_utf8_column(), 3);
  /// # }
  /// ```
  pub fn naive_get_utf8_column(&self) -> usize {
    let before_self = self.get_columns_and_bytes_before().1;
    naive_num_chars(before_self) + 1
  }


}

// impl Default for Span<SourceType<'_>> {
//   fn default() -> Span<SourceType> {
//     Span::initial()
//   }
// }




impl<'s> Display for Span<SourceType<'s>> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.len() > 9 {

      let end = self.len() - 4;
      write!(
        f,
        "Span<{}—{}>(`{}…{}`)",
        self.start,
        self.start.to_usize() + self.len(),
        &self.fragment[..4],
        &self.fragment[end..]
      )

    } else {

      write!(
        f,
        "Span<{}—{}>(`{}`)",
        self.start,
        self.start.to_usize() + self.len(),
        self.fragment()
      )

    }
  }
}


impl From<Span<SourceType<'_>>> for Range<usize> {
  fn from(span: Span<SourceType>) -> Range<usize> {
    span.start.into()..span.end().into()
  }
}

impl From<Span<SourceType<'_>>> for Range<RawIndex> {
  fn from(span: Span<SourceType>) -> Range<RawIndex> {
    span.start.0..span.end().0
  }
}

//--------------------

// region Nom (from `located_span`)


#[cfg(feature = "nom-parsing")]
impl PartialEq for Span<SourceType<'_>> {
  fn eq(&self, other: &Self) -> bool {
    self.source_id == other.source_id &&
        self.start == other.start &&
        self.fragment == other.fragment
  }
}

#[cfg(feature = "nom-parsing")]
impl Eq for Span<SourceType<'_>> {}

#[cfg(feature = "nom-parsing")]
impl AsBytes for Span<SourceType<'_>> {
  fn as_bytes(&self) -> &[u8] {
    self.fragment.as_bytes()
  }
}

#[cfg(feature = "nom-parsing")]
impl InputLength for Span<SourceType<'_>> {
  fn input_len(&self) -> usize {
    self.fragment.len()
  }
}

#[cfg(feature = "nom-parsing")]
impl InputTake for Span<SourceType<'_>>
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

#[cfg(feature = "nom-parsing")]
impl<'s> InputTakeAtPosition for Span<SourceType<'s>>
  // where
  //     SourceType: InputTakeAtPosition + InputLength + InputIter,
  //     Self: Slice<RangeFrom<usize>> + Slice<RangeTo<usize>> + Clone,
{
  type Item = <SourceType<'s> as InputIter>::Item;

  fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
  {
    match self.fragment.position(predicate) {
      Some(n) => Ok(self.take_split(n)),
      None => Err(Err::Incomplete(nom::Needed::Size(NonZeroUsize::new(1).unwrap()))),
    }
  }

  fn split_at_position1<P, E: ParseError<Self>>(
    &self,
    predicate: P,
    e: ErrorKind,
  ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
  {
    match self.fragment.position(predicate) {
      Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
      Some(n) => Ok(self.take_split(n)),
      None => Err(Err::Incomplete(nom::Needed::Size(NonZeroUsize::new(1).unwrap()))),
    }
  }

  fn split_at_position_complete<P, E: ParseError<Self>>(
    &self,
    predicate: P,
  ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
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
    where
        P: Fn(Self::Item) -> bool,
  {
    match self.fragment.position(predicate) {
      Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
      Some(n) => Ok(self.take_split(n)),
      None => {
        if self.fragment.input_len() == 0 {
          Err(Err::Error(E::from_error_kind(self.clone(), e)))
        } else {
          Ok(self.take_split(self.input_len()))
        }
      }
    }
  }
}

/// Implement nom::InputIter for a specific fragment type
///
/// # Parameters
/// * `$fragment_type` - The Span's `fragment` type
/// * `$item` - The type of the item being iterated (a reference for fragments of type `&[SourceType]`).
/// * `$raw_item` - The raw type of the item being iterating (dereferenced type of $item for
/// `&[SourceType]`, otherwise same as `$item`)
/// * `$iter` - The iterator type for `iter_indices()`
/// * `$iter_elem` - The iterator type for `iter_elements()`
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ```ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// impl_input_iter!(&'a str, char, char, CharIndices<'a>, Chars<'a>);
/// impl_input_iter!(&'a [u8], &'a u8, u8, Enumerate<Iter<'a, Self::RawItem>>,
///                  Iter<'a, Self::RawItem>);
/// ```
#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! impl_input_iter {
    ($fragment_type:ty, $item:ty, $raw_item:ty, $iter:ty, $iter_elem:ty) => {
        impl<'a> InputIter for Span<$fragment_type> {
            type Item = $item;
            type Iter = $iter;
            type IterElem = $iter_elem;
            #[inline]
            fn iter_indices(&self) -> Self::Iter {
                self.fragment.iter_indices()
            }
            #[inline]
            fn iter_elements(&self) -> Self::IterElem {
                self.fragment.iter_elements()
            }
            #[inline]
            fn position<P>(&self, predicate: P) -> Option<usize>
            where
                P: Fn(Self::Item) -> bool,
            {
                self.fragment.position(predicate)
            }
            #[inline]
            fn slice_index(&self, count: usize) -> Option<usize> {
                self.fragment.slice_index(count)
            }
        }
    };
}
#[cfg(feature = "nom-parsing")]
impl_input_iter!(&'a str, char, char, CharIndices<'a>, Chars<'a>);
#[cfg(feature = "nom-parsing")]
impl_input_iter!(
    &'a [u8],
    u8,
    u8,
    Enumerate<Self::IterElem>,
    Map<Iter<'a, Self::Item>, fn(&u8) -> u8>
);

/// Implement nom::Compare for a specific fragment type.
///
/// # Parameters
/// * `$fragment_type` - The Span's `fragment` type
/// * `$compare_to_type` - The type to be comparable to `Span<$fragment_type>`
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
/// impl_compare!(&'b str, &'a str);
/// impl_compare!(&'b [u8], &'a [u8]);
/// impl_compare!(&'b [u8], &'a str);
/// ````
#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! impl_compare {
    ( $fragment_type:ty, $compare_to_type:ty ) => {
        impl<'a, 'b> Compare<$compare_to_type> for Span<$fragment_type> {
            #[inline(always)]
            fn compare(&self, t: $compare_to_type) -> CompareResult {
                self.fragment.compare(t)
            }

            #[inline(always)]
            fn compare_no_case(&self, t: $compare_to_type) -> CompareResult {
                self.fragment.compare_no_case(t)
            }
        }
    };
}

#[cfg(feature = "nom-parsing")]
impl_compare!(&'b str, &'a str);
#[cfg(feature = "nom-parsing")]
impl_compare!(&'b [u8], &'a [u8]);
#[cfg(feature = "nom-parsing")]
impl_compare!(&'b [u8], &'a str);

#[cfg(feature = "nom-parsing")]
impl<A: Compare<B>, B> Compare<Span<B>> for Span<A> {
  #[inline(always)]
  fn compare(&self, t: Span<B>) -> CompareResult {
    self.fragment.compare(t.fragment)
  }

  #[inline(always)]
  fn compare_no_case(&self, t: Span<B>) -> CompareResult {
    self.fragment.compare_no_case(t.fragment)
  }
}

// TODO(future): replace impl_compare! with below default specialization?
// default impl<A: Compare<B>, B> Compare<B> for Span<A> {
//     #[inline(always)]
//     fn compare(&self, t: B) -> CompareResult {
//         self.fragment.compare(t)
//     }
//
//     #[inline(always)]
//     fn compare_no_case(&self, t: B) -> CompareResult {
//         self.fragment.compare_no_case(t)
//     }
// }

/// Implement nom::Slice for a specific fragment type and range type.
///
/// **You'd probably better use impl_`slice_ranges`**,
/// unless you use a specific Range.
///
/// # Parameters
/// * `$fragment_type` - The Span's `fragment` type
/// * `$range_type` - The range type to be use with `slice()`.
/// * `$can_return_self` - A bool-returning lambda telling whether we
///    can avoid creating a new `Span`. If unsure, use `|_| false`.
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// #[macro_export]
/// macro_rules! impl_slice_ranges {
///     ( $fragment_type:ty ) => {
///         impl_slice_range! {$fragment_type, Range<usize>, |_| false }
///         impl_slice_range! {$fragment_type, RangeTo<usize>, |_| false }
///         impl_slice_range! {$fragment_type, RangeFrom<usize>, |range:&RangeFrom<usize>| range.start == 0}
///         impl_slice_range! {$fragment_type, RangeFull, |_| true}
///     }
/// }
///
/// ````
#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! impl_slice_range {
    ( $fragment_type:ty, $range_type:ty, $can_return_self:expr ) => {
        impl<'a> Slice<$range_type> for Span<$fragment_type> {
            fn slice(&self, range: $range_type) -> Self {
                if $can_return_self(&range) {
                    return self.clone();
                }
                let next_fragment = self.fragment.slice(range);
                let consumed_len = self.fragment.offset(&next_fragment);
                if consumed_len == 0 {
                    return Span {
                        start: self.start,
                        fragment: next_fragment,
                        source_id: self.source_id,
                    };
                }

                let next_offset = self.start + ByteOffset(consumed_len as i64);
                Span {
                    start: next_offset,
                    fragment: next_fragment,
                    source_id: self.source_id,
                }
            }
        }
    };
}

/// Implement nom::Slice for a specific fragment type and for these types of range:
/// * `Range<usize>`
/// * `RangeTo<usize>`
/// * `RangeFrom<usize>`
/// * `RangeFull`
///
/// # Parameters
/// * `$fragment_type` - The Span's `fragment` type
///
/// # Example of use
///
/// NB: This example is an extract from the nom_locate source code.
///
/// ````ignore
/// #[macro_use]
/// extern crate nom_locate;
///
/// impl_slice_ranges! {&'a str}
/// impl_slice_ranges! {&'a [u8]}
/// ````
#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! impl_slice_ranges {
    ( $fragment_type:ty ) => {
        impl_slice_range! {$fragment_type, Range<usize>, |_| false }
        impl_slice_range! {$fragment_type, RangeTo<usize>, |_| false }
        impl_slice_range! {$fragment_type, RangeFrom<usize>, |range:&RangeFrom<usize>| range.start == 0}
        impl_slice_range! {$fragment_type, RangeFull, |_| true}
    }
}

#[cfg(feature = "nom-parsing")]
impl_slice_ranges! {&'a str}
#[cfg(feature = "nom-parsing")]
impl_slice_ranges! {&'a [u8]}


#[cfg(feature = "nom-parsing")]
impl<Fragment: FindToken<Token>, Token> FindToken<Token> for Span<Fragment> {
  fn find_token(&self, token: Token) -> bool {
    self.fragment.find_token(token)
  }
}


#[cfg(feature = "nom-parsing")]
impl<'a> FindSubstring<&'a str> for Span<SourceType<'a>>
  where
      SourceType<'a>: FindSubstring<&'a str>,
{
  #[inline]
  fn find_substring(&self, substr: &'a str) -> Option<usize> {
    self.fragment.find_substring(substr)
  }
}

#[cfg(feature = "nom-parsing")]
impl<R: FromStr> ParseTo<R> for Span<SourceType<'_>>
  // where
  //     SourceType: ParseTo<R>,
{
  #[inline]
  fn parse_to(&self) -> Option<R> {
    self.fragment.parse_to()
  }
}

#[cfg(feature = "nom-parsing")]
impl Offset for Span<SourceType<'_>> {
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
/// impl_extend_into!(&'a str, char, String);
/// impl_extend_into!(&'a [u8], u8, Vec<u8>);
/// ````
#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! impl_extend_into {
    ($fragment_type:ty, $item:ty, $extender:ty) => {
        impl<'a> ExtendInto for Span<$fragment_type> {
            type Item = $item;
            type Extender = $extender;

            #[inline]
            fn new_builder(&self) -> Self::Extender {
                self.fragment.new_builder()
            }

            #[inline]
            fn extend_into(&self, acc: &mut Self::Extender) {
                self.fragment.extend_into(acc)
            }
        }
    };
}

#[cfg(feature = "nom-parsing")]
impl_extend_into!(&'a str, char, String);
#[cfg(feature = "nom-parsing")]
impl_extend_into!(&'a [u8], u8, Vec<u8>);

#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! impl_hex_display {
    ($fragment_type:ty) => {
        #[cfg(feature = "alloc")]
        impl<'a> nom::HexDisplay for Span<$fragment_type> {
            fn to_hex(&self, chunk_size: usize) -> String {
                self.fragment.to_hex(chunk_size)
            }

            fn to_hex_from(&self, chunk_size: usize, from: usize) -> String {
                self.fragment.to_hex_from(chunk_size, from)
            }
        }
    };
}

#[cfg(feature = "nom-parsing")]
impl_hex_display!(&'a str);
#[cfg(feature = "nom-parsing")]
impl_hex_display!(&'a [u8]);

/// Capture the position of the current fragment
#[cfg(feature = "nom-parsing")]
#[macro_export]
macro_rules! position {
    ($input:expr,) => {
        tag!($input, "")
    };
}

/// Capture the position of the current fragment
#[cfg(feature = "nom-parsing")]
pub fn position<SourceType, E>(s: SourceType) -> IResult<SourceType, SourceType, E>
  where
      E: ParseError<SourceType>,
      SourceType: InputIter + InputTake,
{
  nom::bytes::complete::take(0usize)(s)
}


// endregion
