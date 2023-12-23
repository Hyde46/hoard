use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CommandError {
    details: String,
}

impl CommandError {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for CommandError {
    fn description(&self) -> &str {
        &self.details
    }
}