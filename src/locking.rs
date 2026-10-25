use std::sync::{Mutex, MutexGuard};

/// Lock a shared device, recovering the guard if a previous holder panicked.
///
/// A poisoned device leaves the emulator in an unknown state, but aborting the
/// whole UI would also discard the user's unsaved work, so recover and keep
/// going.
pub(crate) fn lock_or_recover<T: ?Sized>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn recovers_a_poisoned_mutex() {
        let mutex = Arc::new(Mutex::new(41));
        let poisoned = Arc::clone(&mutex);

        let handle = thread::spawn(move || {
            let _guard = poisoned.lock().unwrap();
            panic!("poison the mutex");
        });
        let _ = handle.join();

        assert!(mutex.is_poisoned());
        assert_eq!(*lock_or_recover(&mutex), 41);
    }
}
