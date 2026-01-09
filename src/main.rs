mod tasks;
use clap::Parser;
use tasks::{Cli, Task, handle_command, interactive_mode, load_tasks};

fn main() {
    let mut tasks: Vec<Task> = load_tasks();

    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => {
            if handle_command(cmd, &mut tasks) {
                return;
            }
        }
        None => {
            if interactive_mode(&mut tasks) {
                return;
            }
        }
    }
}
