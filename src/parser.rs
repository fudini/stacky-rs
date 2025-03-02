#![allow(unused)]
use crate::types::{Backtrace, Entry, Location};

pub fn parse_location(line: &str) -> Location {
  let mut file_line =
    line.trim().split(' ').last().unwrap_or("-:0:0").split(':');
  let path = file_line.next().unwrap_or("");
  let line = file_line
    .next()
    .map(str::parse)
    .map(Result::ok)
    .flatten()
    .unwrap_or(0);
  let column = file_line
    .next()
    .map(str::parse)
    .map(Result::ok)
    .flatten()
    .unwrap_or(0);

  Location {
    path: path.to_string(),
    line,
    column,
  }
}

pub fn parse(backtrace: &str) -> Backtrace {
  let mut entries = Vec::new();
  let mut lines = backtrace.lines();

  let mut function: Option<String> = None;

  while let Some(line) = lines.next() {
    if line.contains("at ") {
      let location = parse_location(line);
      if let Some(function) = function.take() {
        entries.push(Entry::new(function, location));
        continue;
      }
    }

    // FUNCTION NAME WITHOUT THE LAST PART (THE HASH)
    let f = line.trim().split(' ').last().unwrap_or("ERR").to_string();
    let parsed_function = match f.rsplit_once("::") {
      Some((cleaned, _)) => cleaned.to_string(),
      None => f,
    };
    if let Some(function) = function.replace(parsed_function) {
      entries.push(Entry::no_location(function));
    }
  }

  // last lonely function if exists, probably <unknown>
  if let Some(function) = function.take() {
    entries.push(Entry::no_location(function));
  }

  Backtrace::with_entries(entries)
}
