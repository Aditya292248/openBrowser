# my_browser

A Rust CLI that controls one persistent Chrome or Chromium tab through a local browser daemon. Separate CLI invocations share the daemon's browser session until `close` stops it.

## Build and test

Install a Rust toolchain with Cargo, a linker for your platform, and Chrome or Chromium. Then run from this directory:

```sh
cargo build
cargo test
cargo run -- --help
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the local checks and PR guidelines.

## Use

```sh
cargo run -- open https://example.com
cargo run -- url
cargo run -- title
cargo run -- snapshot
cargo run -- click "a.more-info"
cargo run -- fill "input[name=q]" "hello world"
cargo run -- close
```

`open` accepts HTTP and HTTPS URLs. It starts the daemon in the background if needed, launches a headless browser, and navigates the shared tab. `url` returns that tab's current URL; `title` returns its document title. `snapshot` prints the full page HTML. `click` and `fill` use CSS selectors. `fill` replaces the existing value of an input, textarea, or editable element; an empty text argument clears it. `close` closes the browser and stops the daemon. Quote arguments that contain spaces.

## JSON output

Put `--json` before a command to get one JSON line on stdout. A success has `ok: true`, `command`, and a string `result`. An error has `ok: false`, `command`, and an `error` string. The `command` is `null` when arguments cannot be parsed. JSON mode preserves exit codes: 0 for success, 1 for execution errors, and 2 for invalid arguments. Text output remains the default.

```sh
cargo run -- open https://example.com
cargo run -- --json title
# {"command":"title","ok":true,"result":"Example Domain"}
cargo run -- close
```

`--json --help` returns the usage text as a JSON result. The long-running `daemon serve` command does not support JSON mode.

Use `cargo run -- daemon start` to start the daemon without opening a page, or `cargo run -- daemon serve` to run it in the current terminal and see startup errors. The browser uses an isolated temporary profile; closing the daemon discards its browsing state. Set `MY_BROWSER_CHROME_PATH` to a Chrome or Chromium executable if automatic detection fails. Set `MY_BROWSER_HEADLESS=0` before starting the daemon to show the browser window.

The daemon listens only on a random loopback port and requires a random token stored in a per-user runtime file. The file is under `%LOCALAPPDATA%\my_browser` on Windows, or `$XDG_RUNTIME_DIR/my_browser` (falling back to `$HOME/.cache/my_browser`) on Unix. `MY_BROWSER_RUNTIME_DIR` overrides the parent directory. A stale file from a crashed daemon is replaced on the next start.

The code is split into `commands` (typed commands and parsing), `execution` (daemon client), `daemon` and `protocol` (local request handling), `browser` (Chrome actions), `session` (daemon discovery), and `output` (CLI messages).
