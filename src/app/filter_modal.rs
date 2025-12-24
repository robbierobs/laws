//! Generic filter modal abstraction
//!
//! Provides a reusable filter modal system that can be configured for different services.
//! Supports text input fields, date/time fields, toggle fields, and cycle-through options.
//!
//! # Example
//! ```ignore
//! let config = FilterModalConfig::new("CloudTrail Event Filters")
//!     .field(FilterFieldConfig::date("Start Date"))
//!     .field(FilterFieldConfig::time("Start Time"))
//!     .field(FilterFieldConfig::text("Event Name").placeholder("e.g., CreateBucket"))
//!     .field(FilterFieldConfig::toggle("Read Only", &["All", "Read", "Write"]));
//! ```

use std::collections::HashMap;

/// Type of filter field
#[derive(Debug, Clone, PartialEq)]
pub enum FilterFieldType {
    /// Free-form text input
    Text,
    /// Date input (YYYY-MM-DD)
    Date,
    /// Time input (HH:MM)
    Time,
    /// Boolean toggle (yes/no)
    Toggle,
    /// Cycle through predefined options
    Cycle(Vec<String>),
}

/// Configuration for a single filter field
#[derive(Debug, Clone)]
pub struct FilterFieldConfig {
    /// Unique identifier for the field
    pub id: String,
    /// Display label
    pub label: String,
    /// Field type
    pub field_type: FilterFieldType,
    /// Optional placeholder text
    pub placeholder: Option<String>,
    /// Width in characters (for layout)
    pub width: u16,
}

impl FilterFieldConfig {
    /// Create a text input field
    pub fn text(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FilterFieldType::Text,
            placeholder: None,
            width: 30,
        }
    }

    /// Create a date input field (YYYY-MM-DD)
    pub fn date(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FilterFieldType::Date,
            placeholder: Some("YYYY-MM-DD".into()),
            width: 12,
        }
    }

    /// Create a time input field (HH:MM)
    pub fn time(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FilterFieldType::Time,
            placeholder: Some("HH:MM".into()),
            width: 8,
        }
    }

    /// Create a toggle field (cycles through options)
    pub fn toggle(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FilterFieldType::Toggle,
            placeholder: None,
            width: 6,
        }
    }

    /// Create a cycle field with custom options
    pub fn cycle(id: impl Into<String>, label: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type: FilterFieldType::Cycle(options),
            placeholder: None,
            width: 15,
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set field width
    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }
}

/// Configuration for the entire filter modal
#[derive(Debug, Clone)]
pub struct FilterModalConfig {
    /// Modal title
    pub title: String,
    /// Field configurations in display order
    pub fields: Vec<FilterFieldConfig>,
    /// Width percentage of screen (0-100)
    pub width_percent: u16,
    /// Height percentage of screen (0-100)
    pub height_percent: u16,
}

impl FilterModalConfig {
    /// Create a new filter modal configuration
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            fields: Vec::new(),
            width_percent: 60,
            height_percent: 70,
        }
    }

    /// Add a field to the modal
    pub fn field(mut self, config: FilterFieldConfig) -> Self {
        self.fields.push(config);
        self
    }

    /// Set modal dimensions
    pub fn dimensions(mut self, width: u16, height: u16) -> Self {
        self.width_percent = width;
        self.height_percent = height;
        self
    }
}

/// Runtime state for a single field
#[derive(Debug, Clone, Default)]
pub struct FilterFieldState {
    /// Current text value (for Text, Date, Time fields)
    pub text_value: String,
    /// Current toggle state (for Toggle fields)
    pub toggle_value: bool,
    /// Current cycle index (for Cycle fields)
    pub cycle_index: usize,
}

impl FilterFieldState {
    /// Get the display value for this field
    pub fn display_value(&self, field_type: &FilterFieldType) -> String {
        match field_type {
            FilterFieldType::Text | FilterFieldType::Date | FilterFieldType::Time => {
                self.text_value.clone()
            }
            FilterFieldType::Toggle => {
                if self.toggle_value { "Yes".into() } else { "No".into() }
            }
            FilterFieldType::Cycle(options) => {
                options.get(self.cycle_index).cloned().unwrap_or_default()
            }
        }
    }

