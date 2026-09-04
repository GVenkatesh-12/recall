# Recall

A modern, lightweight **terminal command memory and clipboard utility** for the terminal.

Ever Google the same command twice? `recall` remembers the commands you want to keep — store them, list them, copy them to your clipboard, all from your shell. Built in Rust, `recall` is designed to be fast, minimal, and run completely locally on your system. It is stored in a SQLite database and provides a sleek terminal interface with colored accents.

<div align="center">

[![Release](https://img.shields.io/github/v/release/GVenkatesh-12/recall?style=flat&label=release)](https://github.com/GVenkatesh-12/recall/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/GVenkatesh-12/recall/ci.yml?style=flat&label=ci)](https://github.com/GVenkatesh-12/recall/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/GVenkatesh-12/recall?style=flat)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-58a6ff?style=flat)](#installation)

</div>

🌐 **Website:** [https://GVenkatesh-12.github.io/recall/](https://GVenkatesh-12.github.io/recall/)

---

## Features

- **Command Memory**: Save the terminal commands you rarely remember (git, docker, ssh, find — anything) and recall them instantly.
- **Local Storage**: Notes are persisted in a single-user local SQLite database at `~/.recall/recall.db`.
- **Clipboard Integration**: Instantly copy saved terminal commands to your system clipboard — paste and run.
- **Beautiful UX**: Polished console output with color hierarchy, unicode decorators, and table layout dividers.
- **Stateless Mapping**: Refer to notes by their simple displayed index when copying or deleting.
- **Cross-Platform**: Support for Linux, macOS, and Windows.

---

## Installation

### One-line install (recommended)

Download the latest prebuilt binary for your OS/architecture, verify its SHA-256 checksum, and install it — no build tools required:

```bash
curl -fsSL https://GVenkatesh-12.github.io/recall/install.sh | bash
```

The installer works on Linux, macOS, and Windows (Git Bash / MSYS2). It detects your platform automatically and installs to `~/.local/bin` (falling back to `sudo` + `/usr/local/bin` if needed).

Customize the installer with environment variables:

| Variable | Description | Default |
| --- | --- | --- |
| `RECALL_VERSION` | Version tag to install | `latest` |
| `RECALL_INSTALL_DIR` | Install directory | `$HOME/.local/bin` |

```bash
# Pin a specific version
RECALL_VERSION=v1.0.0 curl -fsSL https://GVenkatesh-12.github.io/recall/install.sh | bash

# Install to a custom directory
RECALL_INSTALL_DIR=~/bin curl -fsSL https://GVenkatesh-12.github.io/recall/install.sh | bash
```

### Manual download

Grab the archive plus its `.sha256` file from the [latest release](https://github.com/GVenkatesh-12/recall/releases):

| Platform | Asset |
| --- | --- |
| Linux · x86_64 | `recall-x86_64-unknown-linux-gnu.tar.gz` |
| Linux · arm64 | `recall-aarch64-unknown-linux-gnu.tar.gz` |
| macOS · Intel | `recall-x86_64-apple-darwin.tar.gz` |
| macOS · Apple Silicon | `recall-aarch64-apple-darwin.tar.gz` |
| Windows · x86_64 | `recall-x86_64-pc-windows-msvc.zip` |

```bash
# Example: Linux x86_64
curl -fsSL -o recall.tar.gz \
  https://github.com/GVenkatesh-12/recall/releases/download/v1.0.1/recall-x86_64-unknown-linux-gnu.tar.gz
tar -xzf recall.tar.gz
./recall --version
```

### From crates.io

```bash
cargo install recall
```

### From Source

```bash
git clone https://github.com/GVenkatesh-12/recall.git
cd recall
cargo install --path .
```

*Note: On Linux, ensure that X11 or Wayland development libraries (e.g. `libxcb`) are installed if compiling from source, as required by the `arboard` clipboard library.*

---

## Usage

### 1. List Notes
Running `recall` with no arguments lists all notes. The list is ordered with the most recent notes first.

```bash
recall
```

**Output:**
```text
  ✦ RECALL  v1.1.1                                      3 commands
  ╭─────────────────────────────────────────────────────────────╮
  │   #01  git commit -am "fix: cache invalidation bug"   2h ago│
  │   #02  find . -type f -name '*.log' -size +10M        1h ago│
  │   #03  ssh -i ~/.ssh/id_ed25519 deploy@server       just now│
  ╰─────────────────────────────────────────────────────────────╯
  • recall <id> copy  •  recall -s add  •  recall -e edit  •  recall -d delete
```

### 2. Save a Command
Use `-s` or `--save` to add a new terminal command.

```bash
recall -s 'git commit -am "fix: cache invalidation bug"'
```

**Output:**
```text
  ✔ Saved note #3
```

### 3. Copy a Command
Provide the displayed list index to copy its full contents to your clipboard.

```bash
recall 1
```

**Output:**
```text
  ✔ Copied command #1 to clipboard

  ╭── Command #1 ───────────────────────────────────────────────╮
  │   git commit -am "fix: cache invalidation bug"              │
  ╰─────────────────────────────────────────────────────────────╯
```

### 4. Edit a Command
Use `-e` or `--edit` followed by the displayed list index to edit an existing command interactively.

```bash
recall -e 1
```

**Output:**
```text
Edit command: git commit -am "fix: cache invalidation bug"
  ✔ Updated note #1
```

### 5. Delete a Note
Use `-d` or `--delete` followed by the displayed list index. You will be prompted to confirm the deletion.

```bash
recall -d 1
```

**Output:**
```text
Delete this note? [y/N] y
  ✔ Deleted note #1 (2 remaining)
```

#### Skip Confirmation
Use `-f` or `--force` to skip the interactive confirmation prompt:

```bash
recall -d 1 --force
```

### 6. Update Recall
Run `recall update` or use `-u` / `--update` to check for and install the latest version from GitHub:

```bash
recall update
# or
recall -u
```

**Output:**
```text
  ✦ RECALL UPDATER
  ───────────────────────────────────────────────────────
  ℹ Checking for the latest release on GitHub...
  ▲ New version available: v1.1.1 (installed: v1.1.0)
  ℹ Downloading recall-x86_64-unknown-linux-gnu.tar.gz...
  ✔ SHA-256 checksum verified
  ✔ Installed recall v1.1.1 to /home/user/.local/bin/recall
  ✔ Updated recall from v1.1.0 to v1.1.1
  ───────────────────────────────────────────────────────
```

If you are already on the latest version:

```text
  ✦ RECALL UPDATER
  ───────────────────────────────────────────────────────
  ℹ Checking for the latest release on GitHub...
  ✔ recall is already up to date (v1.1.1)
  ───────────────────────────────────────────────────────
```

`recall update` replaces the running binary in place; if the current install directory
is not writable it installs the update to `~/.local/bin/recall` instead.

---

## Configuration & Storage

Upon first launch, `recall` automatically:
1. Creates the configuration/database directory at `~/.recall/`.
2. Creates the SQLite database file at `~/.recall/recall.db`.
3. Runs database migrations to set up the necessary tables.

No manual configuration is required.

---

## Publishing to crates.io

Before publishing, verify all files are formatted and compile cleanly:

1. Log in to crates.io using your API token:
   ```bash
   cargo login <token>
   ```
2. Run a dry run package verification to ensure there are no packaging errors:
   ```bash
   cargo publish --dry-run
   ```
3. Publish your crate:
   ```bash
   cargo publish
   ```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
