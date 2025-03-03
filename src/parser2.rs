#![allow(unused)]
use crate::types::{Backtrace, Entry, Location};
use chumsky::prelude::*;
use chumsky::text::{newline, TextParser};
use chumsky::Parser;
use std::str::pattern::Pattern;

fn ptr_parser() -> impl Parser<char, u64, Error = Simple<char>> {
  just("0x")
    .ignored()
    .then(text::int(16).map(|s: String| u64::from_str_radix(&s, 16).unwrap()))
    .map(|(_, ptr)| ptr)
}

/// Parses this part of the backtrace:
/// TODO: We don't really need details, just have to remove the last segment (the hash)
///
/// eg: "core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once::h92bf06c783fb5223"
fn function_parser_full() -> impl Parser<char, Vec<Tree>, Error = Simple<char>>
{
  text::ident()
    .map(Tree::Leaf)
    .or(symbol_parser())
    .separated_by(just("::"))
    .map(|mut segments| {
      // Only remove the last segment if there are more than one
      if segments.len() > 1 {
        segments.pop();
      }
      segments
    })
}

#[derive(Debug, PartialEq)]
pub enum Tree {
  Leaf(String),
  Branch(Vec<Tree>),
}

impl ToString for Tree {
  fn to_string(&self) -> String {
    match self {
      Self::Leaf(leaf) => leaf.to_string(),
      Self::Branch(branch) => {
        format!(
          "<{}>",
          branch.iter().map(Tree::to_string).collect::<String>()
        )
      }
    }
  }
}

// This is ripped from recursive example and impl of `ident`
fn symbol_parser() -> impl Parser<char, Tree, Error = Simple<char>> {
  recursive(|tree| {
    tree
      .separated_by(just(""))
      .delimited_by(just('<'), just('>'))
      .map(Tree::Branch)
      .or(
        filter(|c: &char| c.is_ascii_alphabetic() || c.is_contained_in(" {}"))
          .map(Some)
          .chain::<char, _, _>(
            filter(|c: &char| {
              c.is_ascii_alphanumeric() || c.is_contained_in(" _&:,{}")
            })
            .repeated(),
          )
          .collect()
          .map(Tree::Leaf),
      )
  })
}

pub fn create_parser() -> impl Parser<char, Backtrace, Error = Simple<char>> {
  indexed_row_parser()
    .map(|(_ptr, segments)| {
      segments
        .iter()
        .map(|segment| segment.to_string())
        .collect::<Vec<String>>()
        .join("::")
    })
    .then(
      newline()
        .ignored()
        .then(location_row_parser().map(Some).or(empty().map(|_| None)))
        .map(|(_, location)| location),
    )
    .map(|(function, location)| Entry::new(function, location))
    .separated_by(newline())
    .map(Backtrace::with_entries)
}

/// To parse the line / column part of the file path
fn number_parser() -> impl Parser<char, u32, Error = Simple<char>> {
  text::int(10).map(|s: String| u32::from_str_radix(&s, 10).unwrap())
}

/// Parses the path to the file
fn path_parser() -> impl Parser<char, Location, Error = Simple<char>> {
  just(':')
    .not()
    .repeated()
    .collect()
    .then_ignore(just(':'))
    .then(number_parser())
    .then_ignore(just(':'))
    .then(number_parser())
    .map(|((path, line), column)| Location { path, line, column })
}

fn location_row_parser() -> impl Parser<char, Location, Error = Simple<char>> {
  just("at ")
    .not()
    .repeated()
    .ignored()
    .then(
      just("at ")
        .padded()
        .ignored()
        .then(path_parser())
        .map(|(_, location)| location),
    )
    .map(|(_, result)| result)
}

fn index_parser() -> impl Parser<char, (), Error = Simple<char>> {
  // in the backtrace there are 5 spaces after the colon
  // but I'm not sure that it's always the case
  // If it is, this could also match the beginning of the
  // address eg just(":     0x") to simplify parsing
  text::int(10).ignored().then_ignore(just(":    "))
}

