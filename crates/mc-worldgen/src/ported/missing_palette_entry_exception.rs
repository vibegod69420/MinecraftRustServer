use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingPaletteEntryException {
    index: i32,
    message: String,
}

impl MissingPaletteEntryException {
    // JAVA: net.minecraft.world.level.chunk.MissingPaletteEntryException#<init>
    pub fn new(index: i32) -> Self {
        Self {
            index,
            message: format!("Missing Palette entry for index {}.", index),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

impl fmt::Display for MissingPaletteEntryException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for MissingPaletteEntryException {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn message_format() {
        let e = MissingPaletteEntryException::new(5);
        assert_eq!(e.to_string(), "Missing Palette entry for index 5.");
        assert_eq!(e.index(), 5);
    }

    #[test]
    fn negative_index() {
        let e = MissingPaletteEntryException::new(-1);
        assert_eq!(e.to_string(), "Missing Palette entry for index -1.");
        assert_eq!(e.index(), -1);
    }

    #[test]
    fn extreme_indices() {
        let min = MissingPaletteEntryException::new(i32::MIN);
        assert_eq!(
            min.to_string(),
            format!("Missing Palette entry for index {}.", i32::MIN)
        );
        let max = MissingPaletteEntryException::new(i32::MAX);
        assert_eq!(
            max.to_string(),
            format!("Missing Palette entry for index {}.", i32::MAX)
        );
    }

    #[test]
    fn implements_error_trait() {
        let e = MissingPaletteEntryException::new(0);
        let err: &dyn Error = &e;
        assert_eq!(err.to_string(), "Missing Palette entry for index 0.");
        assert!(err.source().is_none());
    }
}
