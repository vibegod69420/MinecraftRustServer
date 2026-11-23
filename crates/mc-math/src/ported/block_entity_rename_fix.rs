pub struct BlockEntityRenameFix<F>
where
    F: Fn(&str) -> String,
{
    name: String,
    name_change_lookup: F,
}

impl<F> BlockEntityRenameFix<F>
where
    F: Fn(&str) -> String,
{
    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityRenameFix#BlockEntityRenameFix
    fn new(_output_schema: (), name: &str, name_change_lookup: F) -> Self {
        Self {
            name: name.to_string(),
            name_change_lookup,
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityRenameFix#makeRule
    pub fn make_rule(&self) -> impl Fn(&str) -> String + '_ {
        move |id| (self.name_change_lookup)(id)
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockEntityRenameFix#create
    pub fn create(output_schema: (), name: &str, name_change_lookup: F) -> Self {
        Self::new(output_schema, name, name_change_lookup)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_matching_id() {
        let fix = BlockEntityRenameFix::create((), "rename_fix", |id: &str| {
            if id == "minecraft:old_sign" {
                "minecraft:sign".to_string()
            } else {
                id.to_string()
            }
        });
        let rule = fix.make_rule();
        assert_eq!(rule("minecraft:old_sign"), "minecraft:sign");
        assert_eq!(rule("minecraft:chest"), "minecraft:chest");
    }

    #[test]
    fn create_preserves_name() {
        let fix = BlockEntityRenameFix::create((), "BlockEntityRenameFix", |id: &str| {
            id.to_string()
        });
        assert_eq!(fix.name(), "BlockEntityRenameFix");
    }

    #[test]
    fn handles_empty_and_negative_like_ids() {
        let fix = BlockEntityRenameFix::create((), "edge", |id: &str| {
            if id.is_empty() {
                "minecraft:empty".to_string()
            } else {
                id.to_string()
            }
        });
        let rule = fix.make_rule();
        assert_eq!(rule(""), "minecraft:empty");
        assert_eq!(rule("-1"), "-1");
    }
}
