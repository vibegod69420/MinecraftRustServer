// Self-contained faithful port of
// net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction
// External loot/registry/component types are minimally stubbed for compilation.

use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLocation(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagKey<T> {
    pub location: ResourceLocation,
    _pd: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    // JAVA: net.minecraft.tags.TagKey#<init> (shape stub)
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            location: ResourceLocation(location.into()),
            _pd: PhantomData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder<T>(pub T);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstrumentComponent(pub Holder<Instrument>);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub count: i32,
    pub instrument: Option<InstrumentComponent>,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#set (DataComponents.INSTRUMENT specialization)
    pub fn set_instrument(&mut self, component: InstrumentComponent) {
        self.instrument = Some(component);
    }
}

pub trait RandomSource {
    fn next_i32(&mut self, bound: i32) -> i32;
}

pub trait InstrumentRegistry {
    fn get_random_element_of(
        &self,
        tag: &TagKey<Instrument>,
        random: &mut dyn RandomSource,
    ) -> Option<Holder<Instrument>>;
}

pub struct LootContext<'a> {
    pub registry: &'a dyn InstrumentRegistry,
    pub random: &'a mut dyn RandomSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootItemFunctionType {
    SetInstrument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootItemCondition;

pub type LootItemConditionalFunctionBuilder<T> =
    Box<dyn FnOnce(Vec<LootItemCondition>) -> T>;

// JAVA: net.minecraft.world.level.storage.loot.functions.LootItemConditionalFunction#simpleBuilder
pub fn simple_builder<T, F>(f: F) -> LootItemConditionalFunctionBuilder<T>
where
    F: FnOnce(Vec<LootItemCondition>) -> T + 'static,
{
    Box::new(f)
}

#[derive(Debug, Clone)]
pub struct SetInstrumentFunction {
    pub conditions: Vec<LootItemCondition>,
    pub options: TagKey<Instrument>,
}

impl SetInstrumentFunction {
    // JAVA: net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction#CODEC
    // Codec placeholder: RecordCodecBuilder.mapCodec(commonFields(...).and(TagKey.hashedCodec(...).fieldOf("options")...))
    pub const CODEC_NAME: &'static str = "minecraft:set_instrument";

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction#<init>
    fn new(conditions: Vec<LootItemCondition>, options: TagKey<Instrument>) -> Self {
        Self { conditions, options }
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction#getType
    pub fn get_type(&self) -> LootItemFunctionType {
        LootItemFunctionType::SetInstrument
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction#run
    pub fn run(&self, mut stack: ItemStack, ctx: &mut LootContext<'_>) -> ItemStack {
        // Java: Registry<Instrument> $$2 = $$1.getLevel().registryAccess().lookupOrThrow(Registries.INSTRUMENT);
        // Java: Optional<Holder<Instrument>> $$3 = $$2.getRandomElementOf(this.options, $$1.getRandom());
        let opt = ctx.registry.get_random_element_of(&self.options, ctx.random);
        if let Some(holder) = opt {
            stack.set_instrument(InstrumentComponent(holder));
        }
        stack
    }

    // JAVA: net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction#setInstrumentOptions
    pub fn set_instrument_options(
        options: TagKey<Instrument>,
    ) -> LootItemConditionalFunctionBuilder<SetInstrumentFunction> {
        simple_builder(move |conditions| SetInstrumentFunction::new(conditions, options))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedRandom(i32);
    impl RandomSource for FixedRandom {
        fn next_i32(&mut self, bound: i32) -> i32 {
            if bound <= 0 {
                0
            } else {
                self.0.rem_euclid(bound)
            }
        }
    }

    struct MockRegistry {
        tag: String,
        elements: Vec<Holder<Instrument>>,
    }

    impl InstrumentRegistry for MockRegistry {
        fn get_random_element_of(
            &self,
            tag: &TagKey<Instrument>,
            random: &mut dyn RandomSource,
        ) -> Option<Holder<Instrument>> {
            if tag.location.0 != self.tag || self.elements.is_empty() {
                return None;
            }
            let idx = random.next_i32(self.elements.len() as i32) as usize;
            self.elements.get(idx).cloned()
        }
    }

    #[test]
    fn set_instrument_when_tag_has_element() {
        let reg = MockRegistry {
            tag: "minecraft:goat_horns".into(),
            elements: vec![Holder(Instrument { id: "minecraft:ponder_goat_horn".into() })],
        };
        let mut rng = FixedRandom(0);
        let mut ctx = LootContext { registry: &reg, random: &mut rng };
        let f = SetInstrumentFunction::set_instrument_options(TagKey::new("minecraft:goat_horns"))(vec![]);
        let out = f.run(ItemStack::default(), &mut ctx);
        assert_eq!(
            out.instrument,
            Some(InstrumentComponent(Holder(Instrument {
                id: "minecraft:ponder_goat_horn".into()
            })))
        );
        assert_eq!(f.get_type(), LootItemFunctionType::SetInstrument);
    }

    #[test]
    fn no_change_when_tag_missing_or_empty() {
        let reg = MockRegistry { tag: "minecraft:other".into(), elements: vec![] };
        let mut rng = FixedRandom(0);
        let mut ctx = LootContext { registry: &reg, random: &mut rng };
        let f = SetInstrumentFunction::new(vec![], TagKey::new("minecraft:goat_horns"));
        let input = ItemStack { count: 1, instrument: None };
        let out = f.run(input.clone(), &mut ctx);
        assert_eq!(out, input);
    }

    #[test]
    fn selects_element_with_negative_random_index_normalized() {
        let reg = MockRegistry {
            tag: "minecraft:goat_horns".into(),
            elements: vec![
                Holder(Instrument { id: "a".into() }),
                Holder(Instrument { id: "b".into() }),
            ],
        };
        let mut rng = FixedRandom(-1); // rem_euclid(2) == 1
        let mut ctx = LootContext { registry: &reg, random: &mut rng };
        let f = SetInstrumentFunction::new(vec![], TagKey::new("minecraft:goat_horns"));
        let out = f.run(ItemStack::default(), &mut ctx);
        assert_eq!(out.instrument.unwrap().0 .0.id, "b");
    }

    #[test]
    fn preserves_existing_count_and_overwrites_instrument() {
        let reg = MockRegistry {
            tag: "t".into(),
            elements: vec![Holder(Instrument { id: "new".into() })],
        };
        let mut rng = FixedRandom(0);
        let mut ctx = LootContext { registry: &reg, random: &mut rng };
        let f = SetInstrumentFunction::new(vec![], TagKey::new("t"));
        let input = ItemStack {
            count: i32::MAX,
            instrument: Some(InstrumentComponent(Holder(Instrument { id: "old".into() }))),
        };
        let out = f.run(input, &mut ctx);
        assert_eq!(out.count, i32::MAX);
        assert_eq!(out.instrument.unwrap().0 .0.id, "new");
    }
}
