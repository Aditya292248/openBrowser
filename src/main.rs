mod browser;
mod commands;
mod daemon;
mod execution;
mod output;
mod protocol;
mod session;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = match std::env::args_os()
        .skip(1)
        .map(|arg| arg.into_string())
        .collect()
    {
        Ok(args) => args,
        Err(_) => {
            output::print_argument_encoding_error();
            return ExitCode::from(2);
        }
    };
    if matches!(args.as_slice(), [flag] if flag == "--help" || flag == "-h") {
        output::print_usage();
        return ExitCode::SUCCESS;
    }

    match args.as_slice() {
        [daemon, action] if daemon == "daemon" && action == "start" => {
            return match daemon::start_detached() {
                Ok(daemon::StartStatus::Started) => {
                    output::print_success("browser daemon started");
                    ExitCode::SUCCESS
                }
                Ok(daemon::StartStatus::AlreadyRunning) => {
                    output::print_success("browser daemon is already running");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    output::print_execution_error(&execution::ExecutionError(error));
                    ExitCode::from(1)
                }
            };
        }
        [daemon, action] if daemon == "daemon" && action == "serve" => {
            return match daemon::serve() {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    output::print_execution_error(&execution::ExecutionError(error));
                    ExitCode::from(1)
                }
            };
        }
        _ => {}
    }

    let command = match commands::parse_args(args) {
        Ok(command) => command,
        Err(error) => {
            output::print_parse_error(&error);
            return ExitCode::from(2);
        }
    };

    if matches!(command, commands::Command::Open { .. }) {
        if let Err(error) = daemon::start_detached() {
            output::print_execution_error(&execution::ExecutionError(error));
            return ExitCode::from(1);
        }
    }

    match execution::execute(&command) {
        Ok(result) => {
            output::print_success(&result);
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::print_execution_error(&error);
            ExitCode::from(1)
        }
    }
}