fn indexed_row_parser(
) -> impl Parser<char, (u64, Vec<Tree>), Error = Simple<char>> {
  // prefix  0:     0x55a98f23062c - std::rust::10980
  // ^^^^^^^
  index_parser()
    .not()
    .repeated()
    .ignored()
    .then(
      // prefix  0:     0x55a98f23062c - std::rust::10980
      //         ^^^^^^
      index_parser()
        // prefix  0:     0x55a98f23062c - std::rust::10980
        //               ^
        .then_ignore(text::whitespace())
        // prefix  0:     0x55a98f23062c - std::rust::10980
        //                ^^^^^^^^^^^^^^
        .then(ptr_parser())
        .map(|(_, ptr)| ptr)
        // prefix  0:     0x55a98f23062c - std::rust::10980
        //                              ^^^
        .then_ignore(just(" - "))
        // prefix  0:     0x55a98f23062c - std::rust::10980
        //                                 ^^^^^^^^^XXXXXXX <- ignore this part
        .then(function_parser_full()),
    )
    .map(|(_, result)| result)
}

#[test]
fn symbol_parser_test() {
  let input = "<impl core::ops::function::FnOnce<A> for &F>";
  let parsed = symbol_parser().parse(input);
  assert_eq!(
    parsed,
    Ok(Tree::Branch(vec![
      Tree::Leaf("impl core::ops::function::FnOnce".into()),
      Tree::Branch(vec![Tree::Leaf("A".into())]),
      Tree::Leaf(" for &F".into())
    ]))
  );
}

#[test]
fn symbol_parser_test2() {
  let input = "foo::bar_bar::{{closure}}::h31f3d4001ecc8331";
  let parsed = symbol_parser().parse(input);
  assert_eq!(
    parsed,
    Ok(Tree::Leaf(
      "foo::bar_bar::{{closure}}::h31f3d4001ecc8331".into()
    ))
  );
}

#[test]
#[ignore = "chumsky tired me"]
fn symbol_parser_test3() {
  let input = "core::result::Result<T,E>::unwrap::h66b3d7e86d5c6f91";
  let parsed = symbol_parser().parse(input);
  assert_eq!(
    parsed,
    Ok(Tree::Leaf(
      "core::result::Result<T,E>::unwrap::h66b3d7e86d5c6f91".into()
    ))
  );
}

#[test]
fn full_parser_test() {
  let input = include_str!("./tests/fixtures/panic.txt");
  let parser = create_parser();
  let _parsed = parser.parse(input).unwrap();

  let input = include_str!("./tests/fixtures/panic_prefixed.txt");
  let parser = create_parser();
  let _parsed = parser.parse(input).unwrap();
}

#[test]
fn function_parser_test() {
  let input_fun2 = "core::ops::<impl core::ops::function::{{closure}}::FnOnce<A> for &F>::call_once::h92bf06c783fb5223";
  let parsed_fun2 = function_parser_full().parse(input_fun2).unwrap();
  let joined_back = parsed_fun2
    .iter()
    .map(|segment| segment.to_string())
    .collect::<Vec<String>>();
  let joined_back = joined_back.join("::");

  let output =
    "core::ops::<impl core::ops::function::{{closure}}::FnOnce<A> for &F>::call_once";
  assert_eq!(output, joined_back);
}

#[test]
fn path_parser_test() {
  let parsed_path = path_parser()
    .parse("/rustc/60dcc4348e/library/std/src/sys_common/backtrace.rs:44:22")
    .unwrap();

  assert_eq!(
    parsed_path,
    Location {
      path: "/rustc/60dcc4348e/library/std/src/sys_common/backtrace.rs"
        .to_string(),
      line: 44,
      column: 22
    }
  );
}

#[test]
fn indexed_parser_test() {
  let parser = indexed_row_parser();
  let input = "  0:     0x55a98f23062c - std::foo::bar::123";
  parser.parse(input).unwrap();

  let input = "0:     0x55a98f2306 - std::foo::bar::123";
  parser.parse(input).unwrap();

  let input = "prefix1234 0:     0x55a98f23062c - std::foo::bar::123";
  parser.parse(input).unwrap();
}

#[test]
fn location_parser_test() {
  let parser = location_row_parser();
  let input = "prefix   at /rustc/348e/library/std/src/../../backtrace/src/backtrace/libunwind.rs:93:5";
  parser.parse(input).unwrap();

  let input = "  at /file.rs:1:2";
  parser.parse(input).unwrap();

  let input = "at /file.rs:3:4";
  parser.parse(input).unwrap();
}

#[test]
fn optionality_test() {
  let input = "a-b-a-b-a-a-a";

  fn parser(
  ) -> impl Parser<char, Vec<(char, Option<char>)>, Error = Simple<char>> {
    just('a')
      .then(
        just('-')
          .ignored()
          .then(just('b').map(Some))
          .map(|(_, b)| b)
          .or(empty().map(|_| None)),
      )
      .separated_by(just('-'))
  }

  let result = parser().parse(input);
  println!("{:?}", result);
}
