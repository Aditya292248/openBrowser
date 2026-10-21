# Architecture

`my_browser` is a short-lived CLI connected to a long-lived local daemon. The daemon owns one Chrome or Chromium process and one tab, so separate CLI invocations act on the same page.

```text
CLI process                                daemon process
main -> commands -> execution --TCP JSON--> daemon -> browser -> Chrome
main <- output   <- execution <--TCP JSON-- daemon
```

## Request path

The root `Cargo.toml` defines a workspace. Its default member is `crates/browser-cli`, which contains the existing `my_browser` package and binary. All modules below live in `crates/browser-cli/src/`. Build artifacts and the shared `Cargo.lock` stay at the repository root.

1. `main.rs` selects text or `--json` output and asks `commands.rs` to parse the arguments. Invalid arguments exit with code 2.
2. An `open` command starts a detached daemon if one is not running. `daemon start` can start it explicitly. The daemon does not launch Chrome until the first `open`.
3. `session.rs` stores the daemon's random loopback port and token in a runtime file. `execution.rs` reads it and connects to the daemon.
4. `protocol.rs` sends one newline-delimited JSON request and reads one response. The request contains the token and typed command. The daemon rejects an incorrect token and messages over 16 MiB.
5. `daemon.rs` handles requests in order with one `ChromeExecutor`. `browser.rs` uses `headless_chrome` to launch Chrome, navigate the tab, read page data, and interact with elements.
6. The CLI formats the response in `output.rs` and exits. A successful `close` drops Chrome and stops the daemon. Execution errors exit with code 1.

The protocol JSON is internal to the CLI and daemon. It is separate from the optional `--json` format printed for a CLI caller. For example, a `title` request and response look like this:

```text
{"token":"<session token>","command":{"command":"title"}}
{"status":"ok","output":"Example Domain"}
```

## Boundaries and state

The daemon listens on a random IPv4 loopback port. The session file provides its address and a random token; on Unix, the session directory and file are restricted to the current user. `MY_BROWSER_RUNTIME_DIR` can isolate sessions for tests or separate runs. The daemon serves one request at a time and keeps one tab until `close`.

Chrome runs headless by default with an isolated temporary profile. The CLI does not implement its own CDP WebSocket client; `headless_chrome` manages the browser connection. `snapshot` returns HTML, not a screenshot.

## Tests

Module tests cover parsing, output formatting, protocol limits, and daemon request handling. `crates/browser-cli/tests/cli_json.rs` checks CLI output and exit codes across processes. `crates/browser-cli/tests/browser_integration.rs` serves a local page and checks all browser commands across separate CLI processes with a real Chrome session. The [browser integration workflow](../.github/workflows/browser-integration.yml) builds and tests the workspace with Chrome on pull requests and pushes to `main`.
