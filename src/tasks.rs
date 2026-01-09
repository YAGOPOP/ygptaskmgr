use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, io};

#[derive(Parser)]
#[command(name = "ygptaskmgr")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(ValueEnum, Clone)]
pub enum StatusFilter {
    Done,
    Undone,
    All,
}

#[derive(ValueEnum, Clone)]
pub enum SortBy {
    NoSort,
    Created,
    Deadline,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Add a new task")]
    Add { text: Vec<String> },

    #[command(about = "List tasks")]
    List {
        #[arg(long, value_enum, default_value = "all")]
        status: StatusFilter,

        #[arg(long, value_enum, default_value = "no-sort")]
        sort: SortBy,
    },

    #[command(about = "Remove task")]
    Remove { index: usize },

    #[command(about = "Mark task as done")]
    Do { index: usize },

    #[command(about = "Mark task as undone")]
    Undo { index: usize },

    #[command(about = "Set task's deadline")]
    Deadline {
        index: usize,

        #[arg(help = "Deadline date")]
        date: String,

        #[arg(
            default_value = "23:59",
            help = "Deadline time (HH:MM) (default: 23:59)"
        )]
        time: String,
    },

    #[command(about = "Exit programme")]
    Exit,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub objective: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
}

// fs processing
pub fn load_tasks() -> Vec<Task> {
    let data_dir = match get_data_dir() {
        Some(v) => v,
        None => {
            println!("something went wrong during getting data_dir");
            return Vec::new();
        }
    };
    match std::fs::read_to_string(data_dir.join("tasks.json")) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| {
            eprintln!("Failed to parse tasks.json, starting empty");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

fn get_data_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("", "yagopop", "ygptaskmgr") {
        let dir = proj_dirs.data_dir(); // стандартная папка для данных
        fs::create_dir_all(dir).ok()?; // создаём, если не существует
        Some(PathBuf::from(dir))
    } else {
        None
    }
}

fn save_tasks(tasks: &[Task]) {
    let json: String = serde_json::to_string_pretty(tasks).expect("Serialize failed");

    let data_dir: PathBuf = match get_data_dir() {
        Some(v) => v,
        None => {
            eprintln!("something went wrong during getting data_dir");
            return;
        }
    };

    if let Err(e) = fs::write(data_dir.join("tasks.json"), json) {
        eprintln!("Failed to save tasks: {}", e);
    }
}

pub fn handle_command(cmd: Commands, tasks: &mut Vec<Task>) -> bool {
    match cmd {
        Commands::Add { text } => {
            add_task_from_string(text.join(" "), tasks);
            return false;
        }
        Commands::List { status, sort } => {
            smart_list_tasks(status, sort, tasks);
            return false;
        }
        Commands::Remove { index } => {
            remove_task_by_index(index, tasks);
            return false;
        }
        Commands::Do { index } => {
            set_status(index, true, tasks);
            return false;
        }
        Commands::Undo { index } => {
            set_status(index, false, tasks);
            return false;
        }
        Commands::Deadline { index, date, time } => {
            set_deadline(index, date, time, tasks);
            return false;
        }
        Commands::Exit => {
            println!("Goodbye");
            return true;
        }
    }
}

pub fn interactive_mode(tasks: &mut Vec<Task>) -> bool {
    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("failed to read input");
            continue;
        }

        let input: &str = input.trim();

        // Разбиваем строку как shell
        let mut args: Vec<&str> = vec!["ygptaskmgr"];
        args.extend(input.split_whitespace());

        match Cli::try_parse_from(args) {
            Ok(cmd) => {
                let c = match cmd.command {
                    Some(v) => v,
                    None => continue,
                };
                if handle_command(c, tasks) {
                    return true;
                };
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn add_task_from_string(text: String, tasks: &mut Vec<Task>) {
    tasks.push(Task {
        objective: text,
        done: false,
        created_at: Utc::now(),
        deadline: None,
    });
    save_tasks(tasks);
    println!("Task added.")
}

fn remove_task_by_index(index: usize, tasks: &mut Vec<Task>) {
    if index > tasks.len() || index == 0 {
        eprintln!("Invalid index");
        return;
    } else {
        tasks.remove(index - 1);
        save_tasks(tasks);
        println!("Task {} was removed", index);
    }
}

fn set_status(num: usize, state: bool, tasks: &mut Vec<Task>) {
    if num < tasks.len() && num != 0 {
        let n = num - 1;
        let s: bool = tasks[n].done;
        if s == state {
            if s == true {
                println!("Task {num} was already done")
            } else {
                println!("Task {num} is still undone")
            }
        } else {
            tasks[n].done = state;
            save_tasks(tasks);
            println!("State of task {num} set accordingly")
        }
    } else {
        eprintln!("Invalid index")
    }
}

fn smart_list_tasks(status: StatusFilter, sort: SortBy, tasks: &[Task]) {
    let mut items: Vec<(usize, &Task)> = tasks.iter().enumerate().collect();

    // фильтрация
    match status {
    StatusFilter::Done => items.retain(|(_, t)| t.done),
    StatusFilter::Undone => items.retain(|(_, t)| !t.done),
    StatusFilter::All => {}
}

    // сортировка
    match sort {
    SortBy::Created => items.sort_by_key(|(_, t)| t.created_at),
    SortBy::Deadline => items.sort_by_key(|(_, t)| t.deadline),
    SortBy::NoSort => {}
}

    // вывод
    for (i, task) in items {
        let created_local = task.created_at.with_timezone(&Local);
        let deadline_local = task.deadline.map(|date| date.with_timezone(&Local));

        println!(
            "{} [{}] {} (created: {}, deadline: {})",
            i + 1,
            if task.done { "x" } else { " " },
            task.objective,
            created_local.format("%Y-%m-%d %H:%M"),
            deadline_local
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "-".into())
                .underline()
        );
    }
}

fn set_deadline(index: usize, date: String, time: String, tasks: &mut Vec<Task>) {
    let string_date: String = format!("{} {}", date, time);

    let naive = match NaiveDateTime::parse_from_str(&string_date, "%Y-%m-%d %H:%M") {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    let local_dt = match Local.from_local_datetime(&naive).single() {
        Some(v) => v,
        None => {
            eprintln!("something went wrong during parsing local_dt");
            return;
        }
    };
    let utc_dt = local_dt.with_timezone(&Utc);
    tasks[index - 1].deadline = Some(utc_dt);
    save_tasks(tasks);
    println!("Date changed");
}
