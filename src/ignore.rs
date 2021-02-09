use crate::database;
use crate::utils;

pub fn set_ignore_flag() {
    let remote_branch = get_remote_branch();

    database::with_db(|db| {
        // List all tasks
        let mut cursor = db
            .prepare(
                "
                INSERT INTO ignored (
                  context,
                  inserted_at,
                  updated_at
                ) VALUES (
                  ?,
                  DATETIME(),
                  DATETIME()
                );",
            )
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::String(remote_branch.to_string())])
            .unwrap();

        cursor.next().unwrap();
    });
}

pub fn get_ignore_flag() -> bool {
    let mut result = false;
    let remote_branch = get_remote_branch();

    database::with_db(|db| {
        // List all tasks
        let mut cursor = db
            .prepare(
                "
                SELECT i.id
                FROM ignored i
                WHERE i.context = ?
                ORDER BY i.updated_at DESC;",
            )
            .unwrap()
            .cursor();

        cursor
            .bind(&[sqlite::Value::String(remote_branch.to_string())])
            .unwrap();

        if let Ok(Some(_id)) = cursor.next() {
            result = true;
        }
    });

    result
}

fn get_remote_branch() -> String {
    // If the code comes here these should have been unwrapped before so I am
    // assuming they're safe at this point.
    let repo = utils::get_current_repo().unwrap();
    let (remote, branch) = utils::get_repo_remote_and_branch(repo).unwrap();
    format!("{}#{}", remote, branch)
}
