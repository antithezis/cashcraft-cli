//! Currency formatting utilities
//!
//! Provides currency formatting and parsing with locale support.

use rust_decimal::Decimal;

/// Currency display position
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolPosition {
    Prefix,
    Suffix,
}

/// Currency format configuration
#[derive(Debug, Clone)]
pub struct CurrencyFormat {
    /// Currency symbol (e.g., "$", "€", "£")
    pub symbol: String,
    /// Symbol position (prefix or suffix)
    pub position: SymbolPosition,
    /// Decimal places
    pub decimal_places: usize,
    /// Decimal separator (e.g., "." or ",")
    pub decimal_separator: char,
    /// Thousands separator (e.g., "," or ".")
    pub thousands_separator: char,
    /// Whether to show sign for positive values
    pub show_positive_sign: bool,
}

impl Default for CurrencyFormat {
    fn default() -> Self {
        Self {
            symbol: "$".to_string(),
            position: SymbolPosition::Prefix,
            decimal_places: 2,
            decimal_separator: '.',
            thousands_separator: ',',
            show_positive_sign: false,
        }
    }
}

impl CurrencyFormat {
    /// Create USD format
    pub fn usd() -> Self {
        Self::default()
    }

    /// Create EUR format
    pub fn eur() -> Self {
        Self {
            symbol: "€".to_string(),
            position: SymbolPosition::Suffix,
            decimal_separator: ',',
            thousands_separator: '.',
            ..Self::default()
        }
    }

    /// Create GBP format
    pub fn gbp() -> Self {
        Self {
            symbol: "£".to_string(),
            ..Self::default()
        }
    }

    /// Format a decimal value as currency
    pub fn format(&self, value: Decimal) -> String {
        let is_negative = value.is_sign_negative();
        let abs_value = value.abs();

        // Round to decimal places
        let rounded = abs_value.round_dp(self.decimal_places as u32);
        let value_str = format!("{:.prec$}", rounded, prec = self.decimal_places);

        // Split into integer and decimal parts
        let parts: Vec<&str> = value_str.split('.').collect();
        let int_part = parts[0];
        let dec_part = parts.get(1).unwrap_or(&"00");

        // Add thousands separators
        let formatted_int = add_thousands_separator(int_part, self.thousands_separator);

        // Combine parts
        let number = if self.decimal_places > 0 {
            format!("{}{}{}", formatted_int, self.decimal_separator, dec_part)
        } else {
            formatted_int
        };

        // Add sign
        let sign = if is_negative {
            "-"
        } else if self.show_positive_sign {
            "+"
        } else {
            ""
        };

        // Format with symbol
        match self.position {
            SymbolPosition::Prefix => format!("{}{}{}", sign, self.symbol, number),
            SymbolPosition::Suffix => format!("{}{} {}", sign, number, self.symbol),
        }
    }

    /// Format as compact currency (e.g., "$1.2K", "$3.5M")
    pub fn format_compact(&self, value: Decimal) -> String {
        let is_negative = value.is_sign_negative();
        let abs_value: f64 = value.abs().try_into().unwrap_or(0.0);

        let (scaled, suffix) = if abs_value >= 1_000_000_000.0 {
            (abs_value / 1_000_000_000.0, "B")
        } else if abs_value >= 1_000_000.0 {
            (abs_value / 1_000_000.0, "M")
        } else if abs_value >= 1_000.0 {
            (abs_value / 1_000.0, "K")
        } else {
            (abs_value, "")
        };

        let sign = if is_negative { "-" } else { "" };

        // Format with 1 decimal place if needed
        let number = if suffix.is_empty() {
            format!("{:.0}", scaled)
        } else if scaled >= 10.0 {
            format!("{:.1}", scaled)
        } else {
            format!("{:.2}", scaled)
        };

        match self.position {
            SymbolPosition::Prefix => format!("{}{}{}{}", sign, self.symbol, number, suffix),
            SymbolPosition::Suffix => format!("{}{}{} {}", sign, number, suffix, self.symbol),
        }
    }
}

/// Add thousands separator to a number string
fn add_thousands_separator(s: &str, separator: char) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    if len <= 3 {
        return s.to_string();
    }

    let mut result = String::with_capacity(len + len / 3);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(separator);
        }
        result.push(*c);
    }

    result
}

/// Format a decimal as a simple currency string (convenience function)
pub fn format_currency(value: Decimal) -> String {
    CurrencyFormat::default().format(value)
}

/// Format a decimal as compact currency (convenience function)
pub fn format_compact(value: Decimal) -> String {
    CurrencyFormat::default().format_compact(value)
}

/// Format a decimal with custom symbol
pub fn format_with_symbol(value: Decimal, symbol: &str) -> String {
    let format = CurrencyFormat {
        symbol: symbol.to_string(),
        ..Default::default()
    };
    format.format(value)
}

/// Format as percentage (0-100)
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format as percentage from decimal ratio (0.0-1.0)
pub fn format_ratio_as_percentage(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_format_usd() {
        let fmt = CurrencyFormat::usd();
        assert_eq!(fmt.format(dec!(1234.56)), "$1,234.56");
        assert_eq!(fmt.format(dec!(-1234.56)), "-$1,234.56");
        assert_eq!(fmt.format(dec!(0)), "$0.00");
        assert_eq!(fmt.format(dec!(999999.99)), "$999,999.99");
    }

    #[test]
    fn test_format_eur() {
        let fmt = CurrencyFormat::eur();
        assert_eq!(fmt.format(dec!(1234.56)), "1.234,56 €");
    }

    #[test]
    fn test_format_compact() {
        let fmt = CurrencyFormat::usd();
        assert_eq!(fmt.format_compact(dec!(999)), "$999");
        assert_eq!(fmt.format_compact(dec!(1500)), "$1.50K");
        assert_eq!(fmt.format_compact(dec!(1500000)), "$1.50M");
        assert_eq!(fmt.format_compact(dec!(1500000000)), "$1.50B");
    }

    #[test]
    fn test_thousands_separator() {
        assert_eq!(add_thousands_separator("123", ','), "123");
        assert_eq!(add_thousands_separator("1234", ','), "1,234");
        assert_eq!(add_thousands_separator("12345678", ','), "12,345,678");
    }
}
