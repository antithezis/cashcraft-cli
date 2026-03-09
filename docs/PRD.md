# CashCraft - Product Requirements Document (PRD)

> **Version:** 1.0.0  
> **Status:** Draft  
> **Last Updated:** March 9, 2026  
> **Tech Stack:** Rust + Ratatui  

---

## 1. Executive Summary

**CashCraft** is a terminal-based (TUI) personal finance management application built in Rust. It combines the efficiency of Vim-style navigation with powerful financial tracking capabilities, including a unique **Playground** feature for real-time calculations using configured income and expense variables.

### 1.1 Vision Statement

*"Financial clarity at the speed of thought"* - A keyboard-centric, visually stunning TUI that makes personal finance management feel like coding.

### 1.2 Core Value Propositions

- **Vim-native navigation** for power users
- **Playground calculator** with variable interpolation
- **Rich visualizations** with ASCII/Unicode charts
- **Zero-friction data entry** with smart shortcuts
- **Beautiful themes** supporting dark and light modes

---

## 2. Target Users

### 2.1 Primary Persona: "The Terminal Native"

- Developers, sysadmins, and power users who live in the terminal
- Familiar with Vim keybindings
- Prefers keyboard over mouse
- Values speed and efficiency over GUI aesthetics
- Wants offline-first, privacy-respecting tools

### 2.2 Secondary Persona: "The Budget-Conscious Coder"

- Early-career professionals managing finances for the first time
- Comfortable with command-line tools
- Appreciates visual feedback and charts
- Needs quick calculations without switching contexts

---

## 3. Feature Specification

### 3.1 Core Financial Features

#### 3.1.1 Income Sources (Multi-source Support)

```
+-------------------------------------------------------------------+
|  INCOME SOURCES                                            [i]    |
+-------------------------------------------------------------------+
|  +--------------------------------------------------------------+ |
|  | $salary      | Primary Job    | $4,500.00 | Monthly | *     | |
|  | $freelance   | Side Projects  | $1,200.00 | Monthly | o     | |
|  | $dividends   | Investments    |   $350.00 | Quarterly| o    | |
|  | $rental      | Property       |   $800.00 | Monthly | *     | |
|  +--------------------------------------------------------------+ |
|                                                                   |
|  [a]dd  [e]dit  [d]elete  [t]oggle active  [?] help              |
+-------------------------------------------------------------------+
```

**Data Model:**
```rust
struct IncomeSource {
    id: Uuid,
    variable_name: String,      // e.g., "salary" (used as $salary in playground)
    display_name: String,       // e.g., "Primary Job"
    amount: Decimal,
    frequency: Frequency,       // Daily, Weekly, BiWeekly, Monthly, Quarterly, Yearly
    is_active: bool,
    category: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

enum Frequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Yearly,
    OneTime,
}
```

#### 3.1.2 Expense Categories (Fixed & Variable)

```
+-------------------------------------------------------------------+
|  EXPENSES                                                  [e]    |
+-------------------------------------------------------------------+
|  FIXED EXPENSES ------------------------------------------------- |
|  | $rent        | Apartment      | $1,500.00 | Monthly | *     | |
|  | $insurance   | Health + Car   |   $450.00 | Monthly | *     | |
|  | $subscriptions| Netflix, etc. |    $85.00 | Monthly | *     | |
|  | $gym         | Fitness        |    $50.00 | Monthly | o     | |
|                                                                   |
|  VARIABLE EXPENSES ---------------------------------------------- |
|  | $groceries   | Food & Home    | ~$600.00  | Monthly | *     | |
|  | $transport   | Gas + Transit  | ~$200.00  | Monthly | *     | |
|  | $dining      | Restaurants    | ~$300.00  | Monthly | o     | |
|                                                                   |
|  ---------------------------------------------------------------- |
|  TOTAL FIXED:     $2,085.00                                       |
|  TOTAL VARIABLE:  ~$1,100.00                                      |
|  ================================================================ |
|  TOTAL EXPENSES:  $3,185.00                                       |
+-------------------------------------------------------------------+
```

**Data Model:**
```rust
struct Expense {
    id: Uuid,
    variable_name: String,       // e.g., "rent" (used as $rent in playground)
    display_name: String,
    amount: Decimal,
    expense_type: ExpenseType,
    frequency: Frequency,
    category: ExpenseCategory,
    is_active: bool,
    is_essential: bool,          // For budgeting priorities
    due_day: Option<u8>,         // Day of month when due (1-31)
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

enum ExpenseType {
    Fixed,
    Variable,
    OneTime,
}

enum ExpenseCategory {
    Housing,
    Transportation,
    Food,
    Healthcare,
    Entertainment,
    Utilities,
    Insurance,
    Subscriptions,
    PersonalCare,
    Education,
    Savings,
    Debt,
    Custom(String),
}
```

#### 3.1.3 Transactions Tracking

```
+-------------------------------------------------------------------+
|  TRANSACTIONS - March 2026                                 [t]    |
+-------------------------------------------------------------------+
|  DATE       | DESCRIPTION          | CATEGORY    |    AMOUNT     |
|  -----------+----------------------+-------------+--------------- |
|  03/09      | Grocery Store        | Food        |    -$127.45   |
|  03/08      | Freelance Payment    | Income      |  +$1,200.00   |
|  03/07      | Electric Bill        | Utilities   |     -$89.00   |
|  03/05      | Coffee Shop          | Dining      |      -$5.50   |
|  03/01      | Salary Deposit       | Income      |  +$4,500.00   |
|  03/01      | Rent Payment         | Housing     |  -$1,500.00   |
|  -----------+----------------------+-------------+--------------- |
|                                                                   |
|  < February                                           April >     |
|  [a]dd  [e]dit  [d]elete  [f]ilter  [/]search  [s]ort            |
+-------------------------------------------------------------------+
```

#### 3.1.4 Budget Management

