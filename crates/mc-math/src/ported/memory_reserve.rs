use std::sync::Mutex;

const RESERVE_SIZE: usize = 10_485_760;

static RESERVE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

// JAVA: net.minecraft.util.MemoryReserve#allocate
pub fn allocate() {
    let mut guard = RESERVE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = Some(vec![0u8; RESERVE_SIZE]);
}

// JAVA: net.minecraft.util.MemoryReserve#release
pub fn release() {
    let mut guard = RESERVE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_some() {
        *guard = None;
        // Java calls System.gc() three times after releasing the reserve.
        // Rust has no direct equivalent; dropping the buffer releases the memory.
    }
}

/// Rust convenience helper (not present in Java) for tests/inspection.
pub fn is_allocated() -> bool {
    let guard = RESERVE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.is_some()
}

/// Rust convenience helper (not present in Java) for tests/inspection.
pub fn allocated_len() -> usize {
    let guard = RESERVE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.as_ref().map_or(0, |v| v.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize tests because they mutate global state.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn allocate_sets_reserve_to_expected_size() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        release();
        assert!(!is_allocated());
        allocate();
        assert!(is_allocated());
        assert_eq!(allocated_len(), RESERVE_SIZE);
        release();
    }

    #[test]
    fn release_clears_reserve() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        allocate();
        assert!(is_allocated());
        release();
        assert!(!is_allocated());
        assert_eq!(allocated_len(), 0);
    }

    #[test]
    fn release_when_empty_is_noop() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        release();
        release();
        assert!(!is_allocated());
    }

    #[test]
    fn allocate_replaces_existing_reserve() {
        let _g = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        allocate();
        allocate();
        assert!(is_allocated());
        assert_eq!(allocated_len(), RESERVE_SIZE);
        release();
    }
}
