use std::collections::BTreeMap;

/// Minimal schema marker for self-contained datafix port.
#[derive(Clone, Debug, Default)]
pub struct Schema;

/// Minimal type-rewrite rule marker for self-contained datafix port.
#[derive(Clone, Debug, Default)]
pub struct TypeRewriteRule;

/// Minimal optic finder marker for self-contained datafix port.
#[derive(Clone, Debug, Default)]
pub struct OpticFinder;

/// Minimal dynamic value representation for self-contained datafix port.
#[derive(Clone, Debug, PartialEq)]
pub enum DynValue {
    Int(i32),
    Double(f64),
    String(String),
    IntArray(Vec<i32>),
    List(Vec<DynValue>),
    Map(BTreeMap<String, DynValue>),
}

/// Numeric wrapper matching Java `Number.intValue()` usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynNumber {
    int_val: i32,
}

impl DynNumber {
    pub fn int_value(self) -> i32 {
        self.int_val
    }
}

/// Optional number result, mirroring DFU `DataResult<Number>.result()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionalNumber(Option<DynNumber>);

impl OptionalNumber {
    pub fn result(self) -> Option<DynNumber> {
        self.0
    }
}

/// Optional dynamic result, mirroring DFU `OptionalDynamic`/`Optional<Dynamic<?>>`.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionalDynamic(Option<Dynamic>);

impl OptionalDynamic {
    pub fn as_number(&self) -> OptionalNumber {
        match &self.0 {
            Some(d) => d.as_number(),
            None => OptionalNumber(None),
        }
    }

    pub fn result(&self) -> Option<Dynamic> {
        self.0.clone()
    }

    pub fn as_double(&self, default: f64) -> f64 {
        match &self.0 {
            Some(d) => d.as_double(default),
            None => default,
        }
    }
}

/// Minimal Dynamic wrapper for self-contained datafix port.
#[derive(Clone, Debug, PartialEq)]
pub struct Dynamic {
    pub value: DynValue,
}

impl Dynamic {
    pub fn new(value: DynValue) -> Self {
        Self { value }
    }

    pub fn as_map(&self) -> Option<&BTreeMap<String, DynValue>> {
        match &self.value {
            DynValue::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> OptionalDynamic {
        match &self.value {
            DynValue::Map(m) => OptionalDynamic(m.get(key).cloned().map(Dynamic::new)),
            _ => OptionalDynamic(None),
        }
    }

    pub fn as_number(&self) -> OptionalNumber {
        let n = match &self.value {
            DynValue::Int(v) => Some(DynNumber { int_val: *v }),
            DynValue::Double(v) => Some(DynNumber { int_val: *v as i32 }),
            _ => None,
        };
        OptionalNumber(n)
    }

    pub fn as_double(&self, default: f64) -> f64 {
        match &self.value {
            DynValue::Double(v) => *v,
            DynValue::Int(v) => f64::from(*v),
            _ => default,
        }
    }

    pub fn create_map(&self, entries: Vec<(Dynamic, Dynamic)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            if let DynValue::String(s) = k.value {
                m.insert(s, v.value);
            }
        }
        Dynamic::new(DynValue::Map(m))
    }

    pub fn create_string(&self, s: &str) -> Dynamic {
        Dynamic::new(DynValue::String(s.to_owned()))
    }

    pub fn create_double(&self, d: f64) -> Dynamic {
        Dynamic::new(DynValue::Double(d))
    }

    pub fn create_int(&self, i: i32) -> Dynamic {
        Dynamic::new(DynValue::Int(i))
    }

    pub fn create_list(&self, items: Vec<Dynamic>) -> Dynamic {
        Dynamic::new(DynValue::List(items.into_iter().map(|d| d.value).collect()))
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut m2 = m.clone();
                m2.remove(key);
                Dynamic::new(DynValue::Map(m2))
            }
            _ => self.clone(),
        }
    }

    pub fn set(&self, key: &str, value: Dynamic) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                let mut m2 = m.clone();
                m2.insert(key.to_owned(), value.value);
                Dynamic::new(DynValue::Map(m2))
            }
            _ => self.clone(),
        }
    }

    pub fn rename_field(&self, old: &str, new_name: &str) -> Dynamic {
        match &self.value {
            DynValue::Map(m) => {
                if let Some(v) = m.get(old).cloned() {
                    let mut m2 = m.clone();
                    m2.remove(old);
                    m2.insert(new_name.to_owned(), v);
                    Dynamic::new(DynValue::Map(m2))
                } else {
                    self.clone()
                }
            }
            _ => self.clone(),
        }
    }

    pub fn copy_field(src: &Dynamic, src_key: &str, dst: Dynamic, dst_key: &str) -> Dynamic {
        match src.get(src_key).result() {
            Some(v) => dst.set(dst_key, v),
            None => dst,
        }
    }
}