```
+-------------------------------------------------------------------+
|  BUDGET - March 2026                                       [b]    |
+-------------------------------------------------------------------+
|                                                                   |
|  Food         ████████████████░░░░░░░░░░░░░░  $480/$600  (80%)   |
|  Transport    ██████████░░░░░░░░░░░░░░░░░░░░  $120/$200  (60%)   |
|  Dining       ██████████████████████████░░░░  $260/$300  (87%)   |
|  Entertainment████░░░░░░░░░░░░░░░░░░░░░░░░░░   $40/$150  (27%)   |
|  Shopping     ████████████████████████████░░  $175/$200  (88%)   |
|                                                                   |
|  +-------------------------------------------------------------+ |
|  | MONTHLY SUMMARY                                             | |
|  | ----------------------------------------------------------- | |
|  | Income:      $5,700.00                                      | |
|  | Expenses:    $3,185.00                                      | |
|  | Remaining:   $2,515.00                                      | |
|  | Savings:     $1,500.00 (target)                             | |
|  | Free:        $1,015.00                                      | |
|  +-------------------------------------------------------------+ |
|                                                                   |
|  [e]dit budget  [r]eset  [c]opy from previous                    |
+-------------------------------------------------------------------+
```

---

### 3.2 Playground Feature (Calculator with Variables)

The **Playground** is a powerful calculation environment that allows users to perform quick financial calculations using both globally defined variables (income/expenses) and locally defined playground variables.

#### 3.2.1 Playground Interface

```
+-------------------------------------------------------------------+
|  PLAYGROUND                                                [p]    |
+-------------------------------------------------------------------+
|  GLOBAL VARIABLES ------------------------------------------------ |
|  $salary = 4500  $freelance = 1200  $rent = 1500  $spents = 3185  |
|  ----------------------------------------------------------------- |
|                                                                   |
|  LOCAL VARIABLES ------------------------------------------------- |
|  x = 8900                                                         |
|  bonus = 2500                                                     |
|  ----------------------------------------------------------------- |
|                                                                   |
|  CALCULATIONS ---------------------------------------------------- |
|                                                             RESULT |
|  | x = 8900                                              8,900    |
|  | bonus = 2500                                          2,500    |
|  | marzo = $salary - $spents + 34455555 / x          5,187.14    |
|  | ahorro = marzo + bonus                              7,687.14   |
|  | yearly_savings = ahorro * 12                       92,245.68   |
|  | $income - $rent                                     4,200.00   |
|  | ($salary + $freelance) * 0.3                        1,710.00   |
|  | _                                                              |
|                                                                   |
|  ----------------------------------------------------------------- |
|  [Enter] evaluate  [Ctrl+S] save session  [Ctrl+L] clear         |
|  [Tab] autocomplete  [$] insert global var  [Ctrl+H] history     |
+-------------------------------------------------------------------+
```

#### 3.2.2 Playground Syntax & Rules

**Variable Definition:**
```
variable_name = expression
```

**Global Variable Reference (prefixed with `$`):**
```
$salary          -> References global income "salary"
$rent            -> References global expense "rent"
$income          -> Special: sum of all active income sources
$expenses        -> Special: sum of all active expenses
$spents          -> Alias for $expenses
$savings         -> Calculated: $income - $expenses
$balance         -> Current account balance (if tracking enabled)
```

**Local Variable Reference (no prefix):**
```
x                -> References local playground variable "x"
myvar            -> References local playground variable "myvar"
```

**Supported Operators:**
```
+                -> Addition
-                -> Subtraction
*                -> Multiplication
/                -> Division
%                -> Modulo
^                -> Power/Exponentiation
()               -> Grouping
```

**Built-in Functions:**
```
round(x)         -> Round to nearest integer
round(x, n)      -> Round to n decimal places
floor(x)         -> Round down
ceil(x)          -> Round up
abs(x)           -> Absolute value
min(a, b, ...)   -> Minimum value
max(a, b, ...)   -> Maximum value
avg(a, b, ...)   -> Average value
sum(a, b, ...)   -> Sum of values
monthly(x)       -> Convert yearly to monthly (x/12)
yearly(x)        -> Convert monthly to yearly (x*12)
```

**Example Calculations:**
```
# Simple assignment
x = 8900                                                     8,900

# Using globals with arithmetic
marzo = $salary - $spents + 34455555 / x                 5,187.14

# Referencing local variables
ahorro = marzo + bonus                                    7,687.14

# Complex expressions
tax_rate = 0.22                                              0.22
net_salary = $salary * (1 - tax_rate)                    3,510.00

# Using functions
monthly_avg = avg($salary, $freelance)                   2,850.00
annual_income = yearly($income)                         68,400.00

# Projections
savings_6mo = $savings * 6                              15,090.00

# Inline calculations (no assignment)
$income - $rent                                          4,200.00
```

#### 3.2.3 Playground Data Model

```rust
struct PlaygroundSession {
    id: Uuid,
    name: Option<String>,
    lines: Vec<PlaygroundLine>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

struct PlaygroundLine {
    input: String,
    result: Option<CalculationResult>,
    line_number: usize,
}

enum CalculationResult {
    Value(Decimal),
    Assignment {
        variable: String,
        value: Decimal,
    },
    Error(CalculationError),
}

struct PlaygroundState {
    local_variables: HashMap<String, Decimal>,
    history: Vec<String>,
    saved_sessions: Vec<PlaygroundSession>,
}
```

#### 3.2.4 Playground Features

| Feature | Description |
|---------|-------------|
| **Auto-evaluation** | Results calculate in real-time as you type |
| **Variable autocomplete** | `Tab` to cycle through matching variables |
| **History** | `Ctrl+H` to browse calculation history |
| **Session save** | `Ctrl+S` to save current session with a name |
| **Session load** | `Ctrl+O` to load a saved session |
| **Clear** | `Ctrl+L` to clear all lines and local variables |
| **Export** | `Ctrl+E` to export session as text/markdown |
| **Multi-line support** | Complex calculations across multiple lines |
| **Error highlighting** | Invalid syntax highlighted in red |
| **Variable preview** | Hover/select to see variable definition source |

---

### 3.3 Charts & Visualizations

#### 3.3.1 Charts Dashboard

```
+-------------------------------------------------------------------+
|  CHARTS & ANALYTICS                                        [g]    |
+-------------------------------------------------------------------+
|  [1] Income vs Expenses  [2] Category Breakdown                   |
|  [3] Savings Trend       [4] Cash Flow        [5] Projections     |
|                                                                   |
|  +-------------------------------------------------------------+ |
|  |  INCOME VS EXPENSES (Last 6 Months)                         | |
|  |                                                             | |
|  |  $6k |                                         ,----        | |
|  |      |        ,----,              ,----,      |             | |
|  |  $5k |   ,----'    |    ,----,   |    '------'    # Income  | |
|  |      |   |         '----'    '---'                          | |
|  |  $4k |---'                                                  | |
|  |      |                                                      | |
|  |  $3k |  ........................................   O Expenses| |
|  |      |                                                      | |
|  |  $2k |                                                      | |
|  |      '-----+-----+-----+-----+-----+-----+-----------       | |
|  |           Oct   Nov   Dec   Jan   Feb   Mar                 | |
|  +-------------------------------------------------------------+ |
|                                                                   |
|  [<-/->] Navigate charts  [Enter] Expand  [r] Refresh            |
+-------------------------------------------------------------------+
```

