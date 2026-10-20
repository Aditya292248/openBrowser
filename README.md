# my_browser

A Rust CLI skeleton for browser commands. It parses and validates commands, but does not connect to or control a browser yet. A valid command exits with status `1` and a clear "browser execution is not implemented" error; invalid syntax exits with status `2`.

## Build and test

Install a Rust toolchain with Cargo and a linker for your platform, then run from this directory:

```sh
cargo build
cargo test
cargo run -- --help
```

## Commands

```text
my_browser open <url>
my_browser url
my_browser title
my_browser snapshot
my_browser click <target>
my_browser fill <target> <text>
my_browser close
```

`open` accepts HTTP and HTTPS URLs. Quote a target or text argument if it contains spaces, for example `cargo run -- fill "#search" "hello world"`. An empty `text` argument is allowed so a future executor can clear a field.

The code is split into `commands` (typed commands and parsing), `execution` (the browser integration boundary), and `output` (CLI messages and usage).
