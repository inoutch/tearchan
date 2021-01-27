// #[cfg(target_arch = "wasm32")]
mod web;

// #[cfg(not(target_arch = "wasm32"))]
// pub type ThreadPool = threadpool::ThreadPool;

// #[cfg(target_arch = "wasm32")]
pub type ThreadPool = web::ThreadPool;
