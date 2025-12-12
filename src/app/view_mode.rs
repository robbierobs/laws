/// Trait for handling view modes in services with multiple tabs/views.
/// This allows for consistent navigation and label generation.
/// 
/// Services with drill-down views (e.g., VPC -> SecurityGroupRules, IAM -> PolicyDocument)
/// should override `main_tabs()` to return only the top-level tabs, and implement
/// `is_drill_down()` to return true for nested views.
pub trait ViewMode: Clone + Copy + PartialEq + Sized + 'static {
    /// Return all available view modes in the correct order.
    /// This includes both main tabs and drill-down views.
    fn all() -> &'static [Self];

    /// Return only the main tabs (excludes drill-down views).
    /// Default implementation returns all views.
    /// Override this for services with drill-down navigation.
    fn main_tabs() -> &'static [Self] {
        Self::all()
    }

    /// Iterate over all view modes.
    fn iterator() -> std::slice::Iter<'static, Self> {
        Self::all().iter()
    }

    /// Return the index of the current view mode (0-based).
    fn index(&self) -> usize;

    /// Create a view mode from a 0-based index.
    /// Should wrap around if the index is out of bounds or return a default.
    fn from_index(i: usize) -> Self;

    /// Returns true if this is a main tab (not a drill-down view).
    /// Default implementation returns true for all views.
    fn is_main_tab(&self) -> bool {
        true
    }

    /// Returns true if we're in a drilled-down view.
    /// This is the inverse of `is_main_tab()`.
    fn is_drill_down(&self) -> bool {
        !self.is_main_tab()
    }

    /// Returns true if tab cycling should be enabled in the current view.
    /// Default: cycling is allowed only in main tabs.
    fn supports_cycling(&self) -> bool {
        self.is_main_tab()
    }

    /// Get the previous view mode (cyclic among main tabs).
    /// If in a drill-down view, returns self unchanged.
    fn prev(&self) -> Self {
        if !self.supports_cycling() {
            return *self;
        }
        let i = self.index();
        let tabs = Self::main_tabs();
        let len = tabs.len();
        if len == 0 {
            return *self;
        }
        let prev_idx = if i == 0 { len - 1 } else { i - 1 };
        Self::from_index(prev_idx)
    }

    /// Get the next view mode (cyclic among main tabs).
    /// If in a drill-down view, returns self unchanged.
    fn next(&self) -> Self {
        if !self.supports_cycling() {
            return *self;
        }
        let i = self.index();
        let tabs = Self::main_tabs();
        let len = tabs.len();
        if len == 0 {
            return *self;
        }
        let next_idx = (i + 1) % len;
        Self::from_index(next_idx)
    }

    /// Get a human-readable label for the view mode (e.g. for tabs).
    fn label(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug, Default)]
    enum TestViewMode {
        #[default]
        Tab1 = 0,
        Tab2 = 1,
        DrillDown = 2,
    }

    impl ViewMode for TestViewMode {
        fn all() -> &'static [Self] {
            &[Self::Tab1, Self::Tab2, Self::DrillDown]
        }

        fn main_tabs() -> &'static [Self] {
            &[Self::Tab1, Self::Tab2]
        }

        fn index(&self) -> usize {
            *self as usize
        }

        fn from_index(i: usize) -> Self {
            match i {
                0 => Self::Tab1,
                1 => Self::Tab2,
                2 => Self::DrillDown,
                _ => Self::Tab1,
            }
        }

        fn is_main_tab(&self) -> bool {
            matches!(self, Self::Tab1 | Self::Tab2)
        }

        fn label(&self) -> &'static str {
            match self {
                Self::Tab1 => "Tab 1",
                Self::Tab2 => "Tab 2",
                Self::DrillDown => "Details",
            }
        }
    }

    #[test]
    fn test_main_tab_cycling() {
        let mode = TestViewMode::Tab1;
        assert_eq!(mode.next(), TestViewMode::Tab2);
        assert_eq!(mode.next().next(), TestViewMode::Tab1); // Wraps around
    }

    #[test]
    fn test_drill_down_no_cycling() {
        let mode = TestViewMode::DrillDown;
        assert_eq!(mode.next(), TestViewMode::DrillDown); // No change
        assert_eq!(mode.prev(), TestViewMode::DrillDown); // No change
    }

    #[test]
    fn test_is_main_tab() {
        assert!(TestViewMode::Tab1.is_main_tab());
        assert!(TestViewMode::Tab2.is_main_tab());
        assert!(!TestViewMode::DrillDown.is_main_tab());
    }

    #[test]
    fn test_is_drill_down() {
        assert!(!TestViewMode::Tab1.is_drill_down());
        assert!(TestViewMode::DrillDown.is_drill_down());
    }

    #[test]
    fn test_supports_cycling() {
        assert!(TestViewMode::Tab1.supports_cycling());
        assert!(!TestViewMode::DrillDown.supports_cycling());
    }
}