// Helper: ExtraDataFixUtils.createBlockPos
fn create_block_pos(_dynamic: &Dynamic, x: i32, y: i32, z: i32) -> Dynamic {
    Dynamic::new(DynValue::IntArray(vec![x, y, z]))
}

// Helper: ExtraDataFixUtils.fixInlineBlockPos
fn fix_inline_block_pos(
    dynamic: Dynamic,
    x_key: &str,
    y_key: &str,
    z_key: &str,
    new_key: &str,
) -> Dynamic {
    let x = dynamic.get(x_key).as_number().result();
    let y = dynamic.get(y_key).as_number().result();
    let z = dynamic.get(z_key).as_number().result();
    if let (Some(xn), Some(yn), Some(zn)) = (x, y, z) {
        let pos = create_block_pos(
            &dynamic,
            xn.int_value(),
            yn.int_value(),
            zn.int_value(),
        );
        return dynamic
            .remove(x_key)
            .remove(y_key)
            .remove(z_key)
            .set(new_key, pos);
    }
    dynamic
}

#[derive(Clone, Debug, Default)]
pub struct InlineBlockPosFormatFix;

impl InlineBlockPosFormatFix {
    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#<init>
    pub fn new(_schema: Schema) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        let _vex = self.entity_finder("minecraft:vex");
        let _phantom = self.entity_finder("minecraft:phantom");
        let _turtle = self.entity_finder("minecraft:turtle");
        let _attached = [
            self.entity_finder("minecraft:item_frame"),
            self.entity_finder("minecraft:glow_item_frame"),
            self.entity_finder("minecraft:painting"),
            self.entity_finder("minecraft:leash_knot"),
        ];
        let _ = _attached;
        TypeRewriteRule
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#entityFinder
    fn entity_finder(&self, _name: &str) -> OpticFinder {
        OpticFinder
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#fixPlayer
    fn fix_player(&self, mut d: Dynamic) -> Dynamic {
        d = self.fix_living_entity(d);
        let x = d.get("SpawnX").as_number().result();
        let y = d.get("SpawnY").as_number().result();
        let z = d.get("SpawnZ").as_number().result();
        if let (Some(xn), Some(yn), Some(zn)) = (x, y, z) {
            let mut respawn = d.create_map(vec![(
                d.create_string("pos"),
                create_block_pos(&d, xn.int_value(), yn.int_value(), zn.int_value()),
            )]);
            respawn = Dynamic::copy_field(&d, "SpawnAngle", respawn, "angle");
            respawn = Dynamic::copy_field(&d, "SpawnDimension", respawn, "dimension");
            respawn = Dynamic::copy_field(&d, "SpawnForced", respawn, "forced");
            d = d
                .remove("SpawnX")
                .remove("SpawnY")
                .remove("SpawnZ")
                .remove("SpawnAngle")
                .remove("SpawnDimension")
                .remove("SpawnForced");
            d = d.set("respawn", respawn);
        }

        let entered = d.get("enteredNetherPosition").result();
        if let Some(en) = entered {
            d = d.remove("enteredNetherPosition").set(
                "entered_nether_pos",
                d.create_list(vec![
                    d.create_double(en.get("x").as_double(0.0)),
                    d.create_double(en.get("y").as_double(0.0)),
                    d.create_double(en.get("z").as_double(0.0)),
                ]),
            );
        }
        d
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#fixLivingEntity
    fn fix_living_entity(&self, d: Dynamic) -> Dynamic {
        fix_inline_block_pos(d, "SleepingX", "SleepingY", "SleepingZ", "sleeping_pos")
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#fixVex
    fn fix_vex(&self, d: Dynamic) -> Dynamic {
        fix_inline_block_pos(
            d.rename_field("LifeTicks", "life_ticks"),
            "BoundX",
            "BoundY",
            "BoundZ",
            "bound_pos",
        )
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#fixPhantom
    fn fix_phantom(&self, d: Dynamic) -> Dynamic {
        fix_inline_block_pos(
            d.rename_field("Size", "size"),
            "AX",
            "AY",
            "AZ",
            "anchor_pos",
        )
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#fixTurtle
    fn fix_turtle(&self, d: Dynamic) -> Dynamic {
        let mut d = d
            .remove("TravelPosX")
            .remove("TravelPosY")
            .remove("TravelPosZ");
        d = fix_inline_block_pos(d, "HomePosX", "HomePosY", "HomePosZ", "home_pos");
        d.rename_field("HasEgg", "has_egg")
    }

    // JAVA: net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix#fixBlockAttached
    fn fix_block_attached(&self, d: Dynamic) -> Dynamic {
        fix_inline_block_pos(d, "TileX", "TileY", "TileZ", "block_pos")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dyn_map(entries: Vec<(&str, DynValue)>) -> Dynamic {
        let mut m = BTreeMap::new();
        for (k, v) in entries {
            m.insert(k.to_owned(), v);
        }
        Dynamic::new(DynValue::Map(m))
    }

    #[test]
    fn fix_player_respawn_and_nether_pos() {
        let fix = InlineBlockPosFormatFix::new(Schema);
        let input = dyn_map(vec![
            ("SpawnX", DynValue::Int(10)),
            ("SpawnY", DynValue::Int(64)),
            ("SpawnZ", DynValue::Int(-20)),
            ("SpawnAngle", DynValue::Double(90.0)),
            ("SpawnDimension", DynValue::String("minecraft:overworld".into())),
            ("SpawnForced", DynValue::Int(1)),
            (
                "enteredNetherPosition",
                DynValue::Map({
                    let mut m = BTreeMap::new();
                    m.insert("x".into(), DynValue::Double(1.5));
                    m.insert("y".into(), DynValue::Double(2.0));
                    m.insert("z".into(), DynValue::Double(-3.25));
                    m
                }),
            ),
        ]);

        let out = fix.fix_player(input);
        let m = out.as_map().unwrap();
        assert!(!m.contains_key("SpawnX"));
        assert!(!m.contains_key("SpawnY"));
        assert!(!m.contains_key("SpawnZ"));
        assert!(!m.contains_key("SpawnAngle"));
        assert!(!m.contains_key("SpawnDimension"));
        assert!(!m.contains_key("SpawnForced"));
        assert!(!m.contains_key("enteredNetherPosition"));

        let respawn = match m.get("respawn").unwrap() {
            DynValue::Map(rm) => rm,
            _ => panic!("expected respawn map"),
        };
        assert_eq!(respawn.get("pos"), Some(&DynValue::IntArray(vec![10, 64, -20])));
        assert_eq!(respawn.get("angle"), Some(&DynValue::Double(90.0)));
        assert_eq!(
            respawn.get("dimension"),
            Some(&DynValue::String("minecraft:overworld".into()))
        );
        assert_eq!(respawn.get("forced"), Some(&DynValue::Int(1)));

        let nether = match m.get("entered_nether_pos").unwrap() {
            DynValue::List(v) => v,
            _ => panic!("expected list"),
        };
        assert_eq!(
            nether,
            &vec![
                DynValue::Double(1.5),
                DynValue::Double(2.0),
                DynValue::Double(-3.25),
            ]
        );
    }

    #[test]
    fn fix_inline_block_pos_negative_and_partial() {
        let fix = InlineBlockPosFormatFix::new(Schema);
        let input = dyn_map(vec![
            ("SleepingX", DynValue::Int(i32::MIN)),
            ("SleepingY", DynValue::Int(-1)),
            ("SleepingZ", DynValue::Int(i32::MAX)),
        ]);
        let out = fix.fix_living_entity(input);
        let m = out.as_map().unwrap();
        assert_eq!(
            m.get("sleeping_pos"),
            Some(&DynValue::IntArray(vec![i32::MIN, -1, i32::MAX]))
        );
        assert!(!m.contains_key("SleepingX"));

        let partial = dyn_map(vec![("TileX", DynValue::Int(1)), ("TileZ", DynValue::Int(3))]);
        let partial_out = fix.fix_block_attached(partial.clone());
        assert_eq!(partial_out, partial);
    }

    #[test]
    fn fix_player_spawn_double_overflow_and_float_bits() {
        let fix = InlineBlockPosFormatFix::new(Schema);
        let neg_subnormal = f64::from_bits(0x8000_0000_0000_0001u64);
        let nan_bits = f64::from_bits(0x7FF8_0000_0000_0001u64);
        let input = dyn_map(vec![
            ("SpawnX", DynValue::Double(3_000_000_000.0)),
            ("SpawnY", DynValue::Double(-3_000_000_000.0)),
            ("SpawnZ", DynValue::Double(42.9)),
            (
                "enteredNetherPosition",
                DynValue::Map({
                    let mut m = BTreeMap::new();
                    m.insert("x".into(), DynValue::Double(f64::NEG_INFINITY));
                    m.insert("y".into(), DynValue::Double(neg_subnormal));
                    m.insert("z".into(), DynValue::Double(nan_bits));
                    m
                }),
            ),
        ]);

        let out = fix.fix_player(input);
        let m = out.as_map().unwrap();
        let respawn = match m.get("respawn").unwrap() {
            DynValue::Map(rm) => rm,
            _ => panic!("expected respawn map"),
        };
        assert_eq!(
            respawn.get("pos"),
            Some(&DynValue::IntArray(vec![i32::MAX, i32::MIN, 42]))
        );

        let nether = match m.get("entered_nether_pos").unwrap() {
            DynValue::List(v) => v,
            _ => panic!("expected list"),
        };
        match &nether[0] {
            DynValue::Double(v) => assert_eq!(v.to_bits(), f64::NEG_INFINITY.to_bits()),
            _ => panic!("expected double"),
        }
        match &nether[1] {
            DynValue::Double(v) => assert_eq!(v.to_bits(), neg_subnormal.to_bits()),
            _ => panic!("expected double"),
        }
        match &nether[2] {
            DynValue::Double(v) => assert_eq!(v.to_bits(), nan_bits.to_bits()),
            _ => panic!("expected double"),
        }
    }

    #[test]
    fn fix_entity_variants_renames_and_removals() {
        let fix = InlineBlockPosFormatFix::new(Schema);

        let vex = dyn_map(vec![
            ("LifeTicks", DynValue::Int(100)),
            ("BoundX", DynValue::Int(1)),
            ("BoundY", DynValue::Int(2)),
            ("BoundZ", DynValue::Int(3)),
        ]);
        let vex_out = fix.fix_vex(vex);
        let vm = vex_out.as_map().unwrap();
        assert_eq!(vm.get("life_ticks"), Some(&DynValue::Int(100)));
        assert!(!vm.contains_key("LifeTicks"));
        assert_eq!(vm.get("bound_pos"), Some(&DynValue::IntArray(vec![1, 2, 3])));

        let phantom = dyn_map(vec![
            ("Size", DynValue::Int(5)),
            ("AX", DynValue::Int(-10)),
            ("AY", DynValue::Int(64)),
            ("AZ", DynValue::Int(20)),
        ]);
        let ph_out = fix.fix_phantom(phantom);
        let pm = ph_out.as_map().unwrap();
        assert_eq!(pm.get("size"), Some(&DynValue::Int(5)));
        assert_eq!(pm.get("anchor_pos"), Some(&DynValue::IntArray(vec![-10, 64, 20])));

        let turtle = dyn_map(vec![
            ("TravelPosX", DynValue::Int(0)),
            ("TravelPosY", DynValue::Int(0)),
            ("TravelPosZ", DynValue::Int(0)),
            ("HomePosX", DynValue::Int(7)),
            ("HomePosY", DynValue::Int(8)),
            ("HomePosZ", DynValue::Int(9)),
            ("HasEgg", DynValue::Int(1)),
        ]);
        let t_out = fix.fix_turtle(turtle);
        let tm = t_out.as_map().unwrap();
        assert!(!tm.contains_key("TravelPosX"));
        assert!(!tm.contains_key("TravelPosY"));
        assert!(!tm.contains_key("TravelPosZ"));
        assert_eq!(tm.get("home_pos"), Some(&DynValue::IntArray(vec![7, 8, 9])));
        assert_eq!(tm.get("has_egg"), Some(&DynValue::Int(1)));
        assert!(!tm.contains_key("HasEgg"));
    }

    #[test]
    fn fix_player_entered_nether_missing_coords_default_zero() {
        let fix = InlineBlockPosFormatFix::new(Schema);
        let input = dyn_map(vec![(
            "enteredNetherPosition",
            DynValue::Map(BTreeMap::new()),
        )]);
        let out = fix.fix_player(input);
        let m = out.as_map().unwrap();
        assert!(!m.contains_key("enteredNetherPosition"));
        let nether = match m.get("entered_nether_pos").unwrap() {
            DynValue::List(v) => v,
            _ => panic!("expected list"),
        };
        assert_eq!(
            nether,
            &vec![
                DynValue::Double(0.0),
                DynValue::Double(0.0),
                DynValue::Double(0.0),
            ]
        );
    }
}
