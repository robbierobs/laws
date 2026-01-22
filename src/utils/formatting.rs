/// Format a byte size as a human-readable string.
pub fn format_bytes(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// Format an optional byte size, returning "-" when unavailable.
pub fn format_bytes_opt(size: Option<i64>) -> String {
    size.and_then(|value| u64::try_from(value).ok())
        .map(format_bytes)
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, format_bytes_opt};

    #[test]
    fn test_format_bytes_returns_bytes_for_small_values() {
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn test_format_bytes_returns_kilobytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
    }

    #[test]
    fn test_format_bytes_returns_megabytes() {
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
    }

    #[test]
    fn test_format_bytes_returns_gigabytes() {
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_format_bytes_opt_handles_missing_values() {
        assert_eq!(format_bytes_opt(None), "-");
    }

    #[test]
    fn test_format_bytes_opt_formats_present_values() {
        assert_eq!(format_bytes_opt(Some(2048)), "2.00 KB");
    }

    #[test]
    fn test_format_bytes_opt_handles_negative_values() {
        assert_eq!(format_bytes_opt(Some(-1)), "-");
    }
}
