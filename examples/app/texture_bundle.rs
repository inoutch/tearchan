use nalgebra_glm::vec2;
use tearchan::core::graphic::image::Image;
use tearchan::core::graphic::texture::{Size, TextureAtlas, TextureFrame};
use texture_packer::exporter::ImageExporter;
use texture_packer::importer::ImageImporter;
use texture_packer::texture::Texture;
use texture_packer::{TexturePacker, TexturePackerConfig};

pub fn generate_texture_bundle() -> (TextureAtlas, Image) {
    let config = TexturePackerConfig {
        max_width: 1024,
        max_height: 1024,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 2,
        ..Default::default()
    };

    let t1 = include_bytes!("../data/img/go_1.png");
    let t2 = include_bytes!("../data/img/go_2.png");
    let t3 = include_bytes!("../data/img/go_3.png");
    let t4 = include_bytes!("../data/img/go_4.png");
    let t5 = include_bytes!("../data/img/go_5.png");
    let t6 = include_bytes!("../data/img/go_6.png");
    let t7 = include_bytes!("../data/img/go_7.png");
    let t8 = include_bytes!("../data/img/go_8.png");

    let mut packer = TexturePacker::new_skyline(config);
    packer
        .pack_own(
            "go_1.png".to_string(),
            ImageImporter::import_from_memory(t1).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_2.png".to_string(),
            ImageImporter::import_from_memory(t2).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_3.png".to_string(),
            ImageImporter::import_from_memory(t3).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_4.png".to_string(),
            ImageImporter::import_from_memory(t4).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_5.png".to_string(),
            ImageImporter::import_from_memory(t5).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_6.png".to_string(),
            ImageImporter::import_from_memory(t6).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_7.png".to_string(),
            ImageImporter::import_from_memory(t7).unwrap(),
        )
        .unwrap();
    packer
        .pack_own(
            "go_8.png".to_string(),
            ImageImporter::import_from_memory(t8).unwrap(),
        )
        .unwrap();

    let texture_atlas = TextureAtlas::new(
        "skeleton.png".to_string(),
        Size::new(packer.width(), packer.height()),
        packer
            .get_frames()
            .iter()
            .map(|(_, frame)| TextureFrame::from(frame.clone()))
            .collect(),
    );
    let dynamic_image = ImageExporter::export(&packer).unwrap();
    let image = Image::new(
        dynamic_image.to_rgba().to_vec(),
        vec2(packer.width(), packer.height()),
    );
    (texture_atlas, image)
}

pub fn generate_window_texture_bundle() -> (TextureAtlas, Image) {
    let config = TexturePackerConfig {
        max_width: 1024,
        max_height: 1024,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 2,
        ..Default::default()
    };

    let mut binary_array: Vec<(&'static [u8], &'static str)> = vec![];
    binary_array.push((include_bytes!("../data/img/window_0.png"), "window_0.png"));
    binary_array.push((include_bytes!("../data/img/window_1.png"), "window_1.png"));
    binary_array.push((include_bytes!("../data/img/window_2.png"), "window_2.png"));
    binary_array.push((include_bytes!("../data/img/window_3.png"), "window_3.png"));
    binary_array.push((include_bytes!("../data/img/window_4.png"), "window_4.png"));
    binary_array.push((include_bytes!("../data/img/window_5.png"), "window_5.png"));
    binary_array.push((include_bytes!("../data/img/window_6.png"), "window_6.png"));
    binary_array.push((include_bytes!("../data/img/window_7.png"), "window_7.png"));
    binary_array.push((include_bytes!("../data/img/window_8.png"), "window_8.png"));

    let mut packer = TexturePacker::new_skyline(config);
    for (binary, name) in binary_array {
        packer
            .pack_own(
                name.to_string(),
                ImageImporter::import_from_memory(binary).unwrap(),
            )
            .unwrap();
    }

    let texture_atlas = TextureAtlas::new(
        "window.png".to_string(),
        Size::new(packer.width(), packer.height()),
        packer
            .get_frames()
            .iter()
            .map(|(_, frame)| TextureFrame::from(frame.clone()))
            .collect(),
    );
    let dynamic_image = ImageExporter::export(&packer).unwrap();
    let image = Image::new(
        dynamic_image.to_rgba().to_vec(),
        vec2(packer.width(), packer.height()),
    );

    (texture_atlas, image)
}
