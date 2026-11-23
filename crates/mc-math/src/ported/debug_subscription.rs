use std::fmt;
use std::sync::Arc;

/// Minimal placeholder for Minecraft's `RegistryFriendlyByteBuf`.
/// The full buffer/serialization API is not modeled in this self-contained port.
pub struct RegistryFriendlyByteBuf;

/// Minimal placeholder for Minecraft's `StreamCodec<B, V>`.
/// The full encode/decode/map/dispatch API is not modeled in this self-contained port.
pub trait StreamCodec<B, V: ?Sized>: Send + Sync {}

/// Placeholder for registry-dispatched stream codecs used by `Event`/`Update`.
/// In the full Minecraft runtime this is produced by
/// `ByteBufCodecs.registry(...).dispatch(...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchStreamCodec;

pub struct DebugSubscription<T: 'static> {
    value_stream_codec: Option<Arc<dyn StreamCodec<RegistryFriendlyByteBuf, T>>>,
    expire_after_ticks: i32,
}

impl<T: 'static> Clone for DebugSubscription<T> {
    fn clone(&self) -> Self {
        Self {
            value_stream_codec: self.value_stream_codec.clone(),
            expire_after_ticks: self.expire_after_ticks,
        }
    }
}

impl<T: 'static> fmt::Debug for DebugSubscription<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DebugSubscription")
            .field("has_value_stream_codec", &self.value_stream_codec.is_some())
            .field("expire_after_ticks", &self.expire_after_ticks)
            .finish()
    }
}

