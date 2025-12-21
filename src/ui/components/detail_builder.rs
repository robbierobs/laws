//! Detail panel builder
//!
//! Provides a fluent builder API for constructing styled detail panel content.
//! Reduces boilerplate across screen render functions.

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use crate::ui::theme::THEME;

/// Fluent builder for constructing styled detail panel lines
///
/// # Example
/// ```ignore
/// let lines = DetailBuilder::new()
///     .field("Name", &cluster.name)
///     .section("Configuration")
///     .optional_field("VPC", cluster.vpc_id.as_ref())
///     .bool_field("Container Insights", enabled, "Enabled", "Disabled")
///     .build();
/// ```
pub struct DetailBuilder<'a> {
    lines: Vec<Line<'a>>,
}

impl<'a> DetailBuilder<'a> {
    /// Create a new empty builder
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a labeled field with a value
    pub fn field(mut self, label: &'static str, value: &'a str) -> Self {
        self.lines.push(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(THEME.primary)),
            Span::raw(value),
        ]));
        self
    }

    /// Add a labeled field with an owned value (for computed values)
    #[allow(dead_code)] // Infrastructure for future screen migrations
    pub fn field_owned(mut self, label: &'static str, value: String) -> Self {
        self.lines.push(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(THEME.primary)),
            Span::raw(value),
        ]));
        self
    }

    /// Add a field only if the value is Some, otherwise skip
    pub fn optional_field<T: AsRef<str>>(mut self, label: &'static str, value: Option<&'a T>) -> Self {
        let display = value.map(|v| v.as_ref()).unwrap_or("-");
        self.lines.push(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(THEME.primary)),
            Span::raw(display),
        ]));
        self
    }

    /// Add a section header with styled separator
    pub fn section(mut self, title: &'static str) -> Self {
        self.lines.push(Line::from(""));
        self.lines.push(Line::from(vec![
            Span::styled(
                format!("─── {} ───", title),
                Style::default().fg(THEME.secondary).add_modifier(Modifier::BOLD),
            ),
        ]));
        self
    }

    /// Add a boolean field with colored yes/no indicators
    pub fn bool_field(mut self, label: &'static str, value: bool, yes_text: &'static str, no_text: &'static str) -> Self {
        let (text, color) = if value {
            (yes_text, THEME.success)
        } else {
            (no_text, THEME.muted)
        };
        self.lines.push(Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(THEME.primary)),
            Span::styled(text, Style::default().fg(color)),
        ]));
        self
    }

    /// Add a blank line for spacing
    pub fn blank(mut self) -> Self {
        self.lines.push(Line::from(""));
        self
    }

    /// Add a raw styled line (for custom formatting)
    #[allow(dead_code)] // Infrastructure for future screen migrations
    pub fn raw_line(mut self, line: Line<'a>) -> Self {
        self.lines.push(line);
        self
    }

    /// Add multiple lines from an iterator
    #[allow(dead_code)] // Infrastructure for future screen migrations
    pub fn lines<I>(mut self, lines: I) -> Self
    where
        I: IntoIterator<Item = Line<'a>>,
    {
        self.lines.extend(lines);
        self
    }

    /// Add a list of items with bullet points
    #[allow(dead_code)] // Infrastructure for future screen migrations
    pub fn list<I, S>(mut self, label: &'static str, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.lines.push(Line::from(vec![
            Span::styled(format!("{}:", label), Style::default().fg(THEME.primary)),
        ]));
        for item in items {
            self.lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::raw(item.as_ref().to_string()),
            ]));
        }
        self
    }

    /// Consume the builder and return the constructed lines
    pub fn build(self) -> Vec<Line<'a>> {
        self.lines
    }
}

impl<'a> Default for DetailBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_creates_line() {
        let lines = DetailBuilder::new()
            .field("Name", "test-value")
            .build();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_section_adds_header() {
        let lines = DetailBuilder::new()
            .section("Network")
            .build();
        // Section adds blank line + header = 2 lines
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_bool_field() {
        let lines = DetailBuilder::new()
            .bool_field("Enabled", true, "Yes", "No")
            .build();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_optional_field_with_none() {
        let value: Option<&String> = None;
        let lines = DetailBuilder::new()
            .optional_field("VPC", value)
            .build();
        assert_eq!(lines.len(), 1);
        // Should show "-" for None
    }

    #[test]
    fn test_chained_builder() {
        let lines = DetailBuilder::new()
            .field("ID", "i-12345")
            .section("Details")
            .field("Type", "t2.micro")
            .blank()
            .bool_field("Running", true, "Yes", "No")
            .build();
        // 1 field + 2 section + 1 field + 1 blank + 1 bool = 6
        assert_eq!(lines.len(), 6);
    }
}
