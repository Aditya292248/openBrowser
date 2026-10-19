use crate::commands::Command;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    BrowserNotIntegrated { command: &'static str },
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrowserNotIntegrated { command } => {
                write!(f, "`{command}` was parsed, but browser execution is not implemented yet")
            }
        }
    }
}

pub fn execute(command: &Command) -> Result<(), ExecutionError> {
    Err(ExecutionError::BrowserNotIntegrated {
        command: command.name(),
    })
}
