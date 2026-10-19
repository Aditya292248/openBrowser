mod commands;
mod execution;
mod output;

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

    let command = match commands::parse_args(args) {
        Ok(command) => command,
        Err(error) => {
            output::print_parse_error(&error);
            return ExitCode::from(2);
        }
    };

    match execution::execute(&command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            output::print_execution_error(&error);
            ExitCode::from(1)
        }
    }
}
