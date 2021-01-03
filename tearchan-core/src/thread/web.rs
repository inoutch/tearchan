pub struct ThreadPool {}

impl ThreadPool {
    pub fn new(_task_num: usize) -> Self {
        Self {}
    }

    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        job();
    }

    pub fn join(&self) {
        // Not need to implement
    }
}

#[cfg(test)]
mod test {
    use crate::thread::ThreadPool;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    const TEST_TASKS: usize = 4;

    #[test]
    fn test_execute_and_join() {
        let counter = Arc::new(AtomicI32::new(0));
        let pool = ThreadPool::new(TEST_TASKS);
        for _ in 0..TEST_TASKS {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                counter.fetch_add(1, Ordering::Relaxed);
            });
        }
        pool.join();
        assert_eq!(counter.load(Ordering::Relaxed), TEST_TASKS as i32);
    }
}
