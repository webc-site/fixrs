use std::{env, process::exit};

use clap::{ArgAction, Args, Command, FromArgMatches, arg};
use current_platform::CURRENT_PLATFORM;
use fixrs::{Options, finish_run, run};

fn main() {
  let is_subcommand = env::args().nth(1).as_deref() == Some("fixrs");

  let Some(matches) = clap_args::parse!(|cmd| {
    if is_subcommand {
      let sub = Options::augment_args(
        Command::new("fixrs")
          .disable_version_flag(true)
          .disable_help_flag(true)
          .arg(arg!(-v --version "show version").action(ArgAction::SetTrue))
          .arg(arg!(--vv "version detail"))
          .arg(arg!(-h --help "print help")),
      );
      cmd.subcommand(Options::apply_i18n(sub))
    } else {
      let cmd = Options::augment_args(cmd);
      Options::apply_i18n(cmd)
    }
  }) else {
    return;
  };

  let options = if is_subcommand {
    if let Some(sub_matches) = matches.subcommand_matches("fixrs") {
      if sub_matches.get_one::<bool>("help") == Some(&true) {
        return;
      }
      if sub_matches.get_one::<bool>("version") == Some(&true) {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
      }
      if sub_matches.get_one::<bool>("vv") == Some(&true) {
        println!(
          "ver:{}\ntarget:{CURRENT_PLATFORM}",
          env!("CARGO_PKG_VERSION")
        );
        return;
      }
      match Options::from_arg_matches(sub_matches) {
        Ok(opts) => opts,
        Err(e) => e.exit(),
      }
    } else {
      Options::default()
    }
  } else {
    match Options::from_arg_matches(&matches) {
      Ok(opts) => opts,
      Err(e) => e.exit(),
    }
  };

  match run(&options) {
    Ok(summary) => finish_run(&summary, &options),
    Err(e) => {
      eprintln!("Error: {e}");
      exit(1);
    }
  }
}
