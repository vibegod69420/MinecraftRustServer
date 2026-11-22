//! Port of `net.minecraft.network.syncher.EntityDataSerializer`.

use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Minimal stub for `net.minecraft.network.RegistryFriendlyByteBuf`.
/// This module is self-contained; the real type lives elsewhere.
#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf;

/// Minimal stub for `net.minecraft.network.codec.StreamCodec`.
/// The real codec carries encode/decode logic; this stub preserves the type shape.
pub struct StreamCodec<B, V> {
    _marker: PhantomData<fn(B) -> V>,
}

impl<B, V> StreamCodec<B, V> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<B, V> Default for StreamCodec<B, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B, V> Clone for StreamCodec<B, V> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<B, V> fmt::Debug for StreamCodec<B, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StreamCodec(..)")
    }
}

impl<B, V> PartialEq for StreamCodec<B, V> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<B, V> Eq for StreamCodec<B, V> {}

/// Minimal stub for `net.minecraft.network.syncher.EntityDataAccessor`.
/// This module is self-contained; the canonical type lives in `entity_data_accessor.rs`.
pub struct EntityDataAccessor<T> {
    pub id: i32,
    pub serializer: Arc<dyn EntityDataSerializer<T>>,
}

impl<T> EntityDataAccessor<T> {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn serializer(&self) -> &Arc<dyn EntityDataSerializer<T>> {
        &self.serializer
    }
}

impl<T> Clone for EntityDataAccessor<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            serializer: Arc::clone(&self.serializer),
        }
    }
}

impl<T> PartialEq for EntityDataAccessor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for EntityDataAccessor<T> {}

impl<T> std::hash::Hash for EntityDataAccessor<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> fmt::Display for EntityDataAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<entity data: {}>", self.id)
    }
}

impl<T> fmt::Debug for EntityDataAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityDataAccessor")
            .field("id", &self.id)
            .finish()
    }
}

/// Port of `net.minecraft.network.syncher.EntityDataSerializer`.
pub trait EntityDataSerializer<T>: Send + Sync {
    // JAVA: net.minecraft.network.syncher.EntityDataSerializer#codec
    fn codec(&self) -> StreamCodec<RegistryFriendlyByteBuf, T>;

    // JAVA: net.minecraft.network.syncher.EntityDataSerializer#copy
    fn copy_value(&self, value: &T) -> T;
}

// JAVA: net.minecraft.network.syncher.EntityDataSerializer#createAccessor
pub fn create_accessor<T: 'static>(
    id: i32,
    serializer: Arc<dyn EntityDataSerializer<T>>,
) -> EntityDataAccessor<T> {
    EntityDataAccessor { id, serializer }
}

/// Port of `net.minecraft.network.syncher.EntityDataSerializer.ForValueType`.
pub trait ForValueType<T: Clone>: EntityDataSerializer<T> {
    // JAVA: net.minecraft.network.syncher.EntityDataSerializer$ForValueType#copy
    fn copy_value_type(&self, value: &T) -> T {
        value.clone()
    }
}

/// Concrete value-type serializer used by `for_value_type`.
struct ValueTypeSerializer<T: Clone + Send + Sync + 'static> {
    codec_value: StreamCodec<RegistryFriendlyByteBuf, T>,
}

impl<T: Clone + Send + Sync + 'static> ValueTypeSerializer<T> {
    fn new(codec: StreamCodec<RegistryFriendlyByteBuf, T>) -> Self {
        Self { codec_value: codec }
    }
}

impl<T: Clone + Send + Sync + 'static> EntityDataSerializer<T> for ValueTypeSerializer<T> {
    fn codec(&self) -> StreamCodec<RegistryFriendlyByteBuf, T> {
        self.codec_value.clone()
    }

    fn copy_value(&self, value: &T) -> T {
        value.clone()
    }
}

impl<T: Clone + Send + Sync + 'static> ForValueType<T> for ValueTypeSerializer<T> {}

// JAVA: net.minecraft.network.syncher.EntityDataSerializer#forValueType
pub fn for_value_type<T: Clone + Send + Sync + 'static>(
    codec: StreamCodec<RegistryFriendlyByteBuf, T>,
) -> Arc<dyn EntityDataSerializer<T>> {
    Arc::new(ValueTypeSerializer::new(codec))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_value_type_copy_returns_cloned_value() {
        let ser = for_value_type(StreamCodec::<RegistryFriendlyByteBuf, i32>::new());
        assert_eq!(ser.copy_value(&42), 42);
        assert_eq!(ser.copy_value(&-100), -100);
        assert_eq!(ser.copy_value(&i32::MIN), i32::MIN);
        assert_eq!(ser.copy_value(&i32::MAX), i32::MAX);
    }

    #[test]
    fn create_accessor_sets_id_and_serializer() {
        let ser = for_value_type(StreamCodec::<RegistryFriendlyByteBuf, i32>::new());
        let acc = create_accessor(7, Arc::clone(&ser));
        assert_eq!(acc.id, 7);
        assert_eq!(acc.id(), 7);
        assert_eq!(acc.serializer().copy_value(&5), 5);
    }

    #[test]
    fn create_accessor_negative_id() {
        let ser = for_value_type(StreamCodec::<RegistryFriendlyByteBuf, u8>::new());
        let acc = create_accessor(-1, Arc::clone(&ser));
        assert_eq!(acc.id, -1);
        assert_eq!(acc.to_string(), "<entity data: -1>");
    }

    #[test]
    fn accessor_equality_by_id_only() {
        let ser = for_value_type(StreamCodec::<RegistryFriendlyByteBuf, i32>::new());
        let a = create_accessor(5, Arc::clone(&ser));
        let b = create_accessor(5, Arc::clone(&ser));
        let c = create_accessor(6, Arc::clone(&ser));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn value_type_serializer_copy_string() {
        let ser = for_value_type(StreamCodec::<RegistryFriendlyByteBuf, String>::new());
        let s = String::from("hello");
        let copied = ser.copy_value(&s);
        assert_eq!(copied, "hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn for_value_type_codec_is_available() {
        let ser = for_value_type(StreamCodec::<RegistryFriendlyByteBuf, i64>::new());
        let codec = ser.codec();
        assert_eq!(format!("{codec:?}"), "StreamCodec(..)");
    }
}
