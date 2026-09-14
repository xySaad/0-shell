# 0-SHELL

## Overview

This project is an implementation of the 0-shell subject of the [01-edu curriculum](https://github.com/01-edu/public/tree/master/subjects/0-shell).
A Unix Shell built from scratch using Rust.

## Built-in Commands

| Command | Description | Notable flags / behaviour |
|---------|-------------|--------------------------|
| `echo [args...]` | Print arguments to stdout separated by spaces | — |
| `cd [dir]` | Change the current working directory. Updates `$PWD` and `$OLDPWD`. | no arg / `--` → `$HOME`; `-` → previous directory (`$OLDPWD`) |
| `pwd` | Print the current working directory | — |
| `ls [options] [path...]` | List directory contents | `-a` show hidden files; `-l` long format; `-F` append type indicator |
| `cat [file...]` | Concatenate and print files. Reads from stdin when no file is given or `-` is passed. | — |
| `cp <src> <dest>` | Copy a file. Supports copying one file to a destination file or into a directory. Multiple sources require the destination to be a directory. | — |
| `mv <src> <dest>` | Move or rename files. Multiple sources require the destination to be a directory. | — |
| `mkdir <dir...>` | Create one or more directories | — |
| `rm [options] <path...>` | Remove files or directories | `-r` / `-R` recursive removal; refuses to remove `.`, `..`, and `/` |
| `clear` | Clear the terminal screen | — |
| `exit` | Exit the shell with status 0 | — |

> **Note:** `cd`, `clear`, and `exit` run directly in the parent process. All other commands are executed in a forked child process.

## Supported Syntax

The shell parses input following the [POSIX Shell Command Language](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html) specification.

### Quoting

| Syntax | Description |
|--------|-------------|
| `'...'` | Single quotes — everything inside is treated as a literal string; no expansions occur |
| `"..."` | Double quotes — allows `$variable`, `$(...)`, `` `...` ``, and `\` escape sequences inside |
| `` `...` `` | Backtick command substitution — executes the enclosed command and substitutes its output |

### Expansions

| Syntax | Description |
|--------|-------------|
| `$VAR` | Parameter expansion — replaced with the value of environment variable `VAR` |
| `$(command)` | Command substitution — replaced with the stdout of `command` |
| `` `command` `` | Command substitution (backtick form) — equivalent to `$(command)` |
| `~` | Tilde expansion — replaced with `$HOME` (not expanded inside double quotes) |

### Escape sequences

| Syntax | Description |
|--------|-------------|
| `\<char>` | Escape the next character, removing its special meaning |
| `\` + newline | Line continuation — the newline is ignored and input reading continues on the next line |

### Redirections

| Operator | Description |
|----------|-------------|
| `< file` | Redirect stdin from `file` |
| `> file` | Redirect stdout to `file` (truncate) |
| `2> file` | Redirect stderr to `file` |
| `&> file` | Redirect both stdout and stderr to `file` |

### Command separators

| Operator | Description |
|----------|-------------|
| `;` | Run commands sequentially |
| newline | Same as `;` — separates commands |

## Installation

**Prerequisites:** [Rust toolchain](https://rustup.rs/) (edition 2024, i.e. Rust ≥ 1.85)

```bash
# Clone the repository
git clone https://github.com/zone01oujda/0-shell.git
cd 0-shell

# Build in release mode
cargo build --release

# The binary will be at
./target/release/shell
```

## Usage

```bash
# Run interactively
./target/release/shell
```

The shell displays a `$ ` prompt and waits for input. Type any supported command and press **Enter**.

```
$ echo "Hello, world!"
Hello, world!
$ ls -la
$ cat file.txt
$ cp src.txt dest.txt
$ cd /tmp && pwd
/tmp
$ exit
```

Alternatively, run directly with Cargo (no separate build step needed):

```bash
cargo run --release
```
