//! Faithful port of `net.minecraft.core.DefaultedRegistry`.

/// Minimal self-contained stub for `net.minecraft.resources.Identifier`.
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

/// Minimal self-contained stub for `net.minecraft.core.Registry`.
pub trait Registry<T> {}

// JAVA: net.minecraft.core.DefaultedRegistry
pub trait DefaultedRegistry<T>: Registry<T> {
    // JAVA: net.minecraft.core.DefaultedRegistry#getKey
    fn get_key(&self, value: &T) -> Identifier;

    // JAVA: net.minecraft.core.DefaultedRegistry#getValue
    fn get_value(&self, key: Option<&Identifier>) -> T;

    // JAVA: net.minecraft.core.DefaultedRegistry#byId
    fn by_id(&self, id: i32) -> T;

    // JAVA: net.minecraft.core.DefaultedRegistry#getDefaultKey
    fn get_default_key(&self) -> Identifier;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyDefaultedRegistry {
        default_key: Identifier,
        default_value: i32,
    }

    impl Registry<i32> for DummyDefaultedRegistry {}

    impl DefaultedRegistry<i32> for DummyDefaultedRegistry {
        fn get_key(&self, value: &i32) -> Identifier {
            if *value == self.default_value {
                self.default_key.clone()
            } else {
                Identifier::new("test", "other")
            }
        }

        fn get_value(&self, key: Option<&Identifier>) -> i32 {
            match key {
                Some(k) if *k == self.default_key => self.default_value,
                _ => self.default_value,
            }
        }

        fn by_id(&self, id: i32) -> i32 {
            if id == 0 {
                self.default_value
            } else {
                id
            }
        }

        fn get_default_key(&self) -> Identifier {
            self.default_key.clone()
        }
    }

    fn make_registry() -> DummyDefaultedRegistry {
        DummyDefaultedRegistry {
            default_key: Identifier::new("minecraft", "air"),
            default_value: 0,
        }
    }

    #[test]
    fn get_default_key_returns_default() {
        let reg = make_registry();
        assert_eq!(reg.get_default_key(), Identifier::new("minecraft", "air"));
    }

    #[test]
    fn get_value_none_returns_default() {
        let reg = make_registry();
        assert_eq!(reg.get_value(None), 0);
    }

    #[test]
    fn get_value_known_key_returns_value() {
        let reg = make_registry();
        let key = Identifier::new("minecraft", "air");
        assert_eq!(reg.get_value(Some(&key)), 0);
    }

    #[test]
    fn by_id_zero_returns_default() {
        let reg = make_registry();
        assert_eq!(reg.by_id(0), 0);
    }

    #[test]
    fn by_id_nonzero_returns_id() {
        let reg = make_registry();
        assert_eq!(reg.by_id(5), 5);
        assert_eq!(reg.by_id(-1), -1);
    }

    #[test]
    fn get_key_for_default_value() {
        let reg = make_registry();
        assert_eq!(reg.get_key(&0), Identifier::new("minecraft", "air"));
    }
}
