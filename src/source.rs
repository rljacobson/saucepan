/*!

The `Source` struct represents a unit of source code, typically the contents of a single file.
The `Sources` struct is a collection of `Sources` that can be queried in various ways. In
general, the `Sources` struct remains in scope during parsing, providing `Span`s to client code.
Client code need not interact with `Source`. Instead, client code hands off the source text to
the `Sources` instance in exchange for a `Span` covering the entirety of the source text. The
`Span` may be used both as an input and an output, e.g. the input span is broken into token
spans. A `Span` knows its `Source` and can be queried for `&str`s and position/location data.

Example:

```rust

```

*/

use std::{
  cmp::{
    max,
    min,
    Ordering
  },
  ops::{
    Bound,
    Range,
    RangeBounds
  },
};
use std::fmt::{Debug, Display};


#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nom-parsing")]
use nom_locate::LocatedSpan;
#[cfg(feature = "reporting")]
use codespan_reporting::files::Files;

use memchr::Memchr;
use bytecount::{naive_num_chars, num_chars};

use crate::{
  error::{
    LineIndexOutOfBoundsError,
    LocationError,
    // NotASourceError
  },
  ByteIndex,
  ByteOffset,
  ColumnIndex,
  LineIndex,
  LineOffset,
  Location,
  Span,
};
use crate::span::Formatter;


#[cfg(feature = "nom-parsing")]
type LSpan<'n, 't> = LocatedSpan<&'t str, &'t Source<'n, 't>>;


/// A file that is stored in the database.
#[derive(Clone, Eq, PartialEq, Hash)]
// `Serialize` is only implemented on `OsString` for windows/unix
#[cfg_attr(
  all(feature = "serialization", any(windows, unix)),
  derive(Deserialize, Serialize)
)]
pub struct Source<'n, 't> {
  /// The filename.
  name: &'n str,
  /// The source text of the file, typically a `&str`.
  text: &'t str,
  /// The byte indices of line starts in the source code.
  line_starts: Vec<ByteIndex>,
}

impl<'n, 't> Source<'n, 't> {
  pub fn new(name: &'n str, text: &'t str) -> Self {
    let line_starts = line_starts(text.as_bytes())
        .map(|i| ByteIndex::from(i as u32))
        .collect();

    Source {
      name,
      text,
      line_starts
    }
  }


  /// Note: This function requires that
  ///   span.fragment == std::mem::transmute(
  ///     self.text.as_bytes()[span.start().into()..span.end().into()]
  ///   )
  pub fn fragment(&self, span: &Span) -> &str {
    unsafe {
      std::mem::transmute(&self.text.as_bytes()[span.start().into()..span.end().into()])
    }
  }


  /// Get a copy of the source (typically a slice).
  pub fn text(&self) -> &str {
    self.text
  }


  /// If the source text has $n$ lines, `last_line_index` returns $n+1$, as the last
  /// index-able "line" is the one following the actual last line of the source text.
  pub fn last_line_index(&self) -> LineIndex {
    LineIndex::new(self.line_starts.len()-1)
  }

  /// Given a `byte_index: ByteIndex`, returns the `LineIndex` of the line in which `byte_index`
  /// exists. If `byte_index` is more than one past the end, or if `byte_index` is one past the
  /// end and the file does not end with a newline, we return `None`, as there is no such line.
  pub fn line_index(&self, byte_index: ByteIndex) -> Result<LineIndex, LocationError> {
    let text_len: ByteIndex = self.text.as_bytes().len().into();

    if (text_len == 0usize.into())
        || (byte_index > text_len)
        || (
            (byte_index == text_len)
            & (self.text.as_bytes()[Into::<usize>::into(text_len) - 1] != b'\n')
        ) {
      Err(
          LocationError::OutOfBounds {
          given: byte_index,
          source: self
        }
      )
    } else {
      let result = // the following match
      match self.line_starts.binary_search(&byte_index) {

        // `byte_index` is the start of a line.
        Ok(line) => LineIndex::from(line as u32),

        // `byte_index` is not itself the start of a line, but `Err` contains "the
        // index where a matching element could be inserted while maintaining
        // sorted order," which is the start of the line following `byte_index`.
        // Thus, the line containing `byte_index` must be `next_line` - 1.
        Err(next_line) => LineIndex::from(next_line as u32 - 1),

      };

      Ok(result)
    }
  }

