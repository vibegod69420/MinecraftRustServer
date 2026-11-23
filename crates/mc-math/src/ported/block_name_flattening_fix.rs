#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    pub fn map_second<C, F: FnOnce(B) -> C>(self, f: F) -> Pair<A, C> {
        Pair(self.0, f(self.1))
    }
}

// JAVA: net.minecraft.util.datafix.schemas.NamespacedSchema#ensureNamespaced
pub fn ensure_namespaced(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("minecraft:{name}")
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlockStateData#upgradeBlock
pub fn upgrade_block_id(id: i32) -> String {
    match id {
        0 => "minecraft:air".to_string(),
        1 => "minecraft:stone".to_string(),
        2 => "minecraft:grass_block".to_string(),
        _ => format!("minecraft:numeric_block_{id}"),
    }
}

// JAVA: net.minecraft.util.datafix.fixes.BlockStateData#upgradeBlock
pub fn upgrade_block_string(name: &str) -> String {
    match name {
        "minecraft:grass" => "minecraft:grass_block".to_string(),
        "minecraft:stonebrick" => "minecraft:stone_bricks".to_string(),
        _ => name.to_string(),
    }
}

pub struct BlockNameFlatteningFix;

impl BlockNameFlatteningFix {
    // JAVA: net.minecraft.util.datafix.fixes.BlockNameFlatteningFix#BlockNameFlatteningFix
    pub fn new(_changes_type: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.BlockNameFlatteningFix#makeRule
    pub fn apply(&self, input: Pair<String, Either<i32, String>>) -> Pair<String, String> {
        input.map_second(|either_name| match either_name {
            Either::Left(id) => upgrade_block_id(id),
            Either::Right(name) => upgrade_block_string(&ensure_namespaced(&name)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_namespaced_adds_minecraft() {
        assert_eq!(ensure_namespaced("stone"), "minecraft:stone");
        assert_eq!(ensure_namespaced("minecraft:dirt"), "minecraft:dirt");
        assert_eq!(ensure_namespaced("custom:block"), "custom:block");
    }

    #[test]
    fn flattens_string_name() {
        let fix = BlockNameFlatteningFix::new(true);
        let input = Pair("id".to_string(), Either::Right("grass".to_string()));
        let out = fix.apply(input);
        assert_eq!(
            out,
            Pair("id".to_string(), "minecraft:grass_block".to_string())
        );
    }

    #[test]
    fn flattens_numeric_id() {
        let fix = BlockNameFlatteningFix::new(true);
        let input = Pair("id".to_string(), Either::Left(1_i32));
        let out = fix.apply(input);
        assert_eq!(out, Pair("id".to_string(), "minecraft:stone".to_string()));
    }

    #[test]
    fn namespaced_string_passthrough_or_upgrade() {
        let fix = BlockNameFlatteningFix::new(false);
        let input = Pair(
            "id".to_string(),
            Either::Right("minecraft:stonebrick".to_string()),
        );
        let out = fix.apply(input);
        assert_eq!(
            out,
            Pair("id".to_string(), "minecraft:stone_bricks".to_string())
        );
    }

    #[test]
    fn negative_and_edge_ids() {
        assert_eq!(upgrade_block_id(0), "minecraft:air");
        assert_eq!(upgrade_block_id(-1), "minecraft:numeric_block_-1");
        assert_eq!(
            upgrade_block_id(i32::MIN),
            format!("minecraft:numeric_block_{}", i32::MIN)
        );
    }
}