#### 3.3.2 Category Breakdown (Pie Chart)

```
+-------------------------------------------------------------------+
|  EXPENSE BREAKDOWN - March 2026                                   |
+-------------------------------------------------------------------+
|                                                                   |
|              ########                                             |
|          ####   Housing  ####                                     |
|        ##         47%       ##                                    |
|       #   ::::::::::::::::    #                                   |
|      #  ::    Food     ::     #     #### Housing    $1,500 47%   |
|      #  ::     19%     ::     #     :::: Food         $600 19%   |
|      #    ::::::::::::::::    #     .... Transport    $200  6%   |
|       #      ..........      #      '''' Utilities    $180  6%   |
|        ##   ..........     ##       ,,,, Insurance    $450 14%   |
|          ####  Transp  ####         ;;;; Other        $255  8%   |
|              ########                                             |
|                                                                   |
|  Total Expenses: $3,185.00                                       |
+-------------------------------------------------------------------+
```

#### 3.3.3 Savings Trend

```
+-------------------------------------------------------------------+
|  SAVINGS TREND (12 Months)                                        |
+-------------------------------------------------------------------+
|                                                                   |
|  $15k |                                              ,--*        |
|       |                                          ,---'           |
|  $12k |                                     ,----'               |
|       |                                 ,---'                    |
|   $9k |                            ,----'                        |
|       |                       ,----'                             |
|   $6k |                  ,----'                                  |
|       |             ,----'                                       |
|   $3k |        ,----'                                            |
|       |   ,----'                                                 |
|    $0 +---'-----+-----+-----+-----+-----+-----+-----+-----+---   |
|       Apr  May  Jun  Jul  Aug  Sep  Oct  Nov  Dec  Jan  Feb Mar  |
|                                                                   |
|  * Current: $14,250    ^ Growth: +$1,820 (+14.7%) this month     |
|  o Target:  $20,000    O Projected: $18,500 by Dec 2026          |
+-------------------------------------------------------------------+
```

#### 3.3.4 Cash Flow Waterfall

```
+-------------------------------------------------------------------+
|  CASH FLOW - March 2026                                           |
+-------------------------------------------------------------------+
|                                                                   |
|             +---+                                                 |
|  $6,000 ----|   |                                                 |
|             | # |                                                 |
|             | # | +---+                                           |
|  $5,000 ----| # | |   |                                           |
|             | # | | + | +---+                                     |
|             | # | | . | |   |                                     |
|  $4,000 ----| # | | . | | - |                                     |
|             | # | | . | | ' | +---+                               |
|             | # | | . | | ' | | - |                               |
|  $3,000 ----| # | | . | | ' | | ' | +---+                         |
|             | # | | . | | ' | | ' | | = |                         |
|  $2,000 ----| # | | . | | ' | | ' | | # |                         |
|             | # | | . | | ' | | ' | | # |                         |
|  $1,000 ----| # | | . | | ' | | ' | | # |                         |
|             | # | | . | | ' | | ' | | # |                         |
|      $0 ----+---+-+---+-+---+-+---+-+---+-------------------      |
|            Salary Freelance Rent  Other  Net                      |
|            +4,500  +1,200  -1,500 -1,685 +2,515                   |
|                                                                   |
+-------------------------------------------------------------------+
```

#### 3.3.5 Projections & Forecasts

```
+-------------------------------------------------------------------+
|  FINANCIAL PROJECTIONS                                            |
+-------------------------------------------------------------------+
|                                                                   |
|  Projection Scenarios (Next 12 Months)                            |
|  ===================================================================|
|                                                                   |
|  $50k |                                            ,--- Best     |
|       |                                       ,----'              |
|  $40k |                                  ,----'      ,--- Base   |
|       |                             ,----'      ,----'            |
|  $30k |                        ,----'      ,----'                 |
|       |                   ,----'      ,----'                      |
|  $20k |              ,----'      ,----'           ,---- Worst    |
|       |         ,----'      ,----'           ,----'               |
|  $10k |    ,----'      ,----'           ,----'                    |
|       |*---'      ,----'           ,----'                         |
|    $0 +--------------------------------------------------         |
|       Mar  Apr  May  Jun  Jul  Aug  Sep  Oct  Nov  Dec  Jan Feb   |
|                                                                   |
|  +---------------------------------------------------------------+|
|  | SCENARIO     | 6 MONTH    | 12 MONTH   | ASSUMPTIONS          ||
|  |--------------+------------+------------+----------------------||
|  | Best Case    | $28,500    | $48,000    | +10% income, -5%     ||
|  | Base Case    | $24,000    | $38,000    | Current trends       ||
|  | Worst Case   | $15,000    | $22,000    | -10% income, +5%     ||
|  +---------------------------------------------------------------+|
|                                                                   |
|  [e]dit scenarios  [r]efresh  [p]rint report                     |
+-------------------------------------------------------------------+
```

---

### 3.4 Animations & Visual Effects

#### 3.4.1 Startup Animation

```
Frame 1:                    Frame 2:                    Frame 3:

    +===+                       +===+                      +===+
    |   |                       | $ |                      | $ |
    +===+                       +===+                      +===+
                                                        CashCraft

Frame 4:                    Frame 5 (Final):

    +===+                      +-----------------------------+
    | $ |                      |     +===+                   |
    +===+                      |     | $ |  CashCraft v1.0   |
  CashCraft                    |     +===+                   |
Loading...                     |   Your finances, your way   |
#####..... 50%                 |                             |
                               |   Press any key to start... |
                               +-----------------------------+
```

#### 3.4.2 Animation Types

