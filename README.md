# Recall

A modern, lightweight note capture and clipboard utility for the terminal.

Built in Rust, `recall` is designed to be fast, minimal, and run completely locally on your system. It is stored in a SQLite database and provides a sleek terminal interface with colored accents.

---

## Features

- **Local Storage**: Notes are persisted in a single-user local SQLite database at `~/.recall/recall.db`.
- **Clipboard Integration**: Instantly copy saved notes to your system clipboard.
- **Beautiful UX**: Polished console output with color hierarchy, unicode decorators, and table layout dividers.
- **Stateless Mapping**: Refer to notes by their simple displayed index when copying or deleting.
- **Cross-Platform**: Support for Linux, macOS, and Windows.

---

## Installation

### From crates.io

To install the latest release of `recall` from crates.io:

```bash
cargo install recall
```

### From Source

Clone the repository and build using Cargo:

```bash
git clone https://github.com/gvenkatesh/recall.git
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
 Recall Notes
 ──────────────────────────────────────────────────

 1  Buy milk
 2  Finish GATE Network Theory revision
 3  Learn Rust ownership

 ──────────────────────────────────────────────────
 Total: 3 notes
```

### 2. Save a Note
Use `-s` or `--save` to add a new note.

```bash
recall -s "Learn Rust ownership"
```

**Output:**
```text
✓ Saved note #3
```

### 3. Copy a Note
Provide the displayed list index to copy its full contents to your clipboard.

```bash
recall 3
```

**Output:**
```text
✓ Copied to clipboard

Learn Rust ownership
```

### 4. Delete a Note
Use `-d` or `--delete` followed by the displayed list index. You will be prompted to confirm the deletion.

```bash
recall -d 3
```

**Output:**
```text
Delete this note? [y/N] y
✓ Deleted note #3
```

#### Skip Confirmation
Use `-f` or `--force` to skip the interactive confirmation prompt:

```bash
recall -d 3 --force
```

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
