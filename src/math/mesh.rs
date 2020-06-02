use crate::core::graphic::texture::TextureFrame;
use crate::math::vec::make_vec4_white;
use nalgebra_glm::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

#[derive(Clone)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub colors: Vec<Vec4>,
    pub texcoords: Vec<Vec2>,
    pub normals: Vec<Vec3>,
}

impl Mesh {
    pub fn new(
        positions: Vec<Vec3>,
        colors: Vec<Vec4>,
        texcoords: Vec<Vec2>,
        normals: Vec<Vec3>,
    ) -> Result<Mesh, &'static str> {
        Ok(Mesh {
            positions,
            colors,
            texcoords,
            normals,
        })
    }

    pub fn size(&self) -> usize {
        self.positions.len()
    }
}

#[derive(Default)]
pub struct MeshBuilder<TPositionsType, TColorsType, TTexcoordsType> {
    positions: TPositionsType,
    colors: TColorsType,
    texcoords: TTexcoordsType,
    normals: Vec<Vec3>,
}

impl MeshBuilder<(), (), ()> {
    pub fn new() -> MeshBuilder<(), (), ()> {
        MeshBuilder {
            positions: (),
            colors: (),
            texcoords: (),
            normals: vec![],
        }
    }
}

impl<TPositionsType, TColorsType, TTexcoordsType>
    MeshBuilder<TPositionsType, TColorsType, TTexcoordsType>
{
    pub fn with_square(self, size: Vec2) -> MeshBuilder<Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            positions: create_square_positions(vec2(0.0f32, 0.0f32), size),
            colors: create_square_colors(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32)),
            texcoords: create_square_texcoords(vec2(0.0f32, 0.0f32), vec2(1.0f32, 1.0f32)),
            normals: self.normals,
        }
    }

    pub fn with_cube(self, size: f32) -> MeshBuilder<Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            positions: create_cube_positions(size),
            colors: create_cube_colors(),
            texcoords: create_cute_texcoords(vec2(0.0f32, 0.0f32), vec2(1.0f32, 1.0f32)),
            normals: create_cube_normals(),
        }
    }

    pub fn with_frame(
        self,
        texture_size: Vec2,
        frame: &TextureFrame,
    ) -> MeshBuilder<Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            positions: create_square_positions_from_frame(frame),
            texcoords: create_square_texcoords_from_frame(texture_size, frame),
            colors: create_square_colors(make_vec4_white()),
            normals: create_square_normals(),
        }
    }

    pub fn with_model(self, model: &tobj::Model) -> MeshBuilder<Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        let mesh = &model.mesh;
        let mut positions: Vec<Vec3> = vec![];
        let mut colors: Vec<Vec4> = vec![];
        let mut texcoords: Vec<Vec2> = vec![];
        let mut normals: Vec<Vec3> = vec![];

        let mut next_face = 0;
        for f in 0..mesh.num_face_indices.len() {
            let end = next_face + mesh.num_face_indices[f] as usize;
            let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
            for v in face_indices {
                positions.push(vec3(
                    model.mesh.positions[(*v * 3) as usize],
                    model.mesh.positions[(*v * 3 + 1) as usize],
                    model.mesh.positions[(*v * 3 + 2) as usize],
                ));
                colors.push(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));
                texcoords.push(vec2(
                    model.mesh.texcoords[(*v * 2) as usize],
                    1.0f32 - model.mesh.texcoords[(*v * 2 + 1) as usize],
                ));
                normals.push(vec3(
                    model.mesh.normals[(*v * 3) as usize],
                    model.mesh.normals[(*v * 3 + 1) as usize],
                    model.mesh.normals[(*v * 3 + 2) as usize],
                ));
            }
            next_face = end;
        }

        MeshBuilder {
            positions,
            colors,
            texcoords,
            normals,
        }
    }

    pub fn positions(
        self,
        positions: Vec<Vec3>,
    ) -> MeshBuilder<Vec<Vec3>, TColorsType, TTexcoordsType> {
        MeshBuilder {
            positions,
            colors: self.colors,
            texcoords: self.texcoords,
            normals: self.normals,
        }
    }

    pub fn colors(
        self,
        colors: Vec<Vec4>,
    ) -> MeshBuilder<TPositionsType, Vec<Vec4>, TTexcoordsType> {
        MeshBuilder {
            positions: self.positions,
            colors,
            texcoords: self.texcoords,
            normals: self.normals,
        }
    }

    pub fn texcoords(
        self,
        texcoords: Vec<Vec2>,
    ) -> MeshBuilder<TPositionsType, TColorsType, Vec<Vec2>> {
        MeshBuilder {
            positions: self.positions,
            colors: self.colors,
            texcoords,
            normals: self.normals,
        }
    }

    pub fn normals(
        self,
        normals: Vec<Vec3>,
    ) -> MeshBuilder<TPositionsType, TColorsType, TTexcoordsType> {
        MeshBuilder {
            positions: self.positions,
            colors: self.colors,
            texcoords: self.texcoords,
            normals,
        }
    }
}

