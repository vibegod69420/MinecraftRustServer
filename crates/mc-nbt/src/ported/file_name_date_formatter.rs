// JAVA: net.minecraft.world.level.storage.FileNameDateFormatter

/// Port of Minecraft's FileNameDateFormatter.
/// Formats date/time components into a file-name-safe string: "yyyy-MM-dd_HH-mm-ss".
pub struct FileNameDateFormatter;

// JAVA: net.minecraft.world.level.storage.FileNameDateFormatter#FORMATTER
pub static FORMATTER: &str = "yyyy-MM-dd_HH-mm-ss";

impl FileNameDateFormatter {
    // JAVA: net.minecraft.world.level.storage.FileNameDateFormatter#format
    pub fn format(year: u32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> String {
        format!(
            "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
            year, month, day, hour, minute, second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_basic() {
        let result = FileNameDateFormatter::format(2023, 10, 25, 14, 30, 59);
        assert_eq!(result, "2023-10-25_14-30-59");
    }

    #[test]
    fn test_format_padding() {
        let result = FileNameDateFormatter::format(999, 1, 2, 3, 4, 5);
        assert_eq!(result, "0999-01-02_03-04-05");
    }

    #[test]
    fn test_format_max_values() {
        let result = FileNameDateFormatter::format(9999, 12, 31, 23, 59, 59);
        assert_eq!(result, "9999-12-31_23-59-59");
    }

    #[test]
    fn test_format_min_values() {
        let result = FileNameDateFormatter::format(0, 1, 1, 0, 0, 0);
        assert_eq!(result, "0000-01-01_00-00-00");
    }

    #[test]
    fn test_format_const_pattern() {
        assert_eq!(FORMATTER, "yyyy-MM-dd_HH-mm-ss");
    }
}
