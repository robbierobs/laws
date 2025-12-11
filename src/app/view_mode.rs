/// Trait for handling view modes in services with multiple tabs/views.
/// This allows for consistent navigation and label generation.
pub trait ViewMode: Clone + Copy + PartialEq + Sized + 'static {
    /// Return all available view modes in the correct order.
    fn all() -> &'static [Self];

    /// Iterate over all view modes.
    fn iterator() -> std::slice::Iter<'static, Self> {
        Self::all().iter()
    }

    /// Return the index of the current view mode (0-based).
    fn index(&self) -> usize;

    /// Create a view mode from a 0-based index.
    /// Should wrap around if the index is out of bounds or return a default.
    fn from_index(i: usize) -> Self;

    /// Get the previous view mode (cyclic).
    fn prev(&self) -> Self {
        let i = self.index();
        let all = Self::all();
        let len = all.len();
        if len == 0 {
            return *self;
        }
        let prev_idx = if i == 0 { len - 1 } else { i - 1 };
        Self::from_index(prev_idx)
    }

    /// Get the next view mode (cyclic).
    fn next(&self) -> Self {
        let i = self.index();
        let all = Self::all();
        let len = all.len();
        if len == 0 {
            return *self;
        }
        let next_idx = (i + 1) % len;
        Self::from_index(next_idx)
    }

    /// Get a human-readable label for the view mode (e.g. for tabs).
    fn label(&self) -> &'static str;
}
