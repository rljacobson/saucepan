use crate::{
  ByteIndex,
  ColumnIndex,
  LineIndex,
  LineOffset,
  Location,
  RawIndex,
  error::LineIndexOutOfBoundsError,
  error::LocationError,
  error::NotASourceError,
  Span,
  Source,
  AsBytes
};

#[cfg(feature = "reporting")]
use codespan_reporting::files::Files;


/**
A database of source files.

The `SourceType` generic parameter determines how source text is stored. Using [`String`] will have
`Sources` take ownership of all source text. Smart pointer types such as [`Cow<'_, str>`],
[`Rc<str>`] or [`Arc<str>`] can be used to share the source text with the rest of the program.

[`Cow<'_, str>`]: std::borrow::Cow
[`Rc<str>`]: std::rc::Rc
[`Arc<str>`]: std::sync::Arc
*/
#[derive(Clone, Debug)]
pub struct Sources<'s>
{
  sources: Vec<Source<'s>>,
}

impl<'s> Default for Sources<'s>
{
  fn default() -> Self {
    Self { sources: vec![] }
  }
}


impl<'s> Sources<'s>
{
  pub fn new() -> Self {
    Sources::<'s>::default()
  }

  /// Add a file to the database, returning a reference to the handle that can be used to refer to
  /// it again.
  pub fn add(&mut self, name: impl Into<String>, text: &'s str) -> &Source<'s> {
    self.sources.push(Source::new(name.into(), text));
    self.sources.last().unwrap()
  }

  pub unsafe fn get_unchecked(&self, source_id: usize) -> &Source<'s> {
    self.sources.get_unchecked(source_id)
  }

  /// Get the source file using the file id.
  pub fn get(&self, source_id: usize) -> Option<&Source<'s>> {
    self.sources.get(source_id)
  }


  /// Get the source file using the file id.
  pub unsafe fn get_unchecked_mut(&'s mut self, source_id: usize) -> &'s mut Source<'s> {
    self.sources.get_unchecked_mut(source_id)
  }


  /// Get the source file using the file id.
  pub fn get_mut(&'s mut self, source_id: usize) -> Option<&'s mut Source<'s>> {
    self.sources.get_mut(source_id)
  }
}


// It's not clear if this is useful anymore.
#[cfg(feature = "reporting")]
impl<'s> Files<'s> for Sources<'s>
{
  type FileId = usize;
  // Index into self.sources
  type Name = &'s String;
  type Source = &'s str;

  fn name(&'s self, id: Self::FileId) -> Option<Self::Name> {
    if id >= self.sources.len() {
      return None;
    }

    Some(self.sources[id].name())
  }

  fn source(&'s self, id: Self::FileId) -> Option<&str> {
    if self.sources.len() < id {
      None
    }
    else {
      Some(self.sources.get(id)?.text())
    }
  }

  fn line_index(&self, id: Self::FileId, byte_index: usize) -> Option<usize> {
    if id >= self.sources.len() {
      None
    } else {
      Some(self.sources[id].line_index(ByteIndex(byte_index as u32)).into())
    }
  }

  fn line_range(&'s self, id: Self::FileId, line_index: usize) -> Option<std::ops::Range<usize>> {
    if id >= self.sources.len() {
      None
    } else {
      self.sources[id].line_range((), line_index)
    }
  }
}
