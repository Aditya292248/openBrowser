use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Open { url: String },
    Url,
    Title,
    Snapshot,
    Click { target: String },
    Fill { target: String, text: String },
    Close,
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Open { .. } => "open",
            Self::Url => "url",
            Self::Title => "title",
            Self::Snapshot => "snapshot",
            Self::Click { .. } => "click",
            Self::Fill { .. } => "fill",
            Self::Close => "close",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    MissingCommand,
    UnknownCommand(String),
    MissingArgument {
        command: &'static str,
        argument: &'static str,
    },
    InvalidArgument {
        command: &'static str,
        argument: &'static str,
        reason: &'static str,
    },
    UnexpectedArgument {
        command: &'static str,
        value: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "missing command"),
            Self::UnknownCommand(command) => write!(f, "unknown command `{command}`"),
            Self::MissingArgument { command, argument } => {
                write!(f, "`{command}` requires <{argument}>")
            }
            Self::InvalidArgument {
                command,
                argument,
                reason,
            } => write!(f, "invalid <{argument}> for `{command}`: {reason}"),
            Self::UnexpectedArgument { command, value } => {
                write!(f, "`{command}` does not accept extra argument `{value}`")
            }
        }
    }
}

/// Parse arguments after the executable name into a typed command.
pub fn parse_args<I, S>(args: I) -> Result<Command, ParseError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let name = args.next().ok_or(ParseError::MissingCommand)?;

    let command = match name.as_str() {
        "open" => {
            let url = required(&mut args, "open", "url")?;
            validate_url(&url)?;
            Command::Open { url }
        }
        "url" => Command::Url,
        "title" => Command::Title,
        "snapshot" => Command::Snapshot,
        "click" => Command::Click {
            target: required(&mut args, "click", "target")?,
        },
        "fill" => Command::Fill {
            target: required(&mut args, "fill", "target")?,
            text: args.next().ok_or(ParseError::MissingArgument {
                command: "fill",
                argument: "text",
            })?,
        },
        "close" => Command::Close,
        _ => return Err(ParseError::UnknownCommand(name)),
    };

    if let Some(value) = args.next() {
        return Err(ParseError::UnexpectedArgument {
            command: command.name(),
            value,
        });
    }

    Ok(command)
}

fn required(
    args: &mut impl Iterator<Item = String>,
    command: &'static str,
    argument: &'static str,
) -> Result<String, ParseError> {
    let value = args.next().ok_or(ParseError::MissingArgument { command, argument })?;
    if value.trim().is_empty() {
        return Err(ParseError::InvalidArgument {
            command,
            argument,
            reason: "must not be empty",
        });
    }
    Ok(value)
}

fn validate_url(url: &str) -> Result<(), ParseError> {
    let invalid = |reason| ParseError::InvalidArgument {
        command: "open",
        argument: "url",
        reason,
    };

    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| invalid("must start with http:// or https://"))?;

    if url.chars().any(char::is_whitespace) {
        return Err(invalid("must not contain whitespace"));
    }

    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority.rsplit('@').next().unwrap_or_default();
    if host_port.is_empty() || host_port.starts_with(':') {
        return Err(invalid("must include a host"));
    }

    if host_port.contains(['<', '>', '"', '\\']) {
        return Err(invalid("contains an invalid host character"));
    }

    if let Some(port) = host_port.strip_prefix('[') {
        let (address, suffix) = port
            .split_once(']')
            .ok_or_else(|| invalid("invalid IPv6 host"))?;
        if address.parse::<std::net::Ipv6Addr>().is_err() {
            return Err(invalid("invalid IPv6 host"));
        }
        if !suffix.is_empty() {
            let port = suffix
                .strip_prefix(':')
                .ok_or_else(|| invalid("invalid port"))?;
            validate_port(port, &invalid)?;
        }
    } else {
        let (host, port) = match host_port.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (host_port, None),
        };
        if host.is_empty() || host.contains(['[', ']', '@', ':']) {
            return Err(invalid("invalid host"));
        }
        if let Some(port) = port {
            validate_port(port, &invalid)?;
        }
    }

    Ok(())
}

