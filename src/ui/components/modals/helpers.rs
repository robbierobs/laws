//! Common helper functions for modal rendering
//!
//! Provides centering utilities and formatting helpers used across all modals.

use crate::ui::theme::THEME;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;

/// Create a centered rect using percentage-based sizing
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a centered rect with fixed width and height (in characters/rows)
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    // Ensure we don't exceed available space
    let actual_width = width.min(r.width);
    let actual_height = height.min(r.height);

    // Calculate centering offsets
    let x_offset = (r.width.saturating_sub(actual_width)) / 2;
    let y_offset = (r.height.saturating_sub(actual_height)) / 2;

    Rect {
        x: r.x + x_offset,
        y: r.y + y_offset,
        width: actual_width,
        height: actual_height,
    }
}

/// Get syntax highlighting style based on file extension
pub fn get_syntax_style(ext: &str) -> Style {
    match ext.to_lowercase().as_str() {
        // JSON - cyan
        "json" => Style::default().fg(THEME.primary),
        // YAML - green
        "yaml" | "yml" => Style::default().fg(THEME.success),
        // XML/HTML - yellow
        "xml" | "html" | "htm" => Style::default().fg(THEME.warning),
        // Code files - default with slight highlight
        "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" => {
            Style::default().fg(THEME.fg)
        }
        // Config files - muted
        "toml" | "ini" | "cfg" | "conf" => Style::default().fg(THEME.secondary),
        // Markdown - default
        "md" | "txt" => Style::default().fg(THEME.fg),
        // Default
        _ => Style::default().fg(THEME.fg),
    }
}

/// Format bytes as a hex dump with ASCII representation
pub fn format_hex_dump(bytes: &[u8], bytes_per_line: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for (offset, chunk) in bytes.chunks(bytes_per_line).enumerate() {
        let addr = offset * bytes_per_line;
        let hex_part: Vec<String> = chunk.iter().map(|b| format!("{:02x}", b)).collect();
        let hex_str = hex_part.join(" ");

        // Pad hex string to fixed width
        let hex_padded = format!("{:width$}", hex_str, width = bytes_per_line * 3 - 1);

        // ASCII representation
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();

        lines.push(format!("{:08x}  {}  |{}|", addr, hex_padded, ascii));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex_dump_basic() {
        let bytes = b"Hello";
        let lines = format_hex_dump(bytes, 16);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("48 65 6c 6c 6f"));
        assert!(lines[0].contains("|Hello|"));
    }

    #[test]
    fn test_format_hex_dump_multi_line() {
        let bytes = b"0123456789ABCDEF0123";
        let lines = format_hex_dump(bytes, 16);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("00000000"));
        assert!(lines[1].starts_with("00000010"));
    }

    #[test]
    fn test_format_hex_dump_non_printable() {
        let bytes = &[0x00, 0x01, 0x41, 0x42]; // null, SOH, A, B
        let lines = format_hex_dump(bytes, 16);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("|..AB|"));
    }

    #[test]
    fn test_get_syntax_style_json() {
        let style = get_syntax_style("json");
        assert_eq!(style.fg, Some(THEME.primary));
    }

    #[test]
    fn test_get_syntax_style_yaml() {
        let style = get_syntax_style("yaml");
        assert_eq!(style.fg, Some(THEME.success));

        let style_yml = get_syntax_style("yml");
        assert_eq!(style_yml.fg, Some(THEME.success));
    }

    #[test]
    fn test_get_syntax_style_xml() {
        let style = get_syntax_style("xml");
        assert_eq!(style.fg, Some(THEME.warning));
    }

    #[test]
    fn test_get_syntax_style_unknown() {
        let style = get_syntax_style("xyz");
        assert_eq!(style.fg, Some(THEME.fg));
    }

    #[test]
    fn test_get_syntax_style_case_insensitive() {
        let style_upper = get_syntax_style("JSON");
        let style_lower = get_syntax_style("json");
        assert_eq!(style_upper.fg, style_lower.fg);
    }
}
