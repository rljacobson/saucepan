use crate::{
  ByteIndex,
  Source,
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
pub struct Sources<'n, 't>
{
  sources: Vec<Source<'n, 't>>,
}

impl<'n, 't> Default for Sources<'n, 't>
{
  fn default() -> Self {
    Self { sources: vec![] }
  }
}


impl<'n, 't> Sources<'n, 't> {

  pub fn new() -> Self {
    Sources::<'n, 't>::default()
  }

  /// Add a file to the database, returning a reference to the handle that can be used to refer to
  /// it again.
  pub fn add(&mut self, name: &'n str, text: &'t str) -> &Source<'n, 't> {
    self.sources.push(Source::new(name, text));
    self.sources.last().unwrap()
  }

  pub unsafe fn get_unchecked(&self, source_id: usize) -> &Source<'n, 't> {
    self.sources.get_unchecked(source_id)
  }

  /// Get the source file using the file id.
  pub fn get(&self, source_id: usize) -> Option<&Source<'n, 't>> {
    self.sources.get(source_id)
  }


  /// Get the source file using the file id.
  pub unsafe fn get_unchecked_mut(& mut self, source_id: usize) -> &mut Source<'n, 't> {
    self.sources.get_unchecked_mut(source_id)
  }


  /// Get the source file using the file id.
  pub fn get_mut(&mut self, source_id: usize) -> Option<&mut Source<'n, 't>> {
    self.sources.get_mut(source_id)
  }
}


// It's not clear if this is useful anymore.
#[cfg(feature = "reporting")]
impl<'n: 't, 't> Files<'t> for Sources<'n, 't>
{
  type FileId = usize;
  // Index into self.sources
  type Name = &'n str;
  type Source = &'t str;

  fn name(&self, id: Self::FileId) -> Option<Self::Name> {
    if id >= self.sources.len() {
      return None;
    }

    Some(self.sources[id].name())
  }

  fn source(&self, id: Self::FileId) -> Option<&str> {
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
      Some((self.sources[id].line_index(ByteIndex(byte_index as u32)).ok()?).into())
    }
  }

  fn line_range(&self, id: Self::FileId, line_index: usize) -> Option<std::ops::Range<usize>> {
    if id >= self.sources.len() {
      None
    } else {
      self.sources[id].line_range((), line_index)
    }
  }
}
