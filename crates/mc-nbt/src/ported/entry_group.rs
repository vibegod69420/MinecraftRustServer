use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct LootContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootPoolEntry(pub i32);

pub type EntryConsumer<'a> = &'a mut dyn FnMut(LootPoolEntry);

pub struct ComposableEntryContainer {
    expand_fn: Box<dyn Fn(&mut LootContext, EntryConsumer<'_>) -> bool>,
}

impl fmt::Debug for ComposableEntryContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ComposableEntryContainer(..)")
    }
}

impl ComposableEntryContainer {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut LootContext, EntryConsumer<'_>) -> bool + 'static,
    {
        Self {
            expand_fn: Box::new(f),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#expand
    pub fn expand(&self, ctx: &mut LootContext, consumer: EntryConsumer<'_>) -> bool {
        (self.expand_fn)(ctx, consumer)
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#and
    pub fn and(self, other: ComposableEntryContainer) -> ComposableEntryContainer {
        ComposableEntryContainer::new(move |ctx, consumer| {
            self.expand(ctx, consumer) && other.expand(ctx, consumer)
        })
    }
}

// JAVA: net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer#ALWAYS_TRUE
pub fn always_true() -> ComposableEntryContainer {
    ComposableEntryContainer::new(|_, _| true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootPoolEntryType(pub &'static str);

pub struct LootPoolEntries;

impl LootPoolEntries {
    pub const GROUP: LootPoolEntryType = LootPoolEntryType("group");
}

#[derive(Debug)]
pub enum LootPoolEntryContainer {
    EntryGroup(EntryGroup),
    /// Placeholder for other entry types in this self-contained module.
    Leaf,
}

#[derive(Debug)]
pub struct EntryGroup {
    pub children: Vec<LootPoolEntryContainer>,
    pub conditions: Vec<LootItemCondition>,
}

impl EntryGroup {
    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup#<init>
    pub fn new(children: Vec<LootPoolEntryContainer>, conditions: Vec<LootItemCondition>) -> Self {
        Self {
            children,
            conditions,
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup#getType
    pub fn get_type(&self) -> LootPoolEntryType {
        LootPoolEntries::GROUP
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup#compose
    pub fn compose(&self, children: Vec<ComposableEntryContainer>) -> ComposableEntryContainer {
        match children.len() {
            0 => always_true(),
            1 => children.into_iter().next().unwrap(),
            2 => {
                let mut it = children.into_iter();
                let first = it.next().unwrap();
                let second = it.next().unwrap();
                ComposableEntryContainer::new(move |ctx, consumer| {
                    first.expand(ctx, consumer);
                    second.expand(ctx, consumer);
                    true
                })
            }
            _ => ComposableEntryContainer::new(move |ctx, consumer| {
                for child in &children {
                    child.expand(ctx, consumer);
                }
                true
            }),
        }
    }
}

// JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup#list
pub fn list(children: Vec<LootPoolEntryContainer>) -> Builder {
    Builder::new(children)
}

pub struct Builder {
    entries: Vec<LootPoolEntryContainer>,
    conditions: Vec<LootItemCondition>,
}

impl Builder {
    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup$Builder#<init>
    pub fn new(children: Vec<LootPoolEntryContainer>) -> Self {
        Self {
            entries: children,
            conditions: Vec::new(),
        }
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup$Builder#getThis
    fn get_this(&mut self) -> &mut Self {
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup$Builder#append
    pub fn append(mut self, child: LootPoolEntryContainer) -> Self {
        self.entries.push(child);
        self
    }

    // JAVA: net.minecraft.world.level.storage.loot.entries.EntryGroup$Builder#build
    pub fn build(self) -> LootPoolEntryContainer {
        LootPoolEntryContainer::EntryGroup(EntryGroup::new(self.entries, self.conditions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn mk_entry(id: i32, result: bool, counter: Rc<Cell<i32>>) -> ComposableEntryContainer {
        ComposableEntryContainer::new(move |_ctx, consumer| {
            counter.set(counter.get().wrapping_add(1));
            consumer(LootPoolEntry(id));
            result
        })
    }

    #[test]
    fn compose_empty_returns_true() {
        let entry = EntryGroup::new(vec![], vec![]);
        let composed = entry.compose(vec![]);
        let mut ctx = LootContext;
        let mut out = Vec::new();
        let ok = composed.expand(&mut ctx, &mut |e| out.push(e));
        assert!(ok);
        assert!(out.is_empty());
    }

    #[test]
    fn compose_single_delegates() {
        let entry = EntryGroup::new(vec![], vec![]);
        let counter = Rc::new(Cell::new(0i32));
        let child = mk_entry(7, false, counter.clone());
        let composed = entry.compose(vec![child]);
        let mut ctx = LootContext;
        let mut out = Vec::new();
        let ok = composed.expand(&mut ctx, &mut |e| out.push(e));
        assert!(!ok);
        assert_eq!(counter.get(), 1);
        assert_eq!(out, vec![LootPoolEntry(7)]);
    }

    #[test]
    fn compose_two_expands_both_even_if_false() {
        let entry = EntryGroup::new(vec![], vec![]);
        let c1 = Rc::new(Cell::new(0i32));
        let c2 = Rc::new(Cell::new(0i32));
        let first = mk_entry(1, false, c1.clone());
        let second = mk_entry(2, false, c2.clone());
        let composed = entry.compose(vec![first, second]);
        let mut ctx = LootContext;
        let mut out = Vec::new();
        let ok = composed.expand(&mut ctx, &mut |e| out.push(e));
        assert!(ok);
        assert_eq!(c1.get(), 1);
        assert_eq!(c2.get(), 1);
        assert_eq!(out, vec![LootPoolEntry(1), LootPoolEntry(2)]);
    }

    #[test]
    fn compose_many_expands_all_and_returns_true() {
        let entry = EntryGroup::new(vec![], vec![]);
        let counters: Vec<_> = (0..4).map(|_| Rc::new(Cell::new(0i32))).collect();
        let children = vec![
            mk_entry(10, false, counters[0].clone()),
            mk_entry(20, true, counters[1].clone()),
            mk_entry(30, false, counters[2].clone()),
            mk_entry(40, true, counters[3].clone()),
        ];
        let composed = entry.compose(children);
        let mut ctx = LootContext;
        let mut out = Vec::new();
        let ok = composed.expand(&mut ctx, &mut |e| out.push(e));
        assert!(ok);
        assert_eq!(counters.iter().map(|c| c.get()).collect::<Vec<_>>(), vec![1, 1, 1, 1]);
        assert_eq!(
            out,
            vec![
                LootPoolEntry(10),
                LootPoolEntry(20),
                LootPoolEntry(30),
                LootPoolEntry(40)
            ]
        );
    }

    #[test]
    fn builder_append_and_build_preserves_structure() {
        let built = list(vec![LootPoolEntryContainer::Leaf])
            .append(LootPoolEntryContainer::Leaf)
            .build();
        match built {
            LootPoolEntryContainer::EntryGroup(g) => {
                assert_eq!(g.children.len(), 2);
                assert!(g.conditions.is_empty());
                assert_eq!(g.get_type(), LootPoolEntries::GROUP);
            }
            _ => panic!("expected EntryGroup"),
        }
    }

    #[test]
    fn negative_extreme_and_float_bits_preserved() {
        let nan_bits = f32::NAN.to_bits();
        let neg_zero_bits = (-0.0f32).to_bits();
        let max_bits = f32::INFINITY.to_bits();
        let entry = EntryGroup::new(vec![], vec![]);
        let children = vec![
            ComposableEntryContainer::new(|_, c| {
                c(LootPoolEntry(i32::MIN));
                true
            }),
            ComposableEntryContainer::new(|_, c| {
                c(LootPoolEntry(i32::MAX));
                true
            }),
            ComposableEntryContainer::new(move |_, c| {
                c(LootPoolEntry(nan_bits as i32));
                false
            }),
            ComposableEntryContainer::new(move |_, c| {
                c(LootPoolEntry(neg_zero_bits as i32));
                true
            }),
            ComposableEntryContainer::new(move |_, c| {
                c(LootPoolEntry(max_bits as i32));
                false
            }),
        ];
        let composed = entry.compose(children);
        let mut ctx = LootContext;
        let mut out = Vec::new();
        assert!(composed.expand(&mut ctx, &mut |e| out.push(e)));
        assert_eq!(out[0].0, i32::MIN);
        assert_eq!(out[1].0, i32::MAX);
        assert_eq!(f32::from_bits(out[2].0 as u32).to_bits(), nan_bits);
        assert_eq!(f32::from_bits(out[3].0 as u32).to_bits(), neg_zero_bits);
        assert_eq!(f32::from_bits(out[4].0 as u32).to_bits(), max_bits);
    }
}
