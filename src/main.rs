mod config;
mod parser3;
mod types;
mod utils;

use config::Config;
use nvim_rs::{create::tokio::new_path, rpc::handler::Dummy};
use parser3::parse_backtrace;
use std::io::Write;
use tokio::{
  io::{self, AsyncBufReadExt},
  sync::mpsc::{unbounded_channel, UnboundedReceiver},
};
use types::Backtrace;
use utils::get_nvim_pipes;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // This will eventually come from somewhere if there's a need
  let config = config::Config::default();

  let stdin = io::stdin();
  let mut stdout = std::io::stdout();
  let stdin_buf = io::BufReader::new(stdin);
  let mut lines = stdin_buf.lines();

  let mut append = false;
  let mut full = String::new();

  let (rx, tx) = unbounded_channel::<Backtrace>();

  // task notifying nvim
  tokio::spawn(nvim_task(tx, config.clone()));

  // Main task reading stdin line by line
  while let Ok(Some(line)) = lines.next_line().await {
    if append {
      // We suppress the output for a trace
      full.push_str(&line);
      full.push('\n');
      if config.verbose {
        println!("Stacky: APPENDING LINE {}", line);
      }
    }

    // The beginning of backtrace
    // Not sure why 2 different string
    if line.contains("stack backtrace:") || line.contains("Stack backtrace:") {
      if config.verbose {
        println!("Stacky: START APPENDING");
      }
      append = true;
    }

    if !append {
      // could be an option to dump backtrace, but probably we only want a short version or none
      // if it's sent to neovim
      stdout.write_all(format!("{}\n", line).as_bytes())?;
    }

    // This looks like the end of backtrace that I'm interested in
    if line.contains("0x0 - ")
      || line.ends_with(" - _start")
      || line.ends_with(": _start")
    {
      if config.verbose {
        println!("Stacky: STOP APPENDING");
      }
      append = false;

      let backtrace = parse_backtrace(full.as_str());

      match backtrace {
        Ok((_, mut backtrace)) => {
          backtrace.filter();
          // Print the short backtrace
          println!("{}", backtrace);

          if let Err(e) = rx.send(backtrace) {
            eprintln!(
              "Stacky error sending backtrace through a channel: {}",
              e
            );
          }

          full.clear();
        }
        Err(err) => {
          println!("--- BACKTRACE PARSE ERROR ------------------------");

          if config.verbose {
            eprintln!("{:?}", err);
          }

          full.clear();
          continue;
        }
      };
    }
  }

  Ok(())
}

async fn nvim_task(
  mut backtraces: UnboundedReceiver<Backtrace>,
  config: Config,
) {
  while let Some(backtrace) = backtraces.recv().await {
    let pipes = get_nvim_pipes();

    for pipe in pipes {
      if config.verbose {
        println!("nvim pipe: {}", pipe);
      }
      let nvim = new_path(pipe.clone(), Dummy::new());
      let (writer, _join_handle) = nvim.await.unwrap();

      let cwd = writer
        .exec_lua("return vim.fn.getcwd()", vec![])
        .await
        .map(|val| val.as_str().unwrap_or("").to_owned())
        .unwrap_or_else(|_e| {
          eprintln!("Couldn't retrieve CWD");
          "/".to_string()
        });

      // Is any entry matching a CWD of neovim instance
      // We only need to send a backgrace to those that match
      let has_location = backtrace.has_location(&cwd);

      let json =
        serde_json::to_string(&backtrace).expect("Failed JSON serialize");

      let stacky_function = &config.stacky_function;
      let lua = format!("{stacky_function}('{json}')");

      // Only send the backtrace if CWD is in it
      if !has_location {
        continue;
      }

      if let Err(e) = writer.exec_lua(&lua, vec![]).await {
        eprintln!(
          "Stacky error sending backtrace to Neovim instance '{}': {}",
          pipe, e
        );
      }
    }
  }
}
