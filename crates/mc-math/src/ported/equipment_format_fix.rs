use std::fmt::Debug;

/// Minimal Unit type mirroring `com.mojang.datafixers.util.Unit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Unit;

/// Minimal Either type mirroring `com.mojang.datafixers.util.Either`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Returns `Some(&R)` when this is `Right`, matching DFU `Either#right()`.
    pub fn right(&self) -> Option<&R> {
        match self {
            Either::Right(r) => Some(r),
            Either::Left(_) => None,
        }
    }

    /// Mirrors DFU `Either#map(Function, Function)`.
    pub fn map<B>(self, left: impl FnOnce(L) -> B, right: impl FnOnce(R) -> B) -> B {
        match self {
            Either::Left(l) => left(l),
            Either::Right(r) => right(r),
        }
    }
}

/// Old entity equipment format:
/// `ArmorItems`, `HandItems`, `body_armor_item`, `saddle`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldEntityEquipment<T> {
    pub armor_items: Vec<T>,
    pub hand_items: Vec<T>,
    pub body_armor_item: Either<T, Unit>,
    pub saddle: Either<T, Unit>,
}

/// New equipment slots in the fixed format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentSlots<T, D = Unit> {
    pub mainhand: Either<T, Unit>,
    pub offhand: Either<T, Unit>,
    pub feet: Either<T, Unit>,
    pub legs: Either<T, Unit>,
    pub chest: Either<T, Unit>,
    pub head: Either<T, Unit>,
    pub body: Either<T, Unit>,
    pub saddle: Either<T, Unit>,
    /// Mirrors the `DSL.remainderType()` / `new Dynamic(...)` tail in Java output.
    pub remainder: D,
}

/// New entity equipment format: optional `equipment` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntityEquipment<T, D = Unit> {
    pub equipment: Either<EquipmentSlots<T, D>, Unit>,
}

/// Port of `net.minecraft.util.datafix.fixes.EquipmentFormatFix`.
///
/// DFU schema/type-rewrite plumbing is represented by self-contained data types;
/// the equipment transformation logic is preserved 1:1.
#[derive(Debug, Clone, Copy, Default)]
pub struct EquipmentFormatFix;

