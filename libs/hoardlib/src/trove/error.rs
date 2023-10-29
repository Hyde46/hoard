use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TroveError {
    details: String
}

impl TroveError {
    pub fn new(msg: &str) -> Self {
        Self{details: msg.to_string()}
    }
}

impl fmt::Display for TroveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for TroveError {
    fn description(&self) -> &str {
        &self.details
    }
}