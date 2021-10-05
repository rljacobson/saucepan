/*!

  Provides a macro for implementing the `Slice` trait inteded to be used with (at least) `&str`
  and `&[u8]`. At present the macro is not used.

*/

#![macro_use]

use super::Slice;
use crate::{Span, ByteIndex, ByteOffset, Source};
use std::ops::{RangeBounds, Bound};
use std::cmp::{min, max};

#[macro_export]
macro_rules! parameterize {
  ($type_name:ident) => {
    $type_name<SourceType>
  };
  ($type_name:ty)=> {
    $type_name
  };

}

#[macro_export]
macro_rules! impl_slice_ranges {

  ($ParentType:ty) => {

impl<'s, &str, RangeType> Slice<RangeType> for $ParentType
  where RangeType: RangeBounds<usize>
{
  fn slice(&self, range: RangeType) -> Self {

    let range_start =
        match range.start_bound() {
          Bound::Included(s) => {*s}
          Bound::Excluded(s) => {s+1}
          Bound::Unbounded => {0}
        };
    let range_end =
        match range.end_bound() {
          Bound::Included(s) => {*s}
          Bound::Excluded(s) => {s-1}
          Bound::Unbounded => {self.len()}
        };

    let start: ByteIndex = max(self.start().into(), range_start).into();
    let length: ByteOffset = max(0, min(self.len().into(), range_end - range_start)).into();

    Span::new(start, length, self.source)
  }
}

};
}

// impl_slice_ranges!(&'s str);
// impl_slice_ranges!(&'s [u8]);
