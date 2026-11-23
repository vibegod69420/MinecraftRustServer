use crate::NbtTag;
use std::collections::BTreeSet;
use std::fmt;

/// Minimal self-contained error corresponding to Brigadier `CommandSyntaxException`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSyntaxException(pub String);

impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CommandSyntaxException {}

/// Minimal abstraction for `net.minecraft.commands.arguments.NbtPathArgument.NbtPath`.
/// `visit_or_create` is a Rust-safe adaptation of Java's `getOrCreate`, preserving behavior.
pub trait NbtPath {
    fn get(&self, root: &NbtTag) -> Result<Vec<NbtTag>, CommandSyntaxException>;
    fn set(&self, root: &mut NbtTag, value: NbtTag) -> Result<(), CommandSyntaxException>;
    fn visit_or_create(
        &self,
        root: &mut NbtTag,
        create: fn() -> NbtTag,
        visitor: &mut dyn FnMut(&mut NbtTag),
    ) -> Result<(), CommandSyntaxException>;
}

/// Simple dot-separated compound-key NBT path for self-contained use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleNbtPath {
    pub segments: Vec<String>,
}

impl SimpleNbtPath {
    pub fn of(path: &str) -> Result<Self, CommandSyntaxException> {
        if path.is_empty() {
            return Err(CommandSyntaxException("empty NBT path".to_string()));
        }
        Ok(Self {
            segments: path.split('.').map(|s| s.to_string()).collect(),
        })
    }
}

impl NbtPath for SimpleNbtPath {
    fn get(&self, root: &NbtTag) -> Result<Vec<NbtTag>, CommandSyntaxException> {
        let mut current = vec![root.clone()];
        for seg in &self.segments {
            let mut next = Vec::new();
            for tag in &current {
                if let NbtTag::Compound(entries) = tag {
                    if let Some((_, v)) = entries.iter().rev().find(|(k, _)| k == seg) {
                        next.push(v.clone());
                    }
                }
            }
            current = next;
        }
        Ok(current)
    }

    fn set(&self, root: &mut NbtTag, value: NbtTag) -> Result<(), CommandSyntaxException> {
        if self.segments.is_empty() {
            return Err(CommandSyntaxException("empty NBT path".to_string()));
        }
        let (prefix, last_slice) = self.segments.split_at(self.segments.len() - 1);
        let last = &last_slice[0];
        let mut targets: Vec<&mut NbtTag> = vec![root];
        for seg in prefix {
            let mut next = Vec::new();
            for tag in targets {
                if let NbtTag::Compound(entries) = tag {
                    if let Some((_, v)) = entries.iter_mut().rev().find(|(k, _)| k == seg) {
                        next.push(v);
                    }
                }
            }
            targets = next;
        }
        for tag in targets {
            if let NbtTag::Compound(entries) = tag {
                if let Some(pos) = entries.iter().rposition(|(k, _)| k == last) {
                    entries[pos].1 = value.clone();
                } else {
                    entries.push((last.clone(), value.clone()));
                }
            }
        }
        Ok(())
    }

    fn visit_or_create(
        &self,
        root: &mut NbtTag,
        create: fn() -> NbtTag,
        visitor: &mut dyn FnMut(&mut NbtTag),
    ) -> Result<(), CommandSyntaxException> {
        if self.segments.is_empty() {
            return Err(CommandSyntaxException("empty NBT path".to_string()));
        }
        let (prefix, last_slice) = self.segments.split_at(self.segments.len() - 1);
        let last = &last_slice[0];
        let mut targets: Vec<&mut NbtTag> = vec![root];
        for seg in prefix {
            let mut next = Vec::new();
            for tag in targets {
                if let NbtTag::Compound(entries) = tag {
                    if entries.iter().rposition(|(k, _)| k == seg).is_none() {
                        entries.push((seg.clone(), NbtTag::Compound(Vec::new())));
                    }
                    let pos = entries.iter().rposition(|(k, _)| k == seg).unwrap();
                    next.push(&mut entries[pos].1);
                }
            }
            targets = next;
        }
        for tag in targets {
            if let NbtTag::Compound(entries) = tag {
                if entries.iter().rposition(|(k, _)| k == last).is_none() {
                    entries.push((last.clone(), create()));
                }
                let pos = entries.iter().rposition(|(k, _)| k == last).unwrap();
                visitor(&mut entries[pos].1);
            }
        }
        Ok(())
    }
}

