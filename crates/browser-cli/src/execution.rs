use crate::commands::Command;
use crate::protocol::{self, Request, Response};
use crate::session::{self, DaemonInfo};
use std::fmt;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
pub struct ExecutionError(pub String);

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn execute(command: &Command) -> Result<String, ExecutionError> {
    let info = session::read().map_err(ExecutionError)?;
    execute_with_info(&info, command)
}

pub(crate) fn execute_with_info(
    info: &DaemonInfo,
    command: &Command,
) -> Result<String, ExecutionError> {
    let mut stream = TcpStream::connect_timeout(&info.address().into(), Duration::from_secs(2))
        .map_err(|error| {
            ExecutionError(format!(
                "cannot connect to browser daemon: {error}; run `my_browser daemon start`"
            ))
        })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(|error| ExecutionError(format!("cannot set read timeout: {error}")))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| ExecutionError(format!("cannot set write timeout: {error}")))?;

    let request = Request {
        token: info.token.clone(),
        command: command.clone(),
    };
    protocol::write_message(&mut stream, &request)
        .map_err(|error| ExecutionError(format!("cannot send command to daemon: {error}")))?;
    match protocol::read_message::<Response>(&mut stream)
        .map_err(|error| ExecutionError(format!("cannot read daemon response: {error}")))?
    {
        Response::Ok { output } => Ok(output),
        Response::Err { error } => Err(ExecutionError(error)),
    }
}
