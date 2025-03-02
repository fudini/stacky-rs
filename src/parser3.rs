use crate::types::{Backtrace, Entry, Location};
use nom::{
  bytes::complete::{tag, take_until, take_while1},
  character::complete::{digit1, hex_digit1, line_ending, newline, space1},
  combinator::{map, map_res, opt},
  multi::separated_list,
  sequence::tuple,
  IResult,
};

fn parse_entry(i: &str) -> IResult<&str, Entry> {
  let (i, (function, location)) = tuple((parse_top, maybe_parse_location))(i)?;
  let entry = Entry::new(function.to_string(), location);
  Ok((i, entry))
}

pub fn parse_backtrace(i: &str) -> IResult<&str, Backtrace> {
  map(
    separated_list(line_ending, parse_entry),
    |entries: Vec<Entry>| Backtrace::with_entries(entries),
  )(i)
}

fn parse_prefix(i: &str) -> IResult<&str, Option<&str>> {
  opt(take_while1(|c: char| !c.is_whitespace()))(i)
}

fn parse_symbol(i: &str) -> IResult<&str, &str> {
  take_until("\n")(i)
}

fn parse_top(i: &str) -> IResult<&str, &str> {
  let (i, _) = tuple((parse_prefix, space1, parse_index, tag(" - ")))(i)?;
  let (i, mut path) = parse_symbol(i)?;
  if let Some(last_index) = path.rfind("::") {
    path = path.split_at(last_index).0
  }
  Ok((i, path))
}

fn parse_index(i: &str) -> IResult<&str, &str> {
  let (i, _) = tuple((digit1, tag(":     0x")))(i)?;
  let (i, r) = hex_digit1(i)?;
  Ok((i, r))
}

fn parse_int(i: &str) -> IResult<&str, u32> {
  map_res(digit1, |i: &str| u32::from_str_radix(i, 10))(i)
}

fn maybe_parse_location(i: &str) -> IResult<&str, Option<Location>> {
  let r = parse_location(i)
    .map(|(i, location)| (i, Some(location)))
    .unwrap_or((i, None));
  Ok(r)
}

fn parse_location(i: &str) -> IResult<&str, Location> {
  let (i, _) = tuple((newline, parse_prefix, space1, tag("at ")))(i)?;
  let (i, (path, _, line, _, column)) =
    tuple((take_until(":"), tag(":"), parse_int, tag(":"), parse_int))(i)?;

  let location = Location {
    path: path.to_string(),
    line,
    column,
  };
  Ok((i, location))
}

#[test]
fn nom_location_parser_test() {
  let input = "\nprefix  at /file.rs:1:2";
  let (_i, parsed) = maybe_parse_location(input).unwrap();
  assert_eq!(
    parsed,
    Some(Location {
      path: "/file.rs".to_string(),
      line: 1,
      column: 2,
    })
  );

  let input = "\n  at /file.rs:1:2";
  let (_i, parsed) = maybe_parse_location(input).unwrap();
  assert_eq!(
    parsed,
    Some(Location {
      path: "/file.rs".to_string(),
      line: 1,
      column: 2,
    })
  );

  let input = "jibberish";
  let (_i, parsed) = maybe_parse_location(input).unwrap();
  assert!(parsed.is_none());
}

#[test]
fn nom_top_indexed_parser_test() {
  let input = "0:     0x55a98f23062c - std::foo::bar::123";
  parse_index(input).unwrap();
}

#[test]
fn nom_top_parser_test() {
  let input = "  0:     0x55a98f23062c - std::foo::bar::123\n";
  let _parsed = parse_top(input).unwrap();

  let input = "prefix1234 0:     0x55a98f23062c - std::foo::bar::123\n";
  let _parsed = parse_top(input).unwrap();
}

#[test]
fn nom_entry_parser_test() {
  let input = "  0:     0x55a98f23062c - std::foo::bar::123\n  at /rustc/348e/library/std/src/../../backtrace/src/backtrace/libunwind.rs:93:5\n";
  let _parsed = parse_top(input).unwrap();
}

#[test]
fn nom_full_parser_test() {
  let input = include_str!("./tests/fixtures/panic.txt");
  let (_i, backtrace) = parse_backtrace(input).unwrap();
  assert_eq!(backtrace.entries().len(), 9);

  let input = include_str!("./tests/fixtures/panic_prefixed.txt");
  let (_i, backtrace) = parse_backtrace(input).unwrap();
  assert_eq!(backtrace.entries().len(), 9);
}