  /// Gives the (row, column) location of `idx` where column is actually the
  /// number of bytes between line start and position defined by idx.
  /// Call with span.start() to use with Span. If `idx` refers to a position
  /// past the end of the file, it returns an error.
  pub fn location_in_bytes(&self, idx: ByteIndex) -> Result<Location, LocationError> {
    let line = self.line_index(idx)?;
    // If `self.line_index(idx)` succeeds, `self.line_start(..)` is guaranteed to succeed, so the
    // (outer) unwrap is safe.
    let line_start = self.line_start( line ).unwrap();

    Ok(
      Location{
        line,
        column: (idx - line_start).to_usize().into() // This is the offset from BOL in BYTES.
      }
    )

  }

  /// Gives the (row, column) location of `idx` where column is the count of UTF-8 chars between
  /// line start and position defined by idx. Call with span.start() to use with Span. An error is
  /// returned if `idx` refers to a position past the end of the file. (See `self.line_index(..)`.)
  pub fn location_utf8(&self, idx: ByteIndex) -> Result<Location, LocationError> {
    let location_in_bytes = self.location_in_bytes(idx)?;
    let start_of_line = (idx.0 - location_in_bytes.column.0) as usize;

    // The column in UTF-8 characters
    let column: ColumnIndex =
      (
        num_chars(
          &self.text.as_bytes()[
              start_of_line .. idx.0 as usize
          ]
        )
      ).into();
    
    Ok(
      Location{
        line: location_in_bytes.line,
        column
      }
    )
  }

  /// Same as location_utf8(..), but uses a fast naive method of counting UTF-8 characters.
  pub fn location_naive_utf8(&self, idx: ByteIndex) -> Result<Location, LocationError> {
    let location_in_bytes = self.location_in_bytes(idx)?;
    // The offset in bytes of the start of the line `idx` lives on
    let start_of_line = (idx.0 - location_in_bytes.column.0) as usize;
    // The column in UTF-8 characters
    let column: ColumnIndex =
        (
          naive_num_chars(
            &self.text.as_bytes()[
                start_of_line .. idx.0 as usize
                ]
          ) + 1
        ).into();

    Ok(
      Location{
        line: location_in_bytes.line,
        column
      }
    )
  }


  pub fn line_span(&self, line_index: LineIndex) -> Result<Span, LineIndexOutOfBoundsError> {
    let line_start = self.line_start(line_index)?;
    let next_line_start = self.line_start(line_index + LineOffset::new(1))?;

    Ok(
      Span::new(
        line_start,
        next_line_start - line_start,
        self
      )
    )
  }


  /// Returns the `ByteIndex` to the start of line number `line_index`, where `line_index`
  /// starts at $0$. If the source text ends in a newline, the `ByteIndex` of the
  /// "line"  with `line_index` will be one past the end. This method can be thought of as a
  /// bounds-checked access of `self.line_starts`.
  pub fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, LineIndexOutOfBoundsError> {
    match line_index.cmp(&self.last_line_index()) {

      Ordering::Less
      | Ordering::Equal => Ok(self.line_starts[usize::from(line_index)]),

      // Ordering::Equal => Ok(ByteIndex::new(self.len())),

      Ordering::Greater => Err(
        LineIndexOutOfBoundsError {
          given: line_index,
          max: self.last_line_index(),
        }
      ),
    }
  }


  /// Create a new Nom nature `LocatedSpan` (`LSpan` here) from this `Source`'s text.
  #[cfg(feature = "nom-parsing")]
  pub fn source_located_span(&'t self) -> LSpan<'n, 't> {
    LSpan::new_extra(
      self.text,
      self,
    )
  }


  /// Convert a `Span` to Nom's native `LocatedSpan` (`LSpan` here)
  #[cfg(any(feature = "nom-parsing", feature = "reporting"))]
  pub fn span_to_located(&'t self, span: &'t Span) -> LSpan<'n, 't> {
    unsafe {
      LSpan::<'n, 't>::new_from_raw_offset(
        span.start().into(),
        self.line_index(span.start()).unwrap().into(),
        span.fragment(),
        self
      )
    }
  }


