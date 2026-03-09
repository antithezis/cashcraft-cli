//! Autocomplete widget
//!
//! A dropdown widget that displays filtered suggestions based on user input.
//! Used with InputState to provide category autocomplete functionality.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::ui::theme::Theme;

/// State for autocomplete suggestions
#[derive(Debug, Clone, Default)]
pub struct AutocompleteState {
    /// All available suggestions (full list)
    pub all_suggestions: Vec<String>,
    /// Filtered suggestions based on current input
    pub filtered: Vec<String>,
    /// Currently selected suggestion index
    pub selected: Option<usize>,
    /// Whether to show the suggestions dropdown
    pub visible: bool,
}

impl AutocompleteState {
    /// Create a new empty autocomplete state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial suggestions
    pub fn with_suggestions(suggestions: Vec<String>) -> Self {
        Self {
            all_suggestions: suggestions.clone(),
            filtered: suggestions,
            selected: None,
            visible: false,
        }
    }

    /// Set the available suggestions
    pub fn set_suggestions(&mut self, suggestions: Vec<String>) {
        self.all_suggestions = suggestions;
        self.filtered = self.all_suggestions.clone();
        self.selected = None;
    }

    /// Filter suggestions based on input value
    pub fn filter(&mut self, input: &str) {
        if input.is_empty() {
            self.filtered = self.all_suggestions.clone();
        } else {
            let input_lower = input.to_lowercase();

            // Prioritize prefix matches, then contains matches
            let mut prefix_matches: Vec<String> = Vec::new();
            let mut contains_matches: Vec<String> = Vec::new();

            for suggestion in &self.all_suggestions {
                let suggestion_lower = suggestion.to_lowercase();
                if suggestion_lower.starts_with(&input_lower) {
                    prefix_matches.push(suggestion.clone());
                } else if suggestion_lower.contains(&input_lower) {
                    contains_matches.push(suggestion.clone());
                }
            }

            self.filtered = prefix_matches;
            self.filtered.extend(contains_matches);
        }

        // Reset selection if current selection is out of bounds
        if let Some(idx) = self.selected {
            if idx >= self.filtered.len() {
                self.selected = if self.filtered.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
        }

        // Auto-show if there are matches
        self.visible = !self.filtered.is_empty();
    }

    /// Select the next suggestion
    pub fn select_next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }

        self.selected = Some(match self.selected {
            None => 0,
            Some(idx) => (idx + 1) % self.filtered.len(),
        });
        self.visible = true;
    }

    /// Select the previous suggestion
    pub fn select_prev(&mut self) {
        if self.filtered.is_empty() {
            return;
        }

        self.selected = Some(match self.selected {
            None => self.filtered.len().saturating_sub(1),
            Some(0) => self.filtered.len().saturating_sub(1),
            Some(idx) => idx - 1,
        });
        self.visible = true;
    }

    /// Get the currently selected suggestion
    pub fn selected_value(&self) -> Option<&str> {
        self.selected
            .and_then(|idx| self.filtered.get(idx))
            .map(|s| s.as_str())
    }

    /// Accept the current selection (returns the selected value)
    pub fn accept(&mut self) -> Option<String> {
        let value = self.selected_value().map(|s| s.to_string());
        self.hide();
        value
    }

    /// Hide the suggestions dropdown
    pub fn hide(&mut self) {
        self.visible = false;
        self.selected = None;
    }

    /// Show the suggestions dropdown
    pub fn show(&mut self) {
        if !self.filtered.is_empty() {
            self.visible = true;
        }
    }

    /// Check if there are any suggestions to show
    pub fn has_suggestions(&self) -> bool {
        !self.filtered.is_empty()
    }
}

/// Autocomplete dropdown widget
pub struct Autocomplete<'a> {
    state: &'a AutocompleteState,
    theme: &'a Theme,
    max_visible: usize,
}

impl<'a> Autocomplete<'a> {
    /// Create a new autocomplete widget
    pub fn new(state: &'a AutocompleteState, theme: &'a Theme) -> Self {
        Self {
            state,
            theme,
            max_visible: 5,
        }
    }

    /// Set maximum number of visible suggestions
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max;
        self
    }
}

