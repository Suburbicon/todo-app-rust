use std::env;
use std::process;
use std::path::Path;
use rand::Rng;
use std::io::{Error};

mod utils;
mod entities;

use utils::{write_or_update_file, read_tasks_from_file};
use entities::{Config, Task, Status};


fn main() -> Result<(), Error> {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1)
    });

    match config.action.as_str() {
        "add" => handle_add_action(config.first_arg),
        "delete" => handle_delete_action(&config.first_arg),
        "edit" => handle_edit_action(config),
        "list" => handle_list_action(),
        unknown_action => {
            eprintln!("Unknown action: {}", unknown_action);
            eprintln!("Available actions: add, list, delete, edit");
            process::exit(1);
        }
    }

    Ok(())
}

fn handle_add_action(first_arg: String) {
    let description = first_arg;
    println!("Adding new task: \"{}\"", description);
    let config_file_path = Path::new("tasks.json");

    let mut tasks = read_tasks_from_file(config_file_path).unwrap_or_else(|err| {
        eprintln!("Warning: Could not read tasks file (may be new or invalid): {}. Starting fresh.", err);
        Vec::new()
    });

    let mut rng = rand::thread_rng();
    let id = rng.gen_range(1..1000);
    let new_task = Task {
        id, description, status: Status::HOLD
    };
    tasks.push(new_task);

    if let Err(e) = write_or_update_file(config_file_path, &tasks) {
        eprintln!("Failed to write initial content: {}", e);
    } else {
        println!("Task added successfully.");
    }
}

fn handle_delete_action(first_arg: &String) {
    let task_id_str = first_arg;
    let task_id: i32 = match task_id_str.parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Error: Invalid Task ID '{}'. Must be a number.", task_id_str);
            process::exit(1);
        }
    };

    println!("Deleting task id: {}", task_id);

    let config_file_path = Path::new("tasks.json");
    let mut tasks = read_tasks_from_file(config_file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read tasks file: {}. Cannot edit.", err);
        process::exit(1);
    });

    tasks.retain(|t| t.id != task_id);

    if let Err(e) = write_or_update_file(config_file_path, &tasks) {
        eprintln!("Failed to task: {}", e);
    } else {
        println!("Task ID {} deleted successfully.", task_id);
    }
}

fn handle_edit_action(config: Config) {
    let task_id_str = config.first_arg;
    let task_id: i32 = match task_id_str.parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Error: Invalid Task ID '{}'. Must be a number.", task_id_str);
            process::exit(1);
        }
    };
    println!("Editing task id: {}", task_id);

    let new_description = config.second_arg.as_deref();
    let new_status = config.third_arg.as_deref();

    if new_description.is_none() && new_status.is_none() {
        eprintln!("Error: For 'edit', you must provide a new description and/or a new status.");
        eprintln!("Usage: todo edit <task_id> \"<new_description>\" [NEW_STATUS:HOLD|PROGRESS|DONE]");
        process::exit(1);
    }

    let config_file_path = Path::new("tasks.json");
    let mut tasks = read_tasks_from_file(config_file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read tasks file: {}. Cannot edit.", err);
        process::exit(1);
    });

    let mut task_is_found = false;
    for task in tasks.iter_mut() {
        if task.id == task_id {
            if let Some(desc) = new_description {
                if !desc.is_empty() {
                    task.description = desc.to_string();
                    println!("Updated description for task ID {}", task_id);
                } else {
                    eprintln!("Warning: New description is empty, not updating description for task ID {}.", task_id)
                }
            }
            if let Some(status) = new_status {
                match Status::from_str(status) {
                    Ok(s) => {
                        task.status = s;
                        println!("Updated status for task ID {}", task_id);
                    },
                    Err(e) => {
                        eprintln!("Error updating status for task ID {}: {}", task_id, e);
                    }
                }
            }
            task_is_found = true;
            break;
        }
    }

    if task_is_found {
        if let Err(e) = write_or_update_file(config_file_path, &tasks) {
            eprintln!("Failed to write updated tasks to file: {}", e);
        } else {
            println!("Task ID {} updated successfully.", task_id);
        }
    } else {
        println!("Task with ID {} not found.", task_id);
    }
}

fn handle_list_action() {
    println!("Listing all tasks...");

    let config_file_path = Path::new("tasks.json");
    match read_tasks_from_file(config_file_path) {
        Ok(tasks) => {
            if tasks.is_empty() {
                println!("No tasks found.");
            } else {
                for task in tasks {
                    println!("ID: {}, Desc: \"{}\", Status: {:?}", task.id, task.description, task.status)
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read tasks file: {}", e);
        }
    }
}