/*!
The `Source` struct represents a unit of source code, typically the contents of a single file.
The `Sources` struct is a collection of `Sources` that can be queried in various ways. In
general, the `Sources` struct remains in scope during parsing, providing `Span`s to client code.
Client code need not interact with `Source`. Instead, client code hands off the source text to
the `Sources` instance in exchange for a `SourceID`. The `SourceID` is then used as an interned
`Source` used wherever a source needs to be referenced. To obtain `&str` slices, `Sources` is
queried with a `Span` or `Range` and `SourceID`.
*/

use std::ffi::{OsStr, OsString};
use std::num::NonZeroU32;
use std::fmt::{Display, Formatter};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::error::{LineIndexOutOfBoundsError, LocationError, SpanOutOfBoundsError};
use crate::{ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span};
use nom_locate::LocatedSpan;

type LSpan<'s> = LocatedSpan<&'s str, SourceID>;

/// A handle that points to a file in the database.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct SourceID(NonZeroU32);

impl SourceID {
  /// Offset of our `SourceID`'s numeric value to an index on `Sources::files`.
    ///
    /// This is to ensure the first `SourceID` is non-zero for memory layout optimisations (e.g.
    /// `Option<SourceID>` is 4 bytes)
  const OFFSET: u32 = 1;

  pub fn new(index: usize) -> SourceID {
    SourceID(NonZeroU32::new(index as u32 + Self::OFFSET).expect("file index cannot be stored"))
  }

  pub fn get(self) -> usize {
    (self.0.get() - Self::OFFSET) as usize
  }
}

impl Display for SourceID {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.get().fmt(f)
  }
}

/// A database of source files.
///
/// The `SourceType` generic parameter determines how source text is stored. Using [`String`] will have
/// `Sources` take ownership of all source text. Smart pointer types such as [`Cow<'_, str>`],
/// [`Rc<str>`] or [`Arc<str>`] can be used to share the source text with the rest of the program.
///
/// [`Cow<'_, str>`]: std::borrow::Cow
/// [`Rc<str>`]: std::rc::Rc
/// [`Arc<str>`]: std::sync::Arc
#[derive(Clone, Debug)]
pub struct Sources<SourceType> {
  sources: Vec<Source<SourceType>>,
}

impl<SourceType> Default for Sources<SourceType>
  where
      SourceType: AsRef<str>,
{
  fn default() -> Self {
    Self { sources: vec![] }
  }
}

