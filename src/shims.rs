/*!
Shims for the case that the feature `nom-parsing` is disabled. This module is conditionally
included in `lib.rs`.

This code is taken from `nom::traits`.
*/

use std::ops::{
  Range,
  RangeFrom,
  RangeTo,
  RangeFull,
};



/// Helper trait for types that can be viewed as a byte slice
pub trait AsBytes {
  /// casts the input type to a byte slice
  fn as_bytes(&self) -> &[u8];
}

impl<'a> AsBytes for &'a str {
  #[inline(always)]
  fn as_bytes(&self) -> &[u8] {
    <str as AsBytes>::as_bytes(self)
  }
}

impl AsBytes for str {
  #[inline(always)]
  fn as_bytes(&self) -> &[u8] {
    self.as_ref()
  }
}

impl<'a> AsBytes for &'a [u8] {
  #[inline(always)]
  fn as_bytes(&self) -> &[u8] {
    *self
  }
}

impl AsBytes for [u8] {
  #[inline(always)]
  fn as_bytes(&self) -> &[u8] {
    self
  }
}

macro_rules! as_bytes_array_impls {
  ($($N:expr)+) => {
    $(
      impl<'a> AsBytes for &'a [u8; $N] {
        #[inline(always)]
        fn as_bytes(&self) -> &[u8] {
          *self
        }
      }

      impl AsBytes for [u8; $N] {
        #[inline(always)]
        fn as_bytes(&self) -> &[u8] {
          self
        }
      }
    )+
  };
}

as_bytes_array_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}


/// slicing operations using ranges
///
/// this trait is loosely based on
/// `Index`, but can actually return
/// something else than a `&[T]` or `&str`
pub trait Slice<R> {
  /// slices self according to the range argument
  fn slice(&self, range: R) -> Self;
}

macro_rules! impl_fn_slice {
  ( $ty:ty ) => {
    fn slice(&self, range: $ty) -> Self {
      &self[range]
    }
  };
}

macro_rules! slice_range_impl {
  ( [ $for_type:ident ], $ty:ty ) => {
    impl<'a, $for_type> Slice<$ty> for &'a [$for_type] {
      impl_fn_slice!($ty);
    }
  };
  ( $for_type:ty, $ty:ty ) => {
    impl<'a> Slice<$ty> for &'a $for_type {
      impl_fn_slice!($ty);
    }
  };
}

macro_rules! slice_ranges_impl {
  ( [ $for_type:ident ] ) => {
    slice_range_impl! {[$for_type], Range<usize>}
    slice_range_impl! {[$for_type], RangeTo<usize>}
    slice_range_impl! {[$for_type], RangeFrom<usize>}
    slice_range_impl! {[$for_type], RangeFull}
  };
  ( $for_type:ty ) => {
    slice_range_impl! {$for_type, Range<usize>}
    slice_range_impl! {$for_type, RangeTo<usize>}
    slice_range_impl! {$for_type, RangeFrom<usize>}
    slice_range_impl! {$for_type, RangeFull}
  };
}

slice_ranges_impl! {str}
slice_ranges_impl! {[T]}