/// Minimal abstraction for `net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider`.
pub trait NbtProvider {
    type Context;
    fn get(&self, ctx: &Self::Context) -> Option<NbtTag>;
    fn get_referenced_context_params(&self) -> BTreeSet<String>;
}

/// Minimal abstraction for item custom data access used by `CopyCustomDataFunction`.
pub trait CustomDataHolder {
    fn get_custom_data_tag(&self) -> NbtTag;
    fn set_custom_data_tag(&mut self, tag: NbtTag);
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.LootContext.EntityTarget`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityTarget {
    This,
    Killer,
    DirectKiller,
    KillerPlayer,
}

/// Minimal stub for `net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextNbtProvider {
    pub target: EntityTarget,
}

impl ContextNbtProvider {
    pub fn for_context_entity(target: EntityTarget) -> Self {
        Self { target }
    }
}

impl NbtProvider for ContextNbtProvider {
    type Context = ();
    fn get(&self, _ctx: &Self::Context) -> Option<NbtTag> {
        None
    }
    fn get_referenced_context_params(&self) -> BTreeSet<String> {
        BTreeSet::new()
    }
}

fn merge_compound(target: &mut Vec<(String, NbtTag)>, source: &[(String, NbtTag)]) {
    for (key, src_tag) in source {
        if let NbtTag::Compound(src_entries) = src_tag {
            let existing = target.iter_mut().rev().find(|(k, _)| k == key);
            if let Some((_, NbtTag::Compound(target_entries))) = existing {
                merge_compound(target_entries, src_entries);
                continue;
            }
        }
        let copy = src_tag.clone();
        if let Some(pos) = target.iter().rposition(|(k, _)| k == key) {
            target[pos].1 = copy;
        } else {
            target.push((key.clone(), copy));
        }
    }
}

/// Port of `net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction.MergeStrategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    Replace,
    Append,
    Merge,
}

impl MergeStrategy {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$MergeStrategy#merge
    pub fn merge<P: NbtPath + ?Sized>(
        &self,
        root: &mut NbtTag,
        path: &P,
        sources: &[NbtTag],
    ) -> Result<(), CommandSyntaxException> {
        match self {
            MergeStrategy::Replace => {
                if let Some(last) = sources.last() {
                    path.set(root, last.clone())?;
                }
                Ok(())
            }
            MergeStrategy::Append => {
                path.visit_or_create(
                    root,
                    || NbtTag::List {
                        elem_type: 0,
                        items: Vec::new(),
                    },
                    &mut |target| {
                        if let NbtTag::List { items, elem_type } = target {
                            for src in sources {
                                items.push(src.clone());
                            }
                            if *elem_type == 0 {
                                if let Some(first) = items.first() {
                                    *elem_type = first.id();
                                }
                            }
                        }
                    },
                )?;
                Ok(())
            }
            MergeStrategy::Merge => {
                path.visit_or_create(
                    root,
                    || NbtTag::Compound(Vec::new()),
                    &mut |target| {
                        if let NbtTag::Compound(target_entries) = target {
                            for src in sources {
                                if let NbtTag::Compound(src_entries) = src {
                                    merge_compound(target_entries, src_entries);
                                }
                            }
                        }
                    },
                )?;
                Ok(())
            }
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$MergeStrategy#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            MergeStrategy::Replace => "replace",
            MergeStrategy::Append => "append",
            MergeStrategy::Merge => "merge",
        }
    }
}

/// Port of `net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction.CopyOperation`.
#[derive(Debug, Clone)]
pub struct CopyOperation<P> {
    pub source_path: P,
    pub target_path: P,
    pub op: MergeStrategy,
}