impl<'a, SourceType: 'a> Sources<SourceType>
  where
      SourceType: AsRef<str>,
{
  /// Create a new, empty database of files.
  pub fn new() -> Self {
    Sources::<SourceType>::default()
  }

  /// Get a `&str` of the span.
  pub fn fragment(&self, span: Span) -> &str {
    self.get_unchecked(span.source_id).source_slice(span).unwrap()
  }

  /// Add a file to the database, returning the handle that can be used to
    /// refer to it again.
  pub fn add(&mut self, name: impl Into<OsString>, text: SourceType) -> SourceID {
    let source_id = SourceID::new(self.sources.len());
    self.sources.push(Source::new(name.into(), text.into(), source_id));
    source_id
  }

  /// Update a source file in place.
    ///
    /// This will mean that any outstanding byte indexes will now point to
    /// invalid locations.
  pub fn update(&mut self, source_id: SourceID, source: SourceType) {
    self.get_unchecked_mut(source_id).update(source.into())
  }

  /// Get the source file using the file id.
  pub fn get_unchecked(&self, source_id: SourceID) -> &Source<SourceType> {
    &self.sources[source_id.get()]
  }

  /// Get the source file using the file id.
  pub fn get(&self, source_id: SourceID) -> Option<&Source<SourceType>> {
    self.sources.get(source_id.get())
  }


  /// Get the source file using the file id.
  pub fn get_unchecked_mut(&mut self, source_id: SourceID) -> &mut Source<SourceType> {
    &mut self.sources[source_id.get()]
  }


  /// Get the source file using the file id.
  pub fn get_mut(&mut self, source_id: SourceID) -> Option<&mut Source<SourceType>> {
    self.sources.get_mut(source_id.get())
  }

  /// Get the name of the source file.
    ///
    /// ```rust
    /// use codespan::Sources;
    ///
    /// let name = "test";
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add(name, "hello world!");
    ///
    /// assert_eq!(files.name(source_id), name);
    /// ```
  pub fn name(&self, source_id: SourceID) -> &OsStr {
    self.get_unchecked(source_id).name()
  }

  /// Get the span at the given line index.
    ///
    /// ```rust
    /// use codespan::{Sources, LineIndex, LineIndexOutOfBoundsError, Span};
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// let line_sources = (0..5)
    ///     .map(|line| files.line_span(source_id, line))
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     line_sources,
    ///     [
    ///         Ok(Span::new(0, 4)),    // 0: "foo\n"
    ///         Ok(Span::new(4, 9)),    // 1: "bar\r\n"
    ///         Ok(Span::new(9, 10)),   // 2: ""
    ///         Ok(Span::new(10, 13)),  // 3: "baz"
    ///         Err(LineIndexOutOfBoundsError {
    ///             given: LineIndex::from(5),
    ///             max: LineIndex::from(4),
    ///         }),
    ///     ]
    /// );
    /// ```
  pub fn line_span(
    &self,
    source_id: SourceID,
    line_index: impl Into<LineIndex>,
  ) -> Result<Span, LineIndexOutOfBoundsError> {
    self.get_unchecked(source_id).line_span(line_index.into())
  }

  /// Get the line index at the given byte in the source file.
    ///
    /// ```rust
    /// use codespan::{Sources, LineIndex};
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.line_index(source_id, 0), LineIndex::from(0));
    /// assert_eq!(files.line_index(source_id, 7), LineIndex::from(1));
    /// assert_eq!(files.line_index(source_id, 8), LineIndex::from(1));
    /// assert_eq!(files.line_index(source_id, 9), LineIndex::from(2));
    /// assert_eq!(files.line_index(source_id, 100), LineIndex::from(3));
    /// ```
  pub fn line_index(&self, source_id: SourceID, byte_index: impl Into<ByteIndex>) -> LineIndex {
    self.get_unchecked(source_id).line_index(byte_index.into())
  }

  /// Get the location at the given byte index in the source file.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Sources, Location, LocationError, Span};
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.location(source_id, 0), Ok(Location::new(0, 0)));
    /// assert_eq!(files.location(source_id, 7), Ok(Location::new(1, 3)));
    /// assert_eq!(files.location(source_id, 8), Ok(Location::new(1, 4)));
    /// assert_eq!(files.location(source_id, 9), Ok(Location::new(2, 0)));
    /// assert_eq!(
    ///     files.location(source_id, 100),
    ///     Err(LocationError::OutOfBounds {
    ///         given: ByteIndex::from(100),
    ///         span: Span::new(0, 13),
    ///     }),
    /// );
    /// ```
  pub fn location(
    &self,
    source_id: SourceID,
    byte_index: impl Into<ByteIndex>,
  ) -> Result<Location, LocationError> {
    self.get_unchecked(source_id).location(byte_index.into())
  }

  /// Get the source of the file.
    ///
    /// ```rust
    /// use codespan::Sources;
    ///
    /// let source = "hello world!";
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add("test", source);
    ///
    /// assert_eq!(*files.source(source_id), source);
    /// ```
  pub fn source(&'a self, source_id: SourceID) -> &'a SourceType {
    self.get_unchecked(source_id).source()
  }

  /// Return the span of the full source.
    ///
    /// ```rust
    /// use codespan::{Sources, Span};
    ///
    /// let source = "hello world!";
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add("test", source);
    ///
    /// assert_eq!(files.source_span(source_id), Span::from_str(source));
    /// ```
  pub fn source_span(&self, source_id: SourceID) -> Span {
    self.get_unchecked(source_id).source_span()
  }

  pub fn source_located(&self, source_id: SourceID) -> Span {
    self.get_unchecked(source_id).source_span()
  }

  /// Return a slice of the source file, given a span.
    ///
    /// ```rust
    /// use codespan::{Sources, Span};
    ///
    /// let mut files = Sources::new();
    /// let source_id = files.add("test",  "hello world!");
    ///
    /// assert_eq!(files.source_slice(source_id, Span::new(0, 5)), Ok("hello"));
    /// assert!(files.source_slice(source_id, Span::new(0, 100)).is_err());
    /// ```
  pub fn source_slice(
    &self,
    source_id: SourceID,
    span: impl Into<Span>,
  ) -> Result<&str, SpanOutOfBoundsError> {
    self.get_unchecked(source_id).source_slice(span.into())
  }


  /// Transforms the given Span to an equivalent LocatedSpan
  pub fn span_to_located(&self, span: Span) -> LSpan {
    self.get_unchecked(span.source_id).span_to_located(span)
  }
}

