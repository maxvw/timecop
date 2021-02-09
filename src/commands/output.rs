use crate::state::State;
use crate::summary::Summary;
use crate::utils;

use clap::ArgMatches;
use std::error;
use std::io;
use termion::{color, style};

pub fn exec<'a>(
    state: State<'a>,
    matches: ArgMatches<'a>,
) -> Result<State<'a>, Box<dyn error::Error>> {
    // Make sure we have an active project
    if state.project.is_none() {
        utils::error_msg("No Project Found", "Timecop requires a project to be defined before you can start\r\nusing timecop to log entries, first run: $ timecop init");
        std::process::exit(1);
    }

    // Get the summary
    let project = &state.project.as_ref().unwrap();
    let summary = project.summary();
    let detailed = matches.is_present("detailed");

    if matches.is_present("csv") {
        display_csv(summary, detailed)
    } else {
        display_summary(summary, detailed)
    }

    Ok(state)
}

fn display_summary(summary: Summary, detailed: bool) {
    utils::info_msg_compact("Project Summary:", &summary.name);
    println!();

    for day in summary.days {
        let day_name = format!("{}", day.date.format("%A"));
        let date = format!("({})", day.date.format("%-e %B, %Y"));
        utils::info_msg_compact(&day_name, &date);

        for task in day.tasks {
            if detailed {
                println!(
                    "  {}{}{}{}",
                    color::Fg(color::LightWhite),
                    style::Bold,
                    task.name,
                    style::Reset,
                );

                for entry in task.entries {
                    let time = format_time(entry.minutes);
                    println!(
                        "    [{}{}{}] {}{}",
                        color::Fg(color::LightWhite),
                        time,
                        style::Reset,
                        entry.name,
                        style::Reset,
                    );
                }
            } else {
                let time = format_time(task.minutes);
                println!(
                    "  [{}{}{}] {}",
                    color::Fg(color::LightWhite),
                    time,
                    style::Reset,
                    task.name,
                );
            }
        }
        println!();
    }
}

fn display_csv(summary: Summary, detailed: bool) {
    // Create CSV writer to STDOUT
    let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut headers: Vec<&str> = vec!["Project", "Date", "Time Spent (Minutes)", "Task"];

    if detailed {
        headers.push("Log Entry");
    }

    // Write our headers first
    wtr.write_record(&headers).unwrap();

    // Write all the records
    for day in summary.days {
        let date = format!("{}", day.date.format("%Y-%m-%d"));

        for task in day.tasks {
            if detailed {
                for entry in task.entries {
                    wtr.write_record(&[
                        &summary.name,
                        &date,
                        &format!("{}", entry.minutes),
                        &task.name,
                        &entry.name,
                    ])
                    .unwrap();
                }
            } else {
                wtr.write_record(&[
                    &summary.name,
                    &date,
                    &format!("{}", task.minutes),
                    &task.name,
                ])
                .unwrap();
            }
        }
    }

    // output csv
    wtr.flush().unwrap();
}

fn format_time(time: usize) -> String {
    let minutes = time % 60;
    let hours = time / 60;
    format!("{:02}h{:02}m", hours, minutes)
}