    /// Toggle a boolean field
    pub fn toggle(&mut self) {
        self.toggle_value = !self.toggle_value;
    }

    /// Cycle to next option
    pub fn cycle_next(&mut self, options_len: usize) {
        if options_len > 0 {
            self.cycle_index = (self.cycle_index + 1) % options_len;
        }
    }

    /// Add a character to text value
    pub fn push_char(&mut self, c: char) {
        self.text_value.push(c);
    }

    /// Remove last character from text value
    pub fn pop_char(&mut self) {
        self.text_value.pop();
    }

    /// Clear text value
    pub fn clear(&mut self) {
        self.text_value.clear();
    }
}

/// Runtime state for the entire filter modal
#[derive(Debug, Clone)]
pub struct FilterModalState {
    /// Whether the modal is currently visible
    pub visible: bool,
    /// Currently selected field index
    pub selected_field: usize,
    /// Field states keyed by field ID
    pub field_states: HashMap<String, FilterFieldState>,
}

impl FilterModalState {
    /// Create a new state from a config
    pub fn new(config: &FilterModalConfig) -> Self {
        let mut field_states = HashMap::new();
        for field in &config.fields {
            field_states.insert(field.id.clone(), FilterFieldState::default());
        }
        Self {
            visible: false,
            selected_field: 0,
            field_states,
        }
    }

    /// Show the modal
    pub fn open(&mut self) {
        self.visible = true;
        self.selected_field = 0;
    }

    /// Hide the modal
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Navigate to next field
    pub fn next_field(&mut self, total_fields: usize) {
        if total_fields > 0 {
            self.selected_field = (self.selected_field + 1) % total_fields;
        }
    }

    /// Navigate to previous field
    pub fn prev_field(&mut self, total_fields: usize) {
        if total_fields > 0 {
            self.selected_field = (self.selected_field + total_fields - 1) % total_fields;
        }
    }

    /// Get mutable reference to current field state
    pub fn current_field_state_mut(&mut self, config: &FilterModalConfig) -> Option<&mut FilterFieldState> {
        config.fields.get(self.selected_field)
            .and_then(|f| self.field_states.get_mut(&f.id))
    }

    /// Get reference to a field state by ID
    pub fn get_field(&self, id: &str) -> Option<&FilterFieldState> {
        self.field_states.get(id)
    }

    /// Get mutable reference to a field state by ID
    pub fn get_field_mut(&mut self, id: &str) -> Option<&mut FilterFieldState> {
        self.field_states.get_mut(id)
    }

    /// Get text value for a field
    pub fn get_text(&self, id: &str) -> String {
        self.field_states.get(id)
            .map(|s| s.text_value.clone())
            .unwrap_or_default()
    }

    /// Get toggle value for a field
    pub fn get_toggle(&self, id: &str) -> bool {
        self.field_states.get(id)
            .map(|s| s.toggle_value)
            .unwrap_or(false)
    }

    /// Get cycle index for a field
    pub fn get_cycle_index(&self, id: &str) -> usize {
        self.field_states.get(id)
            .map(|s| s.cycle_index)
            .unwrap_or(0)
    }

    /// Set text value for a field
    pub fn set_text(&mut self, id: &str, value: impl Into<String>) {
        if let Some(state) = self.field_states.get_mut(id) {
            state.text_value = value.into();
        }
    }

    /// Set toggle value for a field
    pub fn set_toggle(&mut self, id: &str, value: bool) {
        if let Some(state) = self.field_states.get_mut(id) {
            state.toggle_value = value;
        }
    }

    /// Set cycle index for a field
    pub fn set_cycle_index(&mut self, id: &str, index: usize) {
        if let Some(state) = self.field_states.get_mut(id) {
            state.cycle_index = index;
        }
    }

    /// Clear all field values
    pub fn clear_all(&mut self) {
        for state in self.field_states.values_mut() {
            state.text_value.clear();
            state.toggle_value = false;
            state.cycle_index = 0;
        }
    }

