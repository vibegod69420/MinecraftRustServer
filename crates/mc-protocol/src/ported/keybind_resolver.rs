use std::sync::{OnceLock, RwLock};

/// Minimal self-contained component representation for keybind resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub text: String,
}

impl Component {
    pub fn literal(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}

pub type ComponentSupplier = Box<dyn Fn() -> Component + Send + Sync>;
pub type KeyResolver = Box<dyn Fn(&str) -> ComponentSupplier + Send + Sync>;

static KEY_RESOLVER: OnceLock<RwLock<KeyResolver>> = OnceLock::new();

fn default_resolver() -> KeyResolver {
    Box::new(|key: &str| {
        let key = key.to_string();
        Box::new(move || Component::literal(&key))
    })
}

fn resolver_lock() -> &'static RwLock<KeyResolver> {
    KEY_RESOLVER.get_or_init(|| RwLock::new(default_resolver()))
}

// JAVA: net.minecraft.network.chat.contents.KeybindResolver#setKeyResolver
pub fn set_key_resolver(resolver: KeyResolver) {
    let mut guard = resolver_lock()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = resolver;
}

/// Resolves a keybind name using the currently configured resolver.
pub fn resolve_key(key: &str) -> ComponentSupplier {
    let guard = resolver_lock()
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    (guard)(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_default_resolver<T>(f: impl FnOnce() -> T) -> T {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_key_resolver(default_resolver());
        f()
    }

    #[test]
    fn default_resolver_returns_literal_key() {
        with_default_resolver(|| {
            let supplier = resolve_key("key.forward");
            assert_eq!(supplier(), Component::literal("key.forward"));
        });
    }

    #[test]
    fn set_key_resolver_overrides_default() {
        with_default_resolver(|| {
            set_key_resolver(Box::new(|key: &str| {
                let resolved = format!("resolved:{key}");
                Box::new(move || Component::literal(&resolved))
            }));
            let supplier = resolve_key("key.jump");
            assert_eq!(supplier(), Component::literal("resolved:key.jump"));
        });
    }

    #[test]
    fn resolver_handles_empty_and_special_keys() {
        with_default_resolver(|| {
            assert_eq!(resolve_key("")(), Component::literal(""));
            assert_eq!(
                resolve_key("key.special.§")(),
                Component::literal("key.special.§")
            );
        });
    }

    #[test]
    fn supplier_captures_value_at_resolution_time() {
        with_default_resolver(|| {
            let first = resolve_key("first");
            set_key_resolver(Box::new(|_: &str| {
                Box::new(|| Component::literal("other"))
            }));
            let second = resolve_key("second");
            assert_eq!(first(), Component::literal("first"));
            assert_eq!(second(), Component::literal("other"));
        });
    }
}
