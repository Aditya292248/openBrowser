# Contributing

Thanks for helping build a small browser CLI. Keep changes focused and easy to review.

## Setup

Install a Rust toolchain with Cargo and a linker for your platform. Install Chrome or Chromium for the browser integration test. From the repository root, run:

```sh
cargo build --locked
cargo test --locked
cargo run -- --help
```

On Windows, native Rust builds also need a C/C++ linker. `MY_BROWSER_CHROME_PATH` can point to a browser executable when automatic detection fails.

## Where things live

- `src/commands.rs` defines typed commands and validates CLI arguments.
- `src/main.rs` handles CLI flags and exit codes.
- `src/output.rs` formats text and JSON results.
- `src/execution.rs`, `src/protocol.rs`, and `src/session.rs` connect CLI invocations to the daemon.
- `src/daemon.rs` keeps one browser session alive and handles requests.
- `src/browser.rs` performs Chrome actions.
- `tests/` covers the CLI as a process; module tests cover parsers and daemon behavior.

## Pull requests

Keep each PR about one behavior. Describe the user-visible change first, then how you tested it and any remaining limitation. Run `cargo fmt --check`, `cargo test --locked`, and `cargo build --locked` before opening a PR. Add a test for a bug or a changed output contract. Do not commit browser profiles, session tokens, or secrets.

`tests/browser_integration.rs` runs every browser command against a local page through separate CLI processes. It uses an isolated daemon runtime directory and closes the browser when the test ends. The CI job runs this test with Chrome on each PR. Set `MY_BROWSER_CHROME_PATH` when Chrome is not on your `PATH`.
