use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct HoardErr {
    details: String,
}

impl HoardErr {
    pub fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for HoardErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for HoardErr {
    fn description(&self) -> &str {
        &self.details
    }
}
