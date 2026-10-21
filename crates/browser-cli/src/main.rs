mod browser;
mod commands;
mod daemon;
mod execution;
mod output;
mod protocol;
mod session;

use std::process::ExitCode;

fn main() -> ExitCode {
    let raw_args: Vec<_> = std::env::args_os().skip(1).collect();
    let json = raw_args
        .first()
        .is_some_and(|arg| arg == std::ffi::OsStr::new("--json"));
    let format = if json {
        output::OutputFormat::Json
    } else {
        output::OutputFormat::Text
    };
    let args: Vec<String> = match raw_args
        .into_iter()
        .skip(usize::from(json))
        .map(|arg| arg.into_string())
        .collect()
    {
        Ok(args) => args,
        Err(_) => {
            output::print_argument_encoding_error(format);
            return ExitCode::from(2);
        }
    };
    if matches!(args.as_slice(), [flag] if flag == "--help" || flag == "-h") {
        output::print_usage(format);
        return ExitCode::SUCCESS;
    }

    match args.as_slice() {
        [daemon, action] if daemon == "daemon" && action == "start" => {
            return match daemon::start_detached() {
                Ok(daemon::StartStatus::Started) => {
                    output::print_success(format, "daemon start", "browser daemon started");
                    ExitCode::SUCCESS
                }
                Ok(daemon::StartStatus::AlreadyRunning) => {
                    output::print_success(
                        format,
                        "daemon start",
                        "browser daemon is already running",
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    output::print_execution_error(
                        format,
                        "daemon start",
                        &execution::ExecutionError(error),
                    );
                    ExitCode::from(1)
                }
            };
        }
        [daemon, action] if daemon == "daemon" && action == "serve" => {
            if format == output::OutputFormat::Json {
                output::print_error(
                    format,
                    Some("daemon serve"),
                    "`daemon serve` cannot use --json because it keeps running",
                    false,
                );
                return ExitCode::from(2);
            }
            return match daemon::serve() {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    output::print_execution_error(
                        format,
                        "daemon serve",
                        &execution::ExecutionError(error),
                    );
                    ExitCode::from(1)
                }
            };
        }
        _ => {}
    }

    let command = match commands::parse_args(args) {
        Ok(command) => command,
        Err(error) => {
            output::print_parse_error(format, &error);
            return ExitCode::from(2);
        }
    };

    if matches!(command, commands::Command::Open { .. }) {
        if let Err(error) = daemon::start_detached() {
            output::print_execution_error(
                format,
                command.name(),
                &execution::ExecutionError(error),
            );
            return ExitCode::from(1);
        }
    }

    match execution::execute(&command) {
        Ok(result) => {
            output::print_success(format, command.name(), &result);
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::print_execution_error(format, command.name(), &error);
            ExitCode::from(1)
        }
    }
}
