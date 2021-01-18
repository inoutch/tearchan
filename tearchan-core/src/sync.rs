#[cfg(not(target_arch = "wasm32"))]
pub fn run<F: futures::Future>(f: F) {
    futures::executor::block_on(f);
}

#[cfg(target_arch = "wasm32")]
pub fn run<F>(f: F)
where
    F: futures::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(f);
}
