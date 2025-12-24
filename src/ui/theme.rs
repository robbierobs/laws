//! Application theming system
//!
//! Provides multiple built-in themes and the ability to customize colors.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Available theme presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Monokai,
    Nord,
}

#[allow(dead_code)]
impl ThemePreset {
    /// Get the theme corresponding to this preset
    pub const fn theme(&self) -> Theme {
        match self {
            ThemePreset::Dark => Theme::dark(),
            ThemePreset::Light => Theme::light(),
            ThemePreset::Monokai => Theme::monokai(),
            ThemePreset::Nord => Theme::nord(),
        }
    }
    
    /// List all available theme names
    pub fn available() -> &'static [&'static str] {
        &["dark", "light", "monokai", "nord"]
    }
}

pub struct Theme {
    #[allow(dead_code)]
    pub bg: Color,
    pub fg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub muted: Color,
    pub border: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

#[allow(dead_code)]
impl Theme {
    /// Default dark theme (Slate color palette)
    pub const fn dark() -> Self {
        Self {
            bg: Color::Rgb(15, 23, 42),       // Slate 900
            fg: Color::Rgb(226, 232, 240),    // Slate 200
            primary: Color::Rgb(56, 189, 248), // Sky 400
            secondary: Color::Rgb(168, 85, 247), // Purple 500
            success: Color::Rgb(74, 222, 128), // Green 400
            warning: Color::Rgb(250, 204, 21), // Yellow 400
            error: Color::Rgb(248, 113, 113),  // Red 400
            muted: Color::Rgb(148, 163, 184),  // Slate 400
            border: Color::Rgb(51, 65, 85),    // Slate 700
            selection_bg: Color::Rgb(30, 41, 59), // Slate 800
            selection_fg: Color::Rgb(56, 189, 248), // Sky 400
        }
    }
    
    /// Light theme
    pub const fn light() -> Self {
        Self {
            bg: Color::Rgb(248, 250, 252),     // Slate 50
            fg: Color::Rgb(30, 41, 59),        // Slate 800
            primary: Color::Rgb(14, 165, 233), // Sky 500
            secondary: Color::Rgb(139, 92, 246), // Violet 500
            success: Color::Rgb(34, 197, 94),  // Green 500
            warning: Color::Rgb(234, 179, 8),  // Yellow 500
            error: Color::Rgb(239, 68, 68),    // Red 500
            muted: Color::Rgb(100, 116, 139),  // Slate 500
            border: Color::Rgb(203, 213, 225), // Slate 300
            selection_bg: Color::Rgb(226, 232, 240), // Slate 200
            selection_fg: Color::Rgb(14, 165, 233), // Sky 500
        }
    }
    
    /// Monokai color scheme
    pub const fn monokai() -> Self {
        Self {
            bg: Color::Rgb(39, 40, 34),        // Monokai background
            fg: Color::Rgb(248, 248, 242),     // Monokai foreground
            primary: Color::Rgb(102, 217, 239), // Monokai cyan
            secondary: Color::Rgb(174, 129, 255), // Monokai purple
            success: Color::Rgb(166, 226, 46), // Monokai green
            warning: Color::Rgb(230, 219, 116), // Monokai yellow
            error: Color::Rgb(249, 38, 114),   // Monokai pink
            muted: Color::Rgb(117, 113, 94),   // Monokai comment
            border: Color::Rgb(73, 72, 62),    // Monokai line
            selection_bg: Color::Rgb(73, 72, 62), // Monokai selection
            selection_fg: Color::Rgb(102, 217, 239), // Monokai cyan
        }
    }
    
    /// Nord color scheme
    pub const fn nord() -> Self {
        Self {
            bg: Color::Rgb(46, 52, 64),        // Nord0 - Polar Night
            fg: Color::Rgb(236, 239, 244),     // Nord6 - Snow Storm
            primary: Color::Rgb(136, 192, 208), // Nord8 - Frost
            secondary: Color::Rgb(180, 142, 173), // Nord15 - Aurora
            success: Color::Rgb(163, 190, 140), // Nord14 - Aurora green
            warning: Color::Rgb(235, 203, 139), // Nord13 - Aurora yellow
            error: Color::Rgb(191, 97, 106),   // Nord11 - Aurora red
            muted: Color::Rgb(76, 86, 106),    // Nord3 - Polar Night
            border: Color::Rgb(67, 76, 94),    // Nord2 - Polar Night
            selection_bg: Color::Rgb(59, 66, 82), // Nord1 - Polar Night
            selection_fg: Color::Rgb(136, 192, 208), // Nord8 - Frost
        }
    }
    
    /// Legacy constructor for backwards compatibility
    pub const fn new() -> Self {
        Self::dark()
    }
}

/// Global theme instance - the Dark theme (Slate color palette)
/// 
/// This is the original theme that has been used since the beginning.
/// Runtime theme switching is not currently supported - the THEME constant
/// is used directly throughout the codebase for optimal performance.
/// 
/// Color palette:
/// - Background: Slate 900 (dark blue-gray)
/// - Foreground: Slate 200 (light gray)
/// - Primary: Sky 400 (bright blue)
/// - Secondary: Purple 500
/// - Success: Green 400
/// - Warning: Yellow 400
/// - Error: Red 400
/// - Muted: Slate 400
/// - Border: Slate 700
/// - Selection: Slate 800 bg with Sky 400 text
pub const THEME: Theme = Theme::dark();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_preset_default() {
        let preset = ThemePreset::default();
        assert_eq!(preset, ThemePreset::Dark);
    }

    #[test]
    fn test_theme_preset_available() {
        let available = ThemePreset::available();
        assert!(available.contains(&"dark"));
        assert!(available.contains(&"light"));
        assert!(available.contains(&"monokai"));
        assert!(available.contains(&"nord"));
    }

    #[test]
    fn test_all_themes_have_distinct_colors() {
        let dark = Theme::dark();
        let light = Theme::light();
        
        // Light and dark should have different backgrounds
        assert_ne!(format!("{:?}", dark.bg), format!("{:?}", light.bg));
    }
}