| Animation | Location | Description |
|-----------|----------|-------------|
| **Startup splash** | App launch | Logo reveal with loading bar |
| **Panel transitions** | View changes | Smooth fade/slide between panels |
| **Chart drawing** | Charts view | Progressive line/bar rendering |
| **Number counting** | Totals display | Animated counting up/down to values |
| **Success sparkle** | After save | Brief sparkle effect |
| **Error shake** | Invalid input | Subtle horizontal shake |
| **Progress pulse** | Loading states | Pulsing progress indicators |
| **Tab switching** | Navigation | Smooth tab indicator movement |
| **Notification slide** | Alerts/toasts | Slide in from top-right |
| **Modal fade** | Dialogs | Fade in/out with backdrop |

#### 3.4.3 Animation Settings

```rust
struct AnimationConfig {
    enabled: bool,
    speed: AnimationSpeed,      // Slow, Normal, Fast, Instant
    reduced_motion: bool,       // Accessibility option
    startup_animation: bool,
    transition_style: TransitionStyle,
}

enum AnimationSpeed {
    Slow,      // 500ms base duration
    Normal,    // 250ms base duration
    Fast,      // 100ms base duration
    Instant,   // No animation
}

enum TransitionStyle {
    Fade,
    Slide,
    None,
}
```

---

## 4. Navigation & Keybindings

### 4.1 Vim Motions Philosophy

CashCraft embraces Vim's modal editing philosophy with three primary modes:

| Mode | Description | Indicator |
|------|-------------|-----------|
| **Normal** | Navigation and commands | `[N]` in status bar |
| **Insert** | Data entry and editing | `[I]` in status bar |
| **Command** | Execute commands | `:` prefix in command line |

### 4.2 Global Keybindings (All Views)

#### Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `h` / `<-` | Move left | Navigate left in lists/tabs |
| `j` / `v` | Move down | Navigate down in lists |
| `k` / `^` | Move up | Navigate up in lists |
| `l` / `->` | Move right | Navigate right in lists/tabs |
| `gg` | Go to top | Jump to first item |
| `G` | Go to bottom | Jump to last item |
| `Ctrl+d` | Half page down | Scroll down half page |
| `Ctrl+u` | Half page up | Scroll up half page |
| `Ctrl+f` | Full page down | Scroll down full page |
| `Ctrl+b` | Full page up | Scroll up full page |
| `w` | Next word/item | Jump to next logical item |
| `b` | Previous word/item | Jump to previous logical item |
| `0` | Start of line | Jump to start of current row |
| `$` | End of line | Jump to end of current row |
| `{` | Previous section | Jump to previous section |
| `}` | Next section | Jump to next section |

#### Global Shortcuts

| Key | Action | Description |
|-----|--------|-------------|
| `?` | Help | Show context-sensitive help |
| `/` | Search | Open search overlay |
| `:` | Command mode | Enter command mode |
| `Esc` | Cancel/Normal | Return to normal mode |
| `q` | Quit/Close | Close current panel or quit app |
| `Q` | Force quit | Quit without saving |
| `Ctrl+s` | Save | Save current changes |
| `Ctrl+z` | Undo | Undo last action |
| `Ctrl+r` | Redo | Redo last undone action |
| `Tab` | Next panel | Cycle to next panel |
| `Shift+Tab` | Previous panel | Cycle to previous panel |
| `Space` | Select/Toggle | Select item or toggle state |
| `Enter` | Confirm/Open | Confirm action or open item |

### 4.3 View Navigation Shortcuts

```
+-------------------------------------------------------------------+
|  QUICK ACCESS                                                     |
+-------------------------------------------------------------------+
|                                                                   |
|  View Shortcuts (Press from anywhere)                             |
|  ===================================================================|
|                                                                   |
|   gh  ->  Home/Dashboard         gp  ->  Playground               |
|   gt  ->  Transactions           gg  ->  Charts/Graphs            |
|   gi  ->  Income Sources         ge  ->  Expenses                 |
|   gb  ->  Budgets                gs  ->  Settings                 |
|   gm  ->  Current Month          gM  ->  Month Selector           |
|   gc  ->  Calendar View          gr  ->  Reports                  |
|                                                                   |
|  Alternative: Leader key + letter                                 |
|  (Default leader: Space)                                          |
|                                                                   |
|   <Space>h  ->  Home             <Space>p  ->  Playground         |
|   <Space>t  ->  Transactions     <Space>g  ->  Charts             |
|                                                                   |
+-------------------------------------------------------------------+
```

### 4.4 View-Specific Keybindings

#### Dashboard View (`gh`)

| Key | Action |
|-----|--------|
| `r` | Refresh data |
| `1-5` | Quick access to summary sections |
| `Enter` | Drill down into selected summary |

#### Transactions View (`gt`)

| Key | Action |
|-----|--------|
| `a` | Add new transaction |
| `e` | Edit selected transaction |
| `d` | Delete selected transaction |
| `dd` | Delete (Vim-style) |
| `y` | Copy/yank transaction |
| `p` | Paste transaction |
| `f` | Open filter menu |
| `/` | Search transactions |
| `s` | Sort menu |
| `[` | Previous month |
| `]` | Next month |
| `c` | Change category |
| `m` | Move to different month |
| `t` | Toggle transaction type |
| `*` | Mark as recurring |

#### Income View (`gi`)

| Key | Action |
|-----|--------|
| `a` | Add income source |
| `e` | Edit selected source |
| `d` | Delete source |
| `t` | Toggle active/inactive |
| `c` | Copy source |
| `Enter` | View source details |

#### Expenses View (`ge`)

| Key | Action |
|-----|--------|
| `a` | Add expense |
| `e` | Edit selected expense |
| `d` | Delete expense |
| `t` | Toggle active/inactive |
| `f` | Toggle fixed/variable |
| `Enter` | View expense details |

#### Playground View (`gp`)

| Key | Action |
|-----|--------|
| `i` | Enter insert mode |
| `Esc` | Return to normal mode |
| `Enter` | Evaluate current line |
| `Tab` | Autocomplete variable |
| `Ctrl+Space` | Show variable suggestions |
| `Ctrl+l` | Clear playground |
| `Ctrl+s` | Save session |
| `Ctrl+o` | Load session |
| `Ctrl+h` | Show history |
| `Ctrl+e` | Export session |
| `dd` | Delete current line |
| `yy` | Copy current line |
| `p` | Paste line |
| `u` | Undo |
| `Ctrl+r` | Redo |

#### Charts View (`gg`)

| Key | Action |
|-----|--------|
| `1-5` | Select chart type |
| `h/l` | Previous/Next time period |
| `[/]` | Zoom out/in time range |
| `r` | Refresh chart data |
| `e` | Export chart as image |
| `Enter` | Expand chart fullscreen |
| `Esc` | Exit fullscreen |
| `d` | Download data |