impl<P: NbtPath> CopyOperation<P> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$CopyOperation#apply
    pub fn apply(&self, root: &mut Option<NbtTag>, init: &dyn Fn() -> NbtTag, source: &NbtTag) {
        let result: Result<(), CommandSyntaxException> = (|| {
            let list = self.source_path.get(source)?;
            if !list.is_empty() {
                let root_tag = if let Some(existing) = root.as_mut() {
                    existing
                } else {
                    *root = Some(init());
                    root.as_mut().expect("root initialized")
                };
                self.op.merge(root_tag, &self.target_path, &list)?;
            }
            Ok(())
        })();
        let _ = result;
    }
}

/// Port of `net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction`.
#[derive(Debug, Clone)]
pub struct CopyCustomDataFunction<Src, Path, Cond = ()> {
    pub conditions: Vec<Cond>,
    pub source: Src,
    pub operations: Vec<CopyOperation<Path>>,
}

impl<Src, Path, Cond> CopyCustomDataFunction<Src, Path, Cond> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction#<init>
    pub fn new(conditions: Vec<Cond>, source: Src, operations: Vec<CopyOperation<Path>>) -> Self {
        Self {
            conditions,
            source,
            operations,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction#getType
    pub fn get_type(&self) -> &'static str {
        "minecraft:copy_custom_data"
    }
}

