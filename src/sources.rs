use crate::{error::{LineIndexOutOfBoundsError, LocationError, NotASourceError}, ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span, Source, AsBytes};

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
pub struct Sources<SourceType>
    where SourceType: Copy + AsBytes //+ Slice<usize>
{
  sources: Vec<Source<SourceType>>,
}

impl<'s, SourceType: 's> Default for Sources<SourceType>
  where SourceType: 's + Copy + AsBytes //+ Slice<usize>
{
  fn default() -> Self {
    Self { sources: vec![] }
  }
}


impl<'s, SourceType> Sources<SourceType>
  where SourceType: 's + Copy + AsBytes
{
  pub fn new() -> Self {
    Sources::<SourceType>::default()
  }

  /// Add a file to the database, returning a reference to the handle that can be used to refer to
  /// it again.
  pub fn add(&mut self, name: impl Into<String>, text: SourceType) -> &Source<SourceType> {
    let source = Source::new(name.into(), text);
    self.sources.push(source);
    &source
  }

  pub unsafe fn get_unchecked(&self, source_id: usize) -> &Source<SourceType> {
    self.sources.get_unchecked(source_id)
  }

  /// Get the source file using the file id.
  pub fn get(&self, source_id: usize) -> Option<&Source<SourceType>> {
    self.sources.get(source_id)
  }


  /// Get the source file using the file id.
  pub unsafe fn get_unchecked_mut(&'s mut self, source_id: usize) -> &'s mut Source<SourceType> {
    self.sources.get_unchecked_mut(source_id)
  }


  /// Get the source file using the file id.
  pub fn get_mut(&'s mut self, source_id: usize) -> Option<&'s mut Source<SourceType>> {
    self.sources.get_mut(source_id).as_deref_mut()
  }
}


// It's not clear if this is useful anymore.
#[cfg(feature = "reporting")]
impl<'s, SourceType> Files<'s> for Sources<SourceType>
  where SourceType: 's + Copy + AsBytes + AsRef<str>//+ Slice<usize>
{
  type FileId = usize;
  // Index into self.sources
  type Name = &'s String;
  type Source = SourceType;

  fn name(&'s self, id: Self::FileId) -> Option<Self::Name> {
    if id >= self.sources.len() {
      return None;
    }

    Some(self.sources[id].name())
  }

  fn source(&'s self, id: Self::FileId) -> Option<SourceType> {
    Sources::source(self, id)
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