#### Budget View (`gb`)

| Key | Action |
|-----|--------|
| `e` | Edit budget |
| `r` | Reset to defaults |
| `c` | Copy from previous month |
| `+` | Increase budget |
| `-` | Decrease budget |
| `Enter` | View category details |

### 4.5 Command Mode

Command mode is entered by pressing `:` and supports the following commands:

```
:q                    Quit application
:q!                   Force quit without saving
:w                    Save all changes
:wq                   Save and quit
:e <section>          Open section (e.g., :e transactions)
:set <option>         Set configuration option
:theme <name>         Switch theme
:export <format>      Export data (csv, json, pdf)
:import <file>        Import data from file
:help [topic]         Show help
:version              Show version info
:config               Open configuration
:clear                Clear current view
:reload               Reload data from disk
:month <MM/YYYY>      Go to specific month
:search <query>       Search across all data
:sort <field>         Sort current view
:filter <expr>        Filter current view
:undo                 Undo last action
:redo                 Redo last undone action
:stats                Show statistics
:backup               Create backup
:restore <backup>     Restore from backup
```

### 4.6 Keybinding Customization

Users can customize keybindings in the configuration file:

```toml
# ~/.config/cashcraft/keybindings.toml

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
top = "gg"
bottom = "G"

[views]
home = "gh"
transactions = "gt"
playground = "gp"
charts = "gg"
income = "gi"
expenses = "ge"
budget = "gb"
settings = "gs"

[leader]
key = "Space"
```

---

## 5. Theming & Visual Design

### 5.1 Theme System

```
+-------------------------------------------------------------------+
|  THEMES                                                    [s]    |
+-------------------------------------------------------------------+
|                                                                   |
|  * Dracula (Dark)          o Nord (Dark)                          |
|  o Gruvbox Dark            o Tokyo Night                          |
|  o One Dark Pro            o Catppuccin Mocha                     |
|                                                                   |
|  o Solarized Light         o GitHub Light                         |
|  o One Light               o Catppuccin Latte                     |
|                                                                   |
|  +-------------------------------------------------------------+ |
|  |  PREVIEW                                                    | |
|  |  ---------------------------------------------------------- | |
|  |  Background      ####  #282A36                              | |
|  |  Foreground      ####  #F8F8F2                              | |
|  |  Primary         ####  #BD93F9                              | |
|  |  Secondary       ####  #50FA7B                              | |
|  |  Accent          ####  #FF79C6                              | |
|  |  Error           ####  #FF5555                              | |
|  |  Warning         ####  #FFB86C                              | |
|  |  Success         ####  #50FA7B                              | |
|  +-------------------------------------------------------------+ |
|                                                                   |
|  [Enter] Apply  [c] Custom theme  [i] Import                     |
+-------------------------------------------------------------------+
```

### 5.2 Built-in Themes

#### Dark Themes

| Theme | Background | Foreground | Primary | Accent |
|-------|------------|------------|---------|--------|
| **Dracula** | `#282A36` | `#F8F8F2` | `#BD93F9` | `#FF79C6` |
| **Nord** | `#2E3440` | `#ECEFF4` | `#88C0D0` | `#81A1C1` |
| **Gruvbox Dark** | `#282828` | `#EBDBB2` | `#FABD2F` | `#B8BB26` |
| **Tokyo Night** | `#1A1B26` | `#C0CAF5` | `#7AA2F7` | `#BB9AF7` |
| **One Dark** | `#282C34` | `#ABB2BF` | `#61AFEF` | `#C678DD` |
| **Catppuccin Mocha** | `#1E1E2E` | `#CDD6F4` | `#CBA6F7` | `#F5C2E7` |

#### Light Themes

| Theme | Background | Foreground | Primary | Accent |
|-------|------------|------------|---------|--------|
| **Solarized Light** | `#FDF6E3` | `#657B83` | `#268BD2` | `#2AA198` |
| **GitHub Light** | `#FFFFFF` | `#24292F` | `#0969DA` | `#8250DF` |
| **One Light** | `#FAFAFA` | `#383A42` | `#4078F2` | `#A626A4` |
| **Catppuccin Latte** | `#EFF1F5` | `#4C4F69` | `#8839EF` | `#EA76CB` |

### 5.3 Theme Data Model

```rust
struct Theme {
    name: String,
    variant: ThemeVariant,
    colors: ThemeColors,
}

enum ThemeVariant {
    Dark,
    Light,
}

struct ThemeColors {
    // Base colors
    background: Color,
    foreground: Color,
    
    // Surface colors
    surface: Color,
    surface_variant: Color,
    
    // Accent colors
    primary: Color,
    secondary: Color,
    accent: Color,
    
    // Semantic colors
    success: Color,
    warning: Color,
    error: Color,
    info: Color,
    
    // Text colors
    text_primary: Color,
    text_secondary: Color,
    text_muted: Color,
    
    // Border colors
    border: Color,
    border_focus: Color,
    
    // Chart colors
    chart_income: Color,
    chart_expense: Color,
    chart_savings: Color,
    chart_palette: Vec<Color>,
}
```

### 5.4 Visual Elements

#### Box Drawing Characters

```rust
// Single line borders
const BORDER_TOP_LEFT: char = '+';      // or Unicode: '┌'
const BORDER_TOP_RIGHT: char = '+';     // or Unicode: '┐'
const BORDER_BOTTOM_LEFT: char = '+';   // or Unicode: '└'
const BORDER_BOTTOM_RIGHT: char = '+';  // or Unicode: '┘'
const BORDER_HORIZONTAL: char = '-';    // or Unicode: '─'
const BORDER_VERTICAL: char = '|';      // or Unicode: '│'

// Double line borders (for emphasis)
const DOUBLE_TOP_LEFT: char = '#';      // or Unicode: '╔'
const DOUBLE_TOP_RIGHT: char = '#';     // or Unicode: '╗'
const DOUBLE_BOTTOM_LEFT: char = '#';   // or Unicode: '╚'
const DOUBLE_BOTTOM_RIGHT: char = '#';  // or Unicode: '╝'
const DOUBLE_HORIZONTAL: char = '=';    // or Unicode: '═'
const DOUBLE_VERTICAL: char = '#';      // or Unicode: '║'

// Rounded corners (Unicode only)
const ROUND_TOP_LEFT: char = ',';       // or Unicode: '╭'
const ROUND_TOP_RIGHT: char = ',';      // or Unicode: '╮'
const ROUND_BOTTOM_LEFT: char = '`';    // or Unicode: '╰'
const ROUND_BOTTOM_RIGHT: char = '\'';  // or Unicode: '╯'
```

#### Progress Bars & Indicators

```rust
// Block elements for progress bars
const FULL_BLOCK: char = '#';       // or Unicode: '█'
const LIGHT_SHADE: char = '.';      // or Unicode: '░'
const MEDIUM_SHADE: char = ':';     // or Unicode: '▒'
const DARK_SHADE: char = '%';       // or Unicode: '▓'