  pub fn name(&self) -> &'n str {
    &self.name
  }

  pub const fn start(&self) -> ByteIndex{
    ByteIndex(0u32)
  }

  /// The length of the text in bytes.
  pub fn len(&self) -> usize {
    self.text.as_bytes().len()
  }

  pub fn end(&self) -> ByteIndex{
    ByteIndex::new(self.len())
  }

  /// Gives a span for the given range. The span is clipped if range is not a subset of the text
  /// range. That is, the span returned will be the intersection of the given range and the
  /// largest valid range. Thus the span may be empty.
  ///
  /// Note: `Source` does not implement Nom's `Slice` trait, as that trait requires the return
  /// type to be `Self`. We want `slice` to return a `Span`, not a `Source`.
  pub fn slice<RangeType>(&self, range: RangeType) -> Span
    where RangeType : RangeBounds<usize>
  {
    let range_start =
        match range.start_bound() {
          Bound::Included(s) => { *s }
          Bound::Excluded(s) => { *s + 1 }
          Bound::Unbounded => { 0 }
        };
    let range_end =
        match range.end_bound() {
          Bound::Included(s) => { *s }
          Bound::Excluded(s) => { *s - 1 }
          Bound::Unbounded => { self.len() }
        };

    let start: ByteIndex = max(self.start().into(), range_start).into();
    let length: ByteOffset = max(0, min(self.len().into(), range_end - range_start)).into();

    Span::new(start, length, self)
  }

  pub fn source_span(&self) -> Span {
    Span::new(
      ByteIndex::default(),
      self.len(),
      self
    )
  }

  /// Returns the `Location` (row+col) of the given `ByteIndex`.
  pub fn location(&self, byte_index: ByteIndex) -> Result<Location, LocationError> {
    let line_index = self.line_index(byte_index)?;

    // This `unwrap` is ok, because `line_index` is guaranteed to be an existing line.
    let line_start_index = self.line_start(line_index).unwrap();
    let line_src: &str = &self.text[line_start_index.into()..=byte_index.into()];

    Ok(
        Location {
        line: line_index,
        column: ColumnIndex::from(line_src.chars().count() as u32),
      }
    )
  }

}


impl Display for Source<'_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Source {{ name: \"{}\", text: \"{}\" }}", clip(self.name, 20), clip(self.text, 20))
  }
}

// Reuses `Display::fmt()`
impl Debug for Source<'_, '_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}

/**
  `Files` is a trait from `codespan-reporting` and is required if `Span` is to be used with
  `codespan-reporting`.
*/
#[cfg(feature = "reporting")]
#[allow(unused_variables)]
impl<'n: 't, 't> Files<'t> for Source<'n, 't> {
  /// A unique identifier for files in the file provider. This will be used
  /// for rendering `diagnostic::Label`s in the corresponding source files.
  type FileId = ();
  /// The user-facing name of a file, to be displayed in diagnostics.
  type Name = &'n str;
  /// The source code of a file.
  type Source = &'t str;

  /// The user-facing name of a file.
  // #[allow(unused_variables)]
  fn name(&self, id: Self::FileId) -> Option<Self::Name> {
    Some(self.name)
  }

  /// The source code of a file.
  // #[allow(unused_variables)]
  fn source(&'t self, id: Self::FileId) -> Option<Self::Source> {
    Some(self.text())
  }

  /// The index of the line at the given byte index.
  // #[allow(unused_variables)]
  fn line_index(&self, id: Self::FileId, byte_index: usize) -> Option<usize> {
    Some((self.line_index(byte_index.into()).ok()?).into())
  }

  /// The byte range of line in the source of the file.
  fn line_range(&self, id: Self::FileId, line_index: usize) -> Option<Range<usize>> {
    if line_index >= self.line_starts.len() {
      return None;
    }

    // Recall self.line_starts gives line numbers which start at 1.
    let start = self.line_starts[line_index];
    if  start ==( self.line_starts.len() - 1usize).into() {
      return Some(start.into()..start.into());
    }

    Some(start.into()..self.line_starts[line_index + 1].into())
  }
}


/// Produces a list containing 0 followed by the index of the byte following every instance of
/// `b'\n'`. These are the indices of the beginning of every line. Note that if the file ends
/// with a newline, then the last index in this list will be one past the end of the text.
// NOTE: this is copied from `codespan_reporting::files::line_starts` and should be kept in sync.
fn line_starts<'s>(source: &'s [u8]) -> impl 's + Iterator<Item=usize>
{
  std::iter::once(0).chain(Memchr::new(b'\n', source).map(|i| i + 1))
}



/// A utility function that clips `text` if necessary so that the result does not exceed
/// length `n`. It does so by replacing a sufficient amount of the middle of the string with
/// a single "…" to make a new string of the form "prefix…postfix". If `n` is less than 2 and
/// `text.len() > 2`, this function makes no sense, and so the original string is returned
/// unclipped.
fn clip(text: &str, n: usize) -> String {
  let text_len = text.len();

  if text_len <= n || n < 2 {
    return text.to_string();
  }

  // The length of the prefix and postfix of the clipped string. If `n`
  // is even, we give the prefix one more character than the postfix.
  let half_n = n/2;

  format!(
    "{}…{}",
    &text[0..half_n],
    &text[text_len - half_n + ((n+1)%2) .. text_len]
  )
}
