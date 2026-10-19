use crate::commands::ParseError;
use crate::execution::ExecutionError;

pub const USAGE: &str = "Usage: my_browser <command> [arguments]\n\
\n\
Commands:\n\
  open <url>             Open an HTTP or HTTPS URL\n\
  url                    Show the current URL\n\
  title                  Show the page title\n\
  snapshot               Show a page snapshot\n\
  click <target>         Click a target\n\
  fill <target> <text>   Fill a target with text\n\
  close                  Close the browser\n\
  daemon start           Start the background browser daemon\n\
  daemon serve           Run the daemon in this terminal\n\
\n\
Use --help to show this message. Quote arguments containing spaces.";

pub fn print_usage() {
    println!("{USAGE}");
}

pub fn print_parse_error(error: &ParseError) {
    eprintln!("error: {error}\n\n{USAGE}");
}

pub fn print_argument_encoding_error() {
    eprintln!("error: arguments must be valid UTF-8\n\n{USAGE}");
}

pub fn print_execution_error(error: &ExecutionError) {
    eprintln!("error: {error}");
}

pub fn print_success(result: &str) {
    println!("{result}");
}
