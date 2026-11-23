/// Port of `net.minecraft.util.datafix.fixes.ItemRenameFix`.
///
/// In Java this extends DFU `DataFix` and rewrites named item IDs through a
/// type rewrite rule. This self-contained port preserves the renaming behavior
/// via a trait/object abstraction and a factory helper.
pub trait ItemRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemRenameFix#fixItem
    fn fix_item(&self, item: &str) -> String;

    // JAVA: net.minecraft.util.datafix.fixes.ItemRenameFix#makeRule
    fn apply(&self, item: &str) -> String {
        self.fix_item(item)
    }
}

/// Concrete helper mirroring `ItemRenameFix.create(Schema, String, Function<String,String>)`.
#[derive(Clone)]
pub struct FnItemRenameFix<F>
where
    F: Fn(&str) -> String,
{
    name: String,
    fixer: F,
}

impl<F> FnItemRenameFix<F>
where
    F: Fn(&str) -> String,
{
    // JAVA: net.minecraft.util.datafix.fixes.ItemRenameFix#create
    pub fn create(_schema: (), name: &str, fixer: F) -> Self {
        Self {
            name: name.to_owned(),
            fixer,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<F> ItemRenameFix for FnItemRenameFix<F>
where
    F: Fn(&str) -> String,
{
    // JAVA: net.minecraft.util.datafix.fixes.ItemRenameFix#fixItem
    fn fix_item(&self, item: &str) -> String {
        (self.fixer)(item)
    }
}

/// A simple struct form closer to the Java abstract class shape.
#[derive(Clone, Debug)]
pub struct NamedItemRenameFix {
    pub name: String,
}

impl NamedItemRenameFix {
    // JAVA: net.minecraft.util.datafix.fixes.ItemRenameFix#<init>
    pub fn new(_schema: (), name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.ItemRenameFix#create
pub fn create_item_rename_fix<F>(_schema: (), name: &str, fixer: F) -> impl ItemRenameFix
where
    F: Fn(&str) -> String,
{
    FnItemRenameFix::create((), name, fixer)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PrefixFix;
    impl ItemRenameFix for PrefixFix {
        fn fix_item(&self, item: &str) -> String {
            if item == "minecraft:stone" {
                "minecraft:rock".to_owned()
            } else {
                item.to_owned()
            }
        }
    }

    #[test]
    fn trait_impl_renames_matching_item() {
        let fix = PrefixFix;
        assert_eq!(fix.apply("minecraft:stone"), "minecraft:rock");
        assert_eq!(fix.apply("minecraft:dirt"), "minecraft:dirt");
    }

    #[test]
    fn create_factory_applies_function() {
        let fix = FnItemRenameFix::create((), "TestRename", |s: &str| {
            if s == "minecraft:old" {
                "minecraft:new".to_owned()
            } else {
                s.to_owned()
            }
        });
        assert_eq!(fix.name(), "TestRename");
        assert_eq!(fix.fix_item("minecraft:old"), "minecraft:new");
        assert_eq!(fix.apply("minecraft:other"), "minecraft:other");
    }

    #[test]
    fn free_create_helper_works() {
        let fix = create_item_rename_fix((), "HelperRename", |s: &str| s.replace("oak", "spruce"));
        assert_eq!(fix.apply("minecraft:oak_planks"), "minecraft:spruce_planks");
        assert_eq!(fix.apply("minecraft:stone"), "minecraft:stone");
    }

    #[test]
    fn named_struct_constructor_preserves_name() {
        let fix = NamedItemRenameFix::new((), "MyFix");
        assert_eq!(fix.name, "MyFix");
    }

    #[test]
    fn empty_and_namespaced_strings_handled() {
        let fix = create_item_rename_fix((), "Edge", |s: &str| {
            if s.is_empty() {
                "minecraft:air".to_owned()
            } else {
                s.to_owned()
            }
        });
        assert_eq!(fix.apply(""), "minecraft:air");
        assert_eq!(fix.apply("custom:thing"), "custom:thing");
    }
}
