use ratatui::style::Color;

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

impl Theme {
    pub const fn new() -> Self {
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
}

pub const THEME: Theme = Theme::new();
