# rdu - Rust Disk Usage Analyzer

![Release](https://img.shields.io/github/v/release/ShinuToki/rdu?color=brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)

A fast, interactive terminal-based disk usage analyzer written in Rust. Inspired by tools like [dua](https://github.com/Byron/dua-cli) and [ncdu](https://dev.yorhel.nl/ncdu), **rdu** provides a modern TUI experience for exploring disk space consumption.

[![asciicast](https://asciinema.org/a/srCKyJ8uZqNVLwKwAlUPjqsRs.svg)](https://asciinema.org/a/srCKyJ8uZqNVLwKwAlUPjqsRs)

## Features

- **Fast parallel scanning** using [jwalk](https://crates.io/crates/jwalk) for multi-threaded directory traversal
- **Interactive TUI** built with [ratatui](https://crates.io/crates/ratatui) and [crossterm](https://crates.io/crates/crossterm)
- **Multiple sort modes**: by size, modification time, or item count
- **Visual percentage bars** with Unicode block characters for precise display
- **Vim-style navigation** alongside arrow keys
- **Cross-platform** support (Windows and Unix-like systems)
- **Filesystem boundary options** to prevent crossing drives/mounts
- **Symbolic link handling** with optional follow mode

## Installation

### Pre-built Binaries

Pre-compiled binaries for various platforms are available in the [Releases](https://github.com/ShinuToki/rdu/releases) section. Download the appropriate binary for your system and add it to your PATH.

### From Source

```bash
git clone https://github.com/ShinuToki/rdu.git
cd rdu
cargo build --release
```

The binary will be available at `target/release/rdu` (or `rdu.exe` on Windows).

## Usage

```bash
# Scan the current directory
rdu

# Scan a specific path
rdu /path/to/directory

# Stay within the same filesystem (don't cross drive boundaries)
rdu -x /path/to/directory

# Follow symbolic links and junction points (use with caution)
rdu -L /path/to/directory
```

### Command Line Options

| Option                    | Description                                                          |
| :------------------------ | :------------------------------------------------------------------- |
| `[PATH]`                  | Directory to scan (default: current directory)                       |
| `-x`, `--one-file-system` | Do not cross filesystem boundaries (drives on Windows)               |
| `-L`, `--follow-links`    | Follow symbolic links and Junction points (caution: can cause loops) |
| `-h`, `--help`            | Print help information                                               |
| `-V`, `--version`         | Print version information                                            |

## Keyboard Shortcuts

### Navigation

| Key               | Action             |
| :---------------- | :----------------- |
| `j` / `↓`         | Move down one item |
| `k` / `↑`         | Move up one item   |
| `Ctrl+d` / `PgDn` | Move down 10 items |
| `Ctrl+u` / `PgUp` | Move up 10 items   |
| `H` / `Home`      | Go to first item   |
| `G` / `End`       | Go to last item    |

### Actions

| Key                           | Action                   |
| :---------------------------- | :----------------------- |
| `o` / `l` / `Enter` / `→`     | Enter selected directory |
| `u` / `h` / `Backspace` / `←` | Go up one level          |
| `r`                           | Refresh current view     |

### Sorting

| Key | Action                                     |
| :-- | :----------------------------------------- |
| `s` | Toggle sort by size (ascending/descending) |
| `m` | Toggle sort by modification time           |
| `c` | Toggle sort by item count                  |

### Other

| Key         | Action              |
| :---------- | :------------------ |
| `?`         | Toggle help overlay |
| `q` / `Esc` | Quit                |

## How It Works

1. **Parallel Directory Scanning**: When launched, `rdu` uses `jwalk` to traverse the target directory tree in parallel, leveraging multiple CPU cores for faster scanning of large directory structures.

2. **Tree Construction**: After collecting all filesystem entries, the tool builds an in-memory tree structure where each node (`FileNode`) contains:
   - File/directory name and path
   - Size (for files) or cumulative size (for directories)
   - Modification timestamp
   - Child nodes (for directories)

3. **Size Propagation**: Directory sizes are calculated by propagating file sizes from the deepest nodes upward, ensuring accurate cumulative totals at every level.

4. **Interactive Display**: The TUI displays:
   - A header showing the application name and version
   - Current path with item count and total size
   - A sortable list with size, percentage bar, and name for each item
   - A footer with current sort mode and total disk usage

5. **Navigation**: Users can navigate through the directory tree, entering subdirectories and going back up, with the view dynamically updating to show contents and sizes.

## Dependencies

- [clap](https://crates.io/crates/clap) - Command line argument parsing
- [crossterm](https://crates.io/crates/crossterm) - Cross-platform terminal manipulation
- [jwalk](https://crates.io/crates/jwalk) - Parallel filesystem traversal
- [number_prefix](https://crates.io/crates/number_prefix) - Human-readable size formatting
- [ratatui](https://crates.io/crates/ratatui) - Terminal user interface framework

## Development

### Pre-push Validation

Before pushing changes, run the validation script to ensure CI will pass:

```powershell
# Basic checks (formatting, linting, tests)
.\scripts\pre-push-check.ps1

# Include tag version verification (for releases)
.\scripts\pre-push-check.ps1 -CheckTagVersion

# Skip Prettier check
.\scripts\pre-push-check.ps1 -SkipPrettier
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
