use crate::database;
use crate::project::Project;
use crate::utils;

use chrono::NaiveDateTime;
use std::vec::Vec;

#[derive(Debug)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub inserted_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Task {
    pub fn find_existing() -> Option<Task> {
        find_existing_task()
    }

    pub fn get_by_id(id: usize) -> Option<Task> {
        get_by_id(id)
    }

    pub fn list_for(project: &Project) -> Vec<Task> {
        list_all_tasks(project)
    }

    pub fn add_to(project: &Project, name: String) -> Option<Task> {
        create_task(project, name)
    }

    pub fn set_context(&self, project: &Project) {
        save_context(project, &self)
    }

    pub fn add_log(&self, minutes: usize, message: String) {
        save_task_log(&self, minutes, message)
    }

    pub fn touch(&self) {
        touch_task(&self)
    }
}

fn find_existing_task() -> Option<Task> {
    let mut result: Option<Task> = None;
    let remote_branch = get_remote_branch();

    database::with_db(|db| {
        let mut cursor = db
            .prepare(
                "
                SELECT t.id, t.name, t.inserted_at, t.updated_at
                FROM tasks t
                LEFT JOIN contexts c ON c.task_id = t.id
                WHERE c.context = ?
                ORDER BY t.updated_at DESC;",
            )
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::String(remote_branch.to_string())])
            .unwrap();

        result = row_to_task(cursor.next());
    });

    result
}

fn list_all_tasks(project: &Project) -> Vec<Task> {
    let mut results: Vec<Task> = Vec::new();

    database::with_db(|db| {
        let mut cursor = db
            .prepare(
                "
                SELECT t.id, t.name, t.inserted_at, t.updated_at
                FROM tasks t
                WHERE t.project_id = ?
                ORDER BY t.updated_at DESC;",
            )
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(project.id as i64)])
            .unwrap();

        results = rows_to_tasks(cursor);
    });

    results
}

// This function will attempt to create a new task  and then
// it will return said Task (or None if something goes wrong)
fn create_task(project: &Project, name: String) -> Option<Task> {
    let mut result: Option<Task> = None;

    database::with_db(|db| {
        let mut cursor = db
            .prepare("REPLACE INTO tasks VALUES (null, ?, ?, DATETIME(), DATETIME());")
            .unwrap()
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(project.id as i64),
                sqlite::Value::String(name.to_string()),
            ])
            .unwrap();

        if cursor.next().is_ok() {
            cursor = db
                .prepare(
                    "
                SELECT t.id, t.name, t.inserted_at, t.updated_at
                FROM tasks t
                WHERE t.id IN(SELECT last_insert_rowid());
                ",
                )
                .unwrap()
                .cursor();

            result = row_to_task(cursor.next());

            // Attempt to store the context
            if let Some(task) = &result {
                save_context(&project, task);
            }
        }
    });

    result
}

// This function will attempt to store the current context attaching
// the remote to this Project
fn save_task_log(task: &Task, minutes: usize, message: String) {
    database::with_db(|db| {
        let mut cursor = db
            .prepare(
                "INSERT INTO task_logs (
                  task_id,
                  name,
                  minutes,
                  inserted_at,
                  updated_at
                ) VALUES (
                  ?,
                  ?,
                  ?,
                  DATETIME(),
                  DATETIME()
                );",
            )
            .unwrap()
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(task.id as i64),
                sqlite::Value::String(message.to_string()),
                sqlite::Value::Integer(minutes as i64),
            ])
            .unwrap();

        cursor.next().unwrap();
    });
}

// This function will attempt to store the current context attaching
// the remote to this Project
fn save_context(project: &Project, task: &Task) {
    let remote_branch = get_remote_branch();
    database::with_db(|db| {
        let mut cursor = db
            .prepare("REPLACE INTO contexts VALUES (null, ?, ?, ?, DATETIME(), DATETIME());")
            .unwrap()
            .cursor();

        cursor
            .bind(&[
                sqlite::Value::Integer(project.id as i64),
                sqlite::Value::Integer(task.id as i64),
                sqlite::Value::String(remote_branch.to_string()),
            ])
            .unwrap();

        cursor.next().unwrap();
    });
}

fn get_by_id(id: usize) -> Option<Task> {
    let mut result: Option<Task> = None;

    database::with_db(|db| {
        let mut cursor = db
            .prepare(
                "
                SELECT t.id, t.name, t.inserted_at, t.updated_at
                FROM tasks t
                WHERE t.id = ?;",
            )
            .unwrap()
            .cursor();

        cursor.bind(&[sqlite::Value::Integer(id as i64)]).unwrap();

        result = row_to_task(cursor.next());
    });

    result
}

// This function will "touch" the task, updating it's "last updated" timestamp
// Which should result in more usable sorted projects and tasks in the UI.
fn touch_task(task: &Task) {
    database::with_db(|db| {
        let mut cursor = db
            .prepare("UPDATE tasks SET updated_at = DATETIME() WHERE id = ?;")
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::Integer(task.id as i64)])
            .unwrap();

        cursor.next().unwrap();
    });
}

fn rows_to_tasks(mut cursor: sqlite::Cursor) -> Vec<Task> {
    let mut results: Vec<Task> = Vec::new();
    while let Some(task) = row_to_task(cursor.next()) {
        results.push(task);
    }

    results
}

fn row_to_task(row: Result<Option<&[sqlite::Value]>, sqlite::Error>) -> Option<Task> {
    let columns = match row {
        Ok(None) => return None,
        Ok(columns) => columns.unwrap(),
        Err(_) => return None,
    };

    Some(Task {
        id: columns[0].as_integer().unwrap() as usize,
        name: columns[1].as_string().unwrap().to_string(),
        inserted_at: utils::sql_to_datetime(columns[2].as_string()).unwrap(),
        updated_at: utils::sql_to_datetime(columns[3].as_string()).unwrap(),
    })
}

fn get_remote_branch() -> String {
    // If the code comes here these should have been unwrapped before so I am
    // assuming they're safe at this point.
    let repo = utils::get_current_repo().unwrap();
    let (remote, branch) = utils::get_repo_remote_and_branch(repo).unwrap();
    format!("{}#{}", remote, branch)
}
