mod commands;
mod database;
mod ignore;
mod project;
mod state;
mod summary;
mod task;
mod utils;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use clap::{App, AppSettings, Arg, SubCommand};
use state::State;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("timecop")
        .about("helps you keep track of time spent working.")
        .version(VERSION)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::VersionlessSubcommands)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .global_setting(AppSettings::ColorAuto)
        .subcommand(
            SubCommand::with_name("init")
            .about("initialize a new project")
            .long_about(
                "This will let you either create a new project or select an existing project, it will also
prompt you to see if you want to install the included post-commit git-hook. This will make
it slightly easier for you to keep track of your time spent because it will ask you after
each commit you make how much time you think you spent on it.")
            .arg(Arg::with_name("no-hook").long("no-hook").help("skip the git post-commit hook prompt"))
        )
        .subcommand(
            SubCommand::with_name("log")
                .about("add a new entry for this project")
                .arg(
                    Arg::with_name("commit")
                        .long("commit")
                        .help("use last commit message as log entry")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("message")
                        .help("the log entry message")
                        .short("m")
                        .long("message")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("output")
                .about("output the tasks performed by day for this project")
                .arg(
                    Arg::with_name("csv")
                        .long("csv")
                        .help("export as CSV")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("detailed")
                        .long("detail")
                        .help("include task log entries")
                        .takes_value(false)
                        .required(false),
                ),
        );

    // Get clap matches
    let matches = matches.get_matches();

    // Ensure valid repository
    utils::ensure_valid_repo();

    // Generate the State
    let mut state = State::new();
    state.matches = matches;

    // Open the Database
    database::open_db();

    // Execute the given command
    commands::exec(state)?;
    return Ok(());
}
