use std::process::exit;

use clap::{Args, FromArgMatches};
use fixrs::{Options, finish_run, run};

fn main() {
  let Some(matches) = clap_args::parse!(|cmd| {
    let cmd = Options::augment_args(cmd);
    Options::apply_i18n(cmd)
  }) else {
    return;
  };

  let options = match Options::from_arg_matches(&matches) {
    Ok(opts) => opts,
    Err(e) => e.exit(),
  };

  match run(&options) {
    Ok(summary) => finish_run(&summary, &options),
    Err(e) => {
      eprintln!("Error: {e}");
      exit(1);
    }
  }
}
