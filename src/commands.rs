mod init;
mod log;
mod output;

use crate::state::State;
use clap::ArgMatches;

pub fn exec(state: State) -> Result<State, Box<dyn std::error::Error>> {
    let matches = state.matches.clone();
    let subcommand = subcommand_name(&matches)?;
    let submatches = subcommand_matches(&matches, &subcommand)?;

    if subcommand == "init" {
        init::exec(state, submatches)
    } else if subcommand == "log" {
        log::exec(state, submatches)
    } else if subcommand == "output" {
        output::exec(state, submatches)
    } else {
        Err("Unknown command".into())
    }
}

fn subcommand_name(matches: &ArgMatches) -> Result<String, Box<dyn std::error::Error>> {
    match matches.subcommand_name() {
        Some(name) => Ok(name.to_string()),
        None => Err("Failed to get subcommand name".into()),
    }
}

fn subcommand_matches<'a>(
    matches: &ArgMatches<'a>,
    subcommand: &str,
) -> Result<ArgMatches<'a>, Box<dyn std::error::Error>> {
    match matches.subcommand_matches(subcommand) {
        Some(matches) => Ok(matches.clone()),
        None => Err("Failed to get subcommand matches".into()),
    }
}
