# CLAUDE.md

Guidance for AI coding agents working in this repository.

## Project

warpgate-connect is a TUI client for [Warpgate](https://github.com/warp-tech/warpgate), written in Rust (edition 2024) on ratatui + tokio. It fetches SSH targets from the Warpgate API, lets the user search and filter them, then exits the TUI and execs `ssh`/`sftp` against the bastion.

A single bin crate. `main.rs` owns startup, the post-TUI handoff (connect or self-update) and the process spawn; `app.rs` is the state and key handling; `screens/` draws and handles one screen each; `warpgate/` is the API client; `config.rs` is the on-disk `config.toml`.

## Commands

```bash
cargo build
cargo test
cargo fmt                                   # `cargo fmt --check` must stay clean
cargo clippy --all-targets -- -D warnings   # CI denies warnings
cargo run -- --skip-update                  # run without the update check
cargo build --release --target x86_64-unknown-linux-musl   # what releases ship
```

Releases are fully static musl binaries, so anything new must build for that target and must not pull in a C dependency (this is why `reqwest` and `self_update` use rustls, not OpenSSL).

## Conventions

Everything below is a rule `cargo fmt` and clippy cannot express. Those two already cover formatting and lints; this section is the rest, and it is what review checks.

### Comments

- **The default is none.** Write one only where a competent Rust reader would still be guessing — a non-obvious constraint, an invariant, the reason the obvious approach was rejected. Simple code gets no comment at all, and a doc comment is not owed to every item.
- Never say a thing twice. If a function name, error string or log message already carries it, the comment gets deleted, not reworded. Never narrate what the next line does.
- Never list your own call sites, or any other inventory of the code as it stands today — it is wrong the moment someone adds a fourth one. Invariants age well; inventories do not.
- **Write the present, not the change.** What a thing used to do, what was removed, and when it changed belong to git; a reader arriving without that history reads them as claims about the code in front of them. "X once did Y, now it does Z" is just "X does Z" — delete the first clause. Two things are not history and stay: the reason a plausible alternative was **rejected**, and a test comment naming the **regression it guards**.
- **Keep it extremely short: two lines is the ceiling, doc comments included.** One sentence is the target. The failure shape is a comment that names the constraint and then keeps going — a second clause restating the first, a consequence the reader can derive. Delete the continuation; that is a cut, not a rewrite.
- The inverse is a defect too, but only where you can name the thing a reader would be guessing about: an invariant or a rejected alternative left unwritten.

### Names

Names spell things out: `warpgate_api_url`, not `url2`; `selected_connection_type`, not `sct`. Conventional short bindings for errors, contexts and iterator variables (`e`, `ctx`, `cfg`) are fine. What is not is an abbreviation of a field or type the codebase spells out elsewhere — `tgt` for a `target`, `conn_ty` for a `ConnectionType`. That contrast is the test.

### Errors and panics

- `color_eyre` (`color_eyre::Result`), `wrap_err`/`context` naming the resource that failed — the path, the URL. A bare `?` on a filesystem or network call loses it.
- **`unwrap()` on user-supplied data is a defect.** A malformed URL, an absent config field or a token that is not a valid header value must surface as an error or a warning, not a panic — a panic with the terminal in raw mode leaves the user with a wrecked shell. `unwrap()` on a `Mutex` lock is fine and is what the codebase does.
- A discarded error (`let _ = …`, ignored `Result`) is either correct, where fail-open is the policy, or a defect where the failure would otherwise leave no record at all. The second kind gets a `tracing::warn!`.

### Logging

`tracing` macros everywhere the TUI is running — `tui_logger` renders them on the logs screen, and a `println!` there corrupts the frame. `println!` is only for the plain-terminal stretches of `main.rs`, before `ratatui::init` and after `ratatui::restore`.

### Terminal state

Any path that leaves `async_main` must have gone through `ratatui::restore` first. Adding an early `return` or `?` between `init` and `restore` is the way this breaks.

### Tests

- **Every test needs a reason to exist.** Don't assert that a constant still holds its value, that an enum still has its variants or that a constructor assigned its arguments — the compiler already says that, and such a test only ever breaks when someone edits it. Test behaviour instead: feed a real API payload through the structs, drive the filter/search over a realistic target list, push the URL parsing at its edge cases.
- Tests live in a sibling file next to the one they test: `utils.rs` → `utils_test.rs`, declared in the file under test as
  ```rust
  #[cfg(test)]
  #[path = "utils_test.rs"]
  mod tests;
  ```
  so `tests` stays a child module and can read its parent's private items. `tempfile::TempDir` for anything touching the filesystem — never the real config path. A `mod tests` inline in the file under test is wrong.
- **No sleep-based synchronization.** A bare `sleep(50ms)` before an assert is flaky by construction. Wait on a channel or a bounded retry that fails for real at the end.
- Look for a test already covering the behaviour you are about to add. If there is one, extend it rather than duplicate it.

### Secrets

`warpgate_token` is a credential. Check **every channel it can leave the process by** — a log line, an error message, a URL or argument handed to `ssh`, the rendered TUI. It belongs in the `X-Warpgate-Token` header and in `config.toml`, nowhere else.

### Maintainability

If you think you can simplify a piece of code, ask the user before doing it. We want to keep the codebase as small and maintainable as possible.
