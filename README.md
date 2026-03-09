<![CDATA[```
   ██████╗ █████╗ ███████╗██╗  ██╗ ██████╗██████╗  █████╗ ███████╗████████╗
  ██╔════╝██╔══██╗██╔════╝██║  ██║██╔════╝██╔══██╗██╔══██╗██╔════╝╚══██╔══╝
  ██║     ███████║███████╗███████║██║     ██████╔╝███████║█████╗     ██║   
  ██║     ██╔══██║╚════██║██╔══██║██║     ██╔══██╗██╔══██║██╔══╝     ██║   
  ╚██████╗██║  ██║███████║██║  ██║╚██████╗██║  ██║██║  ██║██║        ██║   
   ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝        ╚═╝   
```

<div align="center">

**Financial clarity at the speed of thought**

A Vim-powered terminal personal finance manager built with Rust and Ratatui

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()

</div>

---

## Features

- **Vim-style Navigation** - `hjkl` movement, Normal/Insert/Command modes - feels like home
- **Income & Expense Tracking** - Multi-source income and categorized expenses with frequencies
- **Budget Management** - Visual progress bars showing spending against limits
- **Playground Calculator** - Real-time calculations with `$variable` interpolation from your data
- **Beautiful Charts** - ASCII/Unicode visualizations for spending trends and projections
- **10 Built-in Themes** - 6 dark themes (Dracula, Nord, Gruvbox, etc.) + 4 light themes
- **SQLite Persistence** - Your data stays local, private, and fast
- **CSV/JSON Export** - Take your data anywhere

---

## Screenshots

> Screenshots coming soon

---

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/cashcraft.git
cd cashcraft

# Build release version
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .
```

### Requirements

- **Rust 1.75+** (2021 edition)
- **Terminal with Unicode support** (iTerm2, Alacritty, Windows Terminal, etc.)
- **macOS, Linux, or Windows**

---

## Quick Start

```bash
# Launch the TUI
cashcraft

# Open with a specific theme
cashcraft --theme nord

# Quick add a transaction
cashcraft add transaction

# Export data
cashcraft export csv
```

---

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `h` / `j` / `k` / `l` | Navigate left/down/up/right (Vim-style) |
| `Tab` / `Shift+Tab` | Next/Previous panel |
| `1` - `8` | Quick view access |
| `g` + key | Go to view (see below) |
| `:` | Command mode |
| `/` | Search |
| `?` | Help |
| `q` | Quit |
| `Ctrl+s` | Save |
| `Esc` | Cancel / Return to Normal mode |

### View Navigation

| Keys | View |
|------|------|
| `gh` | Home/Dashboard |
| `gt` | Transactions |
| `gi` | Income Sources |
| `ge` | Expenses |
| `gb` | Budget |
| `gp` | Playground |
| `gg` | Charts/Graphs |
| `gs` | Settings |

### Navigation

| Key | Action |
|-----|--------|
| `gg` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` / `Ctrl+u` | Half page down/up |
| `Ctrl+f` / `Ctrl+b` | Full page down/up |
| `w` / `b` | Next/Previous item |
| `0` / `$` | Start/End of line |

### View-Specific Actions

| Key | Action |
|-----|--------|
| `a` | Add new item |
| `e` | Edit selected |
| `d` / `dd` | Delete selected |
| `t` | Toggle active/inactive |
| `y` / `p` | Copy/Paste |
| `f` | Filter |
| `s` | Sort |
| `[` / `]` | Previous/Next month |

### Playground

| Key | Action |
|-----|--------|
| `i` | Enter insert mode |
| `Esc` | Return to normal mode |
| `Enter` | Evaluate current line |
| `Tab` | Autocomplete variable |
| `Ctrl+l` | Clear playground |
| `Ctrl+s` | Save session |
| `Ctrl+h` | Show history |

### Command Mode

```
:q          Quit application
:q!         Force quit without saving
:w          Save all changes
:wq         Save and quit
:theme <n>  Switch theme
:export csv Export to CSV
:export json Export to JSON
:help       Show help
:version    Show version
```

---

## Playground

The Playground is a powerful calculator that integrates with your financial data. Use `$variables` to reference your income and expenses.

### Variable Reference

```
$salary      -> Your income source named "salary"
$rent        -> Your expense named "rent"
$income      -> Sum of all active income sources
$expenses    -> Sum of all active expenses
$savings     -> Calculated: $income - $expenses
```

### Example Calculations

```
# Using variables from your income/expenses
$salary - $rent = ?

# Define local variables
x = 1000
savings_rate = 0.2
monthly_savings = ($salary - $expenses) * savings_rate

# Built-in functions
monthly($yearly_income)          # Divide by 12
yearly($monthly_expense)         # Multiply by 12
avg($salary, $freelance, $bonus) # Average
round($total, 2)                 # Round to 2 decimals
min($rent, $mortgage)            # Minimum value
max($income, $target)            # Maximum value
```

### Supported Operators

| Operator | Description |
|----------|-------------|
| `+` `-` | Addition, Subtraction |
| `*` `/` | Multiplication, Division |
| `%` | Modulo |
| `^` | Power/Exponentiation |
| `()` | Grouping |

---

## Configuration

Configuration is stored in `~/.config/cashcraft/settings.toml`

```toml
[general]
language = "en"
currency = "USD"
currency_symbol = "$"
currency_position = "before"
date_format = "%m/%d/%Y"
first_day_of_week = "monday"

[appearance]
theme = "dracula"
animations_enabled = true
animation_speed = "normal"       # slow, normal, fast, instant
unicode_borders = true
compact_mode = false

[navigation]
vim_mode = true
leader_key = "Space"

[playground]
auto_evaluate = true
decimal_places = 2
save_history = true
max_history = 100

[data]
auto_backup = true
backup_frequency = "daily"
auto_save = true
auto_save_interval = 60          # seconds
```

### Custom Keybindings

Create `~/.config/cashcraft/keybindings.toml`:

```toml
[global]
quit = "q"
save = "C-s"
help = "?"
search = "/"

[navigation]
up = "k"
down = "j"
left = "h"
right = "l"

[views]
home = "gh"
transactions = "gt"
playground = "gp"

[leader]
key = "Space"
```

---

## Themes

### Dark Themes

| Theme | Background | Primary | Style |
|-------|------------|---------|-------|
| **Dracula** | `#282A36` | `#BD93F9` | Purple-accented dark |
| **Nord** | `#2E3440` | `#88C0D0` | Arctic, muted blues |
| **Gruvbox Dark** | `#282828` | `#FABD2F` | Warm, retro colors |
| **Tokyo Night** | `#1A1B26` | `#7AA2F7` | Neon city vibes |
| **One Dark** | `#282C34` | `#61AFEF` | Atom editor inspired |
| **Catppuccin Mocha** | `#1E1E2E` | `#CBA6F7` | Soothing pastel dark |

### Light Themes

| Theme | Background | Primary | Style |
|-------|------------|---------|-------|
| **Solarized Light** | `#FDF6E3` | `#268BD2` | Classic light theme |
| **GitHub Light** | `#FFFFFF` | `#0969DA` | Clean, minimal |
| **One Light** | `#FAFAFA` | `#4078F2` | Atom editor light |
| **Catppuccin Latte** | `#EFF1F5` | `#8839EF` | Soothing pastel light |

Switch themes with:
```bash
cashcraft --theme nord
```

Or in command mode:
```
:theme catppuccin-mocha
```

---

## Data Storage

CashCraft stores data locally in SQLite:

```
~/.cashcraft/
├── config/
│   ├── settings.toml
│   ├── keybindings.toml
│   └── themes/
├── data/
│   ├── cashcraft.db          # Your data
│   ├── backups/              # Automatic backups
│   └── exports/
├── playground/
│   └── sessions/             # Saved calculator sessions
└── logs/
    └── cashcraft.log
```

---

## Export Formats

### CSV

```bash
cashcraft export csv
```

```csv
date,description,amount,type,category,account,notes
2026-03-09,Grocery Store,-127.45,expense,Food,Checking,Weekly groceries
2026-03-08,Freelance Payment,1200.00,income,Freelance,Checking,Project ABC
```

### JSON

```bash
cashcraft export json
```

```json
{
  "version": "1.0",
  "exported_at": "2026-03-09T10:30:00Z",
  "data": {
    "income_sources": [...],
    "expenses": [...],
    "transactions": [...],
    "budgets": [...]
  }
}
```

---

## CLI Reference

```
cashcraft [OPTIONS] [COMMAND]

OPTIONS:
    -c, --config <PATH>     Use custom config file
    -t, --theme <THEME>     Override theme
    -d, --data <PATH>       Use custom data directory
    -v, --verbose           Enable verbose logging
    --no-animations         Disable animations
    --version               Show version information
    --help                  Show help message

COMMANDS:
    tui                     Launch TUI (default)
    add <type>              Quick add (income/expense/transaction)
    list <type>             List items
    playground              Open playground directly
    export <format>         Export data (csv, json)
    import <file>           Import data
    backup                  Create backup
    restore <file>          Restore from backup
    config                  Edit configuration
```

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## License

MIT License

Copyright (c) 2026 CashCraft

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
]]>