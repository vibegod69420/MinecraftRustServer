use std::any::Any;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

/// Minimal stand-in for a Mojang serialization `Codec`.
#[derive(Clone, Debug)]
pub struct Codec<T: ?Sized + 'static>(PhantomData<fn() -> T>);

impl<T: ?Sized + 'static> Codec<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized + 'static> Default for Codec<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal stand-in for a Mojang serialization `MapCodec`.
#[derive(Clone, Debug)]
pub struct MapCodec<T: ?Sized + 'static>(PhantomData<fn() -> T>);

impl<T: ?Sized + 'static> MapCodec<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized + 'static> Default for MapCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal stand-in for a network `StreamCodec`.
#[derive(Clone, Debug)]
pub struct StreamCodec<T: ?Sized + 'static>(PhantomData<fn() -> T>);

impl<T: ?Sized + 'static> StreamCodec<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: ?Sized + 'static> Default for StreamCodec<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal either type for ported dispatch logic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn map<L2, R2>(self, left: impl FnOnce(L) -> L2, right: impl FnOnce(R) -> R2) -> Either<L2, R2> {
        match self {
            Either::Left(value) => Either::Left(left(value)),
            Either::Right(value) => Either::Right(right(value)),
        }
    }
}

/// Minimal stand-in for `net.minecraft.core.component.DataComponentType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataComponentType {
    id: u32,
}

impl DataComponentType {
    pub const fn new(id: u32) -> Self {
        Self { id }
    }

    pub const fn id(self) -> u32 {
        self.id
    }
}

/// Minimal stand-in for `net.minecraft.core.component.DataComponentGetter`.
pub trait DataComponentGetter: fmt::Debug {
    fn has_component(&self, component_type: DataComponentType) -> bool;
}

/// Port of `net.minecraft.core.component.predicates.DataComponentPredicate`.
pub trait DataComponentPredicate: fmt::Debug + Any {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate#matches
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool;
}

/// Minimal stand-in for `net.minecraft.core.component.predicates.AnyValue`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnyValue {
    component_type: DataComponentType,
}

impl AnyValue {
    pub fn new(component_type: DataComponentType) -> Self {
        Self { component_type }
    }

    pub fn component_type(self) -> DataComponentType {
        self.component_type
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate#matches
    pub fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        getter.has_component(self.component_type)
    }
}

impl DataComponentPredicate for AnyValue {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate#matches
    fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        AnyValue::matches(self, getter)
    }
}

/// Port of `net.minecraft.core.component.predicates.DataComponentPredicate.Type`.
pub trait PredicateType<T: DataComponentPredicate + Clone + 'static>: fmt::Debug + Any {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#codec
    fn codec(&self) -> &Codec<T>;

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#wrappedCodec
    fn wrapped_codec(&self) -> &MapCodec<Single<T>>;

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#singleStreamCodec
    fn single_stream_codec(&self) -> &StreamCodec<Single<T>>;

    fn as_any(&self) -> &dyn Any;
}

/// Port of `net.minecraft.core.component.predicates.DataComponentPredicate.Single`.
#[derive(Clone, Debug)]
pub struct Single<T: DataComponentPredicate + Clone + 'static> {
    type_: Arc<dyn PredicateType<T>>,
    predicate: T,
}

