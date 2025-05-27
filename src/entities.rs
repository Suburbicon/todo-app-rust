use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub struct Config {
    pub action: String,
    pub first_arg: String,
    pub second_arg: Option<String>,
    pub third_arg: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    HOLD,
    PROGRESS,
    DONE,
}

impl Status {
    pub fn from_str(status: &str) -> Result<Status, String> {
        match status.to_uppercase().as_str() {
            "HOLD" => Ok(Status::HOLD),
            "PROGRESS" => Ok(Status::PROGRESS),
            "DONE" => Ok(Status::DONE),
            _ => Err(format!("Invalid status string: '{}'. Allowed: HOLD, PROGRESS, DONE", status)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub description: String,
    pub status: Status
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
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
