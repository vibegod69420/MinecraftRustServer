//! Port of `net.minecraft.network.DisconnectionDetails`.

use std::path::PathBuf;

/// Minimal self-contained stub for `net.minecraft.network.chat.Component`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// Minimal self-contained URI wrapper (Java `java.net.URI`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri(pub String);

/// Port of `net.minecraft.network.DisconnectionDetails`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisconnectionDetails {
    pub reason: Component,
    pub report: Option<PathBuf>,
    pub bug_report_link: Option<Uri>,
}

impl DisconnectionDetails {
    // JAVA: net.minecraft.network.DisconnectionDetails#<init>
    pub fn new(reason: Component, report: Option<PathBuf>, bug_report_link: Option<Uri>) -> Self {
        Self {
            reason,
            report,
            bug_report_link,
        }
    }

    // JAVA: net.minecraft.network.DisconnectionDetails#<init>
    pub fn from_reason(reason: Component) -> Self {
        Self {
            reason,
            report: None,
            bug_report_link: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_reason_sets_empty_optionals() {
        let details = DisconnectionDetails::from_reason(Component::literal("disconnect"));
        assert_eq!(details.reason, Component::literal("disconnect"));
        assert_eq!(details.report, None);
        assert_eq!(details.bug_report_link, None);
    }

    #[test]
    fn full_constructor_preserves_all_fields() {
        let report = PathBuf::from("/tmp/report.txt");
        let link = Uri("https://example.com/bug".to_string());
        let details = DisconnectionDetails::new(
            Component::literal("error"),
            Some(report.clone()),
            Some(link.clone()),
        );
        assert_eq!(details.reason.text, "error");
        assert_eq!(details.report, Some(report));
        assert_eq!(details.bug_report_link, Some(link));
    }

    #[test]
    fn handles_empty_and_negative_like_text_edge_cases() {
        let empty = DisconnectionDetails::from_reason(Component::literal(""));
        assert_eq!(empty.reason.text, "");
        let negative_text = DisconnectionDetails::from_reason(Component::literal("-2147483648"));
        assert_eq!(negative_text.reason.text, "-2147483648");
        assert!(negative_text.report.is_none());
    }

    #[test]
    fn clone_and_equality() {
        let details = DisconnectionDetails::new(
            Component::literal("reason"),
            Some(PathBuf::from("a/b")),
            None,
        );
        let cloned = details.clone();
        assert_eq!(details, cloned);
        assert_ne!(
            details,
            DisconnectionDetails::from_reason(Component::literal("other"))
        );
    }

    #[test]
    fn optional_fields_can_be_mixed() {
        let with_report = DisconnectionDetails::new(
            Component::literal("r"),
            Some(PathBuf::from("report")),
            None,
        );
        let with_link = DisconnectionDetails::new(
            Component::literal("r"),
            None,
            Some(Uri("uri".to_string())),
        );
        assert!(with_report.report.is_some());
        assert!(with_report.bug_report_link.is_none());
        assert!(with_link.report.is_none());
        assert!(with_link.bug_report_link.is_some());
    }
}
