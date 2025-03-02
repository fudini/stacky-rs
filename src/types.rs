use crate::utils::color;
use serde_derive::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub struct Entry {
  function: String,
  location: Option<Location>,
}

impl Entry {
  pub fn new(function: String, location: Option<Location>) -> Self {
    Self { function, location }
  }
}

impl fmt::Display for Entry {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    color(f, 4, 0, &self.function);
    if let Some(location) = &self.location {
      write!(f, " ")?;
      color(f, 2, 0, &location.path);
      write!(f, ":")?;
      color(f, 3, 0, &location.line.to_string());
      write!(f, ":")?;
      color(f, 3, 0, &location.column.to_string());
    }
    Ok(())
  }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Location {
  pub path: String,
  pub line: u32,
  pub column: u32,
}

impl fmt::Display for Location {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}:{}", self.path, self.line, self.column)
  }
}

#[derive(Debug, Serialize)]
pub struct Backtrace(Vec<Entry>);

impl Backtrace {
  pub fn with_entries(entries: Vec<Entry>) -> Self {
    Self(entries)
  }

  #[cfg(test)]
  pub fn entries(&self) -> &Vec<Entry> {
    &self.0
  }

  /// Loops through all entries and check if any of them matches the given location
  pub fn has_location(&self, location: &str) -> bool {
    self
      .0
      .iter()
      .find(|entry| {
        entry
          .location
          .as_ref()
          .map(|l| l.path.starts_with(location))
          .unwrap_or(false)
      })
      .is_some()
  }

  /// Clean up from unwanted entries
  pub fn filter(&mut self) {
    self.0.retain(|entry| {
      let Some(location) = &entry.location else {
        return false;
      };

      let path = &location.path;

      // TODO: configurable somehow?
      !(entry.function.contains("__libc")
        || entry.function.contains("start_thread")
        || entry.function.contains("__GI___clone3")
        || path == "_start"
        || path.contains("/rustc/")
        || path.contains("/sysdeps/")
        || path == "")
    });
  }
}

impl fmt::Display for Backtrace {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    color(f, 1, 0, "\n--- BACKTRACE START ---\n");
    for entry in &self.0 {
      writeln!(f, "{}", entry)?;
    }
    color(f, 1, 0, "--- BACKTRACE END ------\n");
    Ok(())
  }
}
