use std::path::Path;

pub fn get_path_without_prefix(path: &Path, prefix: &str) -> Option<String> {
    if !path.starts_with(prefix) {
        return None;
    }

    path.to_str()
        .map(|path_str| path_str[prefix.len()..].to_string())
}

#[cfg(test)]
mod test {
    use crate::string::get_path_without_prefix;
    use std::path::PathBuf;

    #[test]
    fn test_get_patch_without_prefix_prefix() {
        let mut path = PathBuf::new();
        path.push("@assets://");
        path.push("fruits");
        path.push("apple.jpg");

        let mut expect = PathBuf::new();
        expect.push("fruits");
        expect.push("apple.jpg");
        assert_eq!(
            get_path_without_prefix(&path, "@assets://"),
            expect.to_str().map(|str| str.to_string())
        );
    }

    #[test]
    fn test_get_patch_without_prefix_no_prefix() {
        let path = PathBuf::from("fruits/apple.jpg");
        assert_eq!(get_path_without_prefix(&path, "@assets://"), None);
    }
}
