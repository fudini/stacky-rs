use std::{
  env, fmt,
  fs::{self, DirEntry},
  path::PathBuf,
};

fn entry_to_path(dir_entry: DirEntry) -> Option<PathBuf> {
  let path = dir_entry.path();
  let file_name = path.file_name()?;
  let file_name_string = file_name.to_str()?;
  file_name_string.starts_with("nvim.").then_some(path)
}

pub fn get_nvim_pipes() -> Vec<PathBuf> {
  let xdg_pipes = env::var("XDG_RUNTIME_DIR")
    .map(|dir| get_pipes(&dir))
    .unwrap_or_default();

  if !xdg_pipes.is_empty() {
    return xdg_pipes;
  }

  let osx_pipes = env::var("TMPDIR")
    .map(|dir| get_pipes(&dir))
    .unwrap_or_default();

  if !osx_pipes.is_empty() {
    return osx_pipes;
  }

  Vec::new()
}

fn get_pipes(dir: &str) -> Vec<PathBuf> {
  let Ok(dir) = fs::read_dir(dir) else {
    return vec![];
  };

  let nvim_pipes = dir.filter_map(Result::ok).filter_map(entry_to_path);

  nvim_pipes.collect()
}

/// Colors
pub fn color(f: &mut fmt::Formatter, fg: u8, bg: u8, string: &str) {
  let _ = write!(f, "\x1b[38;5;{fg}m\x1b[48;5;{bg}m{string}\x1b[0m");
}
