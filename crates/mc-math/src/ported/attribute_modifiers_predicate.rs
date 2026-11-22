use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            path: path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub id: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderSet<T> {
    pub values: Vec<T>,
}

impl<T: PartialEq> HolderSet<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.values.iter().any(|v| v == value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Doubles {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Doubles {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn matches(&self, value: f64) -> bool {
        if let Some(min) = self.min {
            if !(min <= value) {
                return false;
            }
        }
        if let Some(max) = self.max {
            if !(max >= value) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlotGroup {
    Any,
    Mainhand,
    Offhand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    pub id: Identifier,
    pub amount: f64,
    pub operation: Operation,
}

impl AttributeModifier {
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn operation(&self) -> Operation {
        self.operation
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemAttributeModifiersEntry {
    pub attribute: Attribute,
    pub modifier: AttributeModifier,
    pub slot: EquipmentSlotGroup,
}

impl ItemAttributeModifiersEntry {
    pub fn attribute(&self) -> &Attribute {
        &self.attribute
    }

    pub fn modifier(&self) -> &AttributeModifier {
        &self.modifier
    }

    pub fn slot(&self) -> EquipmentSlotGroup {
        self.slot
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemAttributeModifiers {
    pub entries: Vec<ItemAttributeModifiersEntry>,
}

impl ItemAttributeModifiers {
    pub fn modifiers(&self) -> &[ItemAttributeModifiersEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataComponentType<T> {
    pub key: &'static str,
    _marker: PhantomData<T>,
}

pub const DATA_COMPONENT_ATTRIBUTE_MODIFIERS: DataComponentType<ItemAttributeModifiers> =
    DataComponentType {
        key: "minecraft:attribute_modifiers",
        _marker: PhantomData,
    };

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionPredicate<P> {
    pub entries: Vec<P>,
}

impl CollectionPredicate<EntryPredicate> {
    pub fn test(&self, items: &[ItemAttributeModifiersEntry]) -> bool {
        self.entries
            .iter()
            .all(|p| items.iter().any(|item| p.test(item)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifiersPredicate {
    pub modifiers: Option<CollectionPredicate<EntryPredicate>>,
}

impl AttributeModifiersPredicate {
    // JAVA: net.minecraft.core.component.predicates.AttributeModifiersPredicate#componentType
    pub fn component_type(&self) -> DataComponentType<ItemAttributeModifiers> {
        DATA_COMPONENT_ATTRIBUTE_MODIFIERS
    }

    // JAVA: net.minecraft.core.component.predicates.AttributeModifiersPredicate#matches
    pub fn matches(&self, value: &ItemAttributeModifiers) -> bool {
        match &self.modifiers {
            Some(pred) => pred.test(value.modifiers()),
            None => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryPredicate {
    pub attribute: Option<HolderSet<Attribute>>,
    pub id: Option<Identifier>,
    pub amount: Doubles,
    pub operation: Option<Operation>,
    pub slot: Option<EquipmentSlotGroup>,
}

impl EntryPredicate {
    // JAVA: net.minecraft.core.component.predicates.AttributeModifiersPredicate$EntryPredicate#test
    pub fn test(&self, entry: &ItemAttributeModifiersEntry) -> bool {
        if let Some(attribute) = &self.attribute {
            if !attribute.contains(entry.attribute()) {
                return false;
            }
        }
        if let Some(id) = &self.id {
            if id != entry.modifier().id() {
                return false;
            }
        }
        if !self.amount.matches(entry.modifier().amount()) {
            return false;
        }
        if let Some(operation) = &self.operation {
            if *operation != entry.modifier().operation() {
                return false;
            }
        }
        if let Some(slot) = &self.slot {
            if *slot != entry.slot() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(ns: &str, path: &str) -> Identifier {
        Identifier::new(ns, path)
    }

    fn make_entry(
        amount: f64,
        operation: Operation,
        slot: EquipmentSlotGroup,
    ) -> ItemAttributeModifiersEntry {
        ItemAttributeModifiersEntry {
            attribute: Attribute {
                id: id("minecraft", "attack_damage"),
            },
            modifier: AttributeModifier {
                id: id("minecraft", "base"),
                amount,
                operation,
            },
            slot,
        }
    }

    #[test]
    fn matches_true_when_no_modifiers_predicate() {
        let pred = AttributeModifiersPredicate { modifiers: None };
        let item = ItemAttributeModifiers { entries: vec![] };
        assert!(pred.matches(&item));
        assert_eq!(
            pred.component_type(),
            DATA_COMPONENT_ATTRIBUTE_MODIFIERS
        );
    }

    #[test]
    fn entry_predicate_filters_by_attribute_id_operation_and_slot() {
        let entry = make_entry(5.0, Operation::AddValue, EquipmentSlotGroup::Mainhand);
        let pred = EntryPredicate {
            attribute: Some(HolderSet {
                values: vec![Attribute {
                    id: id("minecraft", "attack_damage"),
                }],
            }),
            id: Some(id("minecraft", "base")),
            amount: Doubles::ANY,
            operation: Some(Operation::AddValue),
            slot: Some(EquipmentSlotGroup::Mainhand),
        };
        assert!(pred.test(&entry));

        let wrong_attr = EntryPredicate {
            attribute: Some(HolderSet {
                values: vec![Attribute {
                    id: id("minecraft", "armor"),
                }],
            }),
            ..pred.clone()
        };
        assert!(!wrong_attr.test(&entry));

        let wrong_op = EntryPredicate {
            operation: Some(Operation::AddMultipliedBase),
            ..pred.clone()
        };
        assert!(!wrong_op.test(&entry));

        let wrong_slot = EntryPredicate {
            slot: Some(EquipmentSlotGroup::Offhand),
            ..pred
        };
        assert!(!wrong_slot.test(&entry));
    }

    #[test]
    fn entry_predicate_amount_bounds_negatives_and_float_bits() {
        let neg_entry = make_entry(-3.5, Operation::AddValue, EquipmentSlotGroup::Any);
        let pred = EntryPredicate {
            attribute: None,
            id: None,
            amount: Doubles {
                min: Some(-10.0),
                max: Some(-1.0),
            },
            operation: None,
            slot: None,
        };
        assert!(pred.test(&neg_entry));

        let nan_entry = make_entry(f64::NAN, Operation::AddValue, EquipmentSlotGroup::Any);
        let bounded_inf = EntryPredicate {
            amount: Doubles {
                min: Some(f64::NEG_INFINITY),
                max: Some(f64::INFINITY),
            },
            ..pred.clone()
        };
        assert!(!bounded_inf.test(&nan_entry));
        assert!(EntryPredicate {
            amount: Doubles::ANY,
            ..pred.clone()
        }
        .test(&nan_entry));

        let neg_zero = f64::from_bits(0x8000_0000_0000_0000u64);
        let zero_entry = make_entry(neg_zero, Operation::AddValue, EquipmentSlotGroup::Any);
        let zero_pred = EntryPredicate {
            amount: Doubles {
                min: Some(0.0),
                max: Some(0.0),
            },
            ..pred
        };
        assert!(zero_pred.test(&zero_entry));
    }

    #[test]
    fn matches_with_collection_predicate_requires_all_entries() {
        let entry_a = make_entry(1.0, Operation::AddValue, EquipmentSlotGroup::Mainhand);
        let entry_b = ItemAttributeModifiersEntry {
            attribute: Attribute {
                id: id("minecraft", "armor"),
            },
            modifier: AttributeModifier {
                id: id("minecraft", "bonus"),
                amount: 2.0,
                operation: Operation::AddValue,
            },
            slot: EquipmentSlotGroup::Chest,
        };
        let item = ItemAttributeModifiers {
            entries: vec![entry_a, entry_b],
        };

        let pred = AttributeModifiersPredicate {
            modifiers: Some(CollectionPredicate {
                entries: vec![EntryPredicate {
                    attribute: Some(HolderSet {
                        values: vec![Attribute {
                            id: id("minecraft", "armor"),
                        }],
                    }),
                    id: None,
                    amount: Doubles::ANY,
                    operation: None,
                    slot: None,
                }],
            }),
        };
        assert!(pred.matches(&item));

        let miss = AttributeModifiersPredicate {
            modifiers: Some(CollectionPredicate {
                entries: vec![EntryPredicate {
                    attribute: Some(HolderSet {
                        values: vec![Attribute {
                            id: id("minecraft", "luck"),
                        }],
                    }),
                    id: None,
                    amount: Doubles::ANY,
                    operation: None,
                    slot: None,
                }],
            }),
        };
        assert!(!miss.matches(&item));
    }

    #[test]
    fn entry_predicate_id_mismatch_and_empty_collection_fails() {
        let entry = make_entry(1.0, Operation::AddValue, EquipmentSlotGroup::Any);
        let pred = EntryPredicate {
            attribute: None,
            id: Some(id("minecraft", "other")),
            amount: Doubles::ANY,
            operation: None,
            slot: None,
        };
        assert!(!pred.test(&entry));

        let empty_item = ItemAttributeModifiers { entries: vec![] };
        let coll_pred = AttributeModifiersPredicate {
            modifiers: Some(CollectionPredicate {
                entries: vec![pred],
            }),
        };
        assert!(!coll_pred.matches(&empty_item));
    }
}
