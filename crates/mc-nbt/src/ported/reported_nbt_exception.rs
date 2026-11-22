use std::fmt;

/// Minimal self-contained crash report representation.
// JAVA: net.minecraft.CrashReport
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReport {
    message: String,
}

impl CrashReport {
    // JAVA: net.minecraft.CrashReport#<init>
    pub fn new(message: String) -> Self {
        Self { message }
    }

    // JAVA: net.minecraft.CrashReport#getTitle
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Minimal self-contained reported exception base.
// JAVA: net.minecraft.ReportedException
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedException {
    report: CrashReport,
}

impl ReportedException {
    // JAVA: net.minecraft.ReportedException#<init>
    pub fn new(report: CrashReport) -> Self {
        Self { report }
    }

    // JAVA: net.minecraft.ReportedException#getReport
    pub fn report(&self) -> &CrashReport {
        &self.report
    }
}

impl fmt::Display for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReportedException: {}", self.report.message())
    }
}

impl std::error::Error for ReportedException {}

// JAVA: net.minecraft.nbt.ReportedNbtException
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedNbtException {
    inner: ReportedException,
}

impl ReportedNbtException {
    // JAVA: net.minecraft.nbt.ReportedNbtException#<init>
    pub fn new(report: CrashReport) -> Self {
        Self {
            inner: ReportedException::new(report),
        }
    }

    // JAVA: net.minecraft.nbt.ReportedNbtException#getReport
    pub fn report(&self) -> &CrashReport {
        self.inner.report()
    }

    // JAVA: net.minecraft.nbt.ReportedNbtException
    pub fn as_reported_exception(&self) -> &ReportedException {
        &self.inner
    }
}

impl fmt::Display for ReportedNbtException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReportedNbtException: {}", self.report().message())
    }
}

impl std::error::Error for ReportedNbtException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn new_stores_report() {
        let report = CrashReport::new("nbt parse failed".to_string());
        let ex = ReportedNbtException::new(report.clone());
        assert_eq!(ex.report(), &report);
        assert_eq!(ex.report().message(), "nbt parse failed");
    }

    #[test]
    fn display_and_source() {
        let ex = ReportedNbtException::new(CrashReport::new("bad tag".to_string()));
        assert_eq!(ex.to_string(), "ReportedNbtException: bad tag");
        let src = ex.source().expect("source");
        assert_eq!(src.to_string(), "ReportedException: bad tag");
    }

    #[test]
    fn as_reported_exception() {
        let ex = ReportedNbtException::new(CrashReport::new("x".to_string()));
        let base = ex.as_reported_exception();
        assert_eq!(base.report().message(), "x");
    }

    #[test]
    fn empty_and_negative_message_edge_cases() {
        let empty = ReportedNbtException::new(CrashReport::new(String::new()));
        assert_eq!(empty.report().message(), "");
        let neg = ReportedNbtException::new(CrashReport::new("-2147483648".to_string()));
        assert_eq!(neg.report().message(), "-2147483648");
    }
}
