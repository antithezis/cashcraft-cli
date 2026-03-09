//! Input widget with Vim modes
//!
//! A text input widget that supports:
//! - Insert mode text editing
//! - Normal mode navigation
//! - Cursor positioning
//! - Validation feedback
//! - Placeholder text

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::ui::theme::Theme;

/// State for the TextInput widget
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current input value
    pub value: String,
    /// Cursor position (character index)
    pub cursor: usize,
    /// Whether the input is focused
    pub focused: bool,
    /// Whether in insert mode (vs normal mode)
    pub insert_mode: bool,
    /// History of values for undo
    history: Vec<String>,
    /// Current position in history
    history_index: usize,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            focused: false,
            insert_mode: true,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Create with an initial value
    pub fn with_value(value: impl Into<String>) -> Self {
        let v = value.into();
        let len = v.len();
        Self {
            value: v,
            cursor: len,
            focused: false,
            insert_mode: true,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Save current state to history (for undo)
    fn save_history(&mut self) {
        // Don't save if value hasn't changed
        if self.history.last().map(|s| s.as_str()) != Some(&self.value) {
            self.history.push(self.value.clone());
            self.history_index = self.history.len();
        }
    }

    /// Insert a character at cursor position
    pub fn insert(&mut self, c: char) {
        self.save_history();
        let byte_index = self.cursor_to_byte_index();
        self.value.insert(byte_index, c);
        self.cursor += 1;
    }

    /// Insert a string at cursor position
    pub fn insert_str(&mut self, s: &str) {
        self.save_history();
        let byte_index = self.cursor_to_byte_index();
        self.value.insert_str(byte_index, s);
        self.cursor += s.chars().count();
    }

    /// Delete character before cursor (Backspace)
    pub fn delete(&mut self) {
        if self.cursor > 0 {
            self.save_history();
            self.cursor -= 1;
            let byte_index = self.cursor_to_byte_index();
            if byte_index < self.value.len() {
                // Find the byte range for the character at cursor
                let char_len = self.value[byte_index..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                self.value = format!(
                    "{}{}",
                    &self.value[..byte_index],
                    &self.value[byte_index + char_len..]
                );
            }
        }
    }

    /// Delete character at cursor (Delete/x)
    pub fn delete_forward(&mut self) {
        let char_count = self.value.chars().count();
        if self.cursor < char_count {
            self.save_history();
            let byte_index = self.cursor_to_byte_index();
            if byte_index < self.value.len() {
                let char_len = self.value[byte_index..]
                    .chars()
                    .next()
                    .map(|c| c.len_utf8())
                    .unwrap_or(0);
                self.value = format!(
                    "{}{}",
                    &self.value[..byte_index],
                    &self.value[byte_index + char_len..]
                );
            }
        }
    }

    /// Delete word before cursor (Ctrl+W)
    pub fn delete_word(&mut self) {
        if self.cursor > 0 {
            self.save_history();
            let chars: Vec<char> = self.value.chars().collect();
            let mut new_cursor = self.cursor;

            // Skip trailing whitespace
            while new_cursor > 0 && chars[new_cursor - 1].is_whitespace() {
                new_cursor -= 1;
            }

            // Delete word characters
            while new_cursor > 0 && !chars[new_cursor - 1].is_whitespace() {
                new_cursor -= 1;
            }

            let chars_before: String = chars[..new_cursor].iter().collect();
            let chars_after: String = chars[self.cursor..].iter().collect();
            self.value = format!("{}{}", chars_before, chars_after);
            self.cursor = new_cursor;
        }
    }

    /// Delete to end of line (Ctrl+K or D$)
    pub fn delete_to_end(&mut self) {
        let chars: Vec<char> = self.value.chars().collect();
        if self.cursor < chars.len() {
            self.save_history();
            self.value = chars[..self.cursor].iter().collect();
        }
    }

    /// Delete to start of line (Ctrl+U or D0)
    pub fn delete_to_start(&mut self) {
        if self.cursor > 0 {
            self.save_history();
            let chars: Vec<char> = self.value.chars().collect();
            self.value = chars[self.cursor..].iter().collect();
            self.cursor = 0;
        }
    }

    /// Move cursor left (h/Left)
    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Move cursor right (l/Right)
    pub fn move_right(&mut self) {
        let char_count = self.value.chars().count();
        self.cursor = (self.cursor + 1).min(char_count);
    }

    /// Move cursor to start of line (0/Home)
    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end of line ($/End)
    pub fn move_end(&mut self) {
        self.cursor = self.value.chars().count();
    }

    /// Move cursor to next word (w)
    pub fn move_word_forward(&mut self) {
        let chars: Vec<char> = self.value.chars().collect();
        let len = chars.len();

        if self.cursor >= len {
            return;
        }

        let mut pos = self.cursor;

        // Skip current word
        while pos < len && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip whitespace
        while pos < len && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.cursor = pos;
    }

    /// Move cursor to previous word (b)
    pub fn move_word_backward(&mut self) {
        let chars: Vec<char> = self.value.chars().collect();

        if self.cursor == 0 {
            return;
        }

        let mut pos = self.cursor;

        // Skip whitespace before cursor
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip word characters
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.cursor = pos;
    }

    /// Clear the input
    pub fn clear(&mut self) {
        if !self.value.is_empty() {
            self.save_history();
            self.value.clear();
            self.cursor = 0;
        }
    }

    /// Set the value (replacing current content)
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.save_history();
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }

    /// Undo last change
    pub fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.value = self.history[self.history_index].clone();
            self.cursor = self.value.chars().count();
        }
    }

    /// Redo (after undo)
    pub fn redo(&mut self) {
        if self.history_index < self.history.len().saturating_sub(1) {
            self.history_index += 1;
            self.value = self.history[self.history_index].clone();
            self.cursor = self.value.chars().count();
        }
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if the input is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Focus the input
    pub fn focus(&mut self) {
        self.focused = true;
        self.insert_mode = true;
    }

    /// Unfocus the input
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Enter insert mode
    pub fn enter_insert(&mut self) {
        self.insert_mode = true;
    }

    /// Enter normal mode
    pub fn enter_normal(&mut self) {
        self.insert_mode = false;
    }

    /// Convert cursor position (character index) to byte index
    fn cursor_to_byte_index(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len())
    }
}

