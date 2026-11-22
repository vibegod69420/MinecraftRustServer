//! Port of `net.minecraft.network.syncher.SyncedDataHolder`.

use std::marker::PhantomData;

/// Minimal stub for `net.minecraft.network.syncher.EntityDataAccessor`.
/// This module is self-contained; the canonical type lives in `entity_data_accessor.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityDataAccessor<T> {
    pub id: i32,
    _marker: PhantomData<T>,
}

impl<T> EntityDataAccessor<T> {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

/// Minimal stub for `net.minecraft.network.syncher.SynchedEntityData.DataValue`.
/// This module is self-contained; the canonical type lives in `synched_entity_data.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataValue<T> {
    pub id: i32,
    _marker: PhantomData<T>,
}

impl<T> DataValue<T> {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

/// Port of `net.minecraft.network.syncher.SyncedDataHolder`.
pub trait SyncedDataHolder {
    // JAVA: net.minecraft.network.syncher.SyncedDataHolder#onSyncedDataUpdated
    fn on_synced_data_updated_accessor<T>(&mut self, accessor: &EntityDataAccessor<T>);

    // JAVA: net.minecraft.network.syncher.SyncedDataHolder#onSyncedDataUpdated
    fn on_synced_data_updated_values<T>(&mut self, values: &[DataValue<T>]);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHolder {
        accessor_ids: Vec<i32>,
        value_count: usize,
        last_value_ids: Vec<i32>,
    }

    impl MockHolder {
        fn new() -> Self {
            Self {
                accessor_ids: Vec::new(),
                value_count: 0,
                last_value_ids: Vec::new(),
            }
        }
    }

    impl SyncedDataHolder for MockHolder {
        fn on_synced_data_updated_accessor<T>(&mut self, accessor: &EntityDataAccessor<T>) {
            self.accessor_ids.push(accessor.id);
        }

        fn on_synced_data_updated_values<T>(&mut self, values: &[DataValue<T>]) {
            self.value_count = values.len();
            self.last_value_ids = values.iter().map(|v| v.id).collect();
        }
    }

    #[test]
    fn accessor_update_records_id() {
        let mut holder = MockHolder::new();
        let acc = EntityDataAccessor::<i32>::new(5);
        holder.on_synced_data_updated_accessor(&acc);
        assert_eq!(holder.accessor_ids, vec![5]);
    }

    #[test]
    fn accessor_update_negative_id() {
        let mut holder = MockHolder::new();
        let acc = EntityDataAccessor::<()>::new(-1);
        holder.on_synced_data_updated_accessor(&acc);
        assert_eq!(holder.accessor_ids, vec![-1]);
    }

    #[test]
    fn values_update_records_count_and_ids() {
        let mut holder = MockHolder::new();
        let values = vec![DataValue::<i32>::new(1), DataValue::<i32>::new(2)];
        holder.on_synced_data_updated_values(&values);
        assert_eq!(holder.value_count, 2);
        assert_eq!(holder.last_value_ids, vec![1, 2]);
    }

    #[test]
    fn values_update_empty_list() {
        let mut holder = MockHolder::new();
        let values: Vec<DataValue<u8>> = Vec::new();
        holder.on_synced_data_updated_values(&values);
        assert_eq!(holder.value_count, 0);
        assert!(holder.last_value_ids.is_empty());
    }

    #[test]
    fn multiple_accessor_updates() {
        let mut holder = MockHolder::new();
        holder.on_synced_data_updated_accessor(&EntityDataAccessor::<i32>::new(0));
        holder.on_synced_data_updated_accessor(&EntityDataAccessor::<i32>::new(i32::MAX));
        holder.on_synced_data_updated_accessor(&EntityDataAccessor::<i32>::new(i32::MIN));
        assert_eq!(holder.accessor_ids, vec![0, i32::MAX, i32::MIN]);
    }
}
