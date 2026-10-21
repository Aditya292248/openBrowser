use serde_json::Value;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::process::{Command, Output};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tempfile::TempDir;

const PAGE: &str = r#"<!doctype html>
<html>
  <head><title>Browser test</title></head>
  <body>
    <input id="name">
    <button id="submit" onclick="document.title = 'Submitted'; document.querySelector('#result').textContent = document.querySelector('#name').value">Submit</button>
    <p id="result"></p>
  </body>
</html>"#;

struct LocalPage {
    url: String,
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl LocalPage {
    fn start() -> Self {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let (stop, stopped) = mpsc::channel();
        let thread = thread::spawn(move || loop {
            if stopped.try_recv().is_ok() {
                break;
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = [0; 4096];
                    let _ = stream.read(&mut request);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        PAGE.len(), PAGE
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("local page server failed: {error}"),
            }
        });
        Self {
            url,
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for LocalPage {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

struct BrowserSession {
    runtime: TempDir,
    closed: bool,
}

impl BrowserSession {
    fn new() -> Self {
        Self {
            runtime: tempfile::tempdir().unwrap(),
            closed: false,
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_my_browser"))
            .arg("--json")
            .args(args)
            .env("MY_BROWSER_RUNTIME_DIR", self.runtime.path())
            .output()
            .unwrap()
    }

    fn success(&self, args: &[&str]) -> Value {
        let output = self.run(args);
        let record = json_output(&output);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {record}");
        assert_eq!(record["ok"], true, "{args:?}: {record}");
        record
    }

    fn close(&mut self) {
        self.success(&["close"]);
        self.closed = true;
    }
}

impl Drop for BrowserSession {
    fn drop(&mut self) {
        if !self.closed {
            let _ = self.run(&["close"]);
        }
    }
}

fn json_output(output: &Output) -> Value {
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    serde_json::from_str(stdout).unwrap()
}

#[test]
fn commands_share_a_real_browser_across_cli_processes() {
    let page = LocalPage::start();
    let mut browser = BrowserSession::new();

    assert_eq!(browser.success(&["open", &page.url])["result"], page.url);
    assert_eq!(browser.success(&["url"])["result"], page.url);
    assert_eq!(browser.success(&["title"])["result"], "Browser test");
    assert!(browser.success(&["snapshot"])["result"]
        .as_str()
        .unwrap()
        .contains("id=\"submit\""));

    assert_eq!(
        browser.success(&["fill", "#name", "Ada Lovelace"])["result"],
        "filled #name"
    );
    assert_eq!(
        browser.success(&["click", "#submit"])["result"],
        "clicked #submit"
    );
    assert_eq!(browser.success(&["title"])["result"], "Submitted");
    assert!(browser.success(&["snapshot"])["result"]
        .as_str()
        .unwrap()
        .contains("<p id=\"result\">Ada Lovelace</p>"));

    browser.close();
    // The close response can arrive just before the daemon removes its session file.
    for _ in 0..50 {
        let after_close = browser.run(&["url"]);
        let record = json_output(&after_close);
        if after_close.status.code() == Some(1)
            && record["error"]
                .as_str()
                .is_some_and(|error| error.contains("daemon is not running"))
        {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("daemon remained available after close");
}
