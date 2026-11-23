use std::collections::BTreeMap;
use std::marker::PhantomData;

/// Minimal self-contained dynamic value representing DFU `Dynamic`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dynamic {
    kind: DynamicKind,
}

#[derive(Debug, Clone, PartialEq, Default)]
enum DynamicKind {
    #[default]
    Empty,
    Map(BTreeMap<String, Dynamic>),
    String(String),
    Int(i32),
}

impl Dynamic {
    pub fn map(entries: BTreeMap<String, Dynamic>) -> Self {
        Self {
            kind: DynamicKind::Map(entries),
        }
    }

    pub fn string(s: impl Into<String>) -> Self {
        Self {
            kind: DynamicKind::String(s.into()),
        }
    }

    pub fn int(v: i32) -> Self {
        Self {
            kind: DynamicKind::Int(v),
        }
    }

    pub fn empty_map_value() -> Self {
        Self {
            kind: DynamicKind::Map(BTreeMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Dynamic {
        match &self.kind {
            DynamicKind::Map(m) => m.get(key).cloned().unwrap_or_default(),
            _ => Dynamic::default(),
        }
    }

    pub fn as_int(&self, default: i32) -> i32 {
        match &self.kind {
            DynamicKind::Int(v) => *v,
            _ => default,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match &self.kind {
            DynamicKind::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn result(&self) -> Option<Dynamic> {
        match &self.kind {
            DynamicKind::Empty => None,
            _ => Some(self.clone()),
        }
    }

    pub fn remove(&self, key: &str) -> Dynamic {
        match &self.kind {
            DynamicKind::Map(m) => {
                let mut new_map = m.clone();
                new_map.remove(key);
                Dynamic {
                    kind: DynamicKind::Map(new_map),
                }
            }
            _ => self.clone(),
        }
    }

    pub fn empty_map(&self) -> Dynamic {
        Self::empty_map_value()
    }
}

/// Minimal DFU `Pair`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<A, B>(pub A, pub B);

/// Minimal DFU `Either`.
#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Minimal DFU `Unit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit;

/// Minimal DFU `Type<R>` carrying a recipe reader.
pub struct Type<R> {
    read_fn: fn(&Dynamic) -> Option<(R, Dynamic)>,
    _phantom: PhantomData<R>,
}

impl<R> Clone for Type<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for Type<R> {}

impl<R> Type<R> {
    pub fn new(read_fn: fn(&Dynamic) -> Option<(R, Dynamic)>) -> Self {
        Self {
            read_fn,
            _phantom: PhantomData,
        }
    }

    pub fn read(&self, input: &Dynamic) -> Option<(R, Dynamic)> {
        (self.read_fn)(input)
    }
}

/// Minimal DFU `Typed<?>` wrapper holding a remainder dynamic.
#[derive(Debug, Clone, PartialEq)]
pub struct Typed {
    pub remainder: Dynamic,
}

impl Typed {
    pub fn get_or_create_remainder(&self) -> Dynamic {
        self.remainder.clone()
    }
}

/// Placeholder for the DFU pair type parameter in `updateFurnaceContents`.
pub struct PairType<R>(PhantomData<R>);

impl<R> Default for PairType<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Minimal DFU schema stub.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Schema;

/// Minimal DFU type rewrite rule stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRewriteRule {
    pub name: String,
}

pub type RecipesUsedField<R> = Either<Pair<Vec<Pair<R, i32>>, Dynamic>, Unit>;
pub type FurnaceTypedOutput<R> = Pair<RecipesUsedField<R>, Dynamic>;

/// Port of `net.minecraft.util.datafix.fixes.FurnaceRecipeFix`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FurnaceRecipeFix {
    pub schema: Schema,
    pub bypass: bool,
}

impl FurnaceRecipeFix {
    // JAVA: net.minecraft.util.datafix.fixes.FurnaceRecipeFix#<init>
    pub fn new(schema: Schema, bypass: bool) -> Self {
        Self { schema, bypass }
    }

    // JAVA: net.minecraft.util.datafix.fixes.FurnaceRecipeFix#makeRule
    pub fn make_rule(&self) -> TypeRewriteRule {
        self.cap(Type::<Dynamic>::new(|d| Some((d.clone(), Dynamic::default()))))
    }

    // JAVA: net.minecraft.util.datafix.fixes.FurnaceRecipeFix#cap
    fn cap<R: Clone + 'static>(&self, _recipe_type: Type<R>) -> TypeRewriteRule {
        // DFU wiring in Java:
        // - DSL.and(DSL.optional(DSL.field("RecipesUsed", DSL.and(DSL.compoundList(recipe, DSL.intType()), DSL.remainderType()))), DSL.remainderType())
        // - OpticFinders for minecraft:furnace, minecraft:blast_furnace, minecraft:smoker
        // - fixTypeEverywhereTyped("FurnaceRecipesFix", ...) applies updateFurnaceContents to each variant
        TypeRewriteRule {
            name: "FurnaceRecipesFix".to_string(),
        }
    }

    // JAVA: net.minecraft.util.datafix.fixes.FurnaceRecipeFix#updateFurnaceContents
    pub fn update_furnace_contents<R: Clone>(
        &self,
        recipe_type: &Type<R>,
        _pair_type: &PairType<R>,
        typed: Typed,
    ) -> FurnaceTypedOutput<R> {
        let mut dynamic = typed.get_or_create_remainder();
        let recipes_used_size: i32 = dynamic.get("RecipesUsedSize").as_int(0);
        dynamic = dynamic.remove("RecipesUsedSize");
        let mut recipes: Vec<Pair<R, i32>> = Vec::new();

        let mut i: i32 = 0;
        while i < recipes_used_size {
            let location_key = format!("RecipeLocation{}", i);
            let amount_key = format!("RecipeAmount{}", i);
            let location_result = dynamic.get(&location_key).result();
            let amount: i32 = dynamic.get(&amount_key).as_int(0);
            if amount > 0 {
                if let Some(loc_dyn) = location_result {
                    if let Some((recipe, _rest)) = recipe_type.read(&loc_dyn) {
                        recipes.push(Pair(recipe, amount));
                    }
                }
            }
            dynamic = dynamic.remove(&location_key).remove(&amount_key);
            i = i.wrapping_add(1);
        }

        let empty_map = dynamic.empty_map();
        Pair(Either::Left(Pair(recipes, empty_map)), dynamic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_recipe_type() -> Type<String> {
        Type::new(|d| d.as_string().map(|s| (s.to_string(), Dynamic::default())))
    }

    #[test]
    fn basic_furnace_fix() {
        let fix = FurnaceRecipeFix::new(Schema, false);
        let mut map = BTreeMap::new();
        map.insert("RecipesUsedSize".to_string(), Dynamic::int(2));
        map.insert(
            "RecipeLocation0".to_string(),
            Dynamic::string("minecraft:iron_ingot"),
        );
        map.insert("RecipeAmount0".to_string(), Dynamic::int(5));
        map.insert(
            "RecipeLocation1".to_string(),
            Dynamic::string("minecraft:gold_ingot"),
        );
        map.insert("RecipeAmount1".to_string(), Dynamic::int(3));
        map.insert("CookTime".to_string(), Dynamic::int(100));

        let out = fix.update_furnace_contents(
            &string_recipe_type(),
            &PairType::default(),
            Typed {
                remainder: Dynamic::map(map),
            },
        );

        let Pair(Either::Left(Pair(recipes, empty_map)), remainder) = out else {
            panic!("expected Left");
        };
        assert_eq!(recipes.len(), 2);
        assert_eq!(recipes[0], Pair("minecraft:iron_ingot".to_string(), 5));
        assert_eq!(recipes[1], Pair("minecraft:gold_ingot".to_string(), 3));
        assert_eq!(empty_map, Dynamic::empty_map_value());
        assert_eq!(remainder.get("RecipesUsedSize").result(), None);
        assert_eq!(remainder.get("RecipeLocation0").result(), None);
        assert_eq!(remainder.get("RecipeAmount0").result(), None);
        assert_eq!(remainder.get("CookTime").as_int(0), 100);
    }

    #[test]
    fn zero_negative_and_missing_amounts_skipped() {
        let fix = FurnaceRecipeFix::new(Schema, false);
        let mut map = BTreeMap::new();
        map.insert("RecipesUsedSize".to_string(), Dynamic::int(4));
        map.insert("RecipeLocation0".to_string(), Dynamic::string("minecraft:stone"));
        map.insert("RecipeAmount0".to_string(), Dynamic::int(0));
        map.insert("RecipeLocation1".to_string(), Dynamic::string("minecraft:dirt"));
        map.insert("RecipeAmount1".to_string(), Dynamic::int(-5));
        map.insert("RecipeLocation2".to_string(), Dynamic::string("minecraft:sand"));
        map.insert("RecipeAmount2".to_string(), Dynamic::int(1));
        map.insert("RecipeLocation3".to_string(), Dynamic::string("minecraft:gravel"));
        // RecipeAmount3 missing -> defaults to 0

        let out = fix.update_furnace_contents(
            &string_recipe_type(),
            &PairType::default(),
            Typed {
                remainder: Dynamic::map(map),
            },
        );

        let Pair(Either::Left(Pair(recipes, _)), remainder) = out else {
            panic!("expected Left");
        };
        assert_eq!(recipes, vec![Pair("minecraft:sand".to_string(), 1)]);
        for i in 0..4 {
            assert_eq!(
                remainder.get(&format!("RecipeLocation{}", i)).result(),
                None
            );
            assert_eq!(remainder.get(&format!("RecipeAmount{}", i)).result(), None);
        }
    }

    #[test]
    fn negative_recipes_used_size_leaves_entries_untouched() {
        let fix = FurnaceRecipeFix::new(Schema, false);
        let mut map = BTreeMap::new();
        map.insert("RecipesUsedSize".to_string(), Dynamic::int(-1));
        map.insert("RecipeLocation0".to_string(), Dynamic::string("minecraft:stone"));
        map.insert("RecipeAmount0".to_string(), Dynamic::int(1));

        let out = fix.update_furnace_contents(
            &string_recipe_type(),
            &PairType::default(),
            Typed {
                remainder: Dynamic::map(map),
            },
        );

        let Pair(Either::Left(Pair(recipes, _)), remainder) = out else {
            panic!("expected Left");
        };
        assert!(recipes.is_empty());
        assert_eq!(remainder.get("RecipesUsedSize").result(), None);
        assert_eq!(
            remainder.get("RecipeLocation0").as_string(),
            Some("minecraft:stone")
        );
        assert_eq!(remainder.get("RecipeAmount0").as_int(0), 1);
    }

    #[test]
    fn missing_location_or_failed_read_skips_entry() {
        let fix = FurnaceRecipeFix::new(Schema, false);
        let mut map = BTreeMap::new();
        map.insert("RecipesUsedSize".to_string(), Dynamic::int(2));
        // index 0: missing RecipeLocation0
        map.insert("RecipeAmount0".to_string(), Dynamic::int(5));
        // index 1: location present but read fails (not a string)
        map.insert("RecipeLocation1".to_string(), Dynamic::int(42));
        map.insert("RecipeAmount1".to_string(), Dynamic::int(3));

        let out = fix.update_furnace_contents(
            &string_recipe_type(),
            &PairType::default(),
            Typed {
                remainder: Dynamic::map(map),
            },
        );

        let Pair(Either::Left(Pair(recipes, _)), remainder) = out else {
            panic!("expected Left");
        };
        assert!(recipes.is_empty());
        assert_eq!(remainder.get("RecipeLocation0").result(), None);
        assert_eq!(remainder.get("RecipeAmount0").result(), None);
        assert_eq!(remainder.get("RecipeLocation1").result(), None);
        assert_eq!(remainder.get("RecipeAmount1").result(), None);
    }

    #[test]
    fn integer_edge_values_and_missing_size_default() {
        let fix = FurnaceRecipeFix::new(Schema, false);
        let mut map = BTreeMap::new();
        map.insert("RecipesUsedSize".to_string(), Dynamic::int(2));
        map.insert("RecipeLocation0".to_string(), Dynamic::string("minecraft:coal"));
        map.insert("RecipeAmount0".to_string(), Dynamic::int(i32::MAX));
        map.insert("RecipeLocation1".to_string(), Dynamic::string("minecraft:charcoal"));
        map.insert("RecipeAmount1".to_string(), Dynamic::int(i32::MIN));

        let out = fix.update_furnace_contents(
            &string_recipe_type(),
            &PairType::default(),
            Typed {
                remainder: Dynamic::map(map),
            },
        );
        let Pair(Either::Left(Pair(recipes, _)), _) = out else {
            panic!("expected Left");
        };
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0], Pair("minecraft:coal".to_string(), i32::MAX));

        // Missing RecipesUsedSize defaults to 0
        let mut map2 = BTreeMap::new();
        map2.insert("CookTime".to_string(), Dynamic::int(200));
        let out2 = fix.update_furnace_contents(
            &string_recipe_type(),
            &PairType::default(),
            Typed {
                remainder: Dynamic::map(map2),
            },
        );
        let Pair(Either::Left(Pair(recipes2, _)), remainder2) = out2 else {
            panic!("expected Left");
        };
        assert!(recipes2.is_empty());
        assert_eq!(remainder2.get("CookTime").as_int(0), 200);
        assert_eq!(remainder2.get("RecipesUsedSize").result(), None);
    }
}
