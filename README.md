# Warpgate Connect

A TUI client for [Warpgate](https://github.com/warp-tech/warpgate) — a smart SSH/HTTP bastion host. Browse, search, and filter your SSH targets, then open an SSH or SFTP session in seconds.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/github/license/Stax124/warpgate-connect)

![warpgate-connect](/shared/img/warpgate-connect.gif)

## Features

- Browse all SSH targets from your Warpgate instance in a searchable, filterable table
- Fuzzy search over target names and descriptions
- Filter targets by group through a searchable group picker, with color-coded group labels
- Connect over SSH or SFTP
- `list` command that prints targets as tab-separated values for scripting
- Automatic self-update from GitHub releases
- Tiny binary size

## Installation

### Linux / macOS

```sh
curl -fsSL https://raw.githubusercontent.com/Stax124/warpgate-connect/refs/heads/main/install.sh | sh
```

### From source

```sh
git clone https://github.com/Stax124/warpgate-connect
cd warpgate-connect

# Add the musl target if you haven't already
rustup target add x86_64-unknown-linux-musl  # or aarch64-unknown-linux-musl

cargo build --release --target x86_64-unknown-linux-musl
# Binary will be at target/x86_64-unknown-linux-musl/release/warpgate-connect
```

> **Note:** musl builds produce a fully static binary with no glibc dependency. Make sure `musl` (or the appropriate musl cross toolchain for your arch) is installed on your system.

### Windows

Windows is not supported. To get started with Linux and live a happy life:

1. Download [CachyOS](https://cachyos.org/) (or any other Linux distro)
2. Flash it to a USB drive with [Rufus](https://rufus.ie/en/)
3. Boot from the USB and install Linux
4. Continue from the **Linux / macOS** install section above

## Configuration

On first launch, warpgate-connect will open the settings screen. Configuration is stored at:

- **Linux:** `~/.config/warpgate-connect/config.toml`
- **macOS:** `~/Library/Application Support/com.warpgate-connect.warpgate-connect/config.toml`

**config.toml example:**

```toml
warpgate_api_url  = "https://warpgate.example.com/@warpgate/api/targets"
warpgate_username = "admin"
warpgate_token    = "your-api-token"
warpgate_port     = 2222
```

| Field               | Description                                        |
| ------------------- | -------------------------------------------------- |
| `warpgate_api_url`  | Full URL to your Warpgate targets API endpoint     |
| `warpgate_username` | Your Warpgate username                             |
| `warpgate_token`    | Warpgate API token (from your account settings)    |
| `warpgate_port`     | SSH port of the Warpgate gateway (default: `2222`) |

If any required field is missing, the app will open on the settings screen automatically.

## Usage

```sh
warpgate-connect [OPTIONS] [COMMAND]

Commands:
  list  Print the SSH targets and the username each one needs, tab separated
  help  Print this message or the help of the given subcommand(s)

Options:
      --skip-update    Skip the update check and proceed directly to the application.
      --config <PATH>  Read and save the configuration at PATH instead of the default location.
  -h, --help           Print help
  -V, --version        Print version
```

### Workflow

1. Launch `warpgate-connect`
2. The app fetches SSH targets from your Warpgate instance
3. Type to fuzzy-search targets by name or description, or press `Ctrl+G` to pick a group
4. Press `Enter` on a target, choose **SSH** or **SFTP**, and press `Enter` again — the TUI exits and `ssh`/`sftp` is spawned against the bastion

## Keybindings

Press `F1` anywhere to show the key reference.

### Global

| Key                 | Action                                    |
| ------------------- | ----------------------------------------- |
| `Ctrl+Q` / `Ctrl+C` | Quit                                      |
| `Ctrl+N`            | Cycle screens (Targets → Settings → Logs) |
| `Ctrl+R`            | Refresh targets from the API              |
| `Ctrl+U`            | Self-update (when update banner is shown) |
| `F1`                | Toggle the key reference                  |
| `Esc`               | Close the open dialog                     |

### Targets

| Key               | Action                                  |
| ----------------- | --------------------------------------- |
| Any printable key | Type into the search query              |
| `Backspace`       | Delete the character before the cursor  |
| `←` / `→`         | Move the cursor in the search query     |
| `Ctrl+A`          | Select all text in the search query     |
| `Esc`             | Clear the search query                  |
| `↑` / `↓`         | Move through the target list            |
| `Home` / `End`    | Jump to the first / last target         |
| `Enter`           | Open the connection dialog (SSH / SFTP) |
| `Ctrl+G`          | Open the group picker                   |

In the connection dialog and group picker, `↑` / `↓` / `Home` / `End` move the selection and `Enter` confirms it. The group picker also takes typing to filter the group list.

### Settings

| Key               | Action                                    |
| ----------------- | ----------------------------------------- |
| `Tab` / `↓`       | Next field                                |
| `Shift+Tab` / `↑` | Previous field                            |
| `Ctrl+A`          | Select all text in the focused field      |
| `Enter`           | Save the settings and refresh targets     |
| `Esc`             | Back to the targets screen without saving |
