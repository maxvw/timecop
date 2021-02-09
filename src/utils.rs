use chrono::format::ParseError;
use chrono::{NaiveDate, NaiveDateTime};
use git2::Repository;
use std::env;
use std::path::PathBuf;
use termion::{color, style};

// Returns the current woring directory as a path
pub fn get_current_path() -> PathBuf {
    env::current_dir().unwrap()
}

// Return the current repository
pub fn get_current_repo() -> Option<git2::Repository> {
    let path = get_current_path();
    match Repository::discover(path) {
        Err(_) => None,
        Ok(repo) => Some(repo),
    }
}

pub fn get_repo_remote_and_branch(repo: git2::Repository) -> Option<(String, String)> {
    // Is there a remote url defined?
    // NOTE: Maybe we want to check for other remotes than `origin`
    let remote = match repo.find_remote("origin") {
        Err(_) => return None,
        Ok(remote) => remote.url().unwrap().to_string(),
    };

    // Find the current branch
    let branch = match repo.head() {
        Err(_) => return None,
        Ok(head) => head.shorthand().unwrap().to_string(),
    };

    // Return the remote and branch
    Some((remote, branch))
}

pub fn ensure_valid_repo() -> Option<(String, String)> {
    // Make sure we are currently in a repository
    let remote_and_branch = match get_current_repo() {
        None => None,
        Some(repo) => get_repo_remote_and_branch(repo),
    };

    // Nope, no repository found?
    if remote_and_branch.is_none() {
        error_msg(
            "No Repository Found",
            "Timecop requires the directory you are currently in to be a git\r\nrepository, with a valid upstream and active branch.",
        );
        std::process::exit(1);
    }

    remote_and_branch
}

// Convert date from SQLite to NaiveDate
pub fn sql_to_date(input: Option<&str>) -> Result<NaiveDate, ParseError> {
    let date = input.unwrap_or("");
    NaiveDate::parse_from_str(date, "%Y-%m-%d %H:%M:%S")
}

// Convert datetime from SQLite to NaiveDateTime
pub fn sql_to_datetime(input: Option<&str>) -> Result<NaiveDateTime, ParseError> {
    let datetime = input.unwrap_or("");
    NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")
}

// Print a message with a bold white title
pub fn info_msg(title: &str, message: &str) {
    println!(
        "{}{}{}{}\r\n{}\r\n",
        color::Fg(termion::color::LightWhite),
        style::Bold,
        title,
        style::Reset,
        message
    );
}

// Print a message with a bold white title (without newlines)
pub fn info_msg_compact(title: &str, message: &str) {
    println!(
        "{}{}{}{} {}",
        color::Fg(termion::color::LightWhite),
        style::Bold,
        title,
        style::Reset,
        message
    );
}

// Print a message with a bold red title
pub fn error_msg(title: &str, message: &str) {
    println!(
        "{}{}{}{}\r\n{}\r\n",
        color::Fg(termion::color::Red),
        style::Bold,
        title,
        style::Reset,
        message
    );
}