#[cfg(feature = "reporting")]
impl<'a, Source> codespan_reporting::files::Files<'a> for Sources<Source>
  where
      Source: AsRef<str>,
{
  type FileId = SourceID;
  type Name = String;
  type Source = &'a str;

  fn name(&self, id: SourceID) -> Option<String> {
    use std::path::PathBuf;

    Some(PathBuf::from(self.name(id)).display().to_string())
  }

  fn source(&'a self, id: SourceID) -> Option<&'a str> {
    Some(self.source(id).as_ref())
  }

  fn line_index(&self, id: SourceID, byte_index: usize) -> Option<usize> {
    Some(self.line_index(id, byte_index as u32).to_usize())
  }

  fn line_range(&'a self, id: SourceID, line_index: usize) -> Option<std::ops::Range<usize>> {
    let span = self.line_span(id, line_index as u32).ok()?;

    Some(span.start().to_usize()..span.end().to_usize())
  }
}

/// A file that is stored in the database.
#[derive(Debug, Clone)]
// `Serialize` is only implemented on `OsString` for windows/unix
#[cfg_attr(
all(feature = "serialization", any(windows, unix)),
derive(Deserialize, Serialize)
)]
pub struct Source<SourceType> {
  /// The name of the file.
  name: OsString,
  /// The source text of the file.
  text: SourceType,
  /// The starting byte indices in the source code.
  line_starts: Vec<ByteIndex>,
  /// The source_id if this Source lives in a source container.
  source_id: SourceID
}

