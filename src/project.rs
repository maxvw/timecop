use crate::database;
use crate::summary;
use crate::task::Task;
use crate::utils;

use chrono::NaiveDateTime;
use std::vec::Vec;

#[derive(Debug)]
pub struct Project {
    pub id: usize,
    pub name: String,
    pub inserted_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Project {
    pub fn find_existing() -> Option<Project> {
        find_existing_project()
    }

    pub fn get_by_id(id: usize) -> Option<Project> {
        get_by_id(id)
    }

    pub fn create(remote: String, name: String) -> Option<Project> {
        create_project(remote, name)
    }

    pub fn list() -> Vec<Project> {
        list_all_projects()
    }

    pub fn list_tasks(&self) -> Vec<Task> {
        Task::list_for(&self)
    }

    pub fn add_task(&self, name: String) -> Option<Task> {
        Task::add_to(&self, name)
    }

    pub fn set_context(&self, remote: String) {
        save_context(&self, remote)
    }

    pub fn summary(&self) -> summary::Summary {
        summary::for_project(&self)
    }

    pub fn touch(&self) {
        touch_project(&self)
    }
}

fn find_existing_project() -> Option<Project> {
    // Is the current path a Git repository?
    let repo = match utils::get_current_repo() {
        Some(repo) => repo,
        None => return None,
    };

    // Get the current remote and branch
    let (remote, _branch) = match utils::get_repo_remote_and_branch(repo) {
        Some(remote_and_branch) => remote_and_branch,
        None => return None,
    };

    // Let's find this project in our database
    get_by_remote(remote)
}

// This function will attempt to create a new project and then
// it will return said Project (or None if something goes wrong)
fn create_project(remote: String, name: String) -> Option<Project> {
    let mut result: Option<Project> = None;

    database::with_db(|db| {
        let mut cursor = db
            .prepare("REPLACE INTO projects VALUES (null, ?, DATETIME(), DATETIME());")
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::String(name.to_string())])
            .unwrap();

        if cursor.next().is_ok() {
            cursor = db
                .prepare(
                    "
                SELECT p.id, p.name, p.inserted_at, p.updated_at
                FROM projects p
                WHERE p.id IN(SELECT last_insert_rowid());
                ",
                )
                .unwrap()
                .cursor();

            result = row_to_project(cursor.next());

            // Attempt to store the context
            if let Some(project) = &result {
                save_context(&project, remote.to_string());
            }
        }
    });

    result
}

// This function will attempt to store the current context attaching
// the remote to this Project
fn save_context(project: &Project, remote: String) {
    database::with_db(|db| {
        let mut cursor = db
            .prepare("REPLACE INTO contexts VALUES (null, ?, null, ?, DATETIME(), DATETIME());")
            .unwrap()
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(project.id as i64),
                sqlite::Value::String(remote.to_string()),
            ])
            .unwrap();

        cursor.next().unwrap();
    });
}

// This function will "touch" the project, updating it's "last updated" timestamp
// Which should result in more usable sorted projects and tasks in the UI.
fn touch_project(project: &Project) {
    database::with_db(|db| {
        let mut cursor = db
            .prepare("UPDATE projects SET updated_at = DATETIME() WHERE id = ?;")
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(project.id as i64)])
            .unwrap();

        cursor.next().unwrap();
    });
}

fn get_by_id(id: usize) -> Option<Project> {
    let mut result: Option<Project> = None;

    database::with_db(|db| {
        let mut cursor = db
            .prepare(
                "
                SELECT p.id, p.name, p.inserted_at, p.updated_at
                FROM projects p
                WHERE p.id = ?;",
            )
            .unwrap()
            .cursor();

        cursor.bind(&[sqlite::Value::Integer(id as i64)]).unwrap();

        result = row_to_project(cursor.next());
    });

    result
}

fn get_by_remote(remote: String) -> Option<Project> {
    let mut result: Option<Project> = None;

    database::with_db(|db| {
        let mut cursor = db
            .prepare(
                "
                SELECT p.id, p.name, p.inserted_at, p.updated_at
                FROM contexts c
                LEFT JOIN projects p ON p.id = c.project_id
                WHERE c.context = ?
                ORDER BY p.updated_at DESC;",
            )
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::String(remote.to_string())])
            .unwrap();

        result = row_to_project(cursor.next());
    });

    result
}

fn list_all_projects() -> Vec<Project> {
    let mut results: Vec<Project> = Vec::new();

    database::with_db(|db| {
        let cursor = db
            .prepare(
                "
                SELECT p.id, p.name, p.inserted_at, p.updated_at
                FROM projects p
                ORDER BY p.updated_at DESC;",
            )
            .unwrap()
            .cursor();

        results = rows_to_projects(cursor);
    });

    results
}

fn rows_to_projects(mut cursor: sqlite::Cursor) -> Vec<Project> {
    let mut results: Vec<Project> = Vec::new();
    while let Some(project) = row_to_project(cursor.next()) {
        results.push(project);
    }
    results
}

fn row_to_project(row: Result<Option<&[sqlite::Value]>, sqlite::Error>) -> Option<Project> {
    let columns = match row {
        Ok(None) => return None,
        Ok(columns) => columns.unwrap(),
        Err(_) => return None,
    };

    Some(Project {
        id: columns[0].as_integer().unwrap() as usize,
        name: columns[1].as_string().unwrap().to_string(),
        inserted_at: utils::sql_to_datetime(columns[2].as_string()).unwrap(),
        updated_at: utils::sql_to_datetime(columns[3].as_string()).unwrap(),
    })
}
