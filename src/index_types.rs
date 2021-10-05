//! Wrapper types that specify positions in a source file

use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};


/// We use a 32-bit integer here for space efficiency, assuming we won't be working with sources
/// larger than 4GB.
pub type RawIndex = u32;
pub type RawOffset = i64;


#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct LineIndex(pub RawIndex);

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct LineNumber(pub RawIndex);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct LineOffset(pub RawOffset);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ColumnIndex(pub RawIndex);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ColumnNumber(pub RawIndex);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ColumnOffset(pub RawOffset);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ByteIndex(pub RawIndex);


#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ByteOffset(pub RawOffset);


/// A relative offset between two indices
pub trait Offset: Copy + Ord
  where
      Self: Neg<Output=Self>,
      Self: Add<Self, Output=Self>,
      Self: AddAssign<Self>,
      Self: Sub<Self, Output=Self>,
      Self: SubAssign<Self>,
{
  const ZERO: Self;
}


pub trait Index: Copy + Ord
  where
      Self: Add<<Self as Index>::Offset, Output=Self>,
      Self: AddAssign<<Self as Index>::Offset>,
      Self: Sub<<Self as Index>::Offset, Output=Self>,
      Self: SubAssign<<Self as Index>::Offset>,
      Self: Sub<Self, Output=<Self as Index>::Offset>,
{
  type Offset: Offset;
}

/// Implement debug/display, type conversion, and index->number  conversion. Indices are 0 based,
/// while numbers are 1 based. One can think of "numbers" as ordinals, e.g. first==1, second==2,
/// etc. Indices are then cardinals.
macro_rules! impl_number {
  ($Number:ident, $Index:ident) => {

    impl $Index {
      pub const fn number(self) -> $Number {
        $Number(self.0 + 1)
      }
    }


    impl fmt::Debug for $Number {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", stringify!($Number), "(")?;
        self.0.fmt(f)?;
        write!(f, ")")
      }
    }


    impl fmt::Display for $Number {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
      }
    }

    impl From<usize> for $Number{
      #[inline]
      fn from(n: usize) -> $Number {
          $Number(n as RawIndex)
      }
    }
  }
}

/// Implement type conversions, constructors, and arithmetic for a given (index, offset) pair.
macro_rules! impl_index {
  ($Index:ident, $Offset:ident) => {

    impl $Index {
      pub fn new(n: usize) -> Self {
        $Index(n as RawIndex)
      }
    }


    impl $Offset{
      pub fn from_char_len(ch: char) -> $Offset {
        $Offset(ch.len_utf8() as RawOffset)
      }

      pub fn from_str_len(value: &str) -> $Offset {
        $Offset(value.len() as RawOffset)
      }

      pub const fn to_usize(self) -> usize {
        self.0 as usize
      }

      pub fn new(n: usize) -> Self {
        $Offset(n as RawOffset)  //as RawOffset)
      }

    }


    impl fmt::Debug for $Offset {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", stringify!($Offset), "(")?;
        self.0.fmt(f)?;
        write!(f, ")")
      }
    }


    impl fmt::Display for $Offset {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
      }
    }


    impl fmt::Display for $Index {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
      }
    }


    impl fmt::Debug for $Index {
      fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", stringify!($Index), "(")?;
        self.0.fmt(f)?;
        write!(f, ")")
      }
    }


    // region `From`
    impl From<RawOffset> for $Offset {
        #[inline]
        fn from(i: RawOffset) -> Self {
            $Offset(i)
        }
    }

    impl From<RawIndex> for $Index {
        #[inline]
        fn from(i: RawIndex) -> Self {
            $Index(i)
        }
    }

    impl From<usize> for $Offset {
      #[inline]
      fn from(i: usize) -> Self {
          $Offset(i as RawOffset)
      }
    }

    impl From<usize> for $Index {
        #[inline]
        fn from(i: usize) -> Self {
            $Index(i as RawIndex)
        }
    }


    impl From<$Index> for RawIndex {
        #[inline]
        fn from(index: $Index) -> RawIndex {
            index.0
        }
    }

    impl From<$Offset> for RawOffset {
        #[inline]
        fn from(offset: $Offset) -> RawOffset {
            offset.0
        }
    }

    impl From<$Index> for usize {
        #[inline]
        fn from(index: $Index) -> usize {
            index.0 as usize
        }
    }

    impl From<$Offset> for usize {
        #[inline]
        fn from(offset: $Offset) -> usize {
            offset.0 as usize
        }
    }
    // endregion

    impl Offset for $Offset {
        const ZERO: $Offset = $Offset(0);
    }

    impl Index for $Index {
        type Offset = $Offset;

    }


    // region Arithmetic
    impl Add<$Offset> for $Index {
        type Output = $Index;

        #[inline]
        fn add(self, rhs: $Offset) -> $Index {
            $Index((self.0 as RawOffset + rhs.0) as RawIndex)
        }
    }

    impl AddAssign<$Offset> for $Index {
        #[inline]
        fn add_assign(&mut self, rhs: $Offset) {
            *self = *self + rhs;
        }
    }

    impl Neg for $Offset {
        type Output = $Offset;

        #[inline]
        fn neg(self) -> $Offset {
            $Offset(-self.0)
        }
    }

    impl Add<$Offset> for $Offset {
        type Output = $Offset;

        #[inline]
        fn add(self, rhs: $Offset) -> $Offset {
            $Offset(self.0 + rhs.0)
        }
    }

    impl AddAssign<$Offset> for $Offset {
        #[inline]
        fn add_assign(&mut self, rhs: $Offset) {
            self.0 += rhs.0;
        }
    }

    impl Sub<$Offset> for $Offset {
        type Output = $Offset;

        #[inline]
        fn sub(self, rhs: $Offset) -> $Offset {
            $Offset(self.0 - rhs.0)
        }
    }

    impl SubAssign<$Offset> for $Offset {
        #[inline]
        fn sub_assign(&mut self, rhs: $Offset) {
            self.0 -= rhs.0;
        }
    }

    impl Sub for $Index {
        type Output = $Offset;

        #[inline]
        fn sub(self, rhs: $Index) -> $Offset {
            $Offset(self.0 as RawOffset - rhs.0 as RawOffset)
        }
    }

    impl Sub<$Offset> for $Index {
        type Output = $Index;

        #[inline]
        fn sub(self, rhs: $Offset) -> $Index {
            $Index((self.0 as RawOffset - rhs.0 as RawOffset) as u32)
        }
    }

    impl SubAssign<$Offset> for $Index {
        #[inline]
        fn sub_assign(&mut self, rhs: $Offset) {
            self.0 = (self.0 as RawOffset - rhs.0) as RawIndex;
        }
    }
    // endregion

  };
}

impl_index!(LineIndex, LineOffset);
impl_index!(ColumnIndex, ColumnOffset);
impl_index!(ByteIndex, ByteOffset);

impl_number!(ColumnNumber, ColumnIndex);
impl_number!(LineNumber, LineIndex);
