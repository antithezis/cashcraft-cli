# CashCraft

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A terminal personal finance manager with a Vim-inspired TUI, written in Rust using ratatui. Focused on privacy (local SQLite), keyboard-driven workflows, and simple exports.

---

## Table of contents

- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Keybindings](#keybindings)
- [Playground](#playground)
- [Configuration](#configuration)
- [Themes](#themes)
- [Data layout](#data-layout)
- [Export formats](#export-formats)
- [CLI reference](#cli-reference)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- Vim-style navigation and modes (normal/insert/command)
- Track income, expenses and transfers
- Budgeting with visual progress indicators
- Integrated calculator (playground) with variable interpolation
- ASCII/Unicode charts for trends and projections
- Multiple built-in themes (dark + light)
- Local persistence with SQLite and automatic backups
- CSV / JSON export and import

---

## Screenshots

Screenshots coming soon. Use the TUI with a modern terminal emulator (iTerm2, Alacritty, Windows Terminal, etc.).

---

## Installation

### From source

```bash
git clone https://github.com/yourusername/cashcraft.git
cd cashcraft
cargo build --release
# optional: install locally
cargo install --path .
```

### Requirements

- Rust 1.75+
- Terminal with Unicode support
- macOS, Linux or Windows

---

## Quick start

Run the app (default launches the TUI):

```bash
cashcraft
# or launch a specific view / theme
cashcraft --theme nord
cashcraft playground
```

Add a quick transaction from the CLI:

```bash
cashcraft add transaction --amount -12.50 --category Food --account Checking --note "Groceries"
```

---

## Keybindings

These are the common bindings. They are customizable in your config.

Global:

- `h`, `j`, `k`, `l`: move left/down/up/right
- `Tab` / `Shift+Tab`: switch panels
- `:` : enter command mode
- `/` : search
- `q` : quit
- `Ctrl+s` : save

View navigation (examples):

- `gh` — Home/Dashboard
- `gt` — Transactions
- `gi` — Income sources
- `ge` — Expenses
- `gb` — Budgets
- `gp` — Playground

In views:

- `a` add, `e` edit, `d`/`dd` delete, `t` toggle, `y`/`p` yank/paste

---

## Playground

The playground is a small REPL-like calculator that can reference your data with `$` variables.

Variable examples:

```
$salary    # income source named "salary"
$rent      # expense named "rent"
$income    # sum of active incomes
$expenses  # sum of active expenses
$savings   # $income - $expenses
```

Example expressions:

```
monthly($yearly_income)
($salary - $expenses) * 0.2
avg($salary, $freelance)
```

---

## Configuration

Default config location: `~/.config/cashcraft/settings.toml`

Minimal example:

```toml
[general]
language = "en"
currency = "USD"

[appearance]
theme = "dracula"
unicode_borders = true

[navigation]
vim_mode = true
leader_key = "Space"
```

Custom keybindings: `~/.config/cashcraft/keybindings.toml`

---

## Themes

Several dark and light themes are included. Switch with `--theme <name>` or `:theme <name>` in command mode.

---

## Data layout

By default CashCraft stores configuration and data under `~/.cashcraft/`:

```
~/.cashcraft/
├── config/
│   ├── settings.toml
│   └── keybindings.toml
├── data/
│   └── cashcraft.db
├── playground/
└── logs/
```

---

## Export formats

CSV:
```bash
cashcraft export csv > export.csv
```

JSON:
```bash
cashcraft export json > export.json
```

---

## CLI reference

Run `cashcraft --help` for the full CLI reference. Examples:

- `cashcraft add transaction ...`
- `cashcraft list transactions`
- `cashcraft export csv`

---

## Contributing

Contributions welcome — please open issues or PRs. Follow the repo conventions and keep commits small.

---

## License

MIT — see `LICENSE` for details.
