#[macro_export]
macro_rules! get_mut_or_insert {
    ($map:expr, $key:expr, $f:block) => {
        if let Some(x) = $map.get_mut(&$key) {
            x
        } else {
            let new = $f;
            $map.insert($key, new);
            $map.get_mut(&$key).unwrap()
        }
    };
}
