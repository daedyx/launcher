//! Sperren ohne Absturz (SICHERHEIT 2.2): Ein Faden, der beim Halten der Sperre abgestürzt ist, „vergiftet“
//! den Mutex. Die Daten darin (hier: Zuordnungen und Listen) bleiben gültig, also wird weitergearbeitet statt
//! den ganzen Launcher mit `lock().unwrap()` mitzureißen.
use std::sync::{Mutex, MutexGuard, PoisonError};

pub fn lock_or_recover<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn a_poisoned_mutex_is_still_usable() {
        let m = Arc::new(Mutex::new(vec![1]));
        let m2 = Arc::clone(&m);
        let _ = std::thread::spawn(move || {
            let _guard = m2.lock().unwrap();
            panic!("Absturz beim Halten der Sperre");
        })
        .join();
        assert!(m.is_poisoned());
        lock_or_recover(&m).push(2);
        assert_eq!(*lock_or_recover(&m), vec![1, 2]);
    }
}
