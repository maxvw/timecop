use home;
use sqlite;
use std::cell::{Ref, RefCell};
use std::path::PathBuf;

// This refcell keeps track of the current database connection
// I'm honestly not sure if this is the best way to do this, but
// this seemed to work and I'm sure there's room for improvement.
thread_local!(static DBCONN: RefCell<Option<sqlite::Connection>> = RefCell::new(None));

// List of static migration strings which will be executed in order,
// the database will store the 'last used' index and work it's way up
// from there next time the app starts. Only migrates upwards.
static MIGRATIONS: [&str; 1] = ["
  CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,
    inserted_at     DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL
  );

  CREATE TABLE IF NOT EXISTS tasks (
    id              INTEGER PRIMARY KEY,
    project_id      INTEGER NOT NULL,
    name            TEXT NOT NULL,
    inserted_at     DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
  );

  CREATE TABLE IF NOT EXISTS contexts (
    id              INTEGER PRIMARY KEY,
    project_id      INTEGER NOT NULL,
    task_id         INTEGER NULL,
    context         TEXT NOT NULL,
    inserted_at     DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE,
    UNIQUE(project_id, task_id, context)
  );

  CREATE TABLE IF NOT EXISTS task_logs (
    id              INTEGER PRIMARY KEY,
    task_id         INTEGER NOT NULL,
    name            TEXT NOT NULL,
    minutes         INTEGER NOT NULL,
    inserted_at     DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL,

    FOREIGN KEY (task_id) REFERENCES tasks (id) ON DELETE CASCADE
  );

  CREATE TABLE IF NOT EXISTS ignored (
    id              INTEGER PRIMARY KEY,
    context         TEXT NOT NULL,
    inserted_at     DATETIME NOT NULL,
    updated_at      DATETIME NOT NULL
  );
  "];

// Open the database and store it in our refcell for later use.
pub fn open_db() {
    let mut path: PathBuf = home::home_dir().unwrap();
    path.push(".timecopdb");
    let db_path = path.to_str().unwrap();

    DBCONN.with(|db_conn| {
        *db_conn.borrow_mut() = Some(sqlite::open(db_path).unwrap());
    });

    run_migrations();
}

// Execute a function or closure with our current connection
// It will now also make a new connection, if needed.
pub fn with_db<F>(mut f: F)
where
    F: FnMut(&sqlite::Connection),
{
    maybe_open_db();

    DBCONN.with(|db_conn| {
        Ref::map(db_conn.borrow(), |borrow| {
            let connection = borrow.as_ref().unwrap();
            f(connection);
            return borrow;
        });
    });

    return;
}

// This function will see if there is an active connection with
// the database, and if not it will attempt to establish one.
fn maybe_open_db() {
    let mut connected = false;

    // Let's check if we have an active connection or not
    // This feels like it could be done better, but I haven't
    // figured out how yet.
    DBCONN.with(|db_conn| {
        Ref::map(db_conn.borrow(), |borrow| {
            if let None = borrow {
                connected = false;
            } else {
                connected = true;
            }

            return borrow;
        });
    });

    // Now we are no longer borrowing the connection, we can
    // open the database if we weren't using it yet.
    if connected == false {
        open_db();
    }
}

// NOTE: This is an extremely basic implementation of migrations, but it should
// suffice for the purposes of this application. They only support migrating up
// so a rollback is a new migration that manually changes it back to the way it
// was before, but hopefully that's not a very common scenario. This could always
// be revised if needed, I suppose. But it serves a very simple purpose as well.
fn run_migrations() {
    with_db(|db| {
        // Create the migrations table (if needed)
        match db.execute("CREATE TABLE IF NOT EXISTS migrations (id INTEGER UNIQUE);") {
            Ok(_) => true,
            Err(err) => panic!("Failed to create migrations table:\r\n{:?}", err),
        };

        // Find the last executed migration
        let mut cursor = db
            .prepare("SELECT id FROM migrations ORDER BY id DESC LIMIT 1;")
            .unwrap()
            .cursor();

        // Get the first result to calculate the next migration id
        let next_migration_id: usize = match cursor.next() {
            Ok(None) => 0,
            Ok(Some(row)) => (row[0].as_integer().unwrap() + 1) as usize,
            Err(_) => 0,
        };

        // Enable foreign keys
        db.execute("PRAGMA foreign_keys = ON;").unwrap();

        // Make sure the calculated next migration id is within bounds
        if next_migration_id >= MIGRATIONS.len() {
            return;
        }

        // Iterate over all unexecuted migrations
        for i in next_migration_id..MIGRATIONS.len() {
            run_migration(db, i);
        }
    });
}

fn run_migration(db: &sqlite::Connection, migration_id: usize) {
    // Get the migration query from our static migrations array
    let migration = MIGRATIONS[migration_id].to_string();
    println!(
        "executing migration (id: {})\r\n{}\r\n",
        migration_id, migration
    );

    // Attempt to execute the migration
    match db.execute(&migration) {
        Ok(_) => 0,
        Err(err) => {
            panic!("Migration failed:\r\n{}\r\n\r\n{:?}\r\n", migration, err);
        }
    };

    // Insert a new record indicating this migration has been executed.
    let mut statement = db.prepare("INSERT INTO migrations VALUES(?);").unwrap();
    statement.bind(1, migration_id as i64).unwrap();
    match statement.next() {
        Ok(_) => true,
        Err(err) => panic!("Logging the migration failed:\r\n{:?}", err),
    };

    return;
}
