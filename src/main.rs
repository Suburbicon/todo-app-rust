use std::env;
use std::io::Read;
use std::process;
use std::fs::File;
use std::path::Path;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::io::{Error, Write, ErrorKind};

#[derive(Debug)]
struct Config {
    action: String,
    first_arg: String,
    second_arg: Option<String>,
    third_arg: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    HOLD,
    PROGRESS,
    DONE,
}

impl Status {
    fn from_str(status: &str) -> Result<Status, String> {
        match status.to_uppercase().as_str() {
            "HOLD" => Ok(Status::HOLD),
            "PROGRESS" => Ok(Status::PROGRESS),
            "DONE" => Ok(Status::DONE),
            _ => Err(format!("Invalid status string: '{}'. Allowed: HOLD, PROGRESS, DONE", status)),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    id: i32,
    description: String,
    status: Status
}

impl Config {
    fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
        args.next();

        let action = match args.next() {
            Some(val) => val,
            None => return Err("No action in command")
        };

        let first_arg = match args.next() {
            Some(val) => val,
            None => {
                if action.eq_ignore_ascii_case("list") {
                    String::new()
                } else if action.eq_ignore_ascii_case("add") {
                    return Err("No description provided for 'add' action.");
                } else {
                    return Err("Missing required argument (e.g., ID or description).");
                }
            }
        };

        let second_arg = args.next();
        let third_arg = args.next();

        Ok(Self{
            action, first_arg, second_arg, third_arg
        })
    }
}

fn write_or_update_file<T: Serialize>(file_path: &Path, data: &T) -> Result<(), Error> {
    let json_string = serde_json::to_string_pretty(data)
        .map_err(|err| Error::new(ErrorKind::InvalidData, format!("Failed to serialize data to JSON: {}", err)))?;

    let mut file = File::create(file_path)?;
    file.write_all(json_string.as_bytes())?;
    file.flush()?;
    println!("Successfully wrote/updated JSON data to '{}'", file_path.display());
    Ok(())
}

fn read_tasks_from_file(file_path: &Path) -> Result<Vec<Task>, Error> {
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&contents)
        .map_err(|err| Error::new(ErrorKind::InvalidData, format!("Failed to deserialize tasks: {}", err)))
}

fn main() -> Result<(), Error> {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1)
    });

    if config.action.eq("add") {
        let description = config.first_arg;
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
    } else if config.action.eq("delete") {
        let task_id_str = &config.first_arg;
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

    } else if config.action.eq("edit") {
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

    } else if config.action.eq("list") {
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

    Ok(())
}