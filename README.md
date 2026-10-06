# t2d2

A vim-feeling, SIMPLE Rust TUI todo list. Single binary, single keymap, one
mode at a time.

## Install

```
cargo install --path .
```

Or build from source:

```
cargo build --release
./target/release/t2d2
```

## Storage

Tasks live in `~/.local/share/ttd/tasks.json`. Created on first save. The
parent directory is created automatically.

## Keymap

### Normal mode

| Key | Action |
|---|---|
| `j` / `k` | move cursor |
| `gg` / `G` | first / last task |
| `o` / `O` | new task below / above |
| `a` / `i` | edit title (append / insert) |
| `A` | edit note |
| `c` | toggle complete |
| `d` | delete |
| `D` | delete all (with `y/n` confirm) |
| `J` / `K` | move task down / up |
| `u` | undo delete |
| `?` | help |
| `Ctrl+Q` | quit |

### Insert mode (title or note)

| Key | Action |
|---|---|
| printable | insert |
| `Backspace` | delete char before cursor |
| `Left` / `Right` | move cursor |
| `Up` / `Down` | move between note lines (note only) |
| `Tab` | title → note (title only) |
| `Enter` | commit title; newline in note |
| `Esc` | commit and return to normal |
| `Ctrl+U` | kill to line start |
| `Ctrl+W` | kill word before cursor |

Any other key is ignored in insert mode.

## Undo

Single-stack, session-only. Deleting a task pushes it onto the undo stack;
`u` restores it. The stack is unbounded across a single session and is
cleared when you quit.

## Quit

`Ctrl+Q` flushes any pending save synchronously. Saves also happen
automatically 150ms after any mutation.