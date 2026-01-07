use std::io;
mod tasks;
use std::env;
use tasks::*;

fn main() {
    let mut tasks: Vec<Task> = load_tasks();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        handle_cli(args, &mut tasks)
    } else {
        println!("{}", HELP);
        loop {
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                println!("INVALID!!!!");
                continue;
            }
            let input = input.trim();
            let parts: Vec<&str> = input.split_whitespace().collect();
            if genuine_match(parts, &mut tasks) {
                break;
            };
        }
    }
}
