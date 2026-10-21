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

## Planned repository structure

This is the target layout as the project grows. Today, the Rust package still lives in the root `src/` and `tests/` directories, and `headless_chrome` handles Chrome launch and the browser connection. The workspace move and additional components below are future work.

```text
openBrowser/
|-- Cargo.toml                     # Rust workspace configuration
|-- Cargo.lock                     # Shared Rust dependency lockfile
|-- crates/
|   |-- browser-cli/               # CLI and local daemon
|   |   |-- Cargo.toml
|   |   |-- src/
|   |   |   |-- main.rs
|   |   |   |-- commands.rs        # Typed commands and argument validation
|   |   |   |-- execution.rs       # CLI connection to the daemon
|   |   |   |-- daemon.rs          # Persistent session and request handling
|   |   |   |-- protocol.rs        # Local CLI-to-daemon messages
|   |   |   |-- session.rs         # Daemon discovery and runtime state
|   |   |   `-- output.rs          # Text and JSON formatting
|   |   `-- tests/
|   `-- browser-core/              # Reusable browser control, extracted later
|       |-- Cargo.toml
|       |-- src/
|       |   |-- lib.rs
|       |   |-- browser.rs         # Navigation, page data, click, and fill
|       |   |-- chrome.rs          # Executable discovery, process, and profile
|       |   `-- cdp/               # Chrome DevTools Protocol client
|       |       |-- mod.rs
|       |       |-- client.rs      # WebSocket requests, IDs, and timeouts
|       |       |-- events.rs      # Browser event handling
|       |       `-- types.rs       # CDP request and response types
|       `-- tests/
|-- agent/                         # Task planning, model calls, and skill loading
|-- skills/                        # Add with the first usable skill
|   `-- example-skill/
|       `-- SKILL.md
|-- docs/
|   |-- ARCHITECTURE.md
|   |-- CLI.md
|   `-- pull_request_template.md
|-- .github/workflows/
`-- README.md
```

The [Chrome DevTools Protocol (CDP)](https://chromedevtools.github.io/devtools-protocol/) is the interface used to control Chrome. A custom CDP client would live in `browser-core/src/cdp/`; Chrome executable discovery, launch, temporary profiles, and shutdown would live in `chrome.rs`. Chrome itself is installed separately and is not committed to this repository. The CLI-to-daemon protocol stays in `browser-cli` because it serves a different connection.

The agent would load skills, choose browser actions, and call the CLI through its JSON interface. If the agent is written in Rust, it can instead become `crates/browser-agent/`. Skills would contain reusable instructions and supporting resources for that agent.

The first refactor can move the existing Rust package into `crates/browser-cli/` while preserving its behavior. Extract `browser-core` when its API is needed, and add the agent and skills alongside their first implementations.

## Documentation

- [CLI reference](docs/CLI.md): commands, JSON output, errors, and browser settings.
- [Architecture](docs/ARCHITECTURE.md): how CLI processes, the daemon, and Chrome work together.
- [PR description template](docs/pull_request_template.md): the format for describing a change and its checks.