fn validate_port(
    port: &str,
    invalid: &impl Fn(&'static str) -> ParseError,
) -> Result<(), ParseError> {
    if port.is_empty()
        || !port.bytes().all(|byte| byte.is_ascii_digit())
        || port.parse::<u16>().is_err()
    {
        return Err(invalid("port must be a number from 0 to 65535"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Command, ParseError};

    #[test]
    fn parses_each_command() {
        let cases = [
            (
                vec!["open", "https://example.com/path"],
                Command::Open {
                    url: "https://example.com/path".into(),
                },
            ),
            (vec!["url"], Command::Url),
            (vec!["title"], Command::Title),
            (vec!["snapshot"], Command::Snapshot),
            (
                vec!["click", "#submit"],
                Command::Click {
                    target: "#submit".into(),
                },
            ),
            (
                vec!["fill", "#name", "Ada Lovelace"],
                Command::Fill {
                    target: "#name".into(),
                    text: "Ada Lovelace".into(),
                },
            ),
            (vec!["close"], Command::Close),
        ];

        for (args, expected) in cases {
            assert_eq!(parse_args(args), Ok(expected));
        }
    }

    #[test]
    fn reports_missing_command_and_operands() {
        assert_eq!(parse_args(Vec::<&str>::new()), Err(ParseError::MissingCommand));
        assert_eq!(
            parse_args(["open"]),
            Err(ParseError::MissingArgument {
                command: "open",
                argument: "url",
            })
        );
        assert_eq!(
            parse_args(["click"]),
            Err(ParseError::MissingArgument {
                command: "click",
                argument: "target",
            })
        );
        assert_eq!(
            parse_args(["fill", "#name"]),
            Err(ParseError::MissingArgument {
                command: "fill",
                argument: "text",
            })
        );
    }

    #[test]
    fn reports_unknown_and_extra_arguments() {
        assert_eq!(
            parse_args(["navigate"]),
            Err(ParseError::UnknownCommand("navigate".into()))
        );
        assert_eq!(
            parse_args(["url", "extra"]),
            Err(ParseError::UnexpectedArgument {
                command: "url",
                value: "extra".into(),
            })
        );
        assert_eq!(
            parse_args(["fill", "#name", "Ada", "extra"]),
            Err(ParseError::UnexpectedArgument {
                command: "fill",
                value: "extra".into(),
            })
        );
    }

    #[test]
    fn rejects_empty_targets() {
        assert_eq!(
            parse_args(["click", "  "]),
            Err(ParseError::InvalidArgument {
                command: "click",
                argument: "target",
                reason: "must not be empty",
            })
        );
        assert_eq!(
            parse_args(["fill", "", "text"]),
            Err(ParseError::InvalidArgument {
                command: "fill",
                argument: "target",
                reason: "must not be empty",
            })
        );
    }

    #[test]
    fn allows_empty_fill_text_to_clear_a_field() {
        assert_eq!(
            parse_args(["fill", "#name", ""]),
            Ok(Command::Fill {
                target: "#name".into(),
                text: "".into(),
            })
        );
    }

    #[test]
    fn validates_web_urls() {
        assert!(parse_args(["open", "not-a-url"]).is_err());
        assert!(parse_args(["open", "ftp://example.com"]).is_err());
        assert!(parse_args(["open", "https://"]).is_err());
        assert!(parse_args(["open", "http://:80"]).is_err());
        assert!(parse_args(["open", "https://example.com:abc"]).is_err());
        assert!(parse_args(["open", "https://example.com:+80"]).is_err());
        assert!(parse_args(["open", "https://example.com/a b"]).is_err());
        assert!(parse_args(["open", "http://[::1]:8080/"]).is_ok());
    }

    #[test]
    fn error_messages_explain_the_problem() {
        let missing = parse_args(["fill", "#name"]).unwrap_err();
        assert_eq!(missing.to_string(), "`fill` requires <text>");

        let invalid = parse_args(["open", "ftp://example.com"]).unwrap_err();
        assert_eq!(
            invalid.to_string(),
            "invalid <url> for `open`: must start with http:// or https://"
        );
    }
}