// Progress bar characters
const PROGRESS_FILLED: char = '#';  // or Unicode: '█'
const PROGRESS_EMPTY: char = '.';   // or Unicode: '░'
const PROGRESS_PARTIAL: [char; 8] = [' ', '|', '|', '|', '|', '|', '|', '|'];
// Unicode: [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉']
```

#### Status Indicators

```rust
// Status symbols
const CHECK_MARK: char = '*';       // or Unicode: '✓'
const CROSS_MARK: char = 'x';       // or Unicode: '✗'
const WARNING: char = '!';          // or Unicode: '⚠'
const INFO: char = 'i';             // or Unicode: 'ℹ'
const ARROW_RIGHT: char = '>';      // or Unicode: '→'
const ARROW_LEFT: char = '<';       // or Unicode: '←'
const BULLET: char = '*';           // or Unicode: '•'
const DIAMOND: char = '<>';         // or Unicode: '◆'
const CIRCLE_FILLED: char = '*';    // or Unicode: '●'
const CIRCLE_EMPTY: char = 'o';     // or Unicode: '○'
const STAR: char = '*';             // or Unicode: '★'
const SPARKLE: char = '*';          // or Unicode: '✨'
```

---

## 6. Data Storage & Architecture

### 6.1 File Structure

```
~/.cashcraft/
├── config/
│   ├── settings.toml          # Main configuration
│   ├── keybindings.toml       # Custom keybindings
│   └── themes/                # Custom themes
│       └── my_theme.toml
├── data/
│   ├── cashcraft.db           # SQLite database
│   ├── backups/               # Automatic backups
│   │   ├── 2026-03-09.db
│   │   └── ...
│   └── exports/               # Exported files
│       └── ...
├── playground/
│   └── sessions/              # Saved playground sessions
│       ├── session_001.json
│       └── ...
└── logs/
    └── cashcraft.log          # Application logs