impl<SourceType> Source<SourceType>
  where
      SourceType: AsRef<str>,
{
  pub fn new(name: OsString, source: SourceType, source_id: SourceID) -> Self {
    let line_starts = line_starts(source.as_ref())
        .map(|i| ByteIndex::from(i as u32))
        .collect();

    Source {
      name,
      text: source,
      line_starts,
      source_id
    }
  }

  /// An unchecked version of `source_slice()`.
  pub fn fragment(&self, span: Span) -> &str {
    if span.source_id != self.source_id {
      panic!("Tried to slice a source with a span from another source.")
    }
    unsafe {
      self.text.as_ref().get_unchecked(span.start().into()..span.end().into())
    }
  }


  pub fn update(&mut self, text: SourceType) {
    let line_starts = line_starts(text.as_ref())
        .map(|i| ByteIndex::from(i as u32))
        .collect();
    self.text = text;
    self.line_starts = line_starts;
  }

  pub fn name(&self) -> &OsStr {
    &self.name
  }

  pub fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, LineIndexOutOfBoundsError> {
    use std::cmp::Ordering;

    match line_index.cmp(&self.last_line_index()) {
      Ordering::Less => Ok(self.line_starts[line_index.to_usize()]),
      Ordering::Equal => Ok(self.source_span().end()),
      Ordering::Greater => Err(LineIndexOutOfBoundsError {
        given: line_index,
        max: self.last_line_index(),
      }),
    }
  }

  pub fn last_line_index(&self) -> LineIndex {
    LineIndex::from(self.line_starts.len() as RawIndex)
  }

  pub fn line_span(&self, line_index: LineIndex) -> Result<Span, LineIndexOutOfBoundsError> {
    let line_start = self.line_start(line_index)?;
    let next_line_start = self.line_start(line_index + LineOffset::from(1))?;

    Ok(Span::new(line_start, next_line_start, self.source_id))
  }

  pub fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
    match self.line_starts.binary_search(&byte_index) {
      // Found the start of a line
      Ok(line) => LineIndex::from(line as u32),
      Err(next_line) => LineIndex::from(next_line as u32 - 1),
    }
  }

  pub fn location(&self, byte_index: ByteIndex) -> Result<Location, LocationError> {
    let line_index = self.line_index(byte_index);
    let line_start_index =
        self.line_start(line_index)
            .map_err(|_| LocationError::OutOfBounds {
              given: byte_index,
              span: self.source_span(),
            })?;
    let line_src = self
        .text
        .as_ref()
        .get(line_start_index.to_usize()..byte_index.to_usize())
        .ok_or_else(|| {
          let given = byte_index;
          if given >= self.source_span().end() {
            let span = self.source_span();
            LocationError::OutOfBounds { given, span }
          } else {
            LocationError::InvalidCharBoundary { given }
          }
        })?;

    Ok(Location {
      line: line_index,
      column: ColumnIndex::from(line_src.chars().count() as u32),
    })
  }

  pub fn source(&self) -> &SourceType {
    &self.text
  }

  pub fn source_span(&self) -> Span {
    Span::from_str(self.text.as_ref(), self.source_id)
  }

  pub fn source_located_span(&self) -> LSpan {
    unsafe{
      LSpan::new_from_raw_offset(
        self.source().start().into(),
        self.line_index(span.source_id, span.start()).into(),
        self.text.as_ref(),
        self.source_id,
      )
    }
    Span::from_str(, ).into()
  }


  pub fn source_slice(&self, span: Span) -> Result<&str, SpanOutOfBoundsError> {
    let start = span.start().to_usize();
    let end = span.end().to_usize();

    self.text.as_ref().get(start..end).ok_or_else(|| {
      let span = Span::from_str(self.text.as_ref(), self.source_id);
      SpanOutOfBoundsError { given: span, span }
    })
  }


  pub fn span_to_located(&self, span: Span) -> LSpan {
    unsafe{
      LSpan::new_from_raw_offset(
        span.start().to_usize(),
        self.line_index(span.start()).into(),
        self.fragment(span),
        span.source_id,
      )
    }
  }


}

// NOTE: this is copied from `codespan_reporting::files::line_starts` and should be kept in sync.
fn line_starts<'source>(source: &'source str) -> impl 'source + Iterator<Item=usize> {
  std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}

#[cfg(test)]
mod test {
  use super::*;

  const TEST_SOURCE: &str = "foo\nbar\r\n\nbaz";

  #[test]
  fn line_starts() {
    let mut files = Sources::<String>::new();
    let source_id = files.add("test", TEST_SOURCE.to_owned());

    assert_eq!(
      files.get_unchecked(source_id).line_starts,
      [
        ByteIndex::from(0),  // "foo\n"
        ByteIndex::from(4),  // "bar\r\n"
        ByteIndex::from(9),  // ""
        ByteIndex::from(10), // "baz"
      ],
    );
  }

  #[test]
  fn line_span_sources() {
    // Also make sure we can use `Arc` for source
    use std::sync::Arc;

    let mut files = Sources::<Arc<str>>::new();
    let source_id = files.add("test", TEST_SOURCE.into());

    let line_sources = (0..4)
        .map(|line| {
          let line_span = files.line_span(source_id, line).unwrap();
          files.source_slice(source_id, line_span)
        })
        .collect::<Vec<_>>();

    assert_eq!(
      line_sources,
      [Ok("foo\n"), Ok("bar\r\n"), Ok("\n"), Ok("baz")],
    );
  }
}
