#[derive(Clone)]
pub struct Config {
  pub verbose: bool,
  pub stacky_function: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      verbose: false,
      stacky_function: "stacky_global".to_string(),
    }
  }
}
