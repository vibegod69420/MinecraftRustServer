//! Faithful port of `net.minecraft.util.CommonLinks`.

/// Minimal UUID representation for self-contained formatting.
/// Equivalent to Java `UUID` most/least significant bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Uuid {
    pub most_significant_bits: i64,
    pub least_significant_bits: i64,
}

impl Uuid {
    /// Formats UUID as 32 lowercase hex chars with no dashes,
    /// matching `com.mojang.util.UndashedUuid#toString`.
    pub fn to_undashed_string(&self) -> String {
        format!(
            "{:016x}{:016x}",
            self.most_significant_bits as u64,
            self.least_significant_bits as u64
        )
    }
}

pub const GDPR: &str = "https://aka.ms/MinecraftGDPR";
pub const EULA: &str = "https://aka.ms/MinecraftEULA";
pub const PRIVACY_STATEMENT: &str = "http://go.microsoft.com/fwlink/?LinkId=521839";
pub const ATTRIBUTION: &str = "https://aka.ms/MinecraftJavaAttribution";
pub const LICENSES: &str = "https://aka.ms/MinecraftJavaLicenses";
pub const BUY_MINECRAFT_JAVA: &str = "https://aka.ms/BuyMinecraftJava";
pub const ACCOUNT_SETTINGS: &str = "https://aka.ms/JavaAccountSettings";
pub const SNAPSHOT_FEEDBACK: &str = "https://aka.ms/snapshotfeedback?ref=game";
pub const RELEASE_FEEDBACK: &str = "https://aka.ms/javafeedback?ref=game";
pub const SNAPSHOT_BUGS_FEEDBACK: &str = "https://aka.ms/snapshotbugs?ref=game";
pub const GENERAL_HELP: &str = "https://aka.ms/Minecraft-Support";
pub const ACCESSIBILITY_HELP: &str = "https://aka.ms/MinecraftJavaAccessibility";
pub const REPORTING_HELP: &str = "https://aka.ms/aboutjavareporting";
pub const SUSPENSION_HELP: &str = "https://aka.ms/mcjavamoderation";
pub const BLOCKING_HELP: &str = "https://aka.ms/javablocking";
pub const SYMLINK_HELP: &str = "https://aka.ms/MinecraftSymLinks";
pub const START_REALMS_TRIAL: &str = "https://aka.ms/startjavarealmstrial";
pub const BUY_REALMS: &str = "https://aka.ms/BuyJavaRealms";
pub const REALMS_TERMS: &str = "https://aka.ms/MinecraftRealmsTerms";
pub const REALMS_CONTENT_CREATION: &str = "https://aka.ms/MinecraftRealmsContentCreator";
pub const EXTEND_REALMS_LINK: &str = "https://aka.ms/ExtendJavaRealms";
pub const INTENTIONAL_GAME_DESIGN_BUG_ID: &str = "MCPE-28723";
pub const INTENTIONAL_GAME_DESIGN_BUG: &str = "https://bugs.mojang.com/browse/MCPE-28723";

// JAVA: net.minecraft.util.CommonLinks#extendRealms
pub fn extend_realms(subscription_id: Option<&str>, profile_id: Uuid, trial: bool) -> String {
    match subscription_id {
        None => EXTEND_REALMS_LINK.to_owned(),
        Some(id) => {
            let base = extend_realms_with_profile(Some(id), profile_id);
            let ref_value = if trial { "expiredTrial" } else { "expiredRealm" };
            format!("{base}&ref={ref_value}")
        }
    }
}

// JAVA: net.minecraft.util.CommonLinks#extendRealms
pub fn extend_realms_with_profile(subscription_id: Option<&str>, profile_id: Uuid) -> String {
    match subscription_id {
        None => EXTEND_REALMS_LINK.to_owned(),
        Some(id) => format!(
            "https://aka.ms/ExtendJavaRealms?subscriptionId={}&profileId={}",
            id,
            profile_id.to_undashed_string()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_uuid() -> Uuid {
        Uuid {
            most_significant_bits: 0x0123_4567_89ab_cdef_u64 as i64,
            least_significant_bits: 0xfedc_ba98_7654_3210_u64 as i64,
        }
    }

    #[test]
    fn constants_match_java_values() {
        assert_eq!(GDPR, "https://aka.ms/MinecraftGDPR");
        assert_eq!(EULA, "https://aka.ms/MinecraftEULA");
        assert_eq!(EXTEND_REALMS_LINK, "https://aka.ms/ExtendJavaRealms");
        assert_eq!(INTENTIONAL_GAME_DESIGN_BUG_ID, "MCPE-28723");
        assert_eq!(
            INTENTIONAL_GAME_DESIGN_BUG,
            "https://bugs.mojang.com/browse/MCPE-28723"
        );
    }

    #[test]
    fn undashed_uuid_formatting() {
        let uuid = sample_uuid();
        assert_eq!(
            uuid.to_undashed_string(),
            "0123456789abcdeffedcba9876543210"
        );
        let neg = Uuid {
            most_significant_bits: -1,
            least_significant_bits: -2,
        };
        assert_eq!(neg.to_undashed_string(), "fffffffffffffffffffffffffffffffe");
    }

    #[test]
    fn extend_realms_without_subscription_returns_base_link() {
        let uuid = sample_uuid();
        assert_eq!(extend_realms(None, uuid, true), EXTEND_REALMS_LINK);
        assert_eq!(
            extend_realms_with_profile(None, uuid),
            EXTEND_REALMS_LINK
        );
    }

    #[test]
    fn extend_realms_builds_expected_urls() {
        let uuid = sample_uuid();
        let base = extend_realms_with_profile(Some("sub-1"), uuid);
        assert_eq!(
            base,
            "https://aka.ms/ExtendJavaRealms?subscriptionId=sub-1&profileId=0123456789abcdeffedcba9876543210"
        );
        assert_eq!(
            extend_realms(Some("sub-1"), uuid, true),
            format!("{base}&ref=expiredTrial")
        );
        assert_eq!(
            extend_realms(Some("sub-1"), uuid, false),
            format!("{base}&ref=expiredRealm")
        );
    }
}
