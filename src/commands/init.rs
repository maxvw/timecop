use crate::project::Project;
use crate::state::State;
use crate::utils;

use clap::ArgMatches;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::vec::Vec;
use textwrap::indent;

pub fn exec<'a>(
    mut state: State<'a>,
    matches: ArgMatches<'a>,
) -> Result<State<'a>, Box<dyn error::Error>> {
    // Let the user either create a new project, or select an existing one
    let project = match state.project {
        None => create_or_select_project(),
        Some(project) => Some(project),
    };

    // Let the user install the git hook
    if !matches.is_present("no-hook") {
        install_git_hook();
    }

    // Assign state to project
    state.project = project;
    Ok(state)
}

fn create_or_select_project() -> Option<Project> {
    let theme = ColorfulTheme::default();
    let projects = Project::list();
    let mut options: Vec<&str> = Vec::new();

    // We only need the remote now, unwrap feels safe
    // because we already checked for None earlier.
    let (remote, _) = utils::ensure_valid_repo().unwrap();

    // Is this a first-time experience or not?
    if projects.is_empty() {
        options.push("Create your first project");
    } else {
        options.push("Create a new project");
        options.push("Select an existing project");
    }

    options.push("Nothing, thanks timecop!");

    utils::info_msg(
        "No Project Found",
        "This repository does not appear to have a project created for it yet.",
    );

    match Select::with_theme(&theme)
        .with_prompt("What would you like to do?")
        .default(0)
        .items(&options)
        .interact()
    {
        Ok(0) => create_project(remote),
        Ok(1) => {
            if options.len() == 2 {
                std::process::exit(0)
            } else {
                select_project(remote, projects)
            }
        }
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(0),
    }
}

fn create_project(remote: String) -> Option<Project> {
    let theme = ColorfulTheme::default();

    let name: String = Input::with_theme(&theme)
        .with_prompt("Project Name:")
        .interact()
        .unwrap();

    Project::create(remote, name)
}

fn select_project(remote: String, projects: Vec<Project>) -> Option<Project> {
    let theme = ColorfulTheme::default();
    let project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();

    let result = Select::with_theme(&theme)
        .with_prompt("Select an existing project:")
        .default(0)
        .items(&project_names)
        .interact()
        .unwrap();

    if let Some(project) = Project::get_by_id(projects[result].id) {
        project.set_context(remote);
        Some(project)
    } else {
        None
    }
}

fn hook_data() -> String {
    format!(
        "#!/usr/bin/env bash

# Offer a nice interactive experience
exec < /dev/tty
exec < /dev/stdin
exec < /dev/stderr

# Start a new commit based log entry
{} log --commit

# Close stdin again
exec <&-
",
        std::env::current_exe().unwrap().to_str().unwrap()
    )
}

fn install_git_hook() {
    let repo = utils::get_current_repo();
    if repo.is_none() {
        return;
    }

    let hook_path = repo.unwrap().path().join("hooks/post-commit");
    println!("{:?}", hook_path);

    if !prompt_install_hook() {
        return;
    }

    // If there is already a post-commit hook installed, ask if the
    // user wants to overwrite this, or not.
    if hook_path.exists() && !prompt_overwrite_existing_hook() {
        manual_install_hook_text();
        return;
    }

    // Try to create the file
    let mut file = match File::create(&hook_path) {
        Err(err) => panic!("failed to create git hook: {}", err),
        Ok(file) => file,
    };

    // Write the hook contents
    if let Err(why) = file.write_all(hook_data().as_bytes()) {
        panic!("failed to create git hook: {}", why);
    }

    // Make it executable for the user
    fs::set_permissions(hook_path, fs::Permissions::from_mode(0o766)).unwrap();

    println!("Done. The `.git/hooks/post-commit` hook has been installed!");
}

fn prompt_install_hook() -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to install the `.git/hooks/post-commit` hook for timecop?")
        .interact()
        .unwrap()
}

fn prompt_overwrite_existing_hook() -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(
            "It looks like `.git/hooks/post-commit` already exists, do you want to overwrite it?",
        )
        .interact()
        .unwrap()
}

fn manual_install_hook_text() {
    utils::info_msg(
        "Manual Installation",
        &format!(
            "There is already a `.git/hooks/post-commit` file present and you opted to
not overwrite this with the Timecop hook. Since you did try to install it
originally we'll dump our `post-commit` hook here so you can pick what you
want to upgrade your existing `post-commit` hook.

{}",
            indent(&hook_data(), "\t")
        ),
    )
}