impl EquipmentFormatFix {
    // JAVA: net.minecraft.util.datafix.fixes.EquipmentFormatFix#EquipmentFormatFix
    pub fn new() -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EquipmentFormatFix#makeRule
    pub fn make_rule<T: Clone, D: Default>(
        &self,
        old: OldEntityEquipment<T>,
        is_empty_item: &impl Fn(&T) -> bool,
    ) -> NewEntityEquipment<T, D> {
        self.fix(old, is_empty_item)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EquipmentFormatFix#fix
    pub fn fix<T: Clone, D: Default>(
        &self,
        old: OldEntityEquipment<T>,
        is_empty_item: &impl Fn(&T) -> bool,
    ) -> NewEntityEquipment<T, D> {
        let OldEntityEquipment {
            armor_items,
            hand_items,
            body_armor_item,
            saddle,
        } = old;

        let feet = get_item_from_list(0, &armor_items, is_empty_item);
        let legs = get_item_from_list(1, &armor_items, is_empty_item);
        let chest = get_item_from_list(2, &armor_items, is_empty_item);
        let head = get_item_from_list(3, &armor_items, is_empty_item);
        let mainhand = get_item_from_list(0, &hand_items, is_empty_item);
        let offhand = get_item_from_list(1, &hand_items, is_empty_item);

        if are_all_empty(&[
            &body_armor_item,
            &saddle,
            &feet,
            &legs,
            &chest,
            &head,
            &mainhand,
            &offhand,
        ]) {
            NewEntityEquipment {
                equipment: Either::Right(Unit),
            }
        } else {
            NewEntityEquipment {
                equipment: Either::Left(EquipmentSlots {
                    mainhand,
                    offhand,
                    feet,
                    legs,
                    chest,
                    head,
                    body: body_armor_item,
                    saddle,
                    remainder: D::default(),
                }),
            }
        }
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EquipmentFormatFix#areAllEmpty
fn are_all_empty<T>(eithers: &[&Either<T, Unit>]) -> bool {
    for either in eithers {
        if either.right().is_none() {
            return false;
        }
    }
    true
}

// JAVA: net.minecraft.util.datafix.fixes.EquipmentFormatFix#getItemFromList
fn get_item_from_list<T: Clone>(
    index: i32,
    list: &[T],
    is_empty: &impl Fn(&T) -> bool,
) -> Either<T, Unit> {
    let size = list.len() as i32;
    if index >= size {
        return Either::Right(Unit);
    }
    // Java: list.get(index); negative indices throw IndexOutOfBoundsException.
    // Rust indexing with a negative i32 cast to usize will panic on bounds check,
    // matching the Java exception behavior for invalid indices.
    let item = list[index as usize].clone();
    if is_empty(&item) {
        Either::Right(Unit)
    } else {
        Either::Left(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Item(Option<i32>);

    fn is_empty_item(item: &Item) -> bool {
        item.0.is_none()
    }

    #[test]
    fn get_item_from_list_out_of_bounds_returns_empty() {
        let list = vec![Item(Some(1))];
        assert_eq!(
            get_item_from_list(1, &list, &is_empty_item),
            Either::Right(Unit)
        );
        assert_eq!(
            get_item_from_list(i32::MAX, &list, &is_empty_item),
            Either::Right(Unit)
        );
    }

    #[test]
    fn get_item_from_list_negative_index_panics_like_java() {
        let list = vec![Item(Some(1))];
        let result = std::panic::catch_unwind(|| get_item_from_list(-1, &list, &is_empty_item));
        assert!(result.is_err());
    }

    #[test]
    fn get_item_from_list_empty_predicate_and_value() {
        let list = vec![Item(None), Item(Some(42))];
        assert_eq!(
            get_item_from_list(0, &list, &is_empty_item),
            Either::Right(Unit)
        );
        assert_eq!(
            get_item_from_list(1, &list, &is_empty_item),
            Either::Left(Item(Some(42)))
        );
    }

    #[test]
    fn are_all_empty_checks_all_right_unit() {
        let a: Either<i32, Unit> = Either::Right(Unit);
        let b: Either<i32, Unit> = Either::Right(Unit);
        let c: Either<i32, Unit> = Either::Left(7);
        assert!(are_all_empty::<i32>(&[&a, &b]));
        assert!(!are_all_empty::<i32>(&[&a, &c]));
    }

    #[test]
    fn fix_all_empty_returns_no_equipment() {
        let fix = EquipmentFormatFix::new();
        let old = OldEntityEquipment {
            armor_items: vec![Item(None), Item(None), Item(None), Item(None)],
            hand_items: vec![Item(None), Item(None)],
            body_armor_item: Either::Right(Unit),
            saddle: Either::Right(Unit),
        };
        let new: NewEntityEquipment<Item, Unit> = fix.make_rule(old, &is_empty_item);
        assert_eq!(new.equipment, Either::Right(Unit));
    }

    #[test]
    fn fix_maps_slots_and_preserves_partial_equipment() {
        let fix = EquipmentFormatFix::new();
        let old = OldEntityEquipment {
            armor_items: vec![Item(Some(10)), Item(None), Item(Some(30))], // feet, legs, chest; head missing
            hand_items: vec![Item(Some(1))],                               // mainhand only
            body_armor_item: Either::Left(Item(Some(99))),
            saddle: Either::Right(Unit),
        };

        let new: NewEntityEquipment<Item, Unit> = fix.fix(old, &is_empty_item);
        let slots = match new.equipment {
            Either::Left(slots) => slots,
            Either::Right(_) => panic!("expected equipment"),
        };

        assert_eq!(slots.mainhand, Either::Left(Item(Some(1))));
        assert_eq!(slots.offhand, Either::Right(Unit));
        assert_eq!(slots.feet, Either::Left(Item(Some(10))));
        assert_eq!(slots.legs, Either::Right(Unit));
        assert_eq!(slots.chest, Either::Left(Item(Some(30))));
        assert_eq!(slots.head, Either::Right(Unit));
        assert_eq!(slots.body, Either::Left(Item(Some(99))));
        assert_eq!(slots.saddle, Either::Right(Unit));
        assert_eq!(slots.remainder, Unit);
    }
}