impl Widget for Autocomplete<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || self.state.filtered.is_empty() {
            return;
        }

        let items_to_show = self.state.filtered.len().min(self.max_visible);

        // Calculate scroll offset to keep selected item visible
        let scroll_offset = if let Some(selected) = self.state.selected {
            if selected >= items_to_show {
                selected - items_to_show + 1
            } else {
                0
            }
        } else {
            0
        };

        // Render background
        let dropdown_style = Style::default()
            .bg(self.theme.colors.surface)
            .fg(self.theme.colors.text_primary);

        for y in 0..items_to_show {
            let row_y = area.y + y as u16;
            if row_y >= area.y + area.height {
                break;
            }

            // Clear the row
            for x in area.x..area.x + area.width {
                buf.set_string(x, row_y, " ", dropdown_style);
            }
        }

        // Render suggestions
        for (display_idx, suggestion_idx) in
            (scroll_offset..scroll_offset + items_to_show).enumerate()
        {
            if suggestion_idx >= self.state.filtered.len() {
                break;
            }

            let y = area.y + display_idx as u16;
            if y >= area.y + area.height {
                break;
            }

            let suggestion = &self.state.filtered[suggestion_idx];
            let is_selected = self.state.selected == Some(suggestion_idx);

            let style = if is_selected {
                Style::default()
                    .bg(self.theme.colors.accent)
                    .fg(self.theme.colors.background)
                    .add_modifier(Modifier::BOLD)
            } else {
                dropdown_style
            };

            // Clear the line first
            for x in area.x..area.x + area.width {
                buf.set_string(x, y, " ", style);
            }

            // Truncate suggestion if too long
            let display_text: String = suggestion.chars().take(area.width as usize - 2).collect();

            buf.set_string(area.x + 1, y, &display_text, style);
        }

        // Draw border (simple box)
        let border_style = Style::default().fg(self.theme.colors.border_focus);

        // Top border
        if area.y > 0 {
            buf.set_string(area.x, area.y.saturating_sub(1), "├", border_style);
            for x in area.x + 1..area.x + area.width - 1 {
                buf.set_string(x, area.y.saturating_sub(1), "─", border_style);
            }
            buf.set_string(
                area.x + area.width - 1,
                area.y.saturating_sub(1),
                "┤",
                border_style,
            );
        }

        // Bottom border
        let bottom_y = area.y + items_to_show as u16;
        if bottom_y < area.y + area.height {
            buf.set_string(area.x, bottom_y, "└", border_style);
            for x in area.x + 1..area.x + area.width - 1 {
                buf.set_string(x, bottom_y, "─", border_style);
            }
            buf.set_string(area.x + area.width - 1, bottom_y, "┘", border_style);
        }

        // Side borders
        for y in 0..items_to_show {
            let row_y = area.y + y as u16;
            if row_y < area.y + area.height {
                buf.set_string(area.x, row_y, "│", border_style);
                buf.set_string(area.x + area.width - 1, row_y, "│", border_style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_state_filter() {
        let mut state = AutocompleteState::with_suggestions(vec![
            "Food".to_string(),
            "Transportation".to_string(),
            "Entertainment".to_string(),
        ]);

        state.filter("Foo");
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0], "Food");

        state.filter("ent");
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.filtered[0], "Entertainment");

        state.filter("");
        assert_eq!(state.filtered.len(), 3);
    }

    #[test]
    fn test_autocomplete_state_navigation() {
        let mut state = AutocompleteState::with_suggestions(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);
        state.filter("");

        assert_eq!(state.selected, None);

        state.select_next();
        assert_eq!(state.selected, Some(0));
        assert_eq!(state.selected_value(), Some("A"));

        state.select_next();
        assert_eq!(state.selected, Some(1));
        assert_eq!(state.selected_value(), Some("B"));

        state.select_prev();
        assert_eq!(state.selected, Some(0));

        // Wrap around
        state.select_prev();
        assert_eq!(state.selected, Some(2));
    }

    #[test]
    fn test_autocomplete_accept() {
        let mut state =
            AutocompleteState::with_suggestions(vec!["Food".to_string(), "Housing".to_string()]);
        state.filter("");
        state.select_next();

        let accepted = state.accept();
        assert_eq!(accepted, Some("Food".to_string()));
        assert!(!state.visible);
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_filter_prioritizes_prefix() {
        let mut state = AutocompleteState::with_suggestions(vec![
            "Entertainment".to_string(),
            "Rent".to_string(),
        ]);

        state.filter("ent");

        // "Entertainment" starts with "ent", "Rent" contains "ent"
        // Entertainment should come first
        assert_eq!(state.filtered.len(), 2);
        assert_eq!(state.filtered[0], "Entertainment");
        assert_eq!(state.filtered[1], "Rent");
    }
}
