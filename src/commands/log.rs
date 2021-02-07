use crate::ignore::{get_ignore_flag, set_ignore_flag};
use crate::project::Project;
use crate::state::State;
use crate::task::Task;
use crate::utils;

use clap::ArgMatches;
use dialoguer::{theme::ColorfulTheme, Editor, Input, Select};
use std::error;

pub fn exec<'a>(
    mut state: State<'a>,
    matches: ArgMatches<'a>,
) -> Result<State<'a>, Box<dyn error::Error>> {
    // Make sure we have an active project
    if let None = state.project {
        utils::error_msg("No Project Found", "Timecop requires a project to be defined before you can start\r\nusing timecop to log entries, first run: $ timecop init");
        std::process::exit(1);
    }

    // Should we ignore this branch?
    if get_ignore_flag() {
        std::process::exit(0)
    }

    // Let the user either create a new task, or select an existing one
    let task = match state.task {
        None => create_or_select_task(&state.project.as_ref().unwrap()),
        Some(task) => Some(task),
    };

    utils::info_msg_compact("Task:", &task.as_ref().unwrap().name);

    // Either use the last commit message, or prompt the user for a message
    let last_commit = get_last_commit_message();
    let message = if matches.is_present("commit") {
        utils::info_msg_compact("Message:", &last_commit);
        last_commit
    } else if let Some(message) = matches.value_of("message") {
        utils::info_msg_compact("Message:", &message);
        message.to_string()
    } else {
        prompt_message(last_commit, "".to_string())
    };

    // Let's ask the user how many minutes they spent on this task
    let minutes = prompt_minutes();

    // Write this log entry to the database
    if let Some(task) = &task {
        task.add_log(minutes, message);
    }

    // Assign task to state
    state.task = task;
    Ok(state)
}

fn create_or_select_task(project: &Project) -> Option<Task> {
    let theme = ColorfulTheme::default();
    let tasks = project.list_tasks();
    let mut options: Vec<&str> = Vec::new();
    let branch = get_branch();

    // Is this a first-time experience or not?
    if tasks.len() == 0 {
        options.push("Create your first task");
    } else {
        options.push("Create a new task");
        options.push("Select an existing task");
    }

    options.push("Nothing, thanks timecop!");
    let ignore_command = format!("Ignore this branch ({})", branch);
    options.push(&ignore_command);

    utils::info_msg(
        "No Task Found",
        "This branch does not appear to have a task created for it yet.",
    );

    match Select::with_theme(&theme)
        .with_prompt("What would you like to do?")
        .default(0)
        .items(&options)
        .interact()
    {
        Ok(0) => create_task(&project),
        Ok(1) => {
            if options.len() == 4 {
                select_tasks(&project, tasks)
            } else {
                std::process::exit(0)
            }
        }
        Ok(n) => {
            if options.len() == 4 && n != 3 {
                std::process::exit(0)
            } else if options.len() == 3 && n != 2 {
                std::process::exit(0)
            }

            set_ignore_flag();
            std::process::exit(0)
        }
        Err(_) => std::process::exit(0),
    }
}

fn create_task(project: &Project) -> Option<Task> {
    let theme = ColorfulTheme::default();

    let name: String = Input::with_theme(&theme)
        .with_prompt("Task Name:")
        .interact()
        .unwrap();

    project.add_task(name)
}

fn select_tasks(project: &Project, tasks: Vec<Task>) -> Option<Task> {
    let theme = ColorfulTheme::default();
    let task_names: Vec<String> = tasks.iter().map(|t| t.name.clone()).collect();

    let result = Select::with_theme(&theme)
        .with_prompt("Select an existing task:")
        .default(0)
        .items(&task_names)
        .interact()
        .unwrap();

    if let Some(task) = Task::get_by_id(tasks[result].id) {
        task.set_context(project);
        Some(task)
    } else {
        None
    }
}

fn get_last_commit_message() -> String {
    let repo = utils::get_current_repo().unwrap();
    let branch = repo.head().unwrap().shorthand().unwrap().to_string();
    let object = repo.revparse_single(&branch).unwrap();
    let commit = object.peel_to_commit().unwrap();
    return commit
        .message()
        .map(|m| m.to_string())
        .unwrap()
        .trim()
        .to_string();
}

fn prompt_message(default: String, initial: String) -> String {
    let theme = ColorfulTheme::default();

    let message = Input::with_theme(&theme)
        .with_prompt("What did you work on?:")
        .default(default.clone())
        .with_initial_text(initial)
        .interact()
        .unwrap();

    // With \e as a message it will open an external editor (or at least try to)
    if message == "\\e" {
        if let Some(updated_message) = Editor::new()
            .require_save(false)
            .edit(&default.clone())
            .unwrap()
        {
            prompt_message(default, updated_message)
        } else {
            message
        }
    } else {
        message
    }
}

fn prompt_minutes() -> usize {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Estimated time spent? (in minutes)")
        .interact()
        .unwrap()
}

fn get_branch() -> String {
    let repo = utils::get_current_repo().unwrap();
    let (_remote, branch) = utils::get_repo_remote_and_branch(repo).unwrap();
    return branch;
}
