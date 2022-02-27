use tearchan::core::io::file::FileUtil;

pub fn main() {
    let file_util = FileUtil::new().expect("Failed to create path util");
    let assets_path = file_util
        .assets_path()
        .to_str()
        .expect("Failed to get assets path");
    println!("{:?}", assets_path);
}
