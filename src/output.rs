use crate::commands::ParseError;
use crate::execution::ExecutionError;
use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub const USAGE: &str = "Usage: my_browser [--json] <command> [arguments]\n\
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
Use --help to show this message. Put --json before the command for one JSON result. Quote arguments containing spaces.";

pub fn print_usage(format: OutputFormat) {
    print_success(format, "help", USAGE);
}

pub fn print_parse_error(format: OutputFormat, error: &ParseError) {
    print_error(format, None, &error.to_string(), true);
}

pub fn print_argument_encoding_error(format: OutputFormat) {
    print_error(format, None, "arguments must be valid UTF-8", true);
}

pub fn print_execution_error(format: OutputFormat, command: &str, error: &ExecutionError) {
    print_error(format, Some(command), &error.to_string(), false);
}

pub fn print_success(format: OutputFormat, command: &str, result: &str) {
    match format {
        OutputFormat::Text => println!("{result}"),
        OutputFormat::Json => println!("{}", json_success(command, result)),
    }
}

pub fn print_error(format: OutputFormat, command: Option<&str>, message: &str, show_usage: bool) {
    match format {
        OutputFormat::Text if show_usage => eprintln!("error: {message}\n\n{USAGE}"),
        OutputFormat::Text => eprintln!("error: {message}"),
        OutputFormat::Json => println!("{}", json_error(command, message)),
    }
}

fn json_success(command: &str, result: &str) -> Value {
    json!({"ok": true, "command": command, "result": result})
}

fn json_error(command: Option<&str>, message: &str) -> Value {
    json!({"ok": false, "command": command, "error": message})
}

#[cfg(test)]
mod tests {
    use super::{json_error, json_success};

    #[test]
    fn json_result_preserves_multiline_page_content_on_one_line() {
        let result = "<html>\n<title>\"Hi\"</title>\n</html>";
        let output = json_success("snapshot", result).to_string();
        assert!(!output.contains('\n'));
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["command"], "snapshot");
        assert_eq!(parsed["result"], result);
    }

    #[test]
    fn json_error_distinguishes_parse_and_execution_errors() {
        assert!(json_error(None, "missing command")["command"].is_null());
        let output = json_error(Some("title"), "no page is open");
        assert_eq!(output["ok"], false);
        assert_eq!(output["command"], "title");
        assert_eq!(output["error"], "no page is open");
    }
}
