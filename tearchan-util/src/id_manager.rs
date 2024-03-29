use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct IdManager<T> {
    current: Arc<Mutex<T>>,
    incrementer: Arc<fn(val: &T) -> T>,
}

impl<T> IdManager<T> {
    pub fn new(first: T, incrementer: fn(val: &T) -> T) -> IdManager<T> {
        IdManager {
            current: Arc::new(Mutex::new(first)),
            incrementer: Arc::new(incrementer),
        }
    }

    pub fn gen(&self) -> T
    where
        T: Copy,
    {
        gen(&self.current, &self.incrementer)
    }

    pub fn reset(&mut self, first: T) {
        self.current = Arc::new(Mutex::new(first));
    }

    pub fn current(&self) -> MutexGuard<T> {
        self.current.lock().unwrap()
    }

    pub fn create_generator(&self) -> IdGenerator<T> {
        IdGenerator {
            current: Arc::clone(&self.current),
            incrementer: Arc::clone(&self.incrementer),
        }
    }
}

impl<T> Debug for IdManager<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.current)
    }
}

#[derive(Clone)]
pub struct IdGenerator<T> {
    current: Arc<Mutex<T>>,
    incrementer: Arc<fn(val: &T) -> T>,
}

impl<T> IdGenerator<T> {
    pub fn gen(&self) -> T
    where
        T: Copy,
    {
        gen(&self.current, &self.incrementer)
    }
}

#[inline]
fn gen<T: Copy>(current: &Arc<Mutex<T>>, incrementer: &Arc<fn(val: &T) -> T>) -> T {
    let mut current = current.lock().unwrap();
    let next = *current;
    *current = incrementer(current.deref());
    next
}

#[cfg(test)]
mod test {
    use crate::id_manager::IdManager;

    #[test]
    fn test_standard() {
        let mut id_manager: IdManager<i32> = IdManager::new(0, |x| x + 1);
        assert_eq!(id_manager.gen(), 0);
        assert_eq!(id_manager.gen(), 1);
        assert_eq!(id_manager.gen(), 2);

        id_manager.reset(111);
        assert_eq!(id_manager.gen(), 111);
        assert_eq!(id_manager.gen(), 112);
        assert_eq!(id_manager.gen(), 113);
    }

    #[test]
    fn test_in_multi_thread() {
        let id_manager = IdManager::new(0u64, |id| id + 1u64);
        let getter = id_manager.create_generator();

        let thread = std::thread::spawn(move || {
            for _ in 0..1000 {
                getter.gen();
            }
        });
        for _ in 0..1000 {
            id_manager.gen();
        }

        thread.join().unwrap();
        assert_eq!(2000, id_manager.gen());
    }
}
