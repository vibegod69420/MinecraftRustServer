/// Minimal self-contained block state representation for block rename fix logic.
/// Java uses DFU Dynamic/Typed/TypeRewriteRule; here we port the core rename behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockStateDynamic {
    pub name: Option<String>,
    pub properties: Option<Vec<(String, String)>>,
}

impl BlockStateDynamic {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            properties: None,
        }
    }
}

pub trait BlockRenamer {
    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#renameBlock
    fn rename_block(&self, name: &str) -> String;
}

pub struct BlockRenameFix<R: BlockRenamer> {
    pub name: String,
    pub renamer: R,
}

impl<R: BlockRenamer> BlockRenameFix<R> {
    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#<init>
    pub fn new(name: String, renamer: R) -> Self {
        Self { name, renamer }
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#renameBlock
    pub fn rename_block(&self, name: &str) -> String {
        self.renamer.rename_block(name)
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#fixBlockState
    pub fn fix_block_state(&self, mut state: BlockStateDynamic) -> BlockStateDynamic {
        if let Some(name) = state.name.as_ref() {
            let renamed = self.rename_block(name);
            state.name = Some(renamed);
        }
        state
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#fixFlatBlockState
    pub fn fix_flat_block_state(&self, flat: &str) -> String {
        let mut end = flat.len();
        if let Some(i) = flat.find('[') {
            if i > 0 {
                end = i;
            }
        }
        if let Some(i) = flat.find('{') {
            if i > 0 && i < end {
                end = i;
            }
        }
        let base = &flat[..end];
        let renamed = self.rename_block(base);
        let mut out = String::with_capacity(renamed.len() + (flat.len() - end));
        out.push_str(&renamed);
        out.push_str(&flat[end..]);
        out
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#makeRule
    pub fn make_rule_description(&self) -> String {
        format!("{} for block/block_state/flat_block_state", self.name)
    }
}

pub struct FnBlockRenamer<F: Fn(&str) -> String> {
    f: F,
}

impl<F: Fn(&str) -> String> FnBlockRenamer<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn(&str) -> String> BlockRenamer for FnBlockRenamer<F> {
    // JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#renameBlock
    fn rename_block(&self, name: &str) -> String {
        (self.f)(name)
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlockRenameFix#create
pub fn create<F>(name: String, f: F) -> BlockRenameFix<FnBlockRenamer<F>>
where
    F: Fn(&str) -> String,
{
    BlockRenameFix::new(name, FnBlockRenamer::new(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_fix() -> BlockRenameFix<FnBlockRenamer<impl Fn(&str) -> String>> {
        create("RenameTest".to_string(), |s: &str| {
            if s == "minecraft:old" {
                "minecraft:new".to_string()
            } else {
                s.to_string()
            }
        })
    }

    #[test]
    fn rename_block_unchanged_and_changed() {
        let fix = simple_fix();
        assert_eq!(fix.rename_block("minecraft:old"), "minecraft:new");
        assert_eq!(fix.rename_block("minecraft:stone"), "minecraft:stone");
    }

    #[test]
    fn fix_block_state_updates_name_only_when_present() {
        let fix = simple_fix();
        let state = BlockStateDynamic::new(Some("minecraft:old".to_string()));
        let out = fix.fix_block_state(state);
        assert_eq!(out.name.as_deref(), Some("minecraft:new"));

        let none_state = BlockStateDynamic::new(None);
        let out_none = fix.fix_block_state(none_state.clone());
        assert_eq!(out_none, none_state);
    }

    #[test]
    fn fix_flat_block_state_with_properties_and_nbt() {
        let fix = simple_fix();
        assert_eq!(
            fix.fix_flat_block_state("minecraft:old[facing=north]"),
            "minecraft:new[facing=north]"
        );
        assert_eq!(
            fix.fix_flat_block_state("minecraft:old{Custom:1b}"),
            "minecraft:new{Custom:1b}"
        );
        assert_eq!(
            fix.fix_flat_block_state("minecraft:old[facing=north]{Custom:1b}"),
            "minecraft:new[facing=north]{Custom:1b}"
        );
    }

    #[test]
    fn fix_flat_block_state_edge_cases_negative_like_and_long_names() {
        let fix = create("Edge".to_string(), |s: &str| {
            if s == "-1" {
                "0".to_string()
            } else if s == "9223372036854775807" {
                "ok".to_string()
            } else {
                s.to_string()
            }
        });
        assert_eq!(fix.fix_flat_block_state("-1[prop=1]"), "0[prop=1]");
        assert_eq!(fix.fix_flat_block_state("9223372036854775807"), "ok");
        assert_eq!(fix.fix_flat_block_state("[leading]"), "[leading]");
    }

    #[test]
    fn make_rule_description_and_float_like_string_preserved() {
        let fix = create("Floaty".to_string(), |s: &str| {
            if s == "1.5" { "2.5".to_string() } else { s.to_string() }
        });
        assert_eq!(
            fix.make_rule_description(),
            "Floaty for block/block_state/flat_block_state"
        );
        // float-bit-like string payload is preserved/renamed as plain string
        assert_eq!(fix.fix_flat_block_state("1.5[bits=3ff8000000000000]"), "2.5[bits=3ff8000000000000]");
    }

    #[test]
    fn fix_flat_block_state_no_brackets() {
        let fix = simple_fix();
        assert_eq!(fix.fix_flat_block_state("minecraft:old"), "minecraft:new");
        assert_eq!(fix.fix_flat_block_state("minecraft:other"), "minecraft:other");
    }
}
