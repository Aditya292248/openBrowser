# my_browser

A Rust CLI for one persistent Chrome or Chromium tab. Each command is a new CLI process, while a local daemon keeps the browser session alive until `close`.

## Quick start

Install Rust and Cargo, a linker for your platform, and Chrome or Chromium. On Windows, native Rust builds also need a C/C++ linker.

```sh
cargo build --locked
cargo test --locked
cargo run -- open https://example.com
cargo run -- url
cargo run -- title
cargo run -- close
```

Set `MY_BROWSER_CHROME_PATH` to the browser executable if Chrome is not detected automatically. Run `cargo run -- --help` to see every command.

## Documentation

- [CLI reference](docs/CLI.md): commands, JSON output, errors, and browser settings.
- [Architecture](docs/ARCHITECTURE.md): how CLI processes, the daemon, and Chrome work together.
- [PR description template](docs/pull_request_template.md): the format for describing a change and its checks.
