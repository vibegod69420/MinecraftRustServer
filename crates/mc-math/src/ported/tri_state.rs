#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriState {
    True,
    False,
    Default,
}

impl TriState {
    // JAVA: net.minecraft.util.TriState#from
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }

    // JAVA: net.minecraft.util.TriState#toBoolean
    pub fn to_boolean(self, default: bool) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            Self::Default => default,
        }
    }

    // JAVA: net.minecraft.util.TriState#getSerializedName
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Default => "default",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bool_maps_to_true_or_false() {
        assert_eq!(TriState::from_bool(true), TriState::True);
        assert_eq!(TriState::from_bool(false), TriState::False);
    }

    #[test]
    fn to_boolean_uses_default_only_for_default_variant() {
        assert!(TriState::True.to_boolean(false));
        assert!(!TriState::False.to_boolean(true));
        assert!(TriState::Default.to_boolean(true));
        assert!(!TriState::Default.to_boolean(false));
    }

    #[test]
    fn serialized_names_match_java() {
        assert_eq!(TriState::True.serialized_name(), "true");
        assert_eq!(TriState::False.serialized_name(), "false");
        assert_eq!(TriState::Default.serialized_name(), "default");
    }

    #[test]
    fn enum_variants_are_distinct() {
        assert_ne!(TriState::True, TriState::False);
        assert_ne!(TriState::True, TriState::Default);
        assert_ne!(TriState::False, TriState::Default);
    }
}
