use std::any::TypeId;
use std::collections::HashMap;

pub const NO_ID_VALUE: i32 = -1;

/// Faithful port of `net.minecraft.util.ClassTreeIdRegistry`.
///
/// Java uses `Class<?>` for identity and superclass traversal. This port uses
/// `TypeId` for identity and an explicit superclass chain (in traversal order,
/// excluding `java.lang.Object`) to model the hierarchy.
#[derive(Debug)]
pub struct ClassTreeIdRegistry {
    class_to_last_id_cache: HashMap<TypeId, i32>,
}

impl Default for ClassTreeIdRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassTreeIdRegistry {
    pub fn new() -> Self {
        Self {
            class_to_last_id_cache: HashMap::new(),
        }
    }

    // JAVA: net.minecraft.util.ClassTreeIdRegistry#getLastIdFor
    pub fn get_last_id_for(&self, class: TypeId, superclasses: &[TypeId]) -> i32 {
        let id = self
            .class_to_last_id_cache
            .get(&class)
            .copied()
            .unwrap_or(NO_ID_VALUE);
        if id != NO_ID_VALUE {
            return id;
        }

        for &superclass in superclasses {
            let id = self
                .class_to_last_id_cache
                .get(&superclass)
                .copied()
                .unwrap_or(NO_ID_VALUE);
            if id != NO_ID_VALUE {
                return id;
            }
        }

        NO_ID_VALUE
    }

    // JAVA: net.minecraft.util.ClassTreeIdRegistry#getCount
    pub fn get_count(&self, class: TypeId, superclasses: &[TypeId]) -> i32 {
        self.get_last_id_for(class, superclasses).wrapping_add(1)
    }

    // JAVA: net.minecraft.util.ClassTreeIdRegistry#define
    pub fn define(&mut self, class: TypeId, superclasses: &[TypeId]) -> i32 {
        let last_id = self.get_last_id_for(class, superclasses);
        let id = if last_id == NO_ID_VALUE {
            0
        } else {
            last_id.wrapping_add(1)
        };
        self.class_to_last_id_cache.insert(class, id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    struct Base;
    struct Child;
    struct GrandChild;
    struct Unrelated;

    fn id<T: 'static>() -> TypeId {
        TypeId::of::<T>()
    }

    #[test]
    fn define_and_get_last_id() {
        let mut reg = ClassTreeIdRegistry::new();
        let base = id::<Base>();

        assert_eq!(reg.get_last_id_for(base, &[]), NO_ID_VALUE);
        assert_eq!(reg.get_count(base, &[]), 0);

        let defined = reg.define(base, &[]);
        assert_eq!(defined, 0);
        assert_eq!(reg.get_last_id_for(base, &[]), 0);
        assert_eq!(reg.get_count(base, &[]), 1);
    }

    #[test]
    fn superclass_inheritance_and_chain_walk() {
        let mut reg = ClassTreeIdRegistry::new();
        let base = id::<Base>();
        let child = id::<Child>();
        let grandchild = id::<GrandChild>();

        assert_eq!(reg.define(base, &[]), 0);
        // Child extends Base: inherits base id before being defined.
        assert_eq!(reg.get_last_id_for(child, &[base]), 0);
        assert_eq!(reg.get_count(child, &[base]), 1);

        // GrandChild extends Child extends Base: walks chain and finds base.
        assert_eq!(reg.get_last_id_for(grandchild, &[child, base]), 0);

        // Defining child gives it its own id; grandchild now finds child.
        assert_eq!(reg.define(child, &[base]), 1);
        assert_eq!(reg.get_last_id_for(child, &[base]), 1);
        assert_eq!(reg.get_last_id_for(grandchild, &[child, base]), 1);
        assert_eq!(reg.get_last_id_for(base, &[]), 0);
    }

    #[test]
    fn unknown_class_returns_no_id() {
        let reg = ClassTreeIdRegistry::new();
        let unrelated = id::<Unrelated>();
        let base = id::<Base>();

        assert_eq!(reg.get_last_id_for(unrelated, &[base]), NO_ID_VALUE);
        assert_eq!(reg.get_count(unrelated, &[base]), 0);
    }

    #[test]
    fn define_same_class_increments() {
        let mut reg = ClassTreeIdRegistry::new();
        let base = id::<Base>();

        assert_eq!(reg.define(base, &[]), 0);
        assert_eq!(reg.define(base, &[]), 1);
        assert_eq!(reg.define(base, &[]), 2);
        assert_eq!(reg.get_last_id_for(base, &[]), 2);
    }

    #[test]
    fn wrapping_overflow_semantics() {
        let mut reg = ClassTreeIdRegistry::new();
        let base = id::<Base>();
        let child = id::<Child>();

        // Seed cache directly to simulate i32 overflow boundary.
        reg.class_to_last_id_cache.insert(base, i32::MAX);

        // getCount does last_id + 1 with Java int semantics.
        assert_eq!(reg.get_count(base, &[]), i32::MIN);

        // define uses last_id + 1 with Java int semantics.
        let defined = reg.define(child, &[base]);
        assert_eq!(defined, i32::MIN);
        assert_eq!(reg.get_last_id_for(child, &[base]), i32::MIN);
    }
}
