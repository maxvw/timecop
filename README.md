# Timecop
A small utility to help you keep track of time.

## Get started
- Download and unarchive the release
- Copy the `bin/timecop` binary to wherever you want (_/usr/local/bin/timecop?_)
- Run `$ timecop init` in a local Git repo

> **NOTE:** Currently I have only build and used for MacOS. It is not signed and you either have to right-click on the binary and click Open to allow MacOS to run it, or download the source and build it for yourself.

## Description
I needed a way to easily keep track of what I've been working on, I started by keeping track in a simple text file but as expected this quickly became unwieldly. Of course there are a plentitude of tools available to help you do this, but I took this opportunity to have a new little side project. I built it using Rust, a language I am not familiar with at all, and my main goal is to make something that works for me and my workflow. If it works for you too, that's even better!

![Animated demonstration](docs/demo.svg?raw=true "Demo")

The demo really quickly runs through some of the possibilities, but run `timecop help` and play around with it a little bit before actually using it. Resetting is as simple as throwing away the database file (`~/.timecopdb`).

## How does it work?
The basic concept exists out of a `Project` with `Tasks`, and you can log time within a `Task`.

- A `Project` is recognised by the current Git repository's origin url.
- A `Task` is recognised by the current Git branch.

If you use the included `post-commit` hook, it will prompt you for an estimate on the time spent working on this commit. For new branches it will also prompt you to check if this is a new task, or an existing task. Sometimes work on a task gets split over multiple branches (creating, bugfixes, etc.) so a task can be connected with multiple branches.

You can then view your output with `timecop output` (add more detail with `--detail`), or even export them as CSV with `timecop output --csv` to process with whatever tool you have at your disposal.

> **NOTE:** About data storage, it's completely local using a SQLite database located at `~/.timecopdb`, I would still avoid storing sensitive data in your log entries.

## `timecop help`
```
timecop 0.1.0
helps you keep track of time spent working.

USAGE:
    timecop <SUBCOMMAND>

OPTIONS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    init      initialize a new project
    log       add a new entry for this project
    output    output the tasks performed by day for this project
    help      Prints this message or the help of the given subcommand(s)
```

## Contributing
Feel free to open PRs with improvements.