```

### 6.2 Database Schema

```sql
-- Income Sources
CREATE TABLE income_sources (
    id TEXT PRIMARY KEY,
    variable_name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    frequency TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    category TEXT,
    start_date DATE,
    end_date DATE,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Expenses
CREATE TABLE expenses (
    id TEXT PRIMARY KEY,
    variable_name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    expense_type TEXT NOT NULL,
    frequency TEXT NOT NULL,
    category TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    is_essential BOOLEAN DEFAULT FALSE,
    due_day INTEGER,
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Transactions
CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    date DATE NOT NULL,
    description TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    transaction_type TEXT NOT NULL,
    category TEXT NOT NULL,
    account TEXT,
    tags TEXT,
    notes TEXT,
    is_recurring BOOLEAN DEFAULT FALSE,
    recurring_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Budgets
CREATE TABLE budgets (
    id TEXT PRIMARY KEY,
    month INTEGER NOT NULL,
    year INTEGER NOT NULL,
    category TEXT NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(month, year, category)
);

-- Playground Sessions
CREATE TABLE playground_sessions (
    id TEXT PRIMARY KEY,
    name TEXT,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### 6.3 Application Architecture

```
+-------------------------------------------------------------------+
|                        CASHCRAFT ARCHITECTURE                     |
+-------------------------------------------------------------------+
|                                                                   |
|  +-------------------------------------------------------------+ |
|  |                      PRESENTATION LAYER                      | |
|  |  +---------+  +----------+  +--------+  +-----------+       | |
|  |  |   TUI   |  |  Widgets |  | Charts |  | Animations|       | |
|  |  | (Ratatui|  |          |  |        |  |           |       | |
|  |  +----+----+  +----+-----+  +---+----+  +-----+-----+       | |
|  +-------+------------+------------+-------------+---------------+ |
|          |            |            |             |                |
|  +-------+------------+------------+-------------+---------------+ |
|  |                      APPLICATION LAYER                        | |
|  |  +-------------+  +--------------+  +----------------+       | |
|  |  |   Commands  |  |    State     |  |   Event Loop   |       | |
|  |  |   Handler   |  |   Manager    |  |                |       | |
|  |  +------+------+  +------+-------+  +-------+--------+       | |
|  +---------+-----------------+------------------+----------------+ |
|            |                 |                  |                  |
|  +---------+-----------------+------------------+----------------+ |
|  |                       DOMAIN LAYER                            | |
|  |  +----------+ +----------+ +----------+ +-------------+       | |
|  |  |  Income  | | Expenses | |  Budget  | |  Playground |       | |
|  |  |  Service | |  Service | |  Service | |   Engine    |       | |
|  |  +----+-----+ +----+-----+ +----+-----+ +------+------+       | |
|  +-------+------------+------------+--------------+---------------+ |
|          |            |            |              |                |
|  +-------+------------+------------+--------------+---------------+ |
|  |                       DATA LAYER                               | |
|  |  +---------------+  +-------------+  +----------------+        | |
|  |  |   Repository  |  |    SQLite   |  |     Config     |        | |
|  |  |    Pattern    |  |   Database  |  |     Files      |        | |
|  |  +---------------+  +-------------+  +----------------+        | |
|  +----------------------------------------------------------------+ |
|                                                                   |
+-------------------------------------------------------------------+
```

### 6.4 Core Modules

```
cashcraft/
├── src/
│   ├── main.rs
│   ├── app.rs                 // Application state & event loop
│   ├── config/
│   │   ├── mod.rs
│   │   ├── settings.rs        // Settings management
│   │   └── keybindings.rs     // Keybinding configuration
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs          // Layout management
│   │   ├── widgets/           // Custom widgets
│   │   │   ├── mod.rs
│   │   │   ├── table.rs
│   │   │   ├── chart.rs
│   │   │   ├── progress.rs
│   │   │   └── input.rs
│   │   ├── views/             // View components
│   │   │   ├── mod.rs
│   │   │   ├── dashboard.rs
│   │   │   ├── transactions.rs
│   │   │   ├── income.rs
│   │   │   ├── expenses.rs
│   │   │   ├── playground.rs
│   │   │   ├── charts.rs
│   │   │   ├── budget.rs
│   │   │   └── settings.rs
│   │   ├── theme.rs           // Theme management
│   │   └── animation.rs       // Animation system
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── income.rs          // Income domain model
│   │   ├── expense.rs         // Expense domain model
│   │   ├── transaction.rs     // Transaction domain model
│   │   ├── budget.rs          // Budget domain model
│   │   └── playground/
│   │       ├── mod.rs
│   │       ├── parser.rs      // Expression parser
│   │       ├── evaluator.rs   // Expression evaluator
│   │       └── session.rs     // Session management
│   ├── services/
│   │   ├── mod.rs
│   │   ├── income_service.rs
│   │   ├── expense_service.rs
│   │   ├── transaction_service.rs
│   │   ├── budget_service.rs
│   │   ├── chart_service.rs
│   │   └── export_service.rs
│   ├── repository/
│   │   ├── mod.rs
│   │   ├── database.rs        // SQLite connection
│   │   ├── income_repo.rs
│   │   ├── expense_repo.rs
│   │   ├── transaction_repo.rs
│   │   └── budget_repo.rs
│   └── utils/
│       ├── mod.rs
│       ├── currency.rs        // Currency formatting
│       ├── date.rs            // Date utilities
│       └── math.rs            // Math utilities
├── Cargo.toml
└── README.md
```

---

## 7. Configuration

### 7.1 Main Configuration File

```toml
# ~/.config/cashcraft/settings.toml

[general]
language = "en"                    # en, es, pt, etc.
currency = "USD"                   # Currency code
currency_symbol = "$"              # Display symbol
currency_position = "before"       # before, after
decimal_separator = "."
thousands_separator = ","
date_format = "%m/%d/%Y"           # strftime format
first_day_of_week = "monday"       # monday, sunday

[appearance]
theme = "dracula"                  # Theme name
animations_enabled = true
animation_speed = "normal"         # slow, normal, fast, instant
reduced_motion = false
unicode_borders = true             # Use Unicode box drawing
show_icons = true
compact_mode = false

[navigation]
vim_mode = true
leader_key = "Space"
scroll_amount = 10                 # Lines per Ctrl+d/u

[playground]
auto_evaluate = true               # Evaluate on Enter
show_global_vars = true            # Show global variables panel
decimal_places = 2                 # Default decimal precision
save_history = true
max_history = 100

[data]
auto_backup = true
backup_frequency = "daily"         # daily, weekly, monthly
backup_retention = 30              # Days to keep backups
auto_save = true
auto_save_interval = 60            # Seconds

[notifications]
enabled = true
budget_warnings = true
budget_warning_threshold = 80      # Percentage
upcoming_bills = true
bills_reminder_days = 3            # Days before due date
```

### 7.2 CLI Arguments

```bash
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
    export <format>         Export data
    import <file>           Import data
    backup                  Create backup
    restore <file>          Restore from backup
    config                  Edit configuration
```

---

## 8. Technical Requirements

### 8.1 Dependencies (Cargo.toml)

```toml
[package]
name = "cashcraft"
version = "1.0.0"
edition = "2021"
authors = ["Your Name"]
description = "A Vim-powered TUI personal finance manager"
license = "MIT"
repository = "https://github.com/yourusername/cashcraft"
keywords = ["finance", "tui", "cli", "vim", "budget"]
categories = ["command-line-utilities", "finance"]

[dependencies]
# TUI Framework
ratatui = "0.28"
crossterm = "0.28"

# Async Runtime
tokio = { version = "1.40", features = ["full"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Numbers
rust_decimal = { version = "1.36", features = ["serde"] }

# UUID
uuid = { version = "1.10", features = ["v4", "serde"] }

# CLI Parsing
clap = { version = "4.5", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Directories
directories = "5.0"

# Expression Parsing (for Playground)
pest = "2.7"
pest_derive = "2.7"

[dev-dependencies]
pretty_assertions = "1.4"
tempfile = "3.12"

[profile.release]
lto = true
strip = true
codegen-units = 1
panic = "abort"
```

### 8.2 Expression Parser Grammar (Playground)

```pest
// grammar.pest - PEG grammar for playground expressions

WHITESPACE = _{ " " | "\t" }

// Numbers
number = @{ 
    "-"? ~ ("0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT*) ~ ("." ~ ASCII_DIGIT+)?
}

// Identifiers
identifier = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

// Global variable reference (prefixed with $)
global_var = @{ "$" ~ identifier }

// Local variable reference
local_var = @{ identifier }

// Variable (either global or local)
variable = { global_var | local_var }

// Function call
function_name = @{ ASCII_ALPHA ~ ASCII_ALPHANUMERIC* }
function_call = { function_name ~ "(" ~ (expr ~ ("," ~ expr)*)? ~ ")" }

// Atoms
atom = { 
    number 
    | function_call 
    | global_var 
    | local_var 
    | "(" ~ expr ~ ")" 
}

// Operators (precedence from low to high)
add_op = { "+" | "-" }
mul_op = { "*" | "/" | "%" }
pow_op = { "^" }

// Expressions with precedence
power = { atom ~ (pow_op ~ atom)* }
term = { power ~ (mul_op ~ power)* }
expr = { term ~ (add_op ~ term)* }

// Assignment
assignment = { identifier ~ "=" ~ expr }

// Line (either assignment or expression)
line = { assignment | expr }

// Full input
input = _{ SOI ~ line ~ EOI }
```

### 8.3 Performance Requirements

| Metric | Target |
|--------|--------|
| Startup time | < 200ms |
| Frame rate | 60 FPS |
| Input latency | < 16ms |
| Memory usage | < 50MB |
| Database query | < 50ms |
| Chart rendering | < 100ms |

### 8.4 Accessibility Requirements

- Full keyboard navigation (no mouse required)
- High contrast theme option
- Reduced motion mode
- Screen reader compatible text output
- Configurable font sizes (terminal dependent)
- Color-blind friendly palette options

---

## 9. Testing Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playground_simple_expression() {
        let evaluator = PlaygroundEvaluator::new();
        let result = evaluator.evaluate("2 + 2").unwrap();
        assert_eq!(result, Decimal::new(4, 0));
    }

    #[test]
    fn test_playground_variable_assignment() {
        let mut evaluator = PlaygroundEvaluator::new();
        evaluator.evaluate("x = 10").unwrap();
        let result = evaluator.evaluate("x * 2").unwrap();
        assert_eq!(result, Decimal::new(20, 0));
    }

    #[test]
    fn test_playground_global_variable() {
        let mut evaluator = PlaygroundEvaluator::new();
        evaluator.set_global("salary", Decimal::new(4500, 0));
        let result = evaluator.evaluate("$salary * 12").unwrap();
        assert_eq!(result, Decimal::new(54000, 0));
    }

    #[test]
    fn test_income_source_monthly_calculation() {
        let source = IncomeSource {
            amount: Decimal::new(1200, 0),
            frequency: Frequency::Yearly,
            ..Default::default()
        };
        assert_eq!(source.monthly_amount(), Decimal::new(100, 0));
    }
}
```

### 9.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_full_transaction_lifecycle() {
        let dir = tempdir().unwrap();
        let db = Database::new(dir.path().join("test.db")).await.unwrap();
        
        // Create
        let tx = Transaction::new("Test", Decimal::new(100, 0), Category::Food);
        db.transactions().create(&tx).await.unwrap();
        
        // Read
        let loaded = db.transactions().find_by_id(&tx.id).await.unwrap();
        assert_eq!(loaded.description, "Test");
        
        // Update
        let mut updated = loaded;
        updated.description = "Updated".to_string();
        db.transactions().update(&updated).await.unwrap();
        
        // Delete
        db.transactions().delete(&updated.id).await.unwrap();
        assert!(db.transactions().find_by_id(&updated.id).await.is_err());
    }
}
```

---

## 10. Roadmap

### Phase 1: Foundation (v0.1.0)

- [ ] Project setup with Rust + Ratatui
- [ ] Basic TUI framework with Vim navigation
- [ ] SQLite database integration
- [ ] Income source CRUD
- [ ] Expense CRUD
- [ ] Basic theme support (dark/light)

### Phase 2: Core Features (v0.5.0)

- [ ] Transaction tracking
- [ ] Budget management
- [ ] Month navigation
- [ ] Basic search and filtering
- [ ] Configuration system
- [ ] Keybinding customization

### Phase 3: Playground (v0.7.0)

- [ ] Expression parser implementation
- [ ] Variable system (local and global)
- [ ] Built-in functions
- [ ] Session save/load
- [ ] History management
- [ ] Autocomplete

### Phase 4: Visualizations (v0.9.0)

- [ ] ASCII/Unicode chart rendering
- [ ] Income vs Expenses chart
- [ ] Category breakdown
- [ ] Savings trend
- [ ] Cash flow waterfall
- [ ] Projections

### Phase 5: Polish (v1.0.0)

- [ ] Animation system
- [ ] Multiple theme support
- [ ] Export/Import functionality
- [ ] Backup system
- [ ] Performance optimization
- [ ] Documentation
- [ ] Cross-platform testing

### Future Considerations (v1.x+)

- Bank account sync (Plaid integration)
- Multi-currency support
- Recurring transaction automation
- Investment tracking
- Goal setting and tracking
- Mobile companion app (data sync)
- Cloud sync option

---

## 11. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| User adoption | 1,000 GitHub stars in 6 months | GitHub API |
| Performance | 95% operations < 100ms | Telemetry |
| Stability | < 1 crash per 1,000 hours | Error tracking |
| User satisfaction | > 4.5/5 rating | User surveys |
| Code quality | > 80% test coverage | CI metrics |

---

## 12. Appendix

### A. Keyboard Shortcut Quick Reference Card

```
+=====================================================================+
|                    CASHCRAFT QUICK REFERENCE                        |
+=====================================================================+
|  NAVIGATION           |  VIEWS              |  ACTIONS              |
|  ---------------------+---------------------+---------------------- |
|  h/j/k/l   Move       |  gh  Dashboard      |  a    Add             |
|  gg        Top        |  gt  Transactions   |  e    Edit            |
|  G         Bottom     |  gi  Income         |  d    Delete          |
|  Ctrl+d    Half down  |  ge  Expenses       |  /    Search          |
|  Ctrl+u    Half up    |  gb  Budget         |  ?    Help            |
|  w/b       Next/Prev  |  gp  Playground     |  :    Command         |
|  Tab       Next panel |  gg  Charts         |  Space Select         |
|  0/$       Start/End  |  gs  Settings       |  Enter Confirm        |
|                       |                     |  Esc   Cancel         |
+=====================================================================+
|  PLAYGROUND           |  CHARTS             |  COMMAND MODE         |
|  ---------------------+---------------------+---------------------- |
|  i      Insert mode   |  1-5  Select chart  |  :q    Quit           |
|  Esc    Normal mode   |  h/l  Time period   |  :w    Save           |
|  Tab    Autocomplete  |  [/]  Zoom          |  :wq   Save & Quit    |
|  Ctrl+L Clear         |  r    Refresh       |  :help Show help      |
|  Ctrl+S Save session  |  e    Export        |  :theme <name>        |
|  Ctrl+H History       |  Enter Fullscreen   |  :export <fmt>        |
+=====================================================================+
```

### B. Color Palette Reference

```
SEMANTIC COLORS
===============
Income/Positive:    #50FA7B (Green)
Expense/Negative:   #FF5555 (Red)  
Neutral:            #8BE9FD (Cyan)
Warning:            #FFB86C (Orange)
Info:               #BD93F9 (Purple)

CHART PALETTE (12 colors)
=========================
#FF6B6B  #4ECDC4  #45B7D1  #96CEB4
#FFEAA7  #DDA0DD  #98D8C8  #F7DC6F
#BB8FCE  #85C1E9  #F8B500  #00CED1
```

### C. File Format Specifications

#### Export Format: CSV

```csv
date,description,amount,type,category,account,notes
2026-03-09,Grocery Store,-127.45,expense,Food,Checking,Weekly groceries
2026-03-08,Freelance Payment,1200.00,income,Freelance,Checking,Project ABC
```

#### Export Format: JSON

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

**Document Control**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-03-09 | CashCraft Team | Initial PRD |

---

*This PRD is a living document and will be updated as the project evolves.*
