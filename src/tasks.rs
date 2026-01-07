use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const HELP: &str = include_str!(".././assets/help.txt");

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

pub fn save_tasks(tasks: &Vec<Task>) {
    let json = serde_json::to_string_pretty(tasks).expect("Serialize failed");

    let data_dir = match get_data_dir() {
        Some(v) => v,
        None => {
            println!("something went wrong during getting data_dir");
            return;
        }
    };

    if let Err(e) = fs::write(data_dir.join("tasks.json"), json) {
        eprintln!("Failed to save tasks: {}", e);
    }
}

// working with tasks
pub fn add_task(parts: &Vec<&str>, tasks: &mut Vec<Task>) {
    if parts.len() < 2 {
        println!("Usage: add <task>");
        return;
    }
    let task = Task {
        objective: parts[1..].join(" "),
        done: false,
        created_at: Utc::now(),
        deadline: None,
    };

    tasks.push(task);
    save_tasks(&tasks);
    println!("Task added");
}

pub fn parse_index(parts: &Vec<&str>, tasks_len: usize) -> Option<usize> {
    let ind = parts.get(1)?;

    let n: usize = ind.parse().ok()?;

    if n == 0 || n > tasks_len {
        println!("Task does not exist");
        None
    } else {
        Some(n - 1)
    }
}

pub fn list_tasks(tasks: &Vec<Task>) {
    if tasks.is_empty() {
        println!("nothing to list");
    } else {
        for (i, task) in tasks.iter().enumerate() {
            let status = if task.done { "[x]" } else { "[ ]" };
            let created_local = task.created_at.with_timezone(&Local);

            let deadline_local = task.deadline.map(|date| date.with_timezone(&Local));

            println!(
                "{} {} {} (created: {}, deadline: {})",
                i + 1,
                status,
                task.objective,
                created_local.format("%Y-%m-%d %H:%M"),
                deadline_local
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "-".into())
            );
        }
    }
}

pub fn remove_task(parts: &Vec<&str>, tasks: &mut Vec<Task>) {
    let ind = match parse_index(parts, tasks.len()) {
        Some(i) => i,
        None => return,
    };

    tasks.remove(ind);
    save_tasks(&tasks);
    println!("Task {} removed", ind + 1);
}

pub fn done_task(parts: &Vec<&str>, tasks: &mut Vec<Task>, state: bool) {
    let ind = match parse_index(parts, tasks.len()) {
        Some(i) => i,
        None => return,
    };

    tasks[ind].done = state;
    save_tasks(&tasks);
    println!("State of task {} changed accordingly", ind + 1);
}

pub fn set_deadline(parts: &Vec<&str>, tasks: &mut Vec<Task>) {
    let ind = match parse_index(parts, tasks.len()) {
        Some(i) => i,
        None => return,
    };

    let date = match parts.get(2) {
        Some(v) => v,
        None => return,
    };
    let time = parts.get(3).copied().unwrap_or("23:59");
    let string_date = format!("{} {}", date, time);

    let naive = match NaiveDateTime::parse_from_str(&string_date, "%Y-%m-%d %H:%M") {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    let local_dt = match Local.from_local_datetime(&naive).single() {
        Some(v) => v,
        None => {
            println!("something went wrong during parsing local_dt");
            return;
        }
    };

    let utc_dt = local_dt.with_timezone(&Utc);

    tasks[ind].deadline = Some(utc_dt);
    save_tasks(tasks);
    println!("Date changed");
}

pub fn genuine_match(parts: Vec<&str>, tasks: &mut Vec<Task>) -> bool {
    match parts.get(0) {
        Some(&"add") => {
            add_task(&parts, tasks);
            return false;
        }
        Some(&"list") => {
            list_tasks(&tasks);
            return false;
        }
        Some(&"remove") => {
            remove_task(&parts, tasks);
            return false;
        }
        Some(&"do") => {
            done_task(&parts, tasks, true);
            return false;
        }
        Some(&"undo") => {
            done_task(&parts, tasks, false);
            return false;
        }
        Some(&"deadline") => {
            set_deadline(&parts, tasks);
            return false;
        }
        Some(&"help") => {
            println!("{}", HELP);
            return false;
        }
        Some(&"exit") => {
            println!("okay, killing myself(((");
            return true;
        }
        _ => {
            println!("no valid command provided");
            return false;
        }
    }
}

pub fn handle_cli(args: Vec<String>, tasks: &mut Vec<Task>) {
    let parts: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    genuine_match(parts, tasks);
}
