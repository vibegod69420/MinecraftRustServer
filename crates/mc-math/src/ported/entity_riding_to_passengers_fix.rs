use serde_json::Value;

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#fix
pub fn fix_entity_riding_to_passengers(value: &mut Value) {
    if let Value::Object(map) = value {
        if map.contains_key("Riding") {
            let mut current = std::mem::take(value);
            let mut chain: Vec<Value> = Vec::new();

            loop {
                match current {
                    Value::Object(mut obj) => {
                        let riding = obj.remove("Riding");
                        chain.push(Value::Object(obj));
                        match riding {
                            Some(Value::Null) | None => break,
                            Some(next) => current = next,
                        }
                    }
                    other => {
                        chain.push(other);
                        break;
                    }
                }
            }

            if chain.is_empty() {
                return;
            }

            let mut root = chain.pop().unwrap();
            let mut acc: Option<Value> = None;

            for mut node in chain {
                if let Some(passenger) = acc.take() {
                    if let Value::Object(ref mut obj) = node {
                        obj.insert("Passengers".to_string(), Value::Array(vec![passenger]));
                    }
                }
                acc = Some(node);
            }

            if let Some(passenger) = acc {
                if let Value::Object(ref mut obj) = root {
                    obj.insert("Passengers".to_string(), Value::Array(vec![passenger]));
                }
            }

            *value = root;
            fix_entity_riding_to_passengers(value);
            return;
        }
    }

    match value {
        Value::Object(map) => {
            for child in map.values_mut() {
                fix_entity_riding_to_passengers(child);
            }
        }
        Value::Array(items) => {
            for child in items.iter_mut() {
                fix_entity_riding_to_passengers(child);
            }
        }
        _ => {}
    }
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#fix
pub fn fix(value: &mut Value) {
    fix_entity_riding_to_passengers(value);
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#fix
pub fn apply(value: &mut Value) {
    fix_entity_riding_to_passengers(value);
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#fix
pub fn transform(value: &mut Value) {
    fix_entity_riding_to_passengers(value);
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#fix
pub fn entity_riding_to_passengers(value: &mut Value) {
    fix_entity_riding_to_passengers(value);
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityTree<E = Value> {
    pub type_name: String,
    pub riding: Option<Box<EntityTree<E>>>,
    pub passengers: Option<Vec<EntityTree<E>>>,
    pub entity: E,
}

pub type OldEntityTree<E> = EntityTree<E>;
pub type NewEntityTree<E> = EntityTree<E>;

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#cap
pub fn convert_riding_to_passengers<E>(mut old: EntityTree<E>) -> EntityTree<E> {
    let mut chain = Vec::new();
    let mut current = old;

    loop {
        let riding = current.riding.take();
        chain.push(current);
        match riding {
            Some(boxed) => current = *boxed,
            None => break,
        }
    }

    if chain.is_empty() {
        return EntityTree {
            type_name: String::new(),
            riding: None,
            passengers: None,
            entity: panic!("empty entity tree chain"),
        };
    }

    let mut root = chain.pop().unwrap();
    let mut acc: Option<EntityTree<E>> = None;

    for mut node in chain {
        if let Some(passenger) = acc.take() {
            node.passengers = Some(vec![passenger]);
        }
        node.riding = None;
        acc = Some(node);
    }

    if let Some(passenger) = acc {
        root.passengers = Some(vec![passenger]);
    }
    root.riding = None;
    root
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#cap
pub fn fix_entity_tree<E>(old: EntityTree<E>) -> EntityTree<E> {
    convert_riding_to_passengers(old)
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#cap
pub fn apply_fix<E>(old: EntityTree<E>) -> EntityTree<E> {
    convert_riding_to_passengers(old)
}

// JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#cap
pub fn transform_entity_tree<E>(old: EntityTree<E>) -> EntityTree<E> {
    convert_riding_to_passengers(old)
}

#[derive(Debug, Clone, Default)]
pub struct EntityRidingToPassengersFix;

impl EntityRidingToPassengersFix {
    // JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#<init>
    pub fn new<A, B>(_input_schema: A, _output_schema: B, _typed: bool) -> Self {
        Self
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#makeRule
    pub fn make_rule(&self) {}

    // JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#cap
    pub fn fix<E>(old: EntityTree<E>) -> EntityTree<E> {
        convert_riding_to_passengers(old)
    }

    // JAVA: net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix#cap
    pub fn fix_value(value: &mut Value) {
        fix_entity_riding_to_passengers(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn converts_simple_riding_chain() {
        let mut value = json!({"id": "player", "Riding": {"id": "pig", "Health": 10.0}});
        fix_entity_riding_to_passengers(&mut value);
        assert_eq!(
            value,
            json!({"id": "pig", "Health": 10.0, "Passengers": [{"id": "player"}]})
        );
    }

    #[test]
    fn converts_three_level_chain() {
        let mut value = json!({"id": "a", "Riding": {"id": "b", "Riding": {"id": "c"}}});
        fix_entity_riding_to_passengers(&mut value);
        assert_eq!(
            value,
            json!({"id": "c", "Passengers": [{"id": "b", "Passengers": [{"id": "a"}]}]})
        );
    }

    #[test]
    fn leaves_unridden_entity_unchanged() {
        let mut value = json!({"id": "villager", "Health": 20});
        let original = value.clone();
        fix_entity_riding_to_passengers(&mut value);
        assert_eq!(value, original);
    }

    #[test]
    fn handles_null_riding() {
        let mut value = json!({"id": "player", "Riding": null});
        fix_entity_riding_to_passengers(&mut value);
        assert_eq!(value, json!({"id": "player"}));
    }

    #[test]
    fn converts_array_of_entities() {
        let mut value = json!([{"id": "a", "Riding": {"id": "b"}}, {"id": "c"}]);
        fix_entity_riding_to_passengers(&mut value);
        assert_eq!(
            value,
            json!([{"id": "b", "Passengers": [{"id": "a"}]}, {"id": "c"}])
        );
    }

    #[test]
    fn struct_conversion_preserves_chain() {
        let old = EntityTree {
            type_name: "minecraft:entity".to_string(),
            riding: Some(Box::new(EntityTree {
                type_name: "minecraft:entity".to_string(),
                riding: Some(Box::new(EntityTree {
                    type_name: "minecraft:entity".to_string(),
                    riding: None,
                    passengers: None,
                    entity: 3,
                })),
                passengers: None,
                entity: 2,
            })),
            passengers: None,
            entity: 1,
        };

        let new = convert_riding_to_passengers(old);
        assert_eq!(new.entity, 3);
        let passengers = new.passengers.unwrap();
        assert_eq!(passengers.len(), 1);
        assert_eq!(passengers[0].entity, 2);
        assert_eq!(passengers[0].passengers.as_ref().unwrap()[0].entity, 1);
        assert!(new.riding.is_none());
    }
}
