
#[cfg(feature = "nom-parsing")]
use nom::{
  // ParseTo,
  // error::ErrorKind,
  // Compare,
  // CompareResult,
  FindSubstring,
  // FindToken,
  // InputIter,
  // InputTake,
  // InputTakeAtPosition,
  // Offset,
  Slice
};


use crate::{ByteIndex, ColumnNumber, LineNumber};
use crate::source::Source;
use crate::Span;

static SOURCE_NAME: &str = "The Second Coming By William Butler Yeats";
static SOURCE_TEXT: &str =
"Turning and turning in the widening gyre
The falcon cannot hear the falconer;
Things fall apart; the centre cannot hold;
Mere anarchy is loosed upon the world,
The blood-dimmed tide is loosed, and everywhere
The ceremony of innocence is drowned;
The best lack all conviction, while the worst
Are full of passionate intensity.";


// region codespan
// Tests adapted from codespan of the set operations on `Span`s.

#[test]
fn test_merge() {
  let source = Source::new(SOURCE_NAME, SOURCE_TEXT);

  // overlap
  let a: Span = source.slice(1..5);
  let b: Span = source.slice(3..10);
  assert_eq!(a.merge(b).unwrap(), source.slice(1..10));
  assert_eq!(b.merge(a).unwrap(), source.slice(1..10));

  // subset
  let two_four = source.slice(2..4);
  assert_eq!(a.merge(two_four).unwrap(), source.slice(1..5));
  assert_eq!(two_four.merge(a).unwrap(), source.slice(1..5));

  // disjoint
  let ten_twenty = source.slice(10..20);
  assert_eq!(a.merge(ten_twenty).unwrap(), source.slice(1..20));
  assert_eq!(ten_twenty.merge(a).unwrap(), source.slice(1..20));

  // identity
  assert_eq!(a.merge(a).unwrap(), a);
}

#[test]
fn test_disjoint() {
  let source = Source::new(SOURCE_NAME, SOURCE_TEXT);

  // overlap
  let a = source.slice(1..5);
  let b = source.slice(3..10);
  assert!(!a.disjoint(b));
  assert!(!b.disjoint(a));

  // subset
  let two_four = source.slice(2..4);
  assert!(!a.disjoint(two_four));
  assert!(!two_four.disjoint(a));

  // disjoint
  let ten_twenty = source.slice(10..20);
  assert!(a.disjoint(ten_twenty));
  assert!(ten_twenty.disjoint(a));

  // identity
  assert!(!a.disjoint(a));

  // off by one (upper bound)
  let c = source.slice(5..10);
  assert!(a.disjoint(c));
  assert!(c.disjoint(a));
  // off by one (lower bound)
  let d = source.slice(0..1);
  assert!(a.disjoint(d));
  assert!(d.disjoint(a));
}

// endregion codespan


// region located span


#[test]
fn calculate_columns() {
  let source = Source::new(SOURCE_NAME, SOURCE_TEXT);
  let span = source.source_span();

  let found_idx = span.find_substring("falconer").unwrap();
  let location = span.slice(found_idx..).location().unwrap();
  assert_eq!(location.line_index.number(), LineNumber::from(2));
  assert_eq!(location.column_index.number(), ColumnNumber::from(28));
}

#[test]
fn calculate_columns_accurately_with_non_ascii_chars() {
  // Each kana character is three bytes long.
  let source = Source::new("Japanese kana", "メカジキ");
  // `source.slice(6..)` == "ジキ", which starts at column number 3.
  let location = source.slice(6..).location().unwrap();
  assert_eq!(location.column_index.number(), ColumnNumber(3));
}

#[test]
fn error_when_getting_column_if_offset_is_too_big() {
  let source = Source::new("some text", "");
  let location = source.location_in_bytes(ByteIndex(28));

  assert_eq!(location.is_err(), true);
}

