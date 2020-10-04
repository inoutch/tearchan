use std::error::Error;
use std::fs::File;
use std::io::Write;
use tearchan_utility::texture::{Size, TextureAtlas, TextureFrame};
use texture_packer::exporter::ImageExporter;
use texture_packer::importer::ImageImporter;
use texture_packer::texture::Texture;
use texture_packer::{TexturePacker, TexturePackerConfig};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=data/img");

    let t1 = include_bytes!("./data/img/go_1.png");
    let t2 = include_bytes!("./data/img/go_2.png");
    let t3 = include_bytes!("./data/img/go_3.png");
    let t4 = include_bytes!("./data/img/go_4.png");
    let t5 = include_bytes!("./data/img/go_5.png");
    let t6 = include_bytes!("./data/img/go_6.png");
    let t7 = include_bytes!("./data/img/go_7.png");
    let t8 = include_bytes!("./data/img/go_8.png");
    let config = TexturePackerConfig {
        max_width: 1024,
        max_height: 1024,
        allow_rotation: false,
        texture_outlines: false,
        border_padding: 2,
        ..Default::default()
    };

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

    {
        let exporter = ImageExporter::export(&packer).unwrap();
        let mut file = File::create("./data/sprites/skeleton.png").unwrap();
        exporter
            .write_to(&mut file, image::ImageFormat::Png)
            .unwrap();
    }

    {
        let texture_atlas = TextureAtlas {
            image: "skeleton.png".to_string(),
            size: Size::new(packer.width(), packer.height()),
            frames: packer
                .get_frames()
                .iter()
                .map(|(_, frame)| TextureFrame::from(frame.clone()))
                .collect(),
        };
        let texture_atlas_json = serde_json::to_string(&texture_atlas).unwrap();
        let mut file = File::create("./data/sprites/skeleton.json").unwrap();
        file.write_all(texture_atlas_json.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}