impl<T: 'static> DebugSubscription<T> {
    pub const DOES_NOT_EXPIRE: i32 = 0;

    // JAVA: net.minecraft.util.debug.DebugSubscription#<init>
    pub fn new(
        value_stream_codec: Option<Arc<dyn StreamCodec<RegistryFriendlyByteBuf, T>>>,
        expire_after_ticks: i32,
    ) -> Self {
        Self {
            value_stream_codec,
            expire_after_ticks,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#<init>
    pub fn with_no_expiry(
        value_stream_codec: Option<Arc<dyn StreamCodec<RegistryFriendlyByteBuf, T>>>,
    ) -> Self {
        Self::new(value_stream_codec, Self::DOES_NOT_EXPIRE)
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#packUpdate
    pub fn pack_update(&self, value: Option<T>) -> Update<T> {
        Update {
            subscription: self.clone(),
            value,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#emptyUpdate
    pub fn empty_update(&self) -> Update<T> {
        Update {
            subscription: self.clone(),
            value: None,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#packEvent
    pub fn pack_event(&self, value: T) -> Event<T> {
        Event {
            subscription: self.clone(),
            value,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#toString
    pub fn registered_name_fallback(&self) -> String {
        // In full Minecraft this uses Util.getRegisteredName(BuiltInRegistries.DEBUG_SUBSCRIPTION, this).
        // Self-contained port returns a descriptive fallback without registry access.
        format!("DebugSubscription(expireAfterTicks={})", self.expire_after_ticks)
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#valueStreamCodec
    pub fn value_stream_codec(&self) -> Option<&Arc<dyn StreamCodec<RegistryFriendlyByteBuf, T>>> {
        self.value_stream_codec.as_ref()
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription#expireAfterTicks
    pub fn expire_after_ticks(&self) -> i32 {
        self.expire_after_ticks
    }
}

impl<T: 'static> fmt::Display for DebugSubscription<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.registered_name_fallback())
    }
}

pub struct Event<T: 'static> {
    pub subscription: DebugSubscription<T>,
    pub value: T,
}

impl<T: 'static> Event<T> {
    /// Placeholder for `Event.STREAM_CODEC` in Java.
    /// In the full Minecraft runtime this is a registry-dispatched stream codec.
    pub const STREAM_CODEC: DispatchStreamCodec = DispatchStreamCodec;

    // JAVA: net.minecraft.util.debug.DebugSubscription$Event#subscription
    pub fn subscription(&self) -> &DebugSubscription<T> {
        &self.subscription
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription$Event#value
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for Event<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("subscription", &self.subscription)
            .field("value", &self.value)
            .finish()
    }
}

pub struct Update<T: 'static> {
    pub subscription: DebugSubscription<T>,
    pub value: Option<T>,
}

impl<T: 'static> Update<T> {
    /// Placeholder for `Update.STREAM_CODEC` in Java.
    /// In the full Minecraft runtime this is a registry-dispatched stream codec.
    pub const STREAM_CODEC: DispatchStreamCodec = DispatchStreamCodec;

    // JAVA: net.minecraft.util.debug.DebugSubscription$Update#subscription
    pub fn subscription(&self) -> &DebugSubscription<T> {
        &self.subscription
    }

    // JAVA: net.minecraft.util.debug.DebugSubscription$Update#value
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for Update<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Update")
            .field("subscription", &self.subscription)
            .field("value", &self.value)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug)]
    struct DummyCodec;
    impl StreamCodec<RegistryFriendlyByteBuf, i32> for DummyCodec {}
    impl StreamCodec<RegistryFriendlyByteBuf, String> for DummyCodec {}

    #[test]
    fn constructors_and_accessors() {
        let codec: Arc<dyn StreamCodec<RegistryFriendlyByteBuf, i32>> = Arc::new(DummyCodec);
        let sub = DebugSubscription::new(Some(codec.clone()), 20);
        assert_eq!(sub.expire_after_ticks(), 20);
        assert!(sub.value_stream_codec().is_some());
        assert!(Arc::ptr_eq(sub.value_stream_codec().unwrap(), &codec));

        let sub_no_expiry = DebugSubscription::<i32>::with_no_expiry(None);
        assert_eq!(sub_no_expiry.expire_after_ticks(), DebugSubscription::<i32>::DOES_NOT_EXPIRE);
        assert!(sub_no_expiry.value_stream_codec().is_none());
    }

    #[test]
    fn pack_update_and_empty_update() {
        let sub = DebugSubscription::<i32>::new(None, 5);
        let packed = sub.pack_update(Some(42));
        assert_eq!(packed.value(), Some(&42));
        assert_eq!(packed.subscription().expire_after_ticks(), 5);

        let empty = sub.empty_update();
        assert_eq!(empty.value(), None);
        assert_eq!(empty.subscription().expire_after_ticks(), 5);
    }

    #[test]
    fn pack_event_holds_value() {
        let sub = DebugSubscription::<String>::with_no_expiry(None);
        let event = sub.pack_event("hello".to_string());
        assert_eq!(event.value(), "hello");
        assert_eq!(event.subscription().expire_after_ticks(), 0);
    }

    #[test]
    fn clone_is_independent_and_preserves_codec() {
        let codec: Arc<dyn StreamCodec<RegistryFriendlyByteBuf, i32>> = Arc::new(DummyCodec);
        let sub = DebugSubscription::new(Some(codec.clone()), 7);
        let cloned = sub.clone();
        assert_eq!(cloned.expire_after_ticks(), 7);
        assert!(Arc::ptr_eq(cloned.value_stream_codec().unwrap(), &codec));
        // Packed records get their own subscription clone
        let upd = sub.pack_update(Some(1));
        assert_eq!(upd.subscription().expire_after_ticks(), 7);
    }

    #[test]
    fn to_string_fallback_and_display() {
        let sub = DebugSubscription::<i32>::new(None, 123);
        let s = sub.registered_name_fallback();
        assert_eq!(s, "DebugSubscription(expireAfterTicks=123)");
        assert_eq!(sub.to_string(), s);
    }

    #[test]
    fn negative_and_edge_expiry_values() {
        let sub = DebugSubscription::<i32>::new(None, -1);
        assert_eq!(sub.expire_after_ticks(), -1);
        let sub_max = DebugSubscription::<i32>::new(None, i32::MAX);
        assert_eq!(sub_max.expire_after_ticks(), i32::MAX);
        let sub_min = DebugSubscription::<i32>::new(None, i32::MIN);
        assert_eq!(sub_min.expire_after_ticks(), i32::MIN);
    }
}