impl<T: DataComponentPredicate + Clone + 'static> Single<T> {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Single#<init>
    pub fn new(type_: Arc<dyn PredicateType<T>>, predicate: T) -> Self {
        Self { type_, predicate }
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Single#type
    pub fn type_(&self) -> &Arc<dyn PredicateType<T>> {
        &self.type_
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Single#predicate
    pub fn predicate(&self) -> &T {
        &self.predicate
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Single#wrapCodec
    pub fn wrap_codec(_type: Arc<dyn PredicateType<T>>, _codec: Codec<T>) -> MapCodec<Single<T>> {
        MapCodec::new()
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Single#fromEntry
    fn from_entry(entry: (Arc<dyn PredicateType<T>>, T)) -> Self {
        Self::new(entry.0, entry.1)
    }
}

/// Port of `net.minecraft.core.component.predicates.DataComponentPredicate.TypeBase`.
#[derive(Clone, Debug)]
pub struct TypeBase<T: DataComponentPredicate + Clone + 'static> {
    codec: Codec<T>,
    wrapped_codec: MapCodec<Single<T>>,
    single_stream_codec: StreamCodec<Single<T>>,
}

impl<T: DataComponentPredicate + Clone + 'static> TypeBase<T> {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.TypeBase#<init>
    pub fn new(codec: Codec<T>) -> Self {
        Self {
            wrapped_codec: MapCodec::new(),
            single_stream_codec: StreamCodec::new(),
            codec,
        }
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#codec
    pub fn codec(&self) -> &Codec<T> {
        &self.codec
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#wrappedCodec
    pub fn wrapped_codec(&self) -> &MapCodec<Single<T>> {
        &self.wrapped_codec
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#singleStreamCodec
    pub fn single_stream_codec(&self) -> &StreamCodec<Single<T>> {
        &self.single_stream_codec
    }
}

/// Port of `net.minecraft.core.component.predicates.DataComponentPredicate.ConcreteType`.
#[derive(Clone, Debug)]
pub struct ConcreteType<T: DataComponentPredicate + Clone + 'static> {
    base: TypeBase<T>,
}

impl<T: DataComponentPredicate + Clone + 'static> ConcreteType<T> {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.ConcreteType#<init>
    pub fn new(codec: Codec<T>) -> Self {
        Self {
            base: TypeBase::new(codec),
        }
    }
}

impl<T: DataComponentPredicate + Clone + 'static> PredicateType<T> for ConcreteType<T> {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#codec
    fn codec(&self) -> &Codec<T> {
        self.base.codec()
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#wrappedCodec
    fn wrapped_codec(&self) -> &MapCodec<Single<T>> {
        self.base.wrapped_codec()
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#singleStreamCodec
    fn single_stream_codec(&self) -> &StreamCodec<Single<T>> {
        self.base.single_stream_codec()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Port of `net.minecraft.core.component.predicates.DataComponentPredicate.AnyValueType`.
#[derive(Clone, Debug)]
pub struct AnyValueType {
    base: TypeBase<AnyValue>,
    predicate: AnyValue,
}

impl AnyValueType {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.AnyValueType#<init>
    pub fn new(predicate: AnyValue) -> Self {
        Self {
            base: TypeBase::new(Codec::new()),
            predicate,
        }
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.AnyValueType#predicate
    pub fn predicate(&self) -> AnyValue {
        self.predicate
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.AnyValueType#componentType
    pub fn component_type(&self) -> DataComponentType {
        self.predicate.component_type()
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.AnyValueType#create
    pub fn create(component_type: DataComponentType) -> Self {
        Self::new(AnyValue::new(component_type))
    }
}

impl PredicateType<AnyValue> for AnyValueType {
    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#codec
    fn codec(&self) -> &Codec<AnyValue> {
        self.base.codec()
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#wrappedCodec
    fn wrapped_codec(&self) -> &MapCodec<Single<AnyValue>> {
        self.base.wrapped_codec()
    }

    // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#singleStreamCodec
    fn single_stream_codec(&self) -> &StreamCodec<Single<AnyValue>> {
        self.base.single_stream_codec()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#unpackType
pub fn unpack_type(type_obj: &dyn Any) -> Either<&dyn Any, DataComponentType> {
    if let Some(av) = type_obj.downcast_ref::<AnyValueType>() {
        Either::Right(av.component_type())
    } else {
        Either::Left(type_obj)
    }
}

// JAVA: net.minecraft.core.component.predicates.DataComponentPredicate.Type#copyOrCreateType
pub fn copy_or_create_type(either: Either<Box<dyn Any>, DataComponentType>) -> Box<dyn Any> {
    match either {
        Either::Left(t) => t,
        Either::Right(ct) => Box::new(AnyValueType::create(ct)),
    }
}

// JAVA: net.minecraft.core.component.predicates.DataComponentPredicate#singleCodec
pub fn single_codec<T: DataComponentPredicate + Clone + 'static>(
    _field_name: &str,
) -> MapCodec<Single<T>> {
    MapCodec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockGetter {
        present: Vec<u32>,
    }

    impl DataComponentGetter for MockGetter {
        fn has_component(&self, component_type: DataComponentType) -> bool {
            self.present.contains(&component_type.id())
        }
    }

    #[test]
    fn any_value_matches_presence_and_edge_ids() {
        let getter = MockGetter {
            present: vec![0, 1, u32::MAX],
        };
        assert!(AnyValue::new(DataComponentType::new(0)).matches(&getter));
        assert!(AnyValue::new(DataComponentType::new(1)).matches(&getter));
        assert!(!AnyValue::new(DataComponentType::new(2)).matches(&getter));
        assert!(AnyValue::new(DataComponentType::new(u32::MAX)).matches(&getter));
    }

    #[test]
    fn any_value_type_create_predicate_and_component_type() {
        let ct = DataComponentType::new(99);
        let ty = AnyValueType::create(ct);
        assert_eq!(ty.component_type(), ct);
        assert_eq!(ty.predicate(), AnyValue::new(ct));

        let getter = MockGetter { present: vec![99] };
        assert!(ty.predicate().matches(&getter));
        assert!(!AnyValue::new(DataComponentType::new(100)).matches(&getter));
    }

    #[test]
    fn single_wraps_type_and_predicate() {
        let av = AnyValue::new(DataComponentType::new(5));
        let ty: Arc<dyn PredicateType<AnyValue>> =
            Arc::new(AnyValueType::create(DataComponentType::new(5)));
        let single = Single::new(ty.clone(), av);
        assert_eq!(single.predicate().component_type(), DataComponentType::new(5));
        assert!(single.type_().as_any().downcast_ref::<AnyValueType>().is_some());

        let single2 = Single::from_entry((ty, av));
        assert_eq!(single2.predicate(), &av);
        let _wrapped: MapCodec<Single<AnyValue>> =
            Single::wrap_codec(single2.type_().clone(), Codec::new());
    }

    #[test]
    fn unpack_and_copy_or_create_type_dispatch() {
        let ct = DataComponentType::new(7);
        let av_type = AnyValueType::create(ct);

        match unpack_type(&av_type) {
            Either::Right(id) => assert_eq!(id, ct),
            Either::Left(_) => panic!("expected Right for AnyValueType"),
        }

        let concrete = ConcreteType::<AnyValue>::new(Codec::new());
        match unpack_type(&concrete) {
            Either::Left(obj) => {
                assert!(obj.downcast_ref::<ConcreteType<AnyValue>>().is_some());
            }
            Either::Right(_) => panic!("expected Left for ConcreteType"),
        }

        let copied = copy_or_create_type(Either::Right(DataComponentType::new(42)));
        let copied_av = copied
            .downcast_ref::<AnyValueType>()
            .expect("expected AnyValueType");
        assert_eq!(copied_av.component_type(), DataComponentType::new(42));

        let passthrough: Box<dyn Any> = Box::new(ConcreteType::<AnyValue>::new(Codec::new()));
        let copied_left = copy_or_create_type(Either::Left(passthrough));
        assert!(copied_left.downcast_ref::<ConcreteType<AnyValue>>().is_some());
    }

    #[test]
    fn custom_predicate_negative_overflow_and_float_bits() {
        #[derive(Clone, Debug)]
        struct WrapPredicate {
            add: i32,
            bits: u32,
        }

        impl DataComponentPredicate for WrapPredicate {
            // JAVA: net.minecraft.core.component.predicates.DataComponentPredicate#matches
            fn matches(&self, _getter: &dyn DataComponentGetter) -> bool {
                let wrapped = self.add.wrapping_add(1);
                let f = f32::from_bits(self.bits);
                wrapped == i32::MIN && f.is_nan()
            }
        }

        let getter = MockGetter { present: vec![] };
        let nan_bits = f32::NAN.to_bits();
        assert!(WrapPredicate {
            add: i32::MAX,
            bits: nan_bits
        }
        .matches(&getter));
        assert!(!WrapPredicate {
            add: -1,
            bits: nan_bits
        }
        .matches(&getter));

        let neg_zero_bits = (-0.0f32).to_bits();
        assert_eq!(neg_zero_bits, 0x8000_0000u32);
        let neg_inf_bits = f32::NEG_INFINITY.to_bits();
        assert_eq!(neg_inf_bits, 0xFF80_0000u32);
        assert!(!WrapPredicate {
            add: i32::MAX,
            bits: neg_zero_bits
        }
        .matches(&getter));
        assert!(!WrapPredicate {
            add: i32::MAX,
            bits: neg_inf_bits
        }
        .matches(&getter));

        let round_trip = f32::from_bits(neg_inf_bits).to_bits();
        assert_eq!(round_trip, neg_inf_bits);
    }

    #[test]
    fn single_codec_marker_and_type_base_accessors() {
        let _mc: MapCodec<Single<AnyValue>> = single_codec("value");
        let base = TypeBase::<AnyValue>::new(Codec::new());
        let _c: &Codec<AnyValue> = base.codec();
        let _w: &MapCodec<Single<AnyValue>> = base.wrapped_codec();
        let _s: &StreamCodec<Single<AnyValue>> = base.single_stream_codec();
    }
}
