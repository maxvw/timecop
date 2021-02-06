use crate::project::Project;
use crate::task::Task;
use clap::ArgMatches;

#[derive(Debug)]
pub struct State<'a> {
    pub project: Option<Project>,
    pub task: Option<Task>,
    pub matches: ArgMatches<'a>,
}

impl State<'_> {
    pub fn new() -> State<'static> {
        State {
            project: Project::find_existing(),
            task: Task::find_existing(),
            matches: ArgMatches::new(),
        }
    }
}
