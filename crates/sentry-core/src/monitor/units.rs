//! Human-readable formatting for byte counts, throughput, and percentages.
//!
//! Every table-facing struct in [`crate::monitor`] pairs a raw numeric field with a
//! `_display` string built from these helpers, so a UI can sort/filter on the number
//! while rendering the formatted string directly.

const KIB: f64 = 1024.0;
const MIB: f64 = KIB * 1024.0;
const GIB: f64 = MIB * 1024.0;
const TIB: f64 = GIB * 1024.0;

/// Formats a byte count as a human-readable string, e.g. `"12.40 MB"`.
pub fn format_bytes(bytes: u64) -> String {
    let bytes = bytes as f64;
    if bytes >= TIB {
        format!("{:.2} TB", bytes / TIB)
    } else if bytes >= GIB {
        format!("{:.2} GB", bytes / GIB)
    } else if bytes >= MIB {
        format!("{:.2} MB", bytes / MIB)
    } else if bytes >= KIB {
        format!("{:.2} KB", bytes / KIB)
    } else {
        format!("{bytes:.0} B")
    }
}

/// Formats a bytes-per-second rate as a human-readable throughput string, e.g. `"3.10 MB/s"`.
pub fn format_bytes_per_sec(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

/// Formats a percentage to one decimal place with a trailing `%`, e.g. `"12.3%"`.
pub fn format_percent(value: f32) -> String {
    format!("{value:.1}%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_picks_the_right_unit() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(MIB as u64), "1.00 MB");
        assert_eq!(format_bytes(GIB as u64), "1.00 GB");
        assert_eq!(format_bytes(TIB as u64), "1.00 TB");
    }

    #[test]
    fn format_bytes_per_sec_appends_rate_suffix() {
        assert_eq!(format_bytes_per_sec(1024), "1.00 KB/s");
    }

    #[test]
    fn format_percent_rounds_to_one_decimal() {
        assert_eq!(format_percent(12.345), "12.3%");
        assert_eq!(format_percent(0.0), "0.0%");
    }
}
