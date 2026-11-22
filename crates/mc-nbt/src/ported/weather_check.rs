/// Self-contained faithful port of
/// `net.minecraft.world.level.storage.loot.predicates.WeatherCheck` (Java 1.21.11).
///
/// External loot/context/codec infrastructure is modeled with minimal local traits
/// and structs so this module compiles independently.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemConditionType {
    WeatherCheck,
}

/// Minimal context surface required by `WeatherCheck`.
pub trait WeatherCheckContext {
    fn level_is_raining(&self) -> bool;
    fn level_is_thundering(&self) -> bool;
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WeatherCheck {
    is_raining: Option<bool>,
    is_thundering: Option<bool>,
}

impl WeatherCheck {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck#<init>
    pub fn new(is_raining: Option<bool>, is_thundering: Option<bool>) -> Self {
        Self {
            is_raining,
            is_thundering,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck#getType
    pub fn get_type(&self) -> LootItemConditionType {
        LootItemConditionType::WeatherCheck
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck#test
    pub fn test<C: WeatherCheckContext>(&self, context: &C) -> bool {
        let is_raining = context.level_is_raining();
        if let Some(expected) = self.is_raining {
            if expected != is_raining {
                return false;
            }
        }

        let is_thundering = context.level_is_thundering();
        if let Some(expected) = self.is_thundering {
            if expected != is_thundering {
                return false;
            }
        }

        true
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck#weather
    pub fn weather() -> Builder {
        Builder::default()
    }

    pub fn is_raining(&self) -> Option<bool> {
        self.is_raining
    }

    pub fn is_thundering(&self) -> Option<bool> {
        self.is_thundering
    }
}

// JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck$Builder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Builder {
    is_raining: Option<bool>,
    is_thundering: Option<bool>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck$Builder#setRaining
    pub fn set_raining(mut self, value: bool) -> Self {
        self.is_raining = Some(value);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck$Builder#setThundering
    pub fn set_thundering(mut self, value: bool) -> Self {
        self.is_thundering = Some(value);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.predicates.WeatherCheck$Builder#build
    pub fn build(self) -> WeatherCheck {
        WeatherCheck::new(self.is_raining, self.is_thundering)
    }
}

#[cfg(test)]
mod tests {
    use super::{LootItemConditionType, WeatherCheck, WeatherCheckContext};

    #[derive(Debug, Clone, Copy)]
    struct TestContext {
        raining: bool,
        thundering: bool,
    }

    impl WeatherCheckContext for TestContext {
        fn level_is_raining(&self) -> bool {
            self.raining
        }

        fn level_is_thundering(&self) -> bool {
            self.thundering
        }
    }

    #[test]
    fn no_conditions_always_pass() {
        let cond = WeatherCheck::weather().build();
        assert_eq!(cond.get_type(), LootItemConditionType::WeatherCheck);
        assert!(cond.test(&TestContext {
            raining: false,
            thundering: false
        }));
        assert!(cond.test(&TestContext {
            raining: true,
            thundering: true
        }));
    }

    #[test]
    fn raining_check_matches_java_logic() {
        let cond = WeatherCheck::weather().set_raining(true).build();
        assert!(cond.test(&TestContext {
            raining: true,
            thundering: false
        }));
        assert!(!cond.test(&TestContext {
            raining: false,
            thundering: false
        }));
    }

    #[test]
    fn thundering_check_matches_java_logic() {
        let cond = WeatherCheck::weather().set_thundering(false).build();
        assert!(cond.test(&TestContext {
            raining: true,
            thundering: false
        }));
        assert!(!cond.test(&TestContext {
            raining: true,
            thundering: true
        }));
    }

    #[test]
    fn both_checks_require_both_to_match() {
        let cond = WeatherCheck::weather()
            .set_raining(true)
            .set_thundering(false)
            .build();
        assert!(cond.test(&TestContext {
            raining: true,
            thundering: false
        }));
        assert!(!cond.test(&TestContext {
            raining: false,
            thundering: false
        }));
        assert!(!cond.test(&TestContext {
            raining: true,
            thundering: true
        }));
    }

    #[test]
    fn builder_and_accessors_preserve_optionality() {
        let cond = WeatherCheck::new(Some(false), None);
        assert_eq!(cond.is_raining(), Some(false));
        assert_eq!(cond.is_thundering(), None);
        let built = WeatherCheck::weather().set_thundering(true).build();
        assert_eq!(built.is_raining(), None);
        assert_eq!(built.is_thundering(), Some(true));
    }
}