impl MeshBuilder<Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
    pub fn build(self) -> Result<Mesh, String> {
        if self.positions.len() == self.colors.len()
            && self.positions.len() == self.texcoords.len()
            && (self.positions.len() == self.normals.len() || self.normals.is_empty())
        {
            return Ok(Mesh {
                positions: self.positions,
                colors: self.colors,
                texcoords: self.texcoords,
                normals: self.normals,
            });
        }
        Err(format!(
            "Illegal vertex length: pos={}, col={}, tex={}, nom={}",
            self.positions.len(),
            self.colors.len(),
            self.texcoords.len(),
            self.normals.len()
        ))
    }
}

pub fn create_square_positions(position: Vec2, size: Vec2) -> Vec<Vec3> {
    return vec![
        Vec3::new(position.x, position.y, 0.0f32),
        Vec3::new(position.x + size.x, position.y + size.y, 0.0f32),
        Vec3::new(position.x, position.y + size.y, 0.0f32),
        Vec3::new(position.x, position.y, 0.0f32),
        Vec3::new(position.x + size.x, position.y, 0.0f32),
        Vec3::new(position.x + size.x, position.y + size.y, 0.0f32),
    ];
}

pub fn create_square_colors(color: Vec4) -> Vec<Vec4> {
    return vec![color, color, color, color, color, color];
}

pub fn create_square_texcoords(position: Vec2, size: Vec2) -> Vec<Vec2> {
    return vec![
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
    ];
}

pub fn create_square_normals() -> Vec<Vec3> {
    return vec![
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
    ];
}

pub fn create_cube_positions(size: f32) -> Vec<Vec3> {
    let h = size / 2.0f32;
    vec![
        vec3(-h, -h, -h),
        vec3(-h, -h, h),
        vec3(-h, h, h),
        vec3(-h, -h, -h),
        vec3(-h, h, h),
        vec3(-h, h, -h),
        vec3(h, h, -h), //
        vec3(-h, -h, -h),
        vec3(-h, h, -h),
        vec3(h, h, -h), //
        vec3(h, -h, -h),
        vec3(-h, -h, -h),
        vec3(h, -h, h), //
        vec3(-h, -h, -h),
        vec3(h, -h, -h),
        vec3(h, -h, h), //
        vec3(-h, -h, h),
        vec3(-h, -h, -h),
        vec3(h, h, h), //
        vec3(h, -h, -h),
        vec3(h, h, -h),
        vec3(h, -h, -h), //
        vec3(h, h, h),
        vec3(h, -h, h),
        vec3(h, h, h), //
        vec3(h, h, -h),
        vec3(-h, h, -h),
        vec3(h, h, h), //
        vec3(-h, h, -h),
        vec3(-h, h, h),
        vec3(-h, h, h), //
        vec3(-h, -h, h),
        vec3(h, -h, h),
        vec3(h, h, h), //
        vec3(-h, h, h),
        vec3(h, -h, h),
    ]
}

pub fn create_cute_texcoords(position: Vec2, size: Vec2) -> Vec<Vec2> {
    vec![
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
        vec2(position.x, position.y),
        vec2(position.x, position.y + size.y),
        vec2(position.x + size.x, position.y + size.y),
        vec2(position.x + size.x, position.y),
    ]
}

pub fn create_cube_colors() -> Vec<Vec4> {
    vec![
        vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
        vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
    ]
}

pub fn create_cube_normals() -> Vec<Vec3> {
    vec![
        vec3(-1.0f32, 0.0f32, 0.0f32),
        vec3(-1.0f32, 0.0f32, 0.0f32),
        vec3(-1.0f32, 0.0f32, 0.0f32),
        vec3(-1.0f32, 0.0f32, 0.0f32),
        vec3(-1.0f32, 0.0f32, 0.0f32),
        vec3(-1.0f32, 0.0f32, 0.0f32),
        vec3(0.0f32, 0.0f32, -1.0f32),
        vec3(0.0f32, 0.0f32, -1.0f32),
        vec3(0.0f32, 0.0f32, -1.0f32),
        vec3(0.0f32, 0.0f32, -1.0f32),
        vec3(0.0f32, 0.0f32, -1.0f32),
        vec3(0.0f32, 0.0f32, -1.0f32),
        vec3(0.0f32, -1.0f32, 0.0f32),
        vec3(0.0f32, -1.0f32, 0.0f32),
        vec3(0.0f32, -1.0f32, 0.0f32),
        vec3(0.0f32, -1.0f32, 0.0f32),
        vec3(0.0f32, -1.0f32, 0.0f32),
        vec3(0.0f32, -1.0f32, 0.0f32),
        vec3(1.0f32, 0.0f32, 0.0f32),
        vec3(1.0f32, 0.0f32, 0.0f32),
        vec3(1.0f32, 0.0f32, 0.0f32),
        vec3(1.0f32, 0.0f32, 0.0f32),
        vec3(1.0f32, 0.0f32, 0.0f32),
        vec3(1.0f32, 0.0f32, 0.0f32),
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec3(0.0f32, 0.0f32, 1.0f32),
        vec3(0.0f32, 0.0f32, 1.0f32),
        vec3(0.0f32, 0.0f32, 1.0f32),
        vec3(0.0f32, 0.0f32, 1.0f32),
        vec3(0.0f32, 0.0f32, 1.0f32),
        vec3(0.0f32, 0.0f32, 1.0f32),
    ]
}

