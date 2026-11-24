// === FILE: synched_entity_data.rs ===
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

const MAX_ID_VALUE: i32 = 254;

#[derive(Debug, Clone, Default)]
pub struct RegistryFriendlyByteBuf {
    data: Vec<u8>,
    pos: usize,
}

impl RegistryFriendlyByteBuf {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    pub fn write_byte(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn read_byte(&mut self) -> i8 {
        let b = self.data[self.pos] as i8;
        self.pos = self.pos.wrapping_add(1);
        b
    }

    pub fn write_var_int(&mut self, mut value: i32) {
        loop {
            let mut b = (value & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                b |= 0x80;
            }
            self.write_byte(b as i8);
            if value == 0 {
                return;
            }
        }
    }

    pub fn read_var_int(&mut self) -> i32 {
        let mut result: i32 = 0;
        let mut shift: i32 = 0;
        loop {
            let b = self.read_byte() as i32;
            result |= (b & 0x7F).wrapping_shl((shift as u32) & 31);
            if (b & 0x80) == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
            if shift >= 32 {
                panic!("VarInt too big");
            }
        }
    }
}

pub trait EntityDataSerializerOps: fmt::Debug + Send + Sync {
    fn serialized_id(&self) -> i32;
    fn copy_value(&self, value: &dyn Any) -> Box<dyn Any + Send + Sync>;
    fn encode(&self, buf: &mut RegistryFriendlyByteBuf, value: &dyn Any);
    fn decode(&self, buf: &mut RegistryFriendlyByteBuf) -> Box<dyn Any + Send + Sync>;
    fn eq_value(&self, a: &dyn Any, b: &dyn Any) -> bool;
    fn debug_value(&self, value: &dyn Any) -> String;
}

static SERIALIZER_REGISTRY: OnceLock<Mutex<HashMap<i32, Arc<dyn EntityDataSerializerOps>>>> =
    OnceLock::new();

fn serializer_registry() -> &'static Mutex<HashMap<i32, Arc<dyn EntityDataSerializerOps>>> {
    SERIALIZER_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct EntityDataSerializers;

impl EntityDataSerializers {
    pub fn get_serialized_id(serializer: &Arc<dyn EntityDataSerializerOps>) -> i32 {
        serializer.serialized_id()
    }

    pub fn get_serializer(id: i32) -> Option<Arc<dyn EntityDataSerializerOps>> {
        serializer_registry().lock().unwrap().get(&id).cloned()
    }

    pub fn register(serializer: Arc<dyn EntityDataSerializerOps>) {
        let id = serializer.serialized_id();
        serializer_registry().lock().unwrap().insert(id, serializer);
    }
}

pub struct ClassTreeIdRegistry {
    pub counts: HashMap<TypeId, i32>,
}

impl ClassTreeIdRegistry {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn define(&mut self, type_id: TypeId) -> i32 {
        let count = self.counts.entry(type_id).or_insert(0);
        let id = *count;
        *count = id.wrapping_add(1);
        id
    }

    pub fn get_count(&self, type_id: TypeId) -> i32 {
        self.counts.get(&type_id).copied().unwrap_or(0)
    }
}

impl Default for ClassTreeIdRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EntityDataAccessor<T: Send + Sync + 'static> {
    id: i32,
    serializer: Arc<dyn EntityDataSerializerOps>,
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<T: Send + Sync + 'static> Clone for EntityDataAccessor<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            serializer: self.serializer.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> fmt::Debug for EntityDataAccessor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityDataAccessor")
            .field("id", &self.id)
            .finish()
    }
}

impl<T: Send + Sync + 'static> EntityDataAccessor<T> {
    pub fn new(id: i32, serializer: Arc<dyn EntityDataSerializerOps>) -> Self {
        Self {
            id,
            serializer,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn serializer(&self) -> &Arc<dyn EntityDataSerializerOps> {
        &self.serializer
    }
}

pub trait SyncedDataHolder: fmt::Debug + Send + Sync + 'static {
    fn holder_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn on_synced_data_updated(&self, accessor_id: i32);
    fn on_synced_data_updated_values(&self, values: &[DataValue]);
}

static ID_REGISTRY: OnceLock<Mutex<ClassTreeIdRegistry>> = OnceLock::new();

fn id_registry() -> &'static Mutex<ClassTreeIdRegistry> {
    ID_REGISTRY.get_or_init(|| Mutex::new(ClassTreeIdRegistry::new()))
}

pub struct SynchedEntityData {
    entity: Arc<dyn SyncedDataHolder>,
    items_by_id: Vec<DataItem>,
    dirty: bool,
}