    /// Handle character input for current field
    pub fn handle_char(&mut self, c: char, config: &FilterModalConfig) {
        if let Some(field_config) = config.fields.get(self.selected_field) {
            if let Some(state) = self.field_states.get_mut(&field_config.id) {
                match &field_config.field_type {
                    FilterFieldType::Text => state.push_char(c),
                    FilterFieldType::Date => {
                        // Only allow digits and dashes
                        if c.is_ascii_digit() || c == '-' {
                            state.push_char(c);
                        }
                    }
                    FilterFieldType::Time => {
                        // Only allow digits and colons
                        if c.is_ascii_digit() || c == ':' {
                            state.push_char(c);
                        }
                    }
                    FilterFieldType::Toggle => {
                        // Space toggles
                        if c == ' ' {
                            state.toggle();
                        }
                    }
                    FilterFieldType::Cycle(options) => {
                        // Space cycles
                        if c == ' ' {
                            state.cycle_next(options.len());
                        }
                    }
                }
            }
        }
    }

    /// Handle backspace for current field
    pub fn handle_backspace(&mut self, config: &FilterModalConfig) {
        if let Some(field_config) = config.fields.get(self.selected_field) {
            if let Some(state) = self.field_states.get_mut(&field_config.id) {
                match field_config.field_type {
                    FilterFieldType::Text | FilterFieldType::Date | FilterFieldType::Time => {
                        state.pop_char();
                    }
                    _ => {} // Toggle and Cycle don't support backspace
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_field_config_builders() {
        let text = FilterFieldConfig::text("name", "Name");
        assert!(matches!(text.field_type, FilterFieldType::Text));
        assert_eq!(text.id, "name");
        assert_eq!(text.label, "Name");

        let date = FilterFieldConfig::date("start", "Start Date");
        assert!(matches!(date.field_type, FilterFieldType::Date));
        assert_eq!(date.placeholder, Some("YYYY-MM-DD".into()));
    }

    #[test]
    fn test_filter_modal_config_builder() {
        let config = FilterModalConfig::new("Test Filter")
            .field(FilterFieldConfig::text("q", "Query"))
            .field(FilterFieldConfig::toggle("active", "Active Only"));

        assert_eq!(config.fields.len(), 2);
        assert_eq!(config.title, "Test Filter");
    }

    #[test]
    fn test_filter_modal_state_navigation() {
        let config = FilterModalConfig::new("Test")
            .field(FilterFieldConfig::text("a", "A"))
            .field(FilterFieldConfig::text("b", "B"))
            .field(FilterFieldConfig::text("c", "C"));

        let mut state = FilterModalState::new(&config);
        assert_eq!(state.selected_field, 0);

        state.next_field(3);
        assert_eq!(state.selected_field, 1);

        state.next_field(3);
        assert_eq!(state.selected_field, 2);

        state.next_field(3);
        assert_eq!(state.selected_field, 0); // Wraps

        state.prev_field(3);
        assert_eq!(state.selected_field, 2); // Wraps back
    }

    #[test]
    fn test_filter_field_state_char_handling() {
        let config = FilterModalConfig::new("Test")
            .field(FilterFieldConfig::date("date", "Date"));

        let mut state = FilterModalState::new(&config);
        
        // Valid date chars
        state.handle_char('2', &config);
        state.handle_char('0', &config);
        state.handle_char('2', &config);
        state.handle_char('4', &config);
        state.handle_char('-', &config);
        
        assert_eq!(state.get_text("date"), "2024-");

        // Invalid char for date (letter)
        state.handle_char('a', &config);
        assert_eq!(state.get_text("date"), "2024-"); // Unchanged
    }

    #[test]
    fn test_cycle_field() {
        let config = FilterModalConfig::new("Test")
            .field(FilterFieldConfig::cycle("status", "Status", vec!["All".into(), "Active".into(), "Inactive".into()]));

        let mut state = FilterModalState::new(&config);
        assert_eq!(state.get_cycle_index("status"), 0);

        state.handle_char(' ', &config); // Space cycles
        assert_eq!(state.get_cycle_index("status"), 1);

        state.handle_char(' ', &config);
        assert_eq!(state.get_cycle_index("status"), 2);

        state.handle_char(' ', &config);
        assert_eq!(state.get_cycle_index("status"), 0); // Wraps
    }
}
