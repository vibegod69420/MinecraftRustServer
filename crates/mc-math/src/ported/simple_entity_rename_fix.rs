#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dynamic {
    pub value: String,
}

impl Dynamic {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemainderFinder;

pub struct Dsl;

impl Dsl {
    pub fn remainder_finder() -> RemainderFinder {
        RemainderFinder
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typed {
    remainder: Dynamic,
}

impl Typed {
    pub fn new(remainder: Dynamic) -> Self {
        Self { remainder }
    }

    pub fn get_or_create(&self, _finder: RemainderFinder) -> Dynamic {
        self.remainder.clone()
    }

    pub fn set(mut self, _finder: RemainderFinder, value: Dynamic) -> Self {
        self.remainder = value;
        self
    }

    pub fn remainder(&self) -> &Dynamic {
        &self.remainder
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair<A, B> {
    first: A,
    second: B,
}

impl<A, B> Pair<A, B> {
    pub fn of(first: A, second: B) -> Self {
        Self { first, second }
    }

    pub fn get_first(&self) -> &A {
        &self.first
    }

    pub fn get_second(&self) -> &B {
        &self.second
    }

    pub fn into_parts(self) -> (A, B) {
        (self.first, self.second)
    }
}

pub struct SimpleEntityRenameFixBase {
    pub name: String,
    pub schema: Schema,
    pub changes_type: bool,
}

impl SimpleEntityRenameFixBase {
    // JAVA: net.minecraft.util.datafix.fixes.SimpleEntityRenameFix#SimpleEntityRenameFix
    pub fn new(name: String, schema: Schema, changes_type: bool) -> Self {
        Self {
            name,
            schema,
            changes_type,
        }
    }
}

pub trait SimpleEntityRenameFix {
    fn base(&self) -> &SimpleEntityRenameFixBase;

    // JAVA: net.minecraft.util.datafix.fixes.SimpleEntityRenameFix#fix
    fn fix(&self, name: &str, typed: Typed) -> Pair<String, Typed> {
        let pair = self.get_new_name_and_tag(name, typed.get_or_create(Dsl::remainder_finder()));
        let (new_name, new_tag) = pair.into_parts();
        Pair::of(new_name, typed.set(Dsl::remainder_finder(), new_tag))
    }

    // JAVA: net.minecraft.util.datafix.fixes.SimpleEntityRenameFix#getNewNameAndTag
    fn get_new_name_and_tag(&self, name: &str, tag: Dynamic) -> Pair<String, Dynamic>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RenameCow {
        base: SimpleEntityRenameFixBase,
    }

    impl RenameCow {
        fn new() -> Self {
            Self {
                base: SimpleEntityRenameFixBase::new(
                    "RenameCow".to_string(),
                    Schema,
                    true,
                ),
            }
        }
    }

    impl SimpleEntityRenameFix for RenameCow {
        fn base(&self) -> &SimpleEntityRenameFixBase {
            &self.base
        }

        fn get_new_name_and_tag(&self, name: &str, tag: Dynamic) -> Pair<String, Dynamic> {
            if name == "minecraft:cow" {
                Pair::of("minecraft:moo".to_string(), tag)
            } else {
                Pair::of(name.to_string(), tag)
            }
        }
    }

    struct RenameAndRetag {
        base: SimpleEntityRenameFixBase,
    }

    impl RenameAndRetag {
        fn new() -> Self {
            Self {
                base: SimpleEntityRenameFixBase::new(
                    "RenameAndRetag".to_string(),
                    Schema,
                    false,
                ),
            }
        }
    }

    impl SimpleEntityRenameFix for RenameAndRetag {
        fn base(&self) -> &SimpleEntityRenameFixBase {
            &self.base
        }

        fn get_new_name_and_tag(&self, name: &str, _tag: Dynamic) -> Pair<String, Dynamic> {
            Pair::of(format!("{name}_new"), Dynamic::new("replaced_tag"))
        }
    }

    #[test]
    fn renames_matching_entity_and_preserves_tag() {
        let fix = RenameCow::new();
        let typed = Typed::new(Dynamic::new("tag-data"));
        let result = fix.fix("minecraft:cow", typed);
        assert_eq!(result.get_first(), "minecraft:moo");
        assert_eq!(result.get_second().remainder(), &Dynamic::new("tag-data"));
    }

    #[test]
    fn leaves_non_matching_entity_name_unchanged() {
        let fix = RenameCow::new();
        let typed = Typed::new(Dynamic::new("pig-tag"));
        let result = fix.fix("minecraft:pig", typed);
        assert_eq!(result.get_first(), "minecraft:pig");
        assert_eq!(result.get_second().remainder().value, "pig-tag");
    }

    #[test]
    fn replaces_tag_when_implementation_changes_it() {
        let fix = RenameAndRetag::new();
        let typed = Typed::new(Dynamic::new("old"));
        let result = fix.fix("old_name", typed);
        assert_eq!(result.get_first(), "old_name_new");
        assert_eq!(result.get_second().remainder().value, "replaced_tag");
    }

    #[test]
    fn handles_empty_and_numeric_like_names() {
        let fix = RenameAndRetag::new();
        let typed = Typed::new(Dynamic::new("-1.5e308"));
        let result = fix.fix("", typed);
        assert_eq!(result.get_first(), "_new");
        assert_eq!(result.get_second().remainder().value, "replaced_tag");
    }

    #[test]
    fn constructor_stores_base_fields() {
        let base = SimpleEntityRenameFixBase::new("TestFix".to_string(), Schema, true);
        assert_eq!(base.name, "TestFix");
        assert!(base.changes_type);
    }
}
