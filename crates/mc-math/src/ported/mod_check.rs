use std::cmp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    // JAVA: net.minecraft.util.ModCheck$Confidence#PROBABLY_NOT
    ProbablyNot,
    // JAVA: net.minecraft.util.ModCheck$Confidence#VERY_LIKELY
    VeryLikely,
    // JAVA: net.minecraft.util.ModCheck$Confidence#DEFINITELY
    Definitely,
}

impl Confidence {
    // JAVA: net.minecraft.util.ModCheck$Confidence#description
    pub fn description(self) -> &'static str {
        match self {
            Self::ProbablyNot => "Probably not.",
            Self::VeryLikely => "Very likely;",
            Self::Definitely => "Definitely;",
        }
    }

    // JAVA: net.minecraft.util.ModCheck$Confidence#shouldReportAsModified
    pub fn should_report_as_modified(self) -> bool {
        match self {
            Self::ProbablyNot => false,
            Self::VeryLikely => true,
            Self::Definitely => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModCheck {
    pub confidence: Confidence,
    pub description: String,
}

impl ModCheck {
    // JAVA: net.minecraft.util.ModCheck#<init>
    pub fn new(confidence: Confidence, description: String) -> Self {
        Self {
            confidence,
            description,
        }
    }

    // JAVA: net.minecraft.util.ModCheck#identify
    pub fn identify(
        brand: &str,
        get_brand: impl FnOnce() -> String,
        name: &str,
        signers_null: bool,
    ) -> Self {
        let actual_brand = get_brand();
        if brand != actual_brand {
            Self::new(
                Confidence::Definitely,
                format!("{} brand changed to '{}'", name, actual_brand),
            )
        } else if signers_null {
            Self::new(
                Confidence::VeryLikely,
                format!("{} jar signature invalidated", name),
            )
        } else {
            Self::new(
                Confidence::ProbablyNot,
                format!("{} jar signature and brand is untouched", name),
            )
        }
    }

    // JAVA: net.minecraft.util.ModCheck#shouldReportAsModified
    pub fn should_report_as_modified(&self) -> bool {
        self.confidence.should_report_as_modified()
    }

    // JAVA: net.minecraft.util.ModCheck#merge
    pub fn merge(&self, other: &ModCheck) -> Self {
        Self::new(
            cmp::max(self.confidence, other.confidence),
            format!("{}; {}", self.description, other.description),
        )
    }

    // JAVA: net.minecraft.util.ModCheck#fullDescription
    pub fn full_description(&self) -> String {
        format!("{} {}", self.confidence.description(), self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identify_brand_changed() {
        let check = ModCheck::identify("vanilla", || "forge".to_string(), "Client", false);
        assert_eq!(check.confidence, Confidence::Definitely);
        assert!(check.should_report_as_modified());
        assert_eq!(check.description, "Client brand changed to 'forge'");
    }

    #[test]
    fn identify_signers_invalidated() {
        let check = ModCheck::identify("vanilla", || "vanilla".to_string(), "Server", true);
        assert_eq!(check.confidence, Confidence::VeryLikely);
        assert!(check.should_report_as_modified());
        assert_eq!(check.description, "Server jar signature invalidated");
    }

    #[test]
    fn identify_untouched() {
        let check = ModCheck::identify("vanilla", || "vanilla".to_string(), "Client", false);
        assert_eq!(check.confidence, Confidence::ProbablyNot);
        assert!(!check.should_report_as_modified());
        assert_eq!(
            check.full_description(),
            "Probably not. Client jar signature and brand is untouched"
        );
    }

    #[test]
    fn merge_takes_max_confidence_and_joins_descriptions() {
        let a = ModCheck::new(Confidence::ProbablyNot, "a".to_string());
        let b = ModCheck::new(Confidence::Definitely, "b".to_string());
        let merged = a.merge(&b);
        assert_eq!(merged.confidence, Confidence::Definitely);
        assert_eq!(merged.description, "a; b");
        assert_eq!(merged.full_description(), "Definitely; a; b");
    }

    #[test]
    fn confidence_ordering_matches_java_ordinal() {
        assert!(Confidence::ProbablyNot < Confidence::VeryLikely);
        assert!(Confidence::VeryLikely < Confidence::Definitely);
        assert_eq!(
            cmp::max(Confidence::VeryLikely, Confidence::ProbablyNot),
            Confidence::VeryLikely
        );
    }
}