pub fn create_square_positions_from_frame(frame: &TextureFrame) -> Vec<Vec3> {
    let sx = frame.source.x as f32;
    let sy = frame.source.y as f32;
    let sw = frame.rect.w as f32;
    let sh = frame.rect.h as f32;
    create_square_positions(vec2(sx, sy), vec2(sw, sh))
}

pub fn create_square_texcoords_from_frame(texture_size: Vec2, frame: &TextureFrame) -> Vec<Vec2> {
    let fx = frame.rect.x as f32 / texture_size.x;
    let fy = frame.rect.y as f32 / texture_size.y;
    let fw = frame.rect.w as f32 / texture_size.x;
    let fh = frame.rect.h as f32 / texture_size.y;
    create_square_texcoords(vec2(fx, fy), vec2(fw, fh))
}

#[test]
fn test_manually() {
    let mesh = MeshBuilder::new()
        .positions(vec![vec3(0.0f32, 1.0f32, 2.0f32)])
        .colors(vec![vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)])
        .texcoords(vec![vec2(7.0f32, 8.0f32)])
        .normals(vec![vec3(9.0f32, 10.0f32, 11.0f32)])
        .build()
        .unwrap();

    assert_eq!(mesh.positions, [vec3(0.0f32, 1.0f32, 2.0f32)]);
    assert_eq!(mesh.colors, [vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)]);
    assert_eq!(mesh.texcoords, [vec2(7.0f32, 8.0f32)]);
    assert_eq!(mesh.normals, [vec3(9.0f32, 10.0f32, 11.0f32)]);
}

#[test]
fn test_manually_failed() {
    let mesh1 = MeshBuilder::new()
        .positions(vec![
            vec3(0.0f32, 1.0f32, 2.0f32),
            vec3(0.0f32, 1.0f32, 2.0f32),
        ])
        .colors(vec![vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)])
        .texcoords(vec![vec2(7.0f32, 8.0f32)])
        .normals(vec![vec3(9.0f32, 10.0f32, 11.0f32)])
        .build();

    assert!(mesh1.is_err());

    let mesh2 = MeshBuilder::new()
        .positions(vec![vec3(0.0f32, 1.0f32, 2.0f32)])
        .colors(vec![
            vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32),
            vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32),
        ])
        .texcoords(vec![vec2(7.0f32, 8.0f32)])
        .normals(vec![vec3(9.0f32, 10.0f32, 11.0f32)])
        .build();

    assert!(mesh2.is_err());

    let mesh3 = MeshBuilder::new()
        .positions(vec![vec3(0.0f32, 1.0f32, 2.0f32)])
        .colors(vec![vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)])
        .texcoords(vec![vec2(7.0f32, 8.0f32), vec2(7.0f32, 8.0f32)])
        .normals(vec![vec3(9.0f32, 10.0f32, 11.0f32)])
        .build();

    assert!(mesh3.is_err());

    let mesh4 = MeshBuilder::new()
        .positions(vec![vec3(0.0f32, 1.0f32, 2.0f32)])
        .colors(vec![vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)])
        .texcoords(vec![vec2(7.0f32, 8.0f32)])
        .normals(vec![
            vec3(9.0f32, 10.0f32, 11.0f32),
            vec3(9.0f32, 10.0f32, 11.0f32),
        ])
        .build();

    assert!(mesh4.is_err());
}

#[test]
fn test_square() {
    let mesh = MeshBuilder::new()
        .with_square(vec2(32.0f32, 32.0f32))
        .build()
        .unwrap();

    assert_eq!(
        mesh.positions,
        [
            vec3(0.0f32, 0.0f32, 0.0f32),
            vec3(32.0f32, 32.0f32, 0.0f32),
            vec3(0.0f32, 32.0f32, 0.0f32),
            vec3(0.0f32, 0.0f32, 0.0f32),
            vec3(32.0f32, 0.0f32, 0.0f32),
            vec3(32.0f32, 32.0f32, 0.0f32),
        ]
    );

    assert_eq!(
        mesh.colors,
        [
            vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32),
        ]
    );

    assert_eq!(
        mesh.texcoords,
        [
            vec2(0.0f32, 1.0f32),
            vec2(1.0f32, 0.0f32),
            vec2(0.0f32, 0.0f32),
            vec2(0.0f32, 1.0f32),
            vec2(1.0f32, 1.0f32),
            vec2(1.0f32, 0.0f32)
        ]
    );

    assert_eq!(mesh.normals.len(), 0);
}