impl<Src, Path, Cond> CopyCustomDataFunction<Src, Path, Cond>
where
    Src: NbtProvider,
    Path: NbtPath,
{
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction#getReferencedContextParams
    pub fn get_referenced_context_params(&self) -> BTreeSet<String> {
        self.source.get_referenced_context_params()
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction#run
    pub fn run<I: CustomDataHolder>(&self, mut item: I, ctx: &Src::Context) -> I {
        let Some(source_tag) = self.source.get(ctx) else {
            return item;
        };

        let mut custom_data: Option<NbtTag> = None;
        let init = || item.get_custom_data_tag();
        for op in &self.operations {
            op.apply(&mut custom_data, &init, &source_tag);
        }
        if let Some(tag) = custom_data {
            item.set_custom_data_tag(tag);
        }
        item
    }
}

// JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction#copyData
pub fn copy_data<Src>(source: Src) -> Builder<Src, SimpleNbtPath> {
    Builder::new(source)
}

// JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction#copyData
pub fn copy_data_from_entity_target(target: EntityTarget) -> Builder<ContextNbtProvider, SimpleNbtPath> {
    Builder::new(ContextNbtProvider::for_context_entity(target))
}

/// Port of `net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction.Builder`.
#[derive(Debug, Clone)]
pub struct Builder<Src, Path = SimpleNbtPath> {
    source: Src,
    ops: Vec<CopyOperation<Path>>,
}

impl<Src> Builder<Src, SimpleNbtPath> {
    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$Builder#<init>
    pub fn new(source: Src) -> Self {
        Self {
            source,
            ops: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$Builder#copy
    pub fn copy(mut self, source: &str, target: &str, op: MergeStrategy) -> Self {
        let src_path = SimpleNbtPath::of(source).expect("invalid NBT path");
        let tgt_path = SimpleNbtPath::of(target).expect("invalid NBT path");
        self.ops.push(CopyOperation {
            source_path: src_path,
            target_path: tgt_path,
            op,
        });
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$Builder#copy
    pub fn copy_replace(self, source: &str, target: &str) -> Self {
        self.copy(source, target, MergeStrategy::Replace)
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$Builder#getThis
    pub fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction$Builder#build
    pub fn build(self) -> CopyCustomDataFunction<Src, SimpleNbtPath, ()> {
        CopyCustomDataFunction::new(Vec::new(), self.source, self.ops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NbtTag;

    #[derive(Debug, Clone)]
    struct TestProvider(Option<NbtTag>);

    impl NbtProvider for TestProvider {
        type Context = ();
        fn get(&self, _ctx: &Self::Context) -> Option<NbtTag> {
            self.0.clone()
        }
        fn get_referenced_context_params(&self) -> BTreeSet<String> {
            BTreeSet::new()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestItem {
        custom_data: NbtTag,
    }

    impl TestItem {
        fn empty() -> Self {
            Self {
                custom_data: NbtTag::Compound(Vec::new()),
            }
        }

        fn with_custom_data(custom_data: NbtTag) -> Self {
            Self { custom_data }
        }
    }

    impl CustomDataHolder for TestItem {
        fn get_custom_data_tag(&self) -> NbtTag {
            self.custom_data.clone()
        }
        fn set_custom_data_tag(&mut self, tag: NbtTag) {
            self.custom_data = tag;
        }
    }

    #[test]
    fn replace_copies_last_source_and_preserves_unchanged() {
        let source = NbtTag::Compound(vec![
            ("a".to_string(), NbtTag::Int(1)),
            ("b".to_string(), NbtTag::Int(2)),
        ]);
        let func = copy_data(TestProvider(Some(source)))
            .copy("a", "x", MergeStrategy::Replace)
            .copy_replace("b", "y")
            .build();
        let item = func.run(TestItem::empty(), &());
        assert_eq!(item.custom_data.compound_get("x"), Some(&NbtTag::Int(1)));
        assert_eq!(item.custom_data.compound_get("y"), Some(&NbtTag::Int(2)));
    }

    #[test]
    fn append_adds_copies_to_list_targets() {
        let source = NbtTag::Compound(vec![("v".to_string(), NbtTag::Int(10))]);
        let item = TestItem::with_custom_data(NbtTag::Compound(vec![(
            "list".to_string(),
            NbtTag::List {
                elem_type: 3,
                items: vec![NbtTag::Int(1)],
            },
        )]));
        let func = copy_data(TestProvider(Some(source)))
            .copy("v", "list", MergeStrategy::Append)
            .build();
        let out = func.run(item, &());
        match out.custom_data.compound_get("list") {
            Some(NbtTag::List { items, .. }) => {
                assert_eq!(items, &vec![NbtTag::Int(1), NbtTag::Int(10)]);
            }
            other => panic!("expected list, got {:?}", other),
        }
    }

    #[test]
    fn merge_merges_compound_targets_recursively() {
        let source = NbtTag::Compound(vec![(
            "src".to_string(),
            NbtTag::Compound(vec![
                ("b".to_string(), NbtTag::Int(2)),
                (
                    "nested".to_string(),
                    NbtTag::Compound(vec![("y".to_string(), NbtTag::Int(20))]),
                ),
            ]),
        )]);
        let item = TestItem::with_custom_data(NbtTag::Compound(vec![(
            "dst".to_string(),
            NbtTag::Compound(vec![
                ("a".to_string(), NbtTag::Int(1)),
                ("b".to_string(), NbtTag::Int(9)),
                (
                    "nested".to_string(),
                    NbtTag::Compound(vec![("x".to_string(), NbtTag::Int(10))]),
                ),
            ]),
        )]));
        let func = copy_data(TestProvider(Some(source)))
            .copy("src", "dst", MergeStrategy::Merge)
            .build();
        let out = func.run(item, &());
        let dst = out.custom_data.compound_get("dst").unwrap();
        assert_eq!(dst.compound_get("a"), Some(&NbtTag::Int(1)));
        assert_eq!(dst.compound_get("b"), Some(&NbtTag::Int(2)));
        let nested = dst.compound_get("nested").unwrap();
        assert_eq!(nested.compound_get("x"), Some(&NbtTag::Int(10)));
        assert_eq!(nested.compound_get("y"), Some(&NbtTag::Int(20)));
    }

    #[test]
    fn missing_source_returns_item_unchanged() {
        let func = copy_data(TestProvider(None))
            .copy("a", "b", MergeStrategy::Replace)
            .build();
        let item = TestItem::with_custom_data(NbtTag::Compound(vec![(
            "k".to_string(),
            NbtTag::Int(7),
        )]));
        let out = func.run(item.clone(), &());
        assert_eq!(out, item);
        assert_eq!(func.get_type(), "minecraft:copy_custom_data");
    }

    #[test]
    fn negative_and_overflow_integer_values_preserved() {
        let source = NbtTag::Compound(vec![
            ("byte".to_string(), NbtTag::Byte(i8::MIN)),
            ("short".to_string(), NbtTag::Short(i16::MIN)),
            ("int".to_string(), NbtTag::Int(i32::MAX)),
            ("long".to_string(), NbtTag::Long(i64::MIN)),
            (
                "arr".to_string(),
                NbtTag::IntArray(vec![i32::MIN, -1, i32::MAX]),
            ),
        ]);
        let func = copy_data(TestProvider(Some(source)))
            .copy("byte", "out_byte", MergeStrategy::Replace)
            .copy("short", "out_short", MergeStrategy::Replace)
            .copy("int", "out_int", MergeStrategy::Replace)
            .copy("long", "out_long", MergeStrategy::Replace)
            .copy("arr", "out_arr", MergeStrategy::Replace)
            .build();
        let out = func.run(TestItem::empty(), &());
        assert_eq!(
            out.custom_data.compound_get("out_byte"),
            Some(&NbtTag::Byte(i8::MIN))
        );
        assert_eq!(
            out.custom_data.compound_get("out_short"),
            Some(&NbtTag::Short(i16::MIN))
        );
        assert_eq!(
            out.custom_data.compound_get("out_int"),
            Some(&NbtTag::Int(i32::MAX))
        );
        assert_eq!(
            out.custom_data.compound_get("out_long"),
            Some(&NbtTag::Long(i64::MIN))
        );
        assert_eq!(
            out.custom_data.compound_get("out_arr"),
            Some(&NbtTag::IntArray(vec![i32::MIN, -1, i32::MAX]))
        );
    }

    #[test]
    fn float_bits_preserved_in_copy() {
        let nan_f32 = f32::from_bits(0x7FC0_0001);
        let neg_zero_f32 = f32::from_bits(0x8000_0000);
        let neg_inf_f32 = f32::from_bits(0xFF80_0000);
        let nan_f64 = f64::from_bits(0x7FF8_0000_0000_0001);
        let neg_zero_f64 = f64::from_bits(0x8000_0000_0000_0000);
        let pos_inf_f64 = f64::from_bits(0x7FF0_0000_0000_0000);
        let source = NbtTag::Compound(vec![
            ("f_nan".to_string(), NbtTag::Float(nan_f32)),
            ("f_nz".to_string(), NbtTag::Float(neg_zero_f32)),
            ("f_ni".to_string(), NbtTag::Float(neg_inf_f32)),
            ("d_nan".to_string(), NbtTag::Double(nan_f64)),
            ("d_nz".to_string(), NbtTag::Double(neg_zero_f64)),
            ("d_pi".to_string(), NbtTag::Double(pos_inf_f64)),
        ]);
        let func = copy_data(TestProvider(Some(source)))
            .copy("f_nan", "o_f_nan", MergeStrategy::Replace)
            .copy("f_nz", "o_f_nz", MergeStrategy::Replace)
            .copy("f_ni", "o_f_ni", MergeStrategy::Replace)
            .copy("d_nan", "o_d_nan", MergeStrategy::Replace)
            .copy("d_nz", "o_d_nz", MergeStrategy::Replace)
            .copy("d_pi", "o_d_pi", MergeStrategy::Replace)
            .build();
        let out = func.run(TestItem::empty(), &());
        match out.custom_data.compound_get("o_f_nan") {
            Some(NbtTag::Float(v)) => assert_eq!(v.to_bits(), nan_f32.to_bits()),
            other => panic!("{:?}", other),
        }
        match out.custom_data.compound_get("o_f_nz") {
            Some(NbtTag::Float(v)) => assert_eq!(v.to_bits(), neg_zero_f32.to_bits()),
            other => panic!("{:?}", other),
        }
        match out.custom_data.compound_get("o_f_ni") {
            Some(NbtTag::Float(v)) => assert_eq!(v.to_bits(), neg_inf_f32.to_bits()),
            other => panic!("{:?}", other),
        }
        match out.custom_data.compound_get("o_d_nan") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), nan_f64.to_bits()),
            other => panic!("{:?}", other),
        }
        match out.custom_data.compound_get("o_d_nz") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), neg_zero_f64.to_bits()),
            other => panic!("{:?}", other),
        }
        match out.custom_data.compound_get("o_d_pi") {
            Some(NbtTag::Double(v)) => assert_eq!(v.to_bits(), pos_inf_f64.to_bits()),
            other => panic!("{:?}", other),
        }
    }
}
