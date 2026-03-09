//! Theme system for CashCraft
//!
//! Provides 10 built-in themes (6 dark, 4 light) with comprehensive color palettes
//! for all UI elements including charts, text, borders, and semantic colors.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Theme variant indicating dark or light mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    Dark,
    Light,
}

/// Complete color palette for a theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Base colors
    /// Main background color
    pub background: Color,
    /// Main foreground/text color
    pub foreground: Color,

    // Surface colors
    /// Elevated surface (cards, panels)
    pub surface: Color,
    /// Alternative surface for contrast
    pub surface_variant: Color,

    // Accent colors
    /// Primary accent color
    pub primary: Color,
    /// Secondary accent color
    pub secondary: Color,
    /// Tertiary accent color
    pub accent: Color,

    // Semantic colors
    /// Success/positive state
    pub success: Color,
    /// Warning/caution state
    pub warning: Color,
    /// Error/danger state
    pub error: Color,
    /// Informational state
    pub info: Color,

    // Text colors
    /// Primary text color
    pub text_primary: Color,
    /// Secondary text color
    pub text_secondary: Color,
    /// Muted/disabled text color
    pub text_muted: Color,

    // Border colors
    /// Default border color
    pub border: Color,
    /// Focused/active border color
    pub border_focus: Color,

    // Chart colors
    /// Income chart color
    pub chart_income: Color,
    /// Expense chart color
    pub chart_expense: Color,
    /// Savings chart color
    pub chart_savings: Color,
}

/// A complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Display name of the theme
    pub name: String,
    /// Dark or light variant
    pub variant: ThemeVariant,
    /// Color palette
    pub colors: ThemeColors,
}

