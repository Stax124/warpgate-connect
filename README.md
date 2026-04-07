# Warpgate Connect

A TUI client for [Warpgate](https://github.com/warp-tech/warpgate) — a smart SSH/HTTP bastion host. Browse, search, and filter your SSH targets, then connect with a single keypress.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/github/license/Stax124/warpgate-connect)

![warpgate-connect](/shared/img/warpgate-connect.webp)

## Features

- Browse all SSH targets from your Warpgate instance in a searchable, filterable table
- Fuzzy search
- Filter targets by group, with color-coded group labels
- Automatic self-update from GitHub releases
- Tiny, fully static binary built with musl libc (no glibc dependency, should run on any Linux)

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
warpgate-connect [OPTIONS]

Options:
  --skip-update    Skip the GitHub update check on startup
  -h, --help       Print help
  -V, --version    Print version
```

### Workflow

1. Launch `warpgate-connect`
2. The app fetches SSH targets from your Warpgate instance
3. Use the search bar to filter targets by name, or `Shift+G` to filter by group
4. Press `Enter` on a target to connect — the TUI exits and `ssh` is spawned automatically

## Keybindings

| Key                  | Action                                     |
| -------------------- | ------------------------------------------ |
| `Ctrl+C` / `Shift+Q` | Quit                                       |
| `Shift+N`            | Cycle screens (Main → Settings → Logs)     |
| `Shift+R`            | Refresh targets from API                   |
| `Shift+U`            | Self-update (when update banner is shown)  |
| `↑` / `↓`            | Navigate target list                       |
| `Home` / `End`       | Jump to first / last target                |
| `Enter`              | Connect to selected target                 |
| `Shift+G`            | Cycle group filter                         |
| Any printable key    | Append to search query                     |
| `Backspace`          | Delete last search character               |
| `Ctrl+A`             | Select all text (search / settings fields) |
| `Tab` / `Shift+Tab`  | Navigate between settings fields           |