/*

#[cfg(feature = "nom")]
#[test]
fn iterate_indices() {
  let source = Source::new("", "Turning");
  let span = source.source_span();

  assert_eq!(
    span.iter_indices().collect::<Vec<(usize, char)>>(),
    vec![(0, 'T'), (1, 'u'), (2, 'r'), (3, 'n'), (4, 'i'), (5, 'n'), (6, 'g')]
  );

  let source = Source::new("", "");
  let  span = source.source_span();

  assert_eq!(
    span.iter_indices().collect::<Vec<(usize, char)>>(),
    vec![]
  );
}

#[cfg(feature = "alloc")]
#[test]
fn iterate_elements() {
  let str_slice = StrSpan::new("foobar");
  assert_eq!(
    str_slice.iter_elements().collect::<Vec<char>>(),
    vec!['f', 'o', 'o', 'b', 'a', 'r']
  );
  assert_eq!(
    StrSpan::new("").iter_elements().collect::<Vec<char>>(),
    vec![]
  );
}


#[test]
fn compare_elements() {
  let source = Source::new("", "Turning");
  let span_a = source.source_span();

  assert_eq!(span.compare("Turning"), CompareResult::Ok);
  assert_eq!(StrSpan::new("foobar").compare("bar"), CompareResult::Error);
  assert_eq!(StrSpan::new("foobar").compare("foobar"), CompareResult::Ok);
  assert_eq!(
    StrSpan::new("foobar").compare_no_case("fooBar"),
    CompareResult::Ok
  );
  assert_eq!(
    StrSpan::new("foobar").compare("foobarbaz"),
    CompareResult::Incomplete
  );
  assert_eq!(
    BytesSpan::new(b"foobar").compare(b"foo" as &[u8]),
    CompareResult::Ok
  );
}

#[test]
#[allow(unused_parens)]
fn find_token() {
  let source = Source::new("", "Turning");
  let span = source.source_span();

  assert!(span.fragment().find_token('a'));
  assert!(StrSpan::new("foobar").find_token(b'a'));
  assert!(StrSpan::new("foobar").find_token(&(b'a')));
  assert!(!StrSpan::new("foobar").find_token('c'));
  assert!(!StrSpan::new("foobar").find_token(b'c'));
  assert!(!StrSpan::new("foobar").find_token((&b'c')));

  assert!(BytesSpan::new(b"foobar").find_token(b'a'));
  assert!(BytesSpan::new(b"foobar").find_token(&(b'a')));
  assert!(!BytesSpan::new(b"foobar").find_token(b'c'));
  assert!(!BytesSpan::new(b"foobar").find_token((&b'c')));
}

#[test]
fn find_substring() {
  assert_eq!(StrSpan::new("foobar").find_substring("bar"), Some(3));
  assert_eq!(StrSpan::new("foobar").find_substring("baz"), None);
  assert_eq!(BytesSpan::new(b"foobar").find_substring("bar"), Some(3));
  assert_eq!(BytesSpan::new(b"foobar").find_substring("baz"), None);
}

#[cfg(feature = "alloc")]
#[test]
fn parse_to_string() {
  assert_eq!(
    StrSpan::new("foobar").parse_to(),
    Some("foobar".to_string())
  );
  assert_eq!(
    BytesSpan::new(b"foobar").parse_to(),
    Some("foobar".to_string())
  );
}

// https://github.com/Geal/nom/blob/eee82832fafdfdd0505546d224caa466f7d39a15/src/util.rs#L710-L720
#[test]
fn calculate_offset_for_u8() {
  let s = b"abcd123";
  let a = &s[..];
  let b = &a[2..];
  let c = &a[..4];
  let d = &a[3..5];
  assert_eq!(a.offset(b), 2);
  assert_eq!(a.offset(c), 0);
  assert_eq!(a.offset(d), 3);
}

// https://github.com/Geal/nom/blob/eee82832fafdfdd0505546d224caa466f7d39a15/src/util.rs#L722-L732
#[test]
fn calculate_offset_for_str() {
  let s = StrSpan::new("abcřèÂßÇd123");
  let a = s.slice(..);
  let b = a.slice(7..);
  let c = a.slice(..5);
  let d = a.slice(5..9);
  assert_eq!(a.offset(&b), 7);
  assert_eq!(a.offset(&c), 0);
  assert_eq!(a.offset(&d), 5);
}

#[test]
fn take_chars() {
  let s = StrSpanEx::new_extra("abcdefghij", "extra");
  assert_eq!(
    s.take(5),
    StrSpanEx {
      offset: 0,
      line: 1,
      fragment: "abcde",
      extra: "extra",
    }
  );
}

#[test]
fn take_split_chars() {
  let s = StrSpanEx::new_extra("abcdefghij", "extra");
  assert_eq!(
    s.take_split(5),
    (
      StrSpanEx {
        offset: 5,
        line: 1,
        fragment: "fghij",
        extra: "extra",
      },
      StrSpanEx {
        offset: 0,
        line: 1,
        fragment: "abcde",
        extra: "extra",
      }
    )
  );
}

type TestError<'a, 'b> = (LocatedSpan<&'a str, &'b str>, nom::error::ErrorKind);

#[test]
fn split_at_position() {
  let s = StrSpanEx::new_extra("abcdefghij", "extra");
  assert_eq!(
    s.split_at_position::<_, TestError>(|c| { c == 'f' }),
    Ok((
      StrSpanEx {
        offset: 5,
        line: 1,
        fragment: "fghij",
        extra: "extra",
      },
      StrSpanEx {
        offset: 0,
        line: 1,
        fragment: "abcde",
        extra: "extra",
      }
    ))
  );
}

// TODO also test split_at_position with an error

#[test]
fn split_at_position1() {
  let s = StrSpanEx::new_extra("abcdefghij", "extra");
  assert_eq!(
    s.split_at_position1::<_, TestError>(|c| { c == 'f' }, ErrorKind::Alpha),
    s.split_at_position::<_, TestError>(|c| { c == 'f' }),
  );
}

#[test]
fn capture_position() {
  use super::position;
  use nom::bytes::complete::{tag, take_until};
  use nom::IResult;

  fn parser<'a>(s: StrSpan<'a>) -> IResult<StrSpan<'a>, (StrSpan<'a>, &'a str)> {
    let (s, _) = take_until("def")(s)?;
    let (s, p) = position(s)?;
    let (s, t) = tag("def")(s)?;
    Ok((s, (p, t.fragment)))
  }

  let s = StrSpan::new("abc\ndefghij");
  let (_, (s, t)) = parser(s).unwrap();
  assert_eq!(s.offset, 4);
  assert_eq!(s.line, 2);
  assert_eq!(t, "def");
}

 */
// endregion


