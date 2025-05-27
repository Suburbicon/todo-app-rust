use std::path::Path;
use std::io::Read;
use std::fs::File;
use serde::{Serialize};
use std::io::{Error, Write, ErrorKind};

use crate::entities::Task;


pub fn write_or_update_file<T: Serialize>(file_path: &Path, data: &T) -> Result<(), Error> {
    let json_string = serde_json::to_string_pretty(data)
        .map_err(|err| Error::new(ErrorKind::InvalidData, format!("Failed to serialize data to JSON: {}", err)))?;

    let mut file = File::create(file_path)?;
    file.write_all(json_string.as_bytes())?;
    file.flush()?;

    println!("Successfully wrote/updated JSON data to '{}'", file_path.display());

    Ok(())
}

pub fn read_tasks_from_file(file_path: &Path) -> Result<Vec<Task>, Error> {
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