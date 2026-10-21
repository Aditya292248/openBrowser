use crate::browser::{BrowserExecutor, ChromeExecutor};
use crate::commands::Command;
use crate::protocol::{self, Request, Response};
use crate::session::{self, DaemonInfo};
use std::env;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::process::{Command as ProcessCommand, Stdio};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub enum StartStatus {
    Started,
    AlreadyRunning,
}

pub fn start_detached() -> Result<StartStatus, String> {
    if let Ok(info) = session::read() {
        if TcpStream::connect_timeout(&info.address().into(), Duration::from_millis(300)).is_ok() {
            return Ok(StartStatus::AlreadyRunning);
        }
    }

    let executable = env::current_exe().map_err(|error| format!("cannot locate CLI: {error}"))?;
    let mut process = ProcessCommand::new(executable);
    process
        .args(["daemon", "serve"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        process.process_group(0);
    }

    let mut child = process
        .spawn()
        .map_err(|error| format!("could not start browser daemon: {error}"))?;

    for _ in 0..100 {
        if let Ok(info) = session::read() {
            if TcpStream::connect_timeout(&info.address().into(), Duration::from_millis(100))
                .is_ok()
            {
                return Ok(StartStatus::Started);
            }
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("cannot check daemon process: {error}"))?
        {
            return Err(format!(
                "browser daemon exited during startup ({status}); run `my_browser daemon serve` for details"
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }

    let _ = child.kill();
    let _ = child.wait();
    Err("browser daemon did not become ready within 10 seconds".to_string())
}

pub fn serve() -> Result<(), String> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
        .map_err(|error| format!("cannot bind browser daemon: {error}"))?;
    let info = DaemonInfo {
        port: listener
            .local_addr()
            .map_err(|error| format!("cannot read daemon address: {error}"))?
            .port(),
        token: Uuid::new_v4().to_string(),
    };
    let _guard = session::publish(&info)?;
    println!("browser daemon ready (pid {})", std::process::id());
    let mut browser = ChromeExecutor::default();
    serve_requests(listener, &info.token, &mut browser)
}

fn serve_requests<E: BrowserExecutor>(
    listener: TcpListener,
    token: &str,
    executor: &mut E,
) -> Result<(), String> {
    for incoming in listener.incoming() {
        let mut stream = incoming.map_err(|error| format!("daemon connection failed: {error}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|error| format!("cannot set daemon read timeout: {error}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .map_err(|error| format!("cannot set daemon write timeout: {error}"))?;

        let request = match protocol::read_message::<Request>(&mut stream) {
            Ok(request) => request,
            Err(error) => {
                let _ = protocol::write_message(
                    &mut stream,
                    &Response::Err {
                        error: format!("invalid request: {error}"),
                    },
                );
                continue;
            }
        };

        if request.token != token {
            let _ = protocol::write_message(
                &mut stream,
                &Response::Err {
                    error: "unauthorized daemon request".to_string(),
                },
            );
            continue;
        }

        let stop = matches!(&request.command, Command::Close);
        let response = match executor.execute(request.command) {
            Ok(output) => Response::Ok { output },
            Err(error) => Response::Err { error },
        };
        let should_stop = stop && matches!(&response, Response::Ok { .. });
        if let Err(error) = protocol::write_message(&mut stream, &response) {
            if error.kind() == ErrorKind::InvalidData {
                let _ = protocol::write_message(
                    &mut stream,
                    &Response::Err {
                        error: error.to_string(),
                    },
                );
            }
        }
        if should_stop {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::serve_requests;
    use crate::browser::BrowserExecutor;
    use crate::commands::Command;
    use crate::execution::execute_with_info;
    use crate::session::DaemonInfo;
    use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
    use std::thread;

    #[derive(Default)]
    struct FakeBrowser {
        current_url: Option<String>,
    }

    impl BrowserExecutor for FakeBrowser {
        fn execute(&mut self, command: Command) -> Result<String, String> {
            match command {
                Command::Open { url } => {
                    self.current_url = Some(url.clone());
                    Ok(url)
                }
                Command::Url => self
                    .current_url
                    .clone()
                    .ok_or_else(|| "no page is open".to_string()),
                Command::Snapshot => Ok("x".repeat(16 * 1024 * 1024)),
                Command::Close => Ok("closed".to_string()),
                _ => Err("unsupported in fake browser".to_string()),
            }
        }
    }

    #[test]
    fn session_persists_across_connections_and_close_stops_daemon() {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let info = DaemonInfo {
            port: listener.local_addr().unwrap().port(),
            token: "test-token".to_string(),
        };
        let server = thread::spawn(|| {
            serve_requests(listener, "test-token", &mut FakeBrowser::default()).unwrap()
        });

        let missing = execute_with_info(&info, &Command::Url).unwrap_err();
        assert!(missing.to_string().contains("no page is open"));

        let opened = execute_with_info(
            &info,
            &Command::Open {
                url: "https://example.com".to_string(),
            },
        )
        .unwrap();
        assert_eq!(opened, "https://example.com");
        assert_eq!(execute_with_info(&info, &Command::Url).unwrap(), opened);
        assert_eq!(execute_with_info(&info, &Command::Close).unwrap(), "closed");
        server.join().unwrap();
    }

    #[test]
    fn rejects_wrong_token_without_stopping_daemon() {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(|| {
            serve_requests(listener, "right-token", &mut FakeBrowser::default()).unwrap()
        });

        let wrong = DaemonInfo {
            port,
            token: "wrong-token".to_string(),
        };
        let error = execute_with_info(&wrong, &Command::Close).unwrap_err();
        assert!(error.to_string().contains("unauthorized"));

        let right = DaemonInfo {
            port,
            token: "right-token".to_string(),
        };
        assert_eq!(
            execute_with_info(&right, &Command::Close).unwrap(),
            "closed"
        );
        server.join().unwrap();
    }

    #[test]
    fn oversized_snapshot_returns_a_clear_error() {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let info = DaemonInfo {
            port: listener.local_addr().unwrap().port(),
            token: "test-token".to_string(),
        };
        let server = thread::spawn(|| {
            serve_requests(listener, "test-token", &mut FakeBrowser::default()).unwrap()
        });

        let error = execute_with_info(&info, &Command::Snapshot).unwrap_err();
        assert_eq!(error.to_string(), "message exceeds the 16 MiB limit");
        assert_eq!(execute_with_info(&info, &Command::Close).unwrap(), "closed");
        server.join().unwrap();
    }
}
