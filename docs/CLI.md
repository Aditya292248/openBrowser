# CLI reference

Run commands with the built `my_browser` binary, or use `cargo run --` before each command as shown below. Each invocation exits after printing its result; the browser stays open in the daemon.

## Browser commands

| Command | Result |
| --- | --- |
| `open <url>` | Open an HTTP or HTTPS URL in the shared tab. Starts the daemon and Chrome when needed. |
| `url` | Print the shared tab's current URL. It does not navigate. |
| `title` | Print the current page's title. |
| `snapshot` | Print the current page's HTML, not an image. |
| `click <target>` | Click an element matching a CSS selector. |
| `fill <target> <text>` | Replace the value of an input, textarea, or editable element matching a CSS selector. |
| `close` | Close Chrome and stop the daemon. |

Quote selectors and text containing spaces. An empty quoted `<text>` clears a field. `url`, `title`, `snapshot`, `click`, and `fill` need an open page.

```sh
cargo run -- open https://example.com
cargo run -- url
cargo run -- title
cargo run -- snapshot
cargo run -- click "a.more-info"
cargo run -- fill "input[name=q]" "hello world"
cargo run -- close
```

## Daemon commands

- `daemon start` starts the daemon without opening Chrome or a page.
- `daemon serve` runs it in the current terminal, which is useful for seeing startup errors.
- `open` starts the daemon automatically when needed. `close` stops it.

The daemon controls one tab. A later `open` navigates that same tab to a new URL. The browser uses an isolated temporary profile; its browsing state is discarded when the daemon closes.

## JSON output and exit codes

Put `--json` before a command to print one JSON record on stdout. Success records contain `ok: true`, `command`, and a string `result`. Error records contain `ok: false`, `command`, and an `error` string. `command` is `null` when parsing fails.

```sh
cargo run -- --json title
# {"command":"title","ok":true,"result":"Example Domain"}
```

The exit code is `0` for success, `1` for an execution error, and `2` for invalid arguments. Text mode is the default; it prints errors on stderr. `--json --help` returns usage as a JSON result. The long-running `daemon serve` command cannot use `--json`.

## Browser and session settings

| Variable | Effect |
| --- | --- |
| `MY_BROWSER_CHROME_PATH` | Use a specific Chrome or Chromium executable. |
| `MY_BROWSER_HEADLESS=0` | Show the browser window. Set it before the daemon starts. |
| `MY_BROWSER_RUNTIME_DIR` | Override the parent directory for the daemon's `my_browser/session.json` file. |

By default, the session file is under `%LOCALAPPDATA%\my_browser` on Windows. On Unix it is under `$XDG_RUNTIME_DIR/my_browser`, or `$HOME/.cache/my_browser` when `XDG_RUNTIME_DIR` is unset. A stale session file from a crashed daemon is replaced on the next start.