impl Theme {
    /// Get theme by name (case-insensitive, supports aliases)
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dracula" => Some(Self::dracula()),
            "nord" => Some(Self::nord()),
            "gruvbox" | "gruvbox-dark" => Some(Self::gruvbox_dark()),
            "tokyo-night" | "tokyonight" => Some(Self::tokyo_night()),
            "one-dark" | "onedark" => Some(Self::one_dark()),
            "catppuccin-mocha" | "catppuccin" => Some(Self::catppuccin_mocha()),
            "rose-pine" | "rosepine" => Some(Self::rose_pine_main()),
            "solarized-light" | "solarized" => Some(Self::solarized_light()),
            "github-light" | "github" => Some(Self::github_light()),
            "one-light" | "onelight" => Some(Self::one_light()),
            "catppuccin-latte" => Some(Self::catppuccin_latte()),
            _ => None,
        }
    }

    /// List all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        vec![
            "dracula",
            "nord",
            "gruvbox-dark",
            "tokyo-night",
            "one-dark",
            "catppuccin-mocha",
            "rose-pine",
            "solarized-light",
            "github-light",
            "one-light",
            "catppuccin-latte",
        ]
    }

    /// List dark theme names
    pub fn dark_themes() -> Vec<&'static str> {
        vec![
            "dracula",
            "nord",
            "gruvbox-dark",
            "tokyo-night",
            "one-dark",
            "catppuccin-mocha",
            "rose-pine",
        ]
    }

    /// List light theme names
    pub fn light_themes() -> Vec<&'static str> {
        vec![
            "solarized-light",
            "github-light",
            "one-light",
            "catppuccin-latte",
        ]
    }

    /// Default theme (Rosé Pine)
    pub fn default_theme() -> Self {
        Self::rose_pine_main()
    }

    // =========================================================================
    // DARK THEMES
    // =========================================================================

    /// Rosé Pine Main theme - All natural pine, faux fur and a bit of soho vibes
    pub fn rose_pine_main() -> Self {
        Self {
            name: "Rosé Pine".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(25, 23, 36),      // #191724 Base
                foreground: Color::Rgb(224, 222, 244),   // #e0def4 Text
                surface: Color::Rgb(31, 29, 46),         // #1f1d2e Surface
                surface_variant: Color::Rgb(38, 35, 58), // #26233a Overlay
                primary: Color::Rgb(196, 167, 231),      // #c4a7e7 Iris
                secondary: Color::Rgb(49, 116, 143),     // #31748f Pine
                accent: Color::Rgb(235, 188, 186),       // #ebbcba Rose
                success: Color::Rgb(156, 207, 216),      // #9ccfd8 Foam
                warning: Color::Rgb(246, 193, 119),      // #f6c177 Gold
                error: Color::Rgb(235, 111, 146),        // #eb6f92 Love
                info: Color::Rgb(156, 207, 216),         // #9ccfd8 Foam
                text_primary: Color::Rgb(224, 222, 244),
                text_secondary: Color::Rgb(235, 188, 186), // Rose
                text_muted: Color::Rgb(110, 106, 134),     // #6e6a86 Muted
                border: Color::Rgb(38, 35, 58),            // Overlay
                border_focus: Color::Rgb(196, 167, 231),   // Iris
                chart_income: Color::Rgb(156, 207, 216),   // Foam
                chart_expense: Color::Rgb(235, 111, 146),  // Love
                chart_savings: Color::Rgb(49, 116, 143),   // Pine
            },
        }
    }

    /// Dracula theme - A dark theme with vibrant colors
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(40, 42, 54),    // #282A36
                foreground: Color::Rgb(248, 248, 242), // #F8F8F2
                surface: Color::Rgb(68, 71, 90),       // #44475A
                surface_variant: Color::Rgb(55, 57, 74),
                primary: Color::Rgb(189, 147, 249),  // #BD93F9
                secondary: Color::Rgb(80, 250, 123), // #50FA7B
                accent: Color::Rgb(255, 121, 198),   // #FF79C6
                success: Color::Rgb(80, 250, 123),   // #50FA7B
                warning: Color::Rgb(255, 184, 108),  // #FFB86C
                error: Color::Rgb(255, 85, 85),      // #FF5555
                info: Color::Rgb(139, 233, 253),     // #8BE9FD
                text_primary: Color::Rgb(248, 248, 242),
                text_secondary: Color::Rgb(189, 147, 249),
                text_muted: Color::Rgb(98, 114, 164), // #6272A4
                border: Color::Rgb(68, 71, 90),
                border_focus: Color::Rgb(189, 147, 249),
                chart_income: Color::Rgb(80, 250, 123),
                chart_expense: Color::Rgb(255, 85, 85),
                chart_savings: Color::Rgb(139, 233, 253),
            },
        }
    }

    /// Nord theme - An arctic, north-bluish color palette
    pub fn nord() -> Self {
        Self {
            name: "Nord".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(46, 52, 64),    // #2E3440
                foreground: Color::Rgb(236, 239, 244), // #ECEFF4
                surface: Color::Rgb(59, 66, 82),       // #3B4252
                surface_variant: Color::Rgb(67, 76, 94),
                primary: Color::Rgb(136, 192, 208),   // #88C0D0
                secondary: Color::Rgb(163, 190, 140), // #A3BE8C
                accent: Color::Rgb(129, 161, 193),    // #81A1C1
                success: Color::Rgb(163, 190, 140),
                warning: Color::Rgb(235, 203, 139), // #EBCB8B
                error: Color::Rgb(191, 97, 106),    // #BF616A
                info: Color::Rgb(136, 192, 208),
                text_primary: Color::Rgb(236, 239, 244),
                text_secondary: Color::Rgb(129, 161, 193),
                text_muted: Color::Rgb(76, 86, 106), // #4C566A
                border: Color::Rgb(59, 66, 82),
                border_focus: Color::Rgb(136, 192, 208),
                chart_income: Color::Rgb(163, 190, 140),
                chart_expense: Color::Rgb(191, 97, 106),
                chart_savings: Color::Rgb(136, 192, 208),
            },
        }
    }

    /// Gruvbox Dark theme - Retro groove color scheme
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "Gruvbox Dark".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(40, 40, 40),    // #282828
                foreground: Color::Rgb(235, 219, 178), // #EBDBB2
                surface: Color::Rgb(60, 56, 54),       // #3C3836
                surface_variant: Color::Rgb(80, 73, 69),
                primary: Color::Rgb(250, 189, 47),   // #FABD2F
                secondary: Color::Rgb(184, 187, 38), // #B8BB26
                accent: Color::Rgb(211, 134, 155),   // #D3869B
                success: Color::Rgb(184, 187, 38),
                warning: Color::Rgb(254, 128, 25), // #FE8019
                error: Color::Rgb(251, 73, 52),    // #FB4934
                info: Color::Rgb(131, 165, 152),   // #83A598
                text_primary: Color::Rgb(235, 219, 178),
                text_secondary: Color::Rgb(213, 196, 161),
                text_muted: Color::Rgb(146, 131, 116), // #928374
                border: Color::Rgb(60, 56, 54),
                border_focus: Color::Rgb(250, 189, 47),
                chart_income: Color::Rgb(184, 187, 38),
                chart_expense: Color::Rgb(251, 73, 52),
                chart_savings: Color::Rgb(131, 165, 152),
            },
        }
    }

    /// Tokyo Night theme - A clean, dark theme inspired by Tokyo at night
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(26, 27, 38),    // #1A1B26
                foreground: Color::Rgb(192, 202, 245), // #C0CAF5
                surface: Color::Rgb(36, 40, 59),       // #24283B
                surface_variant: Color::Rgb(52, 59, 88),
                primary: Color::Rgb(122, 162, 247),   // #7AA2F7
                secondary: Color::Rgb(158, 206, 106), // #9ECE6A
                accent: Color::Rgb(187, 154, 247),    // #BB9AF7
                success: Color::Rgb(158, 206, 106),
                warning: Color::Rgb(224, 175, 104), // #E0AF68
                error: Color::Rgb(247, 118, 142),   // #F7768E
                info: Color::Rgb(125, 207, 255),    // #7DCFFF
                text_primary: Color::Rgb(192, 202, 245),
                text_secondary: Color::Rgb(169, 177, 214),
                text_muted: Color::Rgb(86, 95, 137), // #565F89
                border: Color::Rgb(41, 46, 66),
                border_focus: Color::Rgb(122, 162, 247),
                chart_income: Color::Rgb(158, 206, 106),
                chart_expense: Color::Rgb(247, 118, 142),
                chart_savings: Color::Rgb(125, 207, 255),
            },
        }
    }

    /// One Dark theme - Atom's iconic dark theme
    pub fn one_dark() -> Self {
        Self {
            name: "One Dark".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(40, 44, 52),    // #282C34
                foreground: Color::Rgb(171, 178, 191), // #ABB2BF
                surface: Color::Rgb(50, 56, 66),
                surface_variant: Color::Rgb(62, 68, 81),
                primary: Color::Rgb(97, 175, 239),    // #61AFEF
                secondary: Color::Rgb(152, 195, 121), // #98C379
                accent: Color::Rgb(198, 120, 221),    // #C678DD
                success: Color::Rgb(152, 195, 121),
                warning: Color::Rgb(229, 192, 123), // #E5C07B
                error: Color::Rgb(224, 108, 117),   // #E06C75
                info: Color::Rgb(86, 182, 194),     // #56B6C2
                text_primary: Color::Rgb(171, 178, 191),
                text_secondary: Color::Rgb(152, 195, 121),
                text_muted: Color::Rgb(92, 99, 112), // #5C6370
                border: Color::Rgb(62, 68, 81),
                border_focus: Color::Rgb(97, 175, 239),
                chart_income: Color::Rgb(152, 195, 121),
                chart_expense: Color::Rgb(224, 108, 117),
                chart_savings: Color::Rgb(86, 182, 194),
            },
        }
    }

    /// Catppuccin Mocha theme - Soothing pastel dark theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".into(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors {
                background: Color::Rgb(30, 30, 46),    // #1E1E2E
                foreground: Color::Rgb(205, 214, 244), // #CDD6F4
                surface: Color::Rgb(49, 50, 68),       // #313244
                surface_variant: Color::Rgb(69, 71, 90),
                primary: Color::Rgb(203, 166, 247),   // #CBA6F7
                secondary: Color::Rgb(166, 227, 161), // #A6E3A1
                accent: Color::Rgb(245, 194, 231),    // #F5C2E7
                success: Color::Rgb(166, 227, 161),
                warning: Color::Rgb(249, 226, 175), // #F9E2AF
                error: Color::Rgb(243, 139, 168),   // #F38BA8
                info: Color::Rgb(137, 220, 235),    // #89DCEB
                text_primary: Color::Rgb(205, 214, 244),
                text_secondary: Color::Rgb(180, 190, 254),
                text_muted: Color::Rgb(108, 112, 134), // #6C7086
                border: Color::Rgb(69, 71, 90),
                border_focus: Color::Rgb(203, 166, 247),
                chart_income: Color::Rgb(166, 227, 161),
                chart_expense: Color::Rgb(243, 139, 168),
                chart_savings: Color::Rgb(137, 220, 235),
            },
        }
    }

    // =========================================================================
    // LIGHT THEMES
    // =========================================================================

    /// Solarized Light theme - Precision colors for machines and people
    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".into(),
            variant: ThemeVariant::Light,
            colors: ThemeColors {
                background: Color::Rgb(253, 246, 227), // #FDF6E3
                foreground: Color::Rgb(101, 123, 131), // #657B83
                surface: Color::Rgb(238, 232, 213),    // #EEE8D5
                surface_variant: Color::Rgb(253, 246, 227),
                primary: Color::Rgb(38, 139, 210),  // #268BD2
                secondary: Color::Rgb(133, 153, 0), // #859900
                accent: Color::Rgb(42, 161, 152),   // #2AA198
                success: Color::Rgb(133, 153, 0),
                warning: Color::Rgb(181, 137, 0), // #B58900
                error: Color::Rgb(220, 50, 47),   // #DC322F
                info: Color::Rgb(38, 139, 210),
                text_primary: Color::Rgb(88, 110, 117), // #586E75
                text_secondary: Color::Rgb(101, 123, 131),
                text_muted: Color::Rgb(147, 161, 161), // #93A1A1
                border: Color::Rgb(238, 232, 213),
                border_focus: Color::Rgb(38, 139, 210),
                chart_income: Color::Rgb(133, 153, 0),
                chart_expense: Color::Rgb(220, 50, 47),
                chart_savings: Color::Rgb(38, 139, 210),
            },
        }
    }

    /// GitHub Light theme - GitHub's clean light theme
    pub fn github_light() -> Self {
        Self {
            name: "GitHub Light".into(),
            variant: ThemeVariant::Light,
            colors: ThemeColors {
                background: Color::Rgb(255, 255, 255), // #FFFFFF
                foreground: Color::Rgb(36, 41, 47),    // #24292F
                surface: Color::Rgb(246, 248, 250),    // #F6F8FA
                surface_variant: Color::Rgb(234, 238, 242),
                primary: Color::Rgb(9, 105, 218),   // #0969DA
                secondary: Color::Rgb(26, 127, 55), // #1A7F37
                accent: Color::Rgb(130, 80, 223),   // #8250DF
                success: Color::Rgb(26, 127, 55),
                warning: Color::Rgb(154, 103, 0), // #9A6700
                error: Color::Rgb(207, 34, 46),   // #CF222E
                info: Color::Rgb(9, 105, 218),
                text_primary: Color::Rgb(36, 41, 47),
                text_secondary: Color::Rgb(87, 96, 106),
                text_muted: Color::Rgb(139, 148, 158),
                border: Color::Rgb(208, 215, 222),
                border_focus: Color::Rgb(9, 105, 218),
                chart_income: Color::Rgb(26, 127, 55),
                chart_expense: Color::Rgb(207, 34, 46),
                chart_savings: Color::Rgb(9, 105, 218),
            },
        }
    }

    /// One Light theme - Atom's iconic light theme
    pub fn one_light() -> Self {
        Self {
            name: "One Light".into(),
            variant: ThemeVariant::Light,
            colors: ThemeColors {
                background: Color::Rgb(250, 250, 250), // #FAFAFA
                foreground: Color::Rgb(56, 58, 66),    // #383A42
                surface: Color::Rgb(240, 240, 240),
                surface_variant: Color::Rgb(230, 230, 230),
                primary: Color::Rgb(64, 120, 242),  // #4078F2
                secondary: Color::Rgb(80, 161, 79), // #50A14F
                accent: Color::Rgb(166, 38, 164),   // #A626A4
                success: Color::Rgb(80, 161, 79),
                warning: Color::Rgb(193, 132, 1), // #C18401
                error: Color::Rgb(228, 86, 73),   // #E45649
                info: Color::Rgb(1, 132, 188),    // #0184BC
                text_primary: Color::Rgb(56, 58, 66),
                text_secondary: Color::Rgb(80, 161, 79),
                text_muted: Color::Rgb(160, 161, 167), // #A0A1A7
                border: Color::Rgb(219, 219, 219),
                border_focus: Color::Rgb(64, 120, 242),
                chart_income: Color::Rgb(80, 161, 79),
                chart_expense: Color::Rgb(228, 86, 73),
                chart_savings: Color::Rgb(1, 132, 188),
            },
        }
    }

    /// Catppuccin Latte theme - Soothing pastel light theme
    pub fn catppuccin_latte() -> Self {
        Self {
            name: "Catppuccin Latte".into(),
            variant: ThemeVariant::Light,
            colors: ThemeColors {
                background: Color::Rgb(239, 241, 245), // #EFF1F5
                foreground: Color::Rgb(76, 79, 105),   // #4C4F69
                surface: Color::Rgb(230, 233, 239),    // #E6E9EF
                surface_variant: Color::Rgb(220, 224, 232),
                primary: Color::Rgb(136, 57, 239),  // #8839EF
                secondary: Color::Rgb(64, 160, 43), // #40A02B
                accent: Color::Rgb(234, 118, 203),  // #EA76CB
                success: Color::Rgb(64, 160, 43),
                warning: Color::Rgb(223, 142, 29), // #DF8E1D
                error: Color::Rgb(210, 15, 57),    // #D20F39
                info: Color::Rgb(4, 165, 229),     // #04A5E5
                text_primary: Color::Rgb(76, 79, 105),
                text_secondary: Color::Rgb(114, 135, 253),
                text_muted: Color::Rgb(140, 143, 161), // #8C8FA1
                border: Color::Rgb(204, 208, 218),
                border_focus: Color::Rgb(136, 57, 239),
                chart_income: Color::Rgb(64, 160, 43),
                chart_expense: Color::Rgb(210, 15, 57),
                chart_savings: Color::Rgb(4, 165, 229),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_themes_count() {
        assert_eq!(Theme::available_themes().len(), 11);
        assert_eq!(Theme::dark_themes().len(), 7);
        assert_eq!(Theme::light_themes().len(), 4);
    }

    #[test]
    fn test_by_name_exact() {
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("nord").is_some());
        assert!(Theme::by_name("gruvbox-dark").is_some());
        assert!(Theme::by_name("tokyo-night").is_some());
        assert!(Theme::by_name("one-dark").is_some());
        assert!(Theme::by_name("catppuccin-mocha").is_some());
        assert!(Theme::by_name("solarized-light").is_some());
        assert!(Theme::by_name("github-light").is_some());
        assert!(Theme::by_name("one-light").is_some());
        assert!(Theme::by_name("catppuccin-latte").is_some());
    }

    #[test]
    fn test_by_name_aliases() {
        // Test case insensitivity
        assert!(Theme::by_name("Dracula").is_some());
        assert!(Theme::by_name("NORD").is_some());

        // Test aliases
        assert!(Theme::by_name("gruvbox").is_some());
        assert!(Theme::by_name("tokyonight").is_some());
        assert!(Theme::by_name("onedark").is_some());
        assert!(Theme::by_name("catppuccin").is_some());
        assert!(Theme::by_name("solarized").is_some());
        assert!(Theme::by_name("github").is_some());
        assert!(Theme::by_name("onelight").is_some());
    }

    #[test]
    fn test_by_name_invalid() {
        assert!(Theme::by_name("nonexistent").is_none());
        assert!(Theme::by_name("").is_none());
    }

    #[test]
    fn test_theme_variants() {
        assert_eq!(Theme::dracula().variant, ThemeVariant::Dark);
        assert_eq!(Theme::nord().variant, ThemeVariant::Dark);
        assert_eq!(Theme::gruvbox_dark().variant, ThemeVariant::Dark);
        assert_eq!(Theme::tokyo_night().variant, ThemeVariant::Dark);
        assert_eq!(Theme::one_dark().variant, ThemeVariant::Dark);
        assert_eq!(Theme::catppuccin_mocha().variant, ThemeVariant::Dark);

        assert_eq!(Theme::solarized_light().variant, ThemeVariant::Light);
        assert_eq!(Theme::github_light().variant, ThemeVariant::Light);
        assert_eq!(Theme::one_light().variant, ThemeVariant::Light);
        assert_eq!(Theme::catppuccin_latte().variant, ThemeVariant::Light);
    }

    #[test]
    fn test_default_theme() {
        let default = Theme::default();
        assert_eq!(default.name, "Rosé Pine");
        assert_eq!(default.variant, ThemeVariant::Dark);
    }

    #[test]
    fn test_theme_names() {
        assert_eq!(Theme::dracula().name, "Dracula");
        assert_eq!(Theme::nord().name, "Nord");
        assert_eq!(Theme::gruvbox_dark().name, "Gruvbox Dark");
        assert_eq!(Theme::tokyo_night().name, "Tokyo Night");
        assert_eq!(Theme::one_dark().name, "One Dark");
        assert_eq!(Theme::catppuccin_mocha().name, "Catppuccin Mocha");
        assert_eq!(Theme::solarized_light().name, "Solarized Light");
        assert_eq!(Theme::github_light().name, "GitHub Light");
        assert_eq!(Theme::one_light().name, "One Light");
        assert_eq!(Theme::catppuccin_latte().name, "Catppuccin Latte");
    }
}
