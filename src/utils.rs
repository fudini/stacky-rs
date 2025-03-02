use std::fmt;

// TODO: This depends on existence of XDG_RUNTIME_DIR, but I don't know where
// to look for neovim pipes otherwise yet
pub fn get_nvim_pipes() -> Vec<String> {
  let xdg = std::env::var("XDG_RUNTIME_DIR")
    .expect("XDG_RUNTIME_DIR env var not found");

  let dir = std::fs::read_dir(xdg).expect("Reading dir failure");

  let nvim_pipes = dir.filter_map(|e| {
    let path = e
      .expect("Error listing neovim pipes")
      .path()
      .into_os_string()
      .into_string()
      .expect("Not a string");

    path.contains("nvim.").then_some(path)
  });

  nvim_pipes.collect()
}

/// Colors
pub fn color(f: &mut fmt::Formatter, fg: u8, bg: u8, string: &str) {
  let _ = write!(f, "\x1b[38;5;{fg}m\x1b[48;5;{bg}m{string}\x1b[0m");
}