/// A text input widget
pub struct TextInput<'a> {
    state: &'a InputState,
    placeholder: Option<&'a str>,
    theme: &'a Theme,
    label: Option<&'a str>,
    block: Option<Block<'a>>,
    validation_error: Option<&'a str>,
    width: Option<u16>,
}

impl<'a> TextInput<'a> {
    pub fn new(state: &'a InputState, theme: &'a Theme) -> Self {
        Self {
            state,
            placeholder: None,
            theme,
            label: None,
            block: None,
            validation_error: None,
            width: None,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn validation_error(mut self, error: &'a str) -> Self {
        self.validation_error = Some(error);
        self
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for TextInput<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        // Determine colors based on state
        let (border_color, text_color) = if self.validation_error.is_some() {
            (self.theme.colors.error, self.theme.colors.error)
        } else if self.state.focused {
            (
                self.theme.colors.border_focus,
                self.theme.colors.text_primary,
            )
        } else {
            (self.theme.colors.border, self.theme.colors.text_secondary)
        };

        // Render block/border if specified, or create default
        let inner = if let Some(block) = self.block {
            let inner = block.inner(area);
            block
                .border_style(Style::default().fg(border_color))
                .render(area, buf);
            inner
        } else {
            // Default: simple border
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        if inner.width < 1 || inner.height < 1 {
            return;
        }

        // Calculate visible portion of text
        let max_visible = inner.width as usize;
        let cursor_pos = self.state.cursor;

        // Calculate scroll offset to keep cursor visible
        let scroll_offset = if cursor_pos >= max_visible {
            cursor_pos - max_visible + 1
        } else {
            0
        };

        // Get visible text
        let display_text: String = if self.state.value.is_empty() {
            if let Some(placeholder) = self.placeholder {
                placeholder.chars().take(max_visible).collect()
            } else {
                String::new()
            }
        } else {
            self.state
                .value
                .chars()
                .skip(scroll_offset)
                .take(max_visible)
                .collect()
        };

        // Text style
        let text_style = if self.state.value.is_empty() && self.placeholder.is_some() {
            Style::default().fg(self.theme.colors.text_muted)
        } else {
            Style::default().fg(text_color)
        };

        // Render text
        buf.set_string(inner.x, inner.y, &display_text, text_style);

        // Render cursor if focused
        if self.state.focused {
            let cursor_x = inner.x + (cursor_pos - scroll_offset) as u16;
            if cursor_x < inner.x + inner.width {
                let cursor_char = self.state.value.chars().nth(cursor_pos).unwrap_or(' ');

                let cursor_style = if self.state.insert_mode {
                    // Thin cursor in insert mode
                    Style::default()
                        .fg(self.theme.colors.background)
                        .bg(self.theme.colors.primary)
                } else {
                    // Block cursor in normal mode
                    Style::default()
                        .fg(self.theme.colors.background)
                        .bg(self.theme.colors.accent)
                        .add_modifier(Modifier::BOLD)
                };

                buf.set_string(cursor_x, inner.y, cursor_char.to_string(), cursor_style);
            }
        }

        // Render validation error below if there's space
        if let Some(error) = self.validation_error {
            if area.height > 1 {
                let error_y = inner.y + 1;
                if error_y < area.y + area.height {
                    let error_style = Style::default().fg(self.theme.colors.error);
                    let truncated: String = error.chars().take(inner.width as usize).collect();
                    buf.set_string(inner.x, error_y, &truncated, error_style);
                }
            }
        }
    }
}

/// Multi-line text input state
#[derive(Debug, Clone)]
pub struct MultiLineInputState {
    pub lines: Vec<InputState>,
    pub current_line: usize,
}

impl Default for MultiLineInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiLineInputState {
    pub fn new() -> Self {
        Self {
            lines: vec![InputState::new()],
            current_line: 0,
        }
    }

    pub fn current(&self) -> &InputState {
        &self.lines[self.current_line]
    }

    pub fn current_mut(&mut self) -> &mut InputState {
        &mut self.lines[self.current_line]
    }

    pub fn move_up(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
            // Keep cursor position if possible
            let len = self.lines[self.current_line].value.chars().count();
            if self.lines[self.current_line].cursor > len {
                self.lines[self.current_line].cursor = len;
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.current_line < self.lines.len() - 1 {
            self.current_line += 1;
            let len = self.lines[self.current_line].value.chars().count();
            if self.lines[self.current_line].cursor > len {
                self.lines[self.current_line].cursor = len;
            }
        }
    }

    pub fn new_line(&mut self) {
        let current = &mut self.lines[self.current_line];
        let cursor = current.cursor;
        let rest: String = current.value.chars().skip(cursor).collect();
        current.value = current.value.chars().take(cursor).collect();

        self.current_line += 1;
        self.lines
            .insert(self.current_line, InputState::with_value(rest));
    }

    pub fn get_text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.value.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn set_text(&mut self, text: &str) {
        self.lines = text.lines().map(InputState::with_value).collect();
        if self.lines.is_empty() {
            self.lines.push(InputState::new());
        }
        self.current_line = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_basic() {
        let mut state = InputState::new();
        assert!(state.is_empty());
        assert_eq!(state.cursor, 0);

        state.insert('h');
        state.insert('e');
        state.insert('l');
        state.insert('l');
        state.insert('o');

        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor, 5);
    }

    #[test]
    fn test_input_state_delete() {
        let mut state = InputState::with_value("hello");

        state.delete(); // Delete 'o'
        assert_eq!(state.value(), "hell");

        state.cursor = 0;
        state.delete(); // Should do nothing at start
        assert_eq!(state.value(), "hell");

        state.delete_forward(); // Delete 'h'
        assert_eq!(state.value(), "ell");
    }

    #[test]
    fn test_input_state_navigation() {
        let mut state = InputState::with_value("hello world");

        state.move_start();
        assert_eq!(state.cursor, 0);

        state.move_end();
        assert_eq!(state.cursor, 11);

        state.move_start();
        state.move_word_forward();
        assert_eq!(state.cursor, 6); // After "hello "

        state.move_word_backward();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn test_input_state_delete_word() {
        let mut state = InputState::with_value("hello world");
        state.cursor = 11; // End

        state.delete_word();
        assert_eq!(state.value(), "hello ");
        assert_eq!(state.cursor, 6);

        state.delete_word();
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn test_input_state_undo() {
        let mut state = InputState::new();

        state.insert('a');
        state.insert('b');
        state.insert('c');
        assert_eq!(state.value(), "abc");

        state.clear();
        assert_eq!(state.value(), "");

        state.undo();
        assert_eq!(state.value(), "abc");
    }

    #[test]
    fn test_input_state_unicode() {
        let mut state = InputState::new();

        state.insert('你');
        state.insert('好');
        state.insert('!');

        assert_eq!(state.value(), "你好!");
        assert_eq!(state.cursor, 3);

        state.move_left();
        assert_eq!(state.cursor, 2);

        state.delete();
        assert_eq!(state.value(), "你!");
    }

    #[test]
    fn test_multiline_input() {
        let mut state = MultiLineInputState::new();

        state.current_mut().set_value("line 1");
        state.new_line();
        state.current_mut().set_value("line 2");

        assert_eq!(state.get_text(), "line 1\nline 2");
        assert_eq!(state.current_line, 1);

        state.move_up();
        assert_eq!(state.current_line, 0);
    }
}
