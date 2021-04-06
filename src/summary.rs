use crate::database;
use crate::project::Project;
use crate::utils;

use chrono::NaiveDate;
use std::vec::Vec;

#[derive(Debug)]
pub struct Summary {
    pub id: usize,
    pub name: String,
    pub days: Vec<SummarizedDay>,
}

#[derive(Debug)]
pub struct SummarizedDay {
    pub date: NaiveDate,
    pub minutes: usize,
    pub tasks: Vec<SummarizedTask>,
}

#[derive(Debug)]
pub struct SummarizedTask {
    pub id: usize,
    pub name: String,
    pub minutes: usize,
    pub entries: Vec<SummarizedTaskEntry>,
}

#[derive(Debug)]
pub struct SummarizedTaskEntry {
    pub name: String,
    pub minutes: usize,
}

pub fn for_project(project: &Project) -> Summary {
    let mut results: Vec<SummarizedDay> = Vec::new();

    database::with_db(|db| {
        // List all projects
        let mut cursor = db
            .prepare(
                "
                    SELECT
                        t.id,
                        t.name,
                        l.name as log,
                        l.minutes,
                        l.inserted_at as date,
                        (
                            SELECT
                                SUM(l2.minutes)
                            FROM task_logs l2
                            WHERE l2.task_id = l.task_id
                            AND DATE(l2.inserted_at) = DATE(l.inserted_at)
                        ) as minutes_total
                    FROM task_logs l
                    LEFT JOIN tasks t ON t.id = l.task_id
                    WHERE t.project_id = ?
                    GROUP BY l.id, DATE(l.inserted_at)
                    ORDER BY DATE(l.inserted_at) DESC, t.id DESC;",
            )
            .unwrap()
            .into_cursor();

        cursor
            .bind(&[sqlite::Value::Integer(project.id as i64)])
            .unwrap();

        results = process_summary(cursor);
    });

    Summary {
        id: project.id,
        name: project.name.to_string(),
        days: results,
    }
}

fn process_summary(mut cursor: sqlite::Cursor) -> Vec<SummarizedDay> {
    let mut results: Vec<SummarizedDay> = Vec::new();

    while let Some(row) = cursor.next().unwrap() {
        let mut summary = process_summary_day(row);
        let mut task_summary = process_summary_task(row);
        let task_entry = process_summary_task_entry(row);
        let last_index = results.len();

        // TODO: There has got to be a better way.
        if last_index > 0 {
            let previous = &mut results[last_index - 1];
            if previous.date == summary.date {
                previous.minutes += summary.minutes;
                let last_task_index = previous.tasks.len();
                if last_task_index > 0 {
                    let previous_task = &mut previous.tasks[last_task_index - 1];
                    if previous_task.id == task_summary.id {
                        previous_task.minutes += task_entry.minutes;
                        previous_task.entries.push(task_entry);
                    } else {
                        task_summary.minutes = task_entry.minutes;
                        task_summary.entries.push(task_entry);
                        previous.tasks.push(task_summary);
                    }
                } else {
                    task_summary.minutes = task_entry.minutes;
                    task_summary.entries.push(task_entry);
                    previous.tasks.push(task_summary);
                }
            } else {
                task_summary.minutes = task_entry.minutes;
                task_summary.entries.push(task_entry);
                summary.tasks.push(task_summary);
                results.push(summary);
            }
        } else {
            task_summary.minutes = task_entry.minutes;
            task_summary.entries.push(task_entry);
            summary.tasks.push(task_summary);
            results.push(summary);
        }
    }

    results
}

fn process_summary_day(row: &[sqlite::Value]) -> SummarizedDay {
    let minutes = row[3].as_integer().unwrap() as usize;
    let date = utils::sql_to_date(row[4].as_string()).unwrap();

    SummarizedDay {
        date,
        minutes,
        tasks: Vec::new(),
    }
}

fn process_summary_task(row: &[sqlite::Value]) -> SummarizedTask {
    let id = row[0].as_integer().unwrap() as usize;
    let name = row[1].as_string().unwrap().to_string();

    SummarizedTask {
        id,
        name,
        minutes: 0,
        entries: Vec::new(),
    }
}

fn process_summary_task_entry(row: &[sqlite::Value]) -> SummarizedTaskEntry {
    let name = row[2].as_string().unwrap().to_string();
    let minutes = row[3].as_integer().unwrap() as usize;
    SummarizedTaskEntry { name, minutes }
}
