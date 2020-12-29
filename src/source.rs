/*!
The `Source` struct represents a unit of source code, typically the contents of a single file.
The `Sources` struct is a collection of `Sources` that can be queried in various ways. In
general, the `Sources` struct remains in scope during parsing, providing `Span`s to client code.
Client code need not interact with `Source`. Instead, client code hands off the source text to
the `Sources` instance in exchange for a `Span` covering the entirety of the source text. The
`Span` is used both as an input and an output: the input span is broken into tokan spans. A
`Span` knows its `Source` and can be queried for `&str`s and position/location data.

*/

// todo: Remove this `allow`
#[allow(unused_imports)]
use std::{
  ops::{Range, Bound, RangeBounds},
  fmt::{Display, Formatter},
  num::NonZeroU32,
  slice::SliceIndex,
  cmp::{min, max},
  mem::size_of_val
};


#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "nom-parsing")]
use nom_locate::LocatedSpan;
#[cfg(feature = "reporting")]
use codespan_reporting::files::Files;

use memchr::Memchr;
use bytecount::{naive_num_chars, num_chars};

// todo: Remove this `allow`
#[allow(unused_imports)]
use crate::{
  error::{LineIndexOutOfBoundsError, LocationError, NotASourceError},
  ByteIndex,
  ByteOffset,
  ColumnIndex,
  LineIndex,
  LineOffset,
  Location,
  RawIndex,
  Span,
  Slice,
  AsBytes,
};
use crate::ColumnNumber;


#[cfg(feature = "nom-parsing")]
type LSpan<'s> = LocatedSpan<&'s str, &'s Source<'s>>;


/// A file that is stored in the database.
#[derive(Debug, Clone, Eq, PartialEq)]
// `Serialize` is only implemented on `OsString` for windows/unix
#[cfg_attr(
all(feature = "serialization", any(windows, unix)),
derive(Deserialize, Serialize)
)]
pub struct Source<'s>
{
  /// The filename
  name: String,
  /// The source text of the file, typically a `&str`.
  text: &'s str,
  /// The starting byte indices in the source code.
  line_starts: Vec<ByteIndex>,
}

impl<'s> Source<'s> {
  pub fn new(name: String, text: &'s str) -> Self {
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


  /// Get a copy of the source (typically a slice)
  pub fn text(&self) -> &str {
    self.text
  }


  pub fn last_line_index(&self) -> LineIndex {
    LineIndex::new(self.line_starts.len())
  }

  pub fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
    match self.line_starts.binary_search(&byte_index) {
      // Found the start of a line
      Ok(line) => LineIndex::from(line as u32),
      Err(next_line) => LineIndex::from(next_line as u32 - 1),
    }
  }

  /// Gives BYTES between line start and position defined by idx.
  /// Call with span.start() to use with Span.
  pub fn column(&self, idx: ByteIndex) -> ColumnIndex {
    let line_start = self.line_start( self.line_index(idx) ).unwrap();
    (idx - line_start).to_usize().into()
  }

  /// Gives count of UTF-8 chars between line start and position defined by idx.
  /// Call with span.start() to use with Span.
  pub fn column_utf8(&self, idx: ByteIndex) -> ColumnNumber {
    let before_self = self.column(idx);
    (num_chars(
      &self.text.as_bytes()[
        (idx.0 - before_self.0) as usize .. idx.0 as usize
      ]
    ) + 1).into()
  }

  /// Gives fast count of UTF-8 chars between line start and position defined by idx.
  /// Call with span.start() to use with Span.
  pub fn column_naive_utf8(&self, idx: ByteIndex) -> ColumnNumber {
    let before_self = self.column(idx);
    (naive_num_chars(
      &self.text.as_bytes()[
          (idx.0 - before_self.0) as usize .. idx.0 as usize
          ]
    ) + 1).into()
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


  pub fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, LineIndexOutOfBoundsError> {
    use std::cmp::Ordering;

    match line_index.cmp(&self.last_line_index()) {
      Ordering::Less => Ok(self.line_starts[Into::<usize>::into(line_index)]),
      Ordering::Equal => Ok(ByteIndex(self.text.as_bytes().len() as u32)),
      Ordering::Greater => Err(LineIndexOutOfBoundsError {
        given: line_index,
        max: self.last_line_index(),
      }),
    }
  }


  #[cfg(feature = "nom-parsing")]
  pub fn source_located_span(&'s self) -> LSpan<'s> {
    LSpan::new_extra(
      self.text,
      self,
    )
  }


  #[cfg(any(feature = "nom-parsing", feature = "reporting"))]
  pub fn span_to_located(&'s self, span: &'s Span) -> LSpan<'s> {
    unsafe {
      LSpan::<'s>::new_from_raw_offset(
        span.start().into(),
        self.line_index(span.start()).into(),
        span.fragment(),
        self
      )
    }
  }


  pub fn name(&self) -> &String {
    &self.name
  }

  pub const fn start(&self) -> ByteIndex{
    ByteIndex(0u32)
  }

  pub fn len(&self) -> usize {
    unsafe {
      std::mem::size_of_val_raw(&self.text)
    }
  }

  pub fn end(&self) -> ByteIndex{
    ByteIndex::new(0usize + self.len())
  }

  pub(crate) fn slice<RangeType>(&'s self, range: RangeType) -> Span<'s>
    where RangeType : RangeBounds<usize>
  {
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

    Span::new(start, length, self)
  }

  pub fn source_span(&self) -> Span {
    Span::new(
      ByteIndex::default(),
      self.len(),
      self
    )
  }

  pub fn location(&self, byte_index: ByteIndex) -> Result<Location, LocationError<Span<'_>>> {
    let line_index = self.line_index(byte_index);
    let line_start_index =
        self.line_start(line_index)
            .map_err(|_| LocationError::OutOfBounds {
              given: byte_index,
              span: self.source_span(),
            })?;
    let line_src: &str = &self.text[line_start_index.into()..byte_index.into()];

    Ok(Location {
      line: line_index,
      column: ColumnIndex::from(line_src.chars().count() as u32),
    })
  }

}



#[cfg(feature = "reporting")]
#[allow(unused_variables)]
impl<'s> Files<'s> for Source<'s> {
  /// A unique identifier for files in the file provider. This will be used
  /// for rendering `diagnostic::Label`s in the corresponding source files.
  type FileId = ();
  /// The user-facing name of a file, to be displayed in diagnostics.
  type Name = String;
  /// The source code of a file.
  type Source = &'s str;

  /// The user-facing name of a file.
  // #[allow(unused_variables)]
  fn name(&'s self, id: Self::FileId) -> Option<Self::Name> {
    Some(self.name.clone())
  }

  /// The source code of a file.
  // #[allow(unused_variables)]
  fn source(&'s self, id: Self::FileId) -> Option<Self::Source> {
    Some(self.text())
  }

  /// The index of the line at the given byte index.
  // #[allow(unused_variables)]
  fn line_index(&'s self, id: Self::FileId, byte_index: usize) -> Option<usize> {
    Some(self.line_index(byte_index.into()).into())
  }

  /// The byte range of line in the source of the file.
  fn line_range(&'s self, id: Self::FileId, line_index: usize) -> Option<Range<usize>> {
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



// NOTE: this is copied from `codespan_reporting::files::line_starts` and should be kept in sync.
fn line_starts<'s>(source: &'s [u8]) -> impl 's + Iterator<Item=usize>
{
  // let nl_iter =;

  std::iter::once(0).chain(Memchr::new(b'\n', source).map(|i| i + 1))
}