impl SynchedEntityData {
    // JAVA: net.minecraft.network.syncher.SynchedEntityData#<init>
    fn new(entity: Arc<dyn SyncedDataHolder>, items_by_id: Vec<DataItem>) -> Self {
        Self {
            entity,
            items_by_id,
            dirty: false,
        }
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#defineId
    pub fn define_id<T: Send + Sync + 'static>(
        holder_type: TypeId,
        serializer: Arc<dyn EntityDataSerializerOps>,
    ) -> EntityDataAccessor<T> {
        let id = id_registry().lock().unwrap().define(holder_type);
        if id > MAX_ID_VALUE {
            panic!("Data value id is too big with {}! (Max is {})", id, MAX_ID_VALUE);
        }
        EntityDataAccessor::new(id, serializer)
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#getItem
    fn get_item(&self, id: i32) -> &DataItem {
        &self.items_by_id[id as usize]
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#get
    pub fn get<T: Send + Sync + 'static>(&self, accessor: &EntityDataAccessor<T>) -> &T {
        self.get_item(accessor.id()).get_value::<T>()
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#set
    pub fn set<T: Send + Sync + 'static>(&mut self, accessor: &EntityDataAccessor<T>, value: T) {
        self.set_force(accessor, value, false);
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#set
    pub fn set_force<T: Send + Sync + 'static>(
        &mut self,
        accessor: &EntityDataAccessor<T>,
        value: T,
        force: bool,
    ) {
        let item = &mut self.items_by_id[accessor.id() as usize];
        let not_equal = !item
            .serializer
            .eq_value((&value as &dyn Any), item.value.as_ref());
        if force || not_equal {
            item.value = Box::new(value);
            self.entity.on_synced_data_updated(accessor.id());
            item.dirty = true;
            self.dirty = true;
        }
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#isDirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#packDirty
    pub fn pack_dirty(&mut self) -> Option<Vec<DataValue>> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        let mut list = Vec::new();
        for item in &mut self.items_by_id {
            if item.dirty {
                item.dirty = false;
                list.push(item.value());
            }
        }
        Some(list)
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#getNonDefaultValues
    pub fn get_non_default_values(&self) -> Option<Vec<DataValue>> {
        let mut list: Option<Vec<DataValue>> = None;
        for item in &self.items_by_id {
            if !item.is_set_to_default() {
                list.get_or_insert_with(Vec::new).push(item.value());
            }
        }
        list
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#assignValues
    pub fn assign_values(&mut self, values: Vec<DataValue>) {
        for dv in &values {
            let item = &mut self.items_by_id[dv.id as usize];
            Self::assign_value(&self.entity, item, dv);
            self.entity.on_synced_data_updated(item.accessor_id);
        }
        self.entity.on_synced_data_updated_values(&values);
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData#assignValue
    fn assign_value(entity: &Arc<dyn SyncedDataHolder>, item: &mut DataItem, data_value: &DataValue) {
        if data_value.serializer.serialized_id() != item.serializer.serialized_id() {
            panic!(
                "Invalid entity data item type for field {} on entity {:?}: old={}, new={}",
                item.accessor_id,
                entity,
                item.serializer.debug_value(item.value.as_ref()),
                data_value.serializer.debug_value(data_value.value.as_ref()),
            );
        }
        item.value = data_value.serializer.copy_value(data_value.value.as_ref());
    }
}

pub struct Builder {
    entity: Arc<dyn SyncedDataHolder>,
    items_by_id: Vec<Option<DataItem>>,
}

impl Builder {
    // JAVA: net.minecraft.network.syncher.SynchedEntityData$Builder#<init>
    pub fn new(entity: Arc<dyn SyncedDataHolder>) -> Self {
        let count = id_registry()
            .lock()
            .unwrap()
            .get_count(entity.holder_type_id());
        let mut items_by_id = Vec::with_capacity(count as usize);
        for _ in 0..count {
            items_by_id.push(None);
        }
        Self { entity, items_by_id }
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$Builder#define
    pub fn define<T: Send + Sync + 'static>(
        mut self,
        accessor: EntityDataAccessor<T>,
        value: T,
    ) -> Self {
        let id = accessor.id();
        if id > self.items_by_id.len() as i32 {
            panic!(
                "Data value id is too big with {}! (Max is {})",
                id,
                self.items_by_id.len()
            );
        }
        let idx = id as usize;
        if self.items_by_id[idx].is_some() {
            panic!("Duplicate id value for {}!", id);
        }
        if EntityDataSerializers::get_serialized_id(accessor.serializer()) < 0 {
            panic!(
                "Unregistered serializer {:?} for {}!",
                accessor.serializer(),
                id
            );
        }
        self.items_by_id[idx] = Some(DataItem::new(accessor, value));
        self
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$Builder#build
    pub fn build(self) -> SynchedEntityData {
        for i in 0..self.items_by_id.len() {
            if self.items_by_id[i].is_none() {
                panic!(
                    "Entity {:?} has not defined synched data value {}",
                    self.entity, i
                );
            }
        }
        let items = self
            .items_by_id
            .into_iter()
            .map(|x| x.unwrap())
            .collect();
        SynchedEntityData::new(self.entity, items)
    }
}

pub struct DataItem {
    accessor_id: i32,
    serializer: Arc<dyn EntityDataSerializerOps>,
    value: Box<dyn Any + Send + Sync>,
    initial_value: Box<dyn Any + Send + Sync>,
    dirty: bool,
}

impl DataItem {
    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#<init>
    pub fn new<T: Send + Sync + 'static>(accessor: EntityDataAccessor<T>, value: T) -> Self {
        let initial_value = accessor.serializer().copy_value(&value);
        Self {
            accessor_id: accessor.id(),
            serializer: accessor.serializer().clone(),
            value: Box::new(value),
            initial_value,
            dirty: false,
        }
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#getAccessor
    pub fn get_accessor<T: Send + Sync + 'static>(&self) -> EntityDataAccessor<T> {
        EntityDataAccessor::new(self.accessor_id, self.serializer.clone())
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#setValue
    pub fn set_value<T: Send + Sync + 'static>(&mut self, value: T) {
        self.value = Box::new(value);
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#getValue
    pub fn get_value<T: Send + Sync + 'static>(&self) -> &T {
        self.value
            .downcast_ref::<T>()
            .expect("DataItem value type mismatch")
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#isDirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#setDirty
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#isSetToDefault
    pub fn is_set_to_default(&self) -> bool {
        self.serializer
            .eq_value(self.initial_value.as_ref(), self.value.as_ref())
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataItem#value
    pub fn value(&self) -> DataValue {
        DataValue::create_erased(self.accessor_id, self.serializer.clone(), self.value.as_ref())
    }
}

pub struct DataValue {
    id: i32,
    serializer: Arc<dyn EntityDataSerializerOps>,
    value: Box<dyn Any + Send + Sync>,
}

impl DataValue {
    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataValue#create
    pub fn create<T: Send + Sync + 'static>(accessor: &EntityDataAccessor<T>, value: &T) -> Self {
        let serializer = accessor.serializer().clone();
        let copied = serializer.copy_value(value);
        Self {
            id: accessor.id(),
            serializer,
            value: copied,
        }
    }

    fn create_erased(id: i32, serializer: Arc<dyn EntityDataSerializerOps>, value: &dyn Any) -> Self {
        let copied = serializer.copy_value(value);
        Self {
            id,
            serializer,
            value: copied,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn serializer(&self) -> &Arc<dyn EntityDataSerializerOps> {
        &self.serializer
    }

    pub fn value<T: Send + Sync + 'static>(&self) -> &T {
        self.value
            .downcast_ref::<T>()
            .expect("DataValue type mismatch")
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataValue#write
    pub fn write(&self, buf: &mut RegistryFriendlyByteBuf) {
        let serializer_id = EntityDataSerializers::get_serialized_id(&self.serializer);
        if serializer_id < 0 {
            panic!("Unknown serializer type {:?}", self.serializer);
        }
        buf.write_byte(self.id as i8);
        buf.write_var_int(serializer_id);
        self.serializer.encode(buf, self.value.as_ref());
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataValue#read
    pub fn read(buf: &mut RegistryFriendlyByteBuf, id: i32) -> Self {
        let serializer_type_id = buf.read_var_int();
        let serializer = EntityDataSerializers::get_serializer(serializer_type_id)
            .unwrap_or_else(|| panic!("Unknown serializer type {}", serializer_type_id));
        Self::read_with_serializer(buf, id, serializer)
    }

    // JAVA: net.minecraft.network.syncher.SynchedEntityData$DataValue#read
    fn read_with_serializer(
        buf: &mut RegistryFriendlyByteBuf,
        id: i32,
        serializer: Arc<dyn EntityDataSerializerOps>,
    ) -> Self {
        let value = serializer.decode(buf);
        Self {
            id,
            serializer,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct IntSerializer {
        id: i32,
    }

    impl EntityDataSerializerOps for IntSerializer {
        fn serialized_id(&self) -> i32 {
            self.id
        }
        fn copy_value(&self, value: &dyn Any) -> Box<dyn Any + Send + Sync> {
            Box::new(*value.downcast_ref::<i32>().unwrap())
        }
        fn encode(&self, buf: &mut RegistryFriendlyByteBuf, value: &dyn Any) {
            buf.write_var_int(*value.downcast_ref::<i32>().unwrap());
        }
        fn decode(&self, buf: &mut RegistryFriendlyByteBuf) -> Box<dyn Any + Send + Sync> {
            Box::new(buf.read_var_int())
        }
        fn eq_value(&self, a: &dyn Any, b: &dyn Any) -> bool {
            a.downcast_ref::<i32>() == b.downcast_ref::<i32>()
        }
        fn debug_value(&self, value: &dyn Any) -> String {
            format!("{}", value.downcast_ref::<i32>().unwrap())
        }
    }

    #[derive(Debug)]
    struct F32Serializer {
        id: i32,
    }

    impl EntityDataSerializerOps for F32Serializer {
        fn serialized_id(&self) -> i32 {
            self.id
        }
        fn copy_value(&self, value: &dyn Any) -> Box<dyn Any + Send + Sync> {
            Box::new(*value.downcast_ref::<f32>().unwrap())
        }
        fn encode(&self, buf: &mut RegistryFriendlyByteBuf, value: &dyn Any) {
            let bits = value.downcast_ref::<f32>().unwrap().to_bits() as i32;
            buf.write_var_int(bits);
        }
        fn decode(&self, buf: &mut RegistryFriendlyByteBuf) -> Box<dyn Any + Send + Sync> {
            Box::new(f32::from_bits(buf.read_var_int() as u32))
        }
        fn eq_value(&self, a: &dyn Any, b: &dyn Any) -> bool {
            a.downcast_ref::<f32>().unwrap().to_bits() == b.downcast_ref::<f32>().unwrap().to_bits()
        }
        fn debug_value(&self, value: &dyn Any) -> String {
            format!("{}", value.downcast_ref::<f32>().unwrap())
        }
    }

    #[test]
    fn test_set_get_dirty_pack_non_default() {
        #[derive(Debug)]
        struct E1;
        impl SyncedDataHolder for E1 {
            fn on_synced_data_updated(&self, _: i32) {}
            fn on_synced_data_updated_values(&self, _: &[DataValue]) {}
        }

        let ser: Arc<dyn EntityDataSerializerOps> = Arc::new(IntSerializer { id: 9201 });
        EntityDataSerializers::register(ser.clone());
        let entity = Arc::new(E1);
        let a0 = SynchedEntityData::define_id::<i32>(entity.holder_type_id(), ser.clone());
        let a1 = SynchedEntityData::define_id::<i32>(entity.holder_type_id(), ser.clone());
        let mut data = Builder::new(entity)
            .define(a0.clone(), 10i32)
            .define(a1.clone(), 20i32)
            .build();

        assert_eq!(*data.get(&a0), 10);
        assert!(!data.is_dirty());
        assert!(data.pack_dirty().is_none());
        assert!(data.get_non_default_values().is_none());

        data.set(&a0, 10i32);
        assert!(!data.is_dirty());

        data.set_force(&a0, 10i32, true);
        assert!(data.is_dirty());
        let packed = data.pack_dirty().unwrap();
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0].id(), a0.id());
        assert_eq!(*packed[0].value::<i32>(), 10);
        assert!(!data.is_dirty());

        data.set(&a1, -5i32);
        let non_default = data.get_non_default_values().unwrap();
        assert_eq!(non_default.len(), 1);
        assert_eq!(*non_default[0].value::<i32>(), -5);
    }

    #[test]
    fn test_negative_overflow_and_varint_round_trip() {
        #[derive(Debug)]
        struct E2;
        impl SyncedDataHolder for E2 {
            fn on_synced_data_updated(&self, _: i32) {}
            fn on_synced_data_updated_values(&self, _: &[DataValue]) {}
        }

        let ser: Arc<dyn EntityDataSerializerOps> = Arc::new(IntSerializer { id: 9202 });
        EntityDataSerializers::register(ser.clone());
        let entity = Arc::new(E2);
        let acc = SynchedEntityData::define_id::<i32>(entity.holder_type_id(), ser.clone());
        let mut data = Builder::new(entity).define(acc.clone(), i32::MIN).build();

        assert_eq!(*data.get(&acc), i32::MIN);
        data.set(&acc, i32::MAX);
        assert_eq!(*data.get(&acc), i32::MAX);
        data.set(&acc, -1i32);
        let packed = data.pack_dirty().unwrap();
        assert_eq!(*packed[0].value::<i32>(), -1);

        let mut buf = RegistryFriendlyByteBuf::new();
        buf.write_var_int(i32::MIN);
        buf.write_var_int(i32::MAX);
        buf.write_var_int(-1);
        buf.write_var_int(0);
        let mut r = RegistryFriendlyByteBuf::from_bytes(buf.into_inner());
        assert_eq!(r.read_var_int(), i32::MIN);
        assert_eq!(r.read_var_int(), i32::MAX);
        assert_eq!(r.read_var_int(), -1);
        assert_eq!(r.read_var_int(), 0);

        let mut reg = ClassTreeIdRegistry::new();
        let tid = TypeId::of::<u16>();
        reg.counts.insert(tid, i32::MAX);
        let id = reg.define(tid);
        assert_eq!(id, i32::MAX);
        assert_eq!(reg.define(tid), i32::MIN);
    }

    #[test]
    fn test_float_bits_nan_and_signed_zero() {
        #[derive(Debug)]
        struct E3;
        impl SyncedDataHolder for E3 {
            fn on_synced_data_updated(&self, _: i32) {}
            fn on_synced_data_updated_values(&self, _: &[DataValue]) {}
        }

        let ser: Arc<dyn EntityDataSerializerOps> = Arc::new(F32Serializer { id: 9203 });
        EntityDataSerializers::register(ser.clone());
        let entity = Arc::new(E3);
        let acc = SynchedEntityData::define_id::<f32>(entity.holder_type_id(), ser.clone());
        let nan_bits = 0x7FC0_0001u32;
        let nan = f32::from_bits(nan_bits);
        let mut data = Builder::new(entity).define(acc.clone(), nan).build();

        assert_eq!(data.get(&acc).to_bits(), nan_bits);
        data.set(&acc, f32::from_bits(nan_bits));
        assert!(!data.is_dirty());

        data.set(&acc, 0.0f32);
        assert!(data.is_dirty());
        let _ = data.pack_dirty();
        data.set(&acc, -0.0f32);
        assert!(data.is_dirty());
        assert_ne!(0.0f32.to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn test_assign_values_callbacks_and_order() {
        #[derive(Debug)]
        struct E4 {
            log: Mutex<Vec<String>>,
        }
        impl E4 {
            fn new() -> Arc<Self> {
                Arc::new(Self {
                    log: Mutex::new(Vec::new()),
                })
            }
        }
        impl SyncedDataHolder for E4 {
            fn on_synced_data_updated(&self, id: i32) {
                self.log.lock().unwrap().push(format!("id:{id}"));
            }
            fn on_synced_data_updated_values(&self, values: &[DataValue]) {
                self.log
                    .lock()
                    .unwrap()
                    .push(format!("bulk:{}", values.len()));
            }
        }

        let ser: Arc<dyn EntityDataSerializerOps> = Arc::new(IntSerializer { id: 9204 });
        EntityDataSerializers::register(ser.clone());
        let entity = E4::new();
        let a0 = SynchedEntityData::define_id::<i32>(entity.holder_type_id(), ser.clone());
        let a1 = SynchedEntityData::define_id::<i32>(entity.holder_type_id(), ser.clone());
        let mut data = Builder::new(entity.clone())
            .define(a0.clone(), 1i32)
            .define(a1.clone(), 2i32)
            .build();

        let v0 = DataValue::create(&a0, &10i32);
        let v1 = DataValue::create(&a1, &20i32);
        data.assign_values(vec![v0, v1]);

        assert_eq!(*data.get(&a0), 10);
        assert_eq!(*data.get(&a1), 20);
        let log = entity.log.lock().unwrap();
        assert_eq!(
            &*log,
            &[
                format!("id:{}", a0.id()),
                format!("id:{}", a1.id()),
                "bulk:2".to_string(),
            ]
        );
    }

    #[test]
    fn test_data_value_wire_round_trip() {
        let ser: Arc<dyn EntityDataSerializerOps> = Arc::new(IntSerializer { id: 9205 });
        EntityDataSerializers::register(ser.clone());
        let acc = EntityDataAccessor::<i32>::new(7, ser.clone());
        let dv = DataValue::create(&acc, &-12345i32);

        let mut buf = RegistryFriendlyByteBuf::new();
        dv.write(&mut buf);
        let bytes = buf.into_inner();

        let mut r = RegistryFriendlyByteBuf::from_bytes(bytes);
        let id = r.read_byte() as i32;
        let dv2 = DataValue::read(&mut r, id);
        assert_eq!(dv2.id(), 7);
        assert_eq!(*dv2.value::<i32>(), -12345);
    }
}
