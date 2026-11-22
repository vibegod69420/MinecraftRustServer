use std::fmt;

/// Collected exception payload preserving Java's primary + suppressed behavior.
#[derive(Debug)]
pub struct CollectedException<T> {
    pub primary: T,
    pub suppressed: Vec<T>,
}

impl<T: fmt::Display> fmt::Display for CollectedException<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.primary)?;
        if !self.suppressed.is_empty() {
            write!(f, " ({} suppressed)", self.suppressed.len())?;
        }
        Ok(())
    }
}

impl<T: fmt::Debug + fmt::Display> std::error::Error for CollectedException<T> {}

/// Faithful port of `net.minecraft.util.ExceptionCollector`.
#[derive(Debug, Default)]
pub struct ExceptionCollector<T> {
    result: Option<T>,
    suppressed: Vec<T>,
}

impl<T> ExceptionCollector<T> {
    pub fn new() -> Self {
        Self {
            result: None,
            suppressed: Vec::new(),
        }
    }

    // JAVA: net.minecraft.util.ExceptionCollector#add
    pub fn add(&mut self, err: T) {
        if self.result.is_none() {
            self.result = Some(err);
        } else {
            self.suppressed.push(err);
        }
    }

    // JAVA: net.minecraft.util.ExceptionCollector#throwIfPresent
    pub fn throw_if_present(self) -> Result<(), CollectedException<T>> {
        match self.result {
            Some(primary) => Err(CollectedException {
                primary,
                suppressed: self.suppressed,
            }),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_error_is_ok() {
        let collector: ExceptionCollector<String> = ExceptionCollector::new();
        assert!(collector.throw_if_present().is_ok());
    }

    #[test]
    fn single_error_becomes_primary() {
        let mut collector = ExceptionCollector::new();
        collector.add("first".to_string());
        let err = collector.throw_if_present().unwrap_err();
        assert_eq!(err.primary, "first");
        assert!(err.suppressed.is_empty());
    }

    #[test]
    fn subsequent_errors_are_suppressed_in_order() {
        let mut collector = ExceptionCollector::new();
        collector.add(1i32);
        collector.add(2i32);
        collector.add(3i32);
        let err = collector.throw_if_present().unwrap_err();
        assert_eq!(err.primary, 1);
        assert_eq!(err.suppressed, vec![2, 3]);
    }

    #[test]
    fn display_includes_suppressed_count() {
        let mut collector = ExceptionCollector::new();
        collector.add("boom");
        collector.add("s1");
        collector.add("s2");
        let err = collector.throw_if_present().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("boom"));
        assert!(msg.contains("2 suppressed"));
    }

    #[test]
    fn default_is_empty() {
        let collector: ExceptionCollector<&str> = ExceptionCollector::default();
        assert!(collector.throw_if_present().is_ok());
    }
}
