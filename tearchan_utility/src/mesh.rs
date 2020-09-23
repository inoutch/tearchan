use crate::math::vec::{vec2_zero, vec4_white};
use crate::mesh::cube::{
    create_cube_colors, create_cube_indices, create_cube_normals, create_cube_positions,
    create_cube_texcoords,
};
use crate::mesh::obj::create_elements_from_mesh;
use crate::mesh::square::{
    create_square_colors, create_square_indices, create_square_normals, create_square_positions,
    create_square_positions_from_frame, create_square_texcoords,
    create_square_texcoords_from_frame,
};
use crate::rect::{rect2, rect3, Rect2, Rect3};
use crate::texture::TextureFrame;
use nalgebra_glm::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
use std::ops::Range;

pub type IndexType = u32;

#[derive(Clone, Debug)]
pub struct Mesh {
    pub indices: Vec<IndexType>,
    pub positions: Vec<Vec3>,
    pub colors: Vec<Vec4>,
    pub texcoords: Vec<Vec2>,
    pub normals: Vec<Vec3>,
}

impl Mesh {
    pub fn new(
        indices: Vec<IndexType>,
        positions: Vec<Vec3>,
        colors: Vec<Vec4>,
        texcoords: Vec<Vec2>,
        normals: Vec<Vec3>,
    ) -> Result<Mesh, &'static str> {
        Ok(Mesh {
            indices,
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
pub struct MeshBuilder<TIndicesType, TPositionsType, TColorsType, TTexcoordsType> {
    indices: TIndicesType,
    positions: TPositionsType,
    colors: TColorsType,
    texcoords: TTexcoordsType,
    normals: Vec<Vec3>,
}

impl MeshBuilder<(), (), (), ()> {
    pub fn new() -> MeshBuilder<(), (), (), ()> {
        MeshBuilder {
            indices: (),
            positions: (),
            colors: (),
            texcoords: (),
            normals: vec![],
        }
    }
}

impl<TIndicesType, TPositionsType, TColorsType, TTexcoordsType>
    MeshBuilder<TIndicesType, TPositionsType, TColorsType, TTexcoordsType>
{
    pub fn with_square(
        self,
        size: Vec2,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            indices: create_square_indices(),
            positions: create_square_positions(&Rect2 {
                origin: vec2(0.0f32, 0.0f32),
                size,
            }),
            colors: create_square_colors(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32)),
            texcoords: create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32)),
            normals: self.normals,
        }
    }

    pub fn with_square_and_color(
        self,
        size: Vec2,
        color: Vec4,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            indices: create_square_indices(),
            positions: create_square_positions(&Rect2 {
                origin: vec2(0.0f32, 0.0f32),
                size,
            }),
            colors: create_square_colors(color),
            texcoords: create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32)),
            normals: self.normals,
        }
    }

    pub fn with_cube(
        self,
        rect: &Rect3<f32>,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            indices: create_cube_indices(),
            positions: create_cube_positions(rect),
            colors: create_cube_colors(),
            texcoords: create_cube_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32)),
            normals: create_cube_normals(),
        }
    }

    pub fn with_simple_cube(
        self,
        size: f32,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        let half = size / 2.0f32;
        MeshBuilder {
            indices: create_cube_indices(),
            positions: create_cube_positions(&rect3(-half, -half, -half, size, size, size)),
            colors: create_cube_colors(),
            texcoords: create_cube_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32)),
            normals: create_cube_normals(),
        }
    }

    pub fn with_frame(
        self,
        texture_size: Vec2,
        frame: &TextureFrame,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        MeshBuilder {
            indices: create_square_indices(),
            positions: create_square_positions_from_frame(&vec2_zero(), frame),
            texcoords: create_square_texcoords_from_frame(texture_size, frame),
            colors: create_square_colors(vec4_white()),
            normals: create_square_normals(),
        }
    }

    pub fn with_model(
        self,
        model: &tobj::Model,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        let mut indices: Vec<IndexType> = vec![];
        let mut positions: Vec<Vec3> = vec![];
        let mut colors: Vec<Vec4> = vec![];
        let mut texcoords: Vec<Vec2> = vec![];
        let mut normals: Vec<Vec3> = vec![];

        create_elements_from_mesh(
            &mut indices,
            &mut positions,
            &mut colors,
            &mut texcoords,
            &mut normals,
            &model.mesh,
            &rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32),
        );

        MeshBuilder {
            indices,
            positions,
            colors,
            texcoords,
            normals,
        }
    }

    pub fn with_model_and_frame(
        self,
        texture_size: Vec2,
        frame: &TextureFrame,
        model: &tobj::Model,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        let mut indices: Vec<IndexType> = vec![];
        let mut positions: Vec<Vec3> = vec![];
        let mut colors: Vec<Vec4> = vec![];
        let mut texcoords: Vec<Vec2> = vec![];
        let mut normals: Vec<Vec3> = vec![];

        let fx = frame.rect.x as f32 / texture_size.x;
        let fy = frame.rect.y as f32 / texture_size.y;
        let fw = frame.rect.w as f32 / texture_size.x;
        let fh = frame.rect.h as f32 / texture_size.y;
        create_elements_from_mesh(
            &mut indices,
            &mut positions,
            &mut colors,
            &mut texcoords,
            &mut normals,
            &model.mesh,
            &rect2(fx, fy, fw, fh),
        );

        MeshBuilder {
            indices,
            positions,
            colors,
            texcoords,
            normals,
        }
    }

    pub fn with_models_and_frame(
        self,
        texture_size: Vec2,
        frame: &TextureFrame,
        models: Vec<&tobj::Model>,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        let mut indices: Vec<IndexType> = vec![];
        let mut positions: Vec<Vec3> = vec![];
        let mut colors: Vec<Vec4> = vec![];
        let mut texcoords: Vec<Vec2> = vec![];
        let mut normals: Vec<Vec3> = vec![];

        let fx = frame.rect.x as f32 / texture_size.x;
        let fy = frame.rect.y as f32 / texture_size.y;
        let fw = frame.rect.w as f32 / texture_size.x;
        let fh = frame.rect.h as f32 / texture_size.y;
        for model in models {
            create_elements_from_mesh(
                &mut indices,
                &mut positions,
                &mut colors,
                &mut texcoords,
                &mut normals,
                &model.mesh,
                &rect2(fx, fy, fw, fh),
            );
        }

        MeshBuilder {
            indices,
            positions,
            colors,
            texcoords,
            normals,
        }
    }

    pub fn with_grid(
        self,
        interval: f32,
        range: Range<(i32, i32)>,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        debug_assert_ne!(range.start, range.end);

        let mut indices = vec![];
        let mut positions = vec![];
        let mut colors = vec![];

        let p1x = range.start.0 as f32 * interval;
        let p2x = range.end.0 as f32 * interval;
        let p1y = range.start.1 as f32 * interval;
        let p2y = range.end.1 as f32 * interval;

        for x in range.start.0..=range.end.0 {
            let p1x = x as f32 * interval;
            positions.push(vec3(p1x, p1y, 0.0f32));
            positions.push(vec3(p1x, p2y, 0.0f32));
            colors.push(vec4_white());
            colors.push(vec4_white());

            indices.push(indices.len() as IndexType);
            indices.push(indices.len() as IndexType);
        }

        for y in range.start.1..=range.end.1 {
            let p1y = y as f32 * interval;
            positions.push(vec3(p1x, p1y, 0.0f32));
            positions.push(vec3(p2x, p1y, 0.0f32));
            colors.push(vec4_white());
            colors.push(vec4_white());

            indices.push(indices.len() as IndexType);
            indices.push(indices.len() as IndexType);
        }

        MeshBuilder {
            indices,
            positions,
            colors,
            texcoords: vec![],
            normals: vec![],
        }
    }

    pub fn with_lines(
        self,
        lines: Vec<(Vec3, Vec3)>,
        color: Vec4,
    ) -> MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
        let mut indices = vec![];
        let mut positions = vec![];
        let mut colors = vec![];

        for (start, end) in lines {
            indices.push(indices.len() as IndexType);
            indices.push(indices.len() as IndexType);
            positions.push(start);
            positions.push(end);
            colors.push(color.clone_owned());
            colors.push(color.clone_owned());
        }

        MeshBuilder {
            indices,
            positions,
            colors,
            texcoords: vec![],
            normals: vec![],
        }
    }

    pub fn indices(
        self,
        indices: Vec<IndexType>,
    ) -> MeshBuilder<Vec<IndexType>, TPositionsType, TColorsType, TTexcoordsType> {
        MeshBuilder {
            indices,
            positions: self.positions,
            colors: self.colors,
            texcoords: self.texcoords,
            normals: self.normals,
        }
    }

    pub fn positions(
        self,
        positions: Vec<Vec3>,
    ) -> MeshBuilder<TIndicesType, Vec<Vec3>, TColorsType, TTexcoordsType> {
        MeshBuilder {
            indices: self.indices,
            positions,
            colors: self.colors,
            texcoords: self.texcoords,
            normals: self.normals,
        }
    }

    pub fn colors(
        self,
        colors: Vec<Vec4>,
    ) -> MeshBuilder<TIndicesType, TPositionsType, Vec<Vec4>, TTexcoordsType> {
        MeshBuilder {
            indices: self.indices,
            positions: self.positions,
            colors,
            texcoords: self.texcoords,
            normals: self.normals,
        }
    }

    pub fn texcoords(
        self,
        texcoords: Vec<Vec2>,
    ) -> MeshBuilder<TIndicesType, TPositionsType, TColorsType, Vec<Vec2>> {
        MeshBuilder {
            indices: self.indices,
            positions: self.positions,
            colors: self.colors,
            texcoords,
            normals: self.normals,
        }
    }

    pub fn normals(
        self,
        normals: Vec<Vec3>,
    ) -> MeshBuilder<TIndicesType, TPositionsType, TColorsType, TTexcoordsType> {
        MeshBuilder {
            indices: self.indices,
            positions: self.positions,
            colors: self.colors,
            texcoords: self.texcoords,
            normals,
        }
    }
}

impl MeshBuilder<Vec<IndexType>, Vec<Vec3>, Vec<Vec4>, Vec<Vec2>> {
    pub fn build(self) -> Result<Mesh, String> {
        if self.positions.len() == self.colors.len()
            && (self.positions.len() == self.texcoords.len() || self.texcoords.is_empty())
            && (self.positions.len() == self.normals.len() || self.normals.is_empty())
        {
            return Ok(Mesh {
                indices: self.indices,
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

pub mod square {
    use crate::mesh::IndexType;
    use crate::rect::{rect2, Rect2};
    use crate::texture::TextureFrame;
    use nalgebra_glm::{vec2, Vec2, Vec3, Vec4};

    pub fn create_square_indices() -> Vec<IndexType> {
        // Index order →x ↑y
        // i2 --------- i1,5
        // |          /  |
        // |       /     |
        // |    /        |
        // | /           |
        // i0,3 ------- i2,4
        // Position order
        // p2 --------- p3
        // |          /  |
        // |       /     |
        // |    /        |
        // | /           |
        // p0 --------- p1
        vec![0, 3, 2, 0, 1, 3]
    }

    pub fn create_square_indices_with_offset(offset: IndexType) -> Vec<IndexType> {
        // Index order →x ↑y
        // i2 --------- i1,5
        // |          /  |
        // |       /     |
        // |    /        |
        // | /           |
        // i0,3 ------- i2,4
        // Position order
        // p2 --------- p3
        // |          /  |
        // |       /     |
        // |    /        |
        // | /           |
        // p0 --------- p1
        vec![
            offset,
            offset + 3,
            offset + 2,
            offset,
            offset + 1,
            offset + 3,
        ]
    }

    pub fn create_square_positions(rect: &Rect2<f32>) -> Vec<Vec3> {
        // Position order →x ↑y
        // p2 --------- p3
        // |          /  |
        // |       /     |
        // |    /        |
        // | /           |
        // p0 --------- p1
        vec![
            Vec3::new(rect.origin.x, rect.origin.y, 0.0f32),
            Vec3::new(rect.origin.x + rect.size.x, rect.origin.y, 0.0f32),
            Vec3::new(rect.origin.x, rect.origin.y + rect.size.y, 0.0f32),
            Vec3::new(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                0.0f32,
            ),
        ]
    }

    pub fn create_square_colors(color: Vec4) -> Vec<Vec4> {
        // Position order →x ↓y
        // t2 --------- t3
        // |          /  |
        // |       /     |
        // |    /        |
        // | /           |
        // t0 --------- t1
        return vec![color, color, color, color];
    }

    pub fn create_square_texcoords(rect: &Rect2<f32>) -> Vec<Vec2> {
        return vec![
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
        ];
    }

    pub fn create_square_normals() -> Vec<Vec3> {
        return vec![
            Vec3::new(0.0f32, 0.0f32, 1.0f32),
            Vec3::new(0.0f32, 0.0f32, 1.0f32),
            Vec3::new(0.0f32, 0.0f32, 1.0f32),
            Vec3::new(0.0f32, 0.0f32, 1.0f32),
        ];
    }

    pub fn create_square_positions_from_frame(origin: &Vec2, frame: &TextureFrame) -> Vec<Vec3> {
        let sx = frame.source.x as f32;
        let sy = frame.source.y as f32;
        let sw = frame.rect.w as f32;
        let sh = frame.rect.h as f32;
        create_square_positions(&Rect2 {
            origin: vec2(sx + origin.x, sy + origin.y),
            size: vec2(sw, sh),
        })
    }

    pub fn create_square_positions_from_frame_with_ratio(
        origin: &Vec2,
        frame: &TextureFrame,
        ratio: &Vec2,
    ) -> Vec<Vec3> {
        let mw = frame.source.w as f32 * ratio.x;
        let mh = frame.source.h as f32 * ratio.y;
        let sx = (frame.source.x as f32).min(mw);
        let sy = (frame.source.y as f32).min(mh);
        let sw = (frame.rect.w as f32).min(mw - sx);
        let sh = (frame.rect.h as f32).min(mh - sy);
        create_square_positions(&Rect2 {
            origin: vec2(sx + origin.x, sy + origin.y),
            size: vec2(sw, sh),
        })
    }

    pub fn create_square_texcoords_from_frame(
        texture_size: Vec2,
        frame: &TextureFrame,
    ) -> Vec<Vec2> {
        let fx = frame.rect.x as f32 / texture_size.x;
        let fy = frame.rect.y as f32 / texture_size.y;
        let fw = frame.rect.w as f32 / texture_size.x;
        let fh = frame.rect.h as f32 / texture_size.y;
        create_square_texcoords(&rect2(fx, fy, fw, fh))
    }

    pub fn create_square_texcoords_from_frame_with_ratio(
        texture_size: Vec2,
        frame: &TextureFrame,
        ratio: &Vec2,
    ) -> Vec<Vec2> {
        let fx = frame.rect.x as f32 / texture_size.x;
        let fy = frame.rect.y as f32 / texture_size.y;
        let fw = frame.rect.w as f32 / texture_size.x * ratio.x;
        let fh = frame.rect.h as f32 / texture_size.y * ratio.y;
        create_square_texcoords(&rect2(fx, fy, fw, fh))
    }

    #[cfg(test)]
    mod test {
        use crate::math::vec::vec4_white;
        use crate::mesh::square::{
            create_square_colors, create_square_indices, create_square_normals,
            create_square_positions, create_square_texcoords,
        };
        use crate::rect::{rect2, Rect2};
        use nalgebra_glm::vec2;

        #[test]
        fn test_len() {
            let indices = create_square_indices();
            let positions = create_square_positions(&Rect2 {
                origin: vec2(0.0f32, 0.0f32),
                size: vec2(1.0f32, 1.0f32),
            });
            let colors = create_square_colors(vec4_white());
            let texcoords = create_square_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32));
            let normals = create_square_normals();
            assert_eq!(indices.len(), 6);
            assert_eq!(positions.len(), 4);
            assert_eq!(positions.len(), colors.len());
            assert_eq!(colors.len(), texcoords.len());
            assert_eq!(texcoords.len(), normals.len());
        }
    }
}

pub mod cube {
    use crate::mesh::IndexType;
    use crate::rect::{Rect2, Rect3};
    use nalgebra_glm::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

    // Index order
    // x→ y↑ z↓→
    //      i5,8,26,28 ---------- i6,9,20,25
    //               | \         | \
    //               |   \       |   \
    //            i2,4,29,30 --------- i18,22,24,27,32,33
    //               |     |     |     |
    //               |     |     |     |
    // i0,3,7,11,13,17 ----|---- i10,14|19,21
    //                 \   |       \   |
    //                   \ |         \ |
    //           i1,16,31,34 --------- i12,15,23,35
    // Position order
    // x→ y↑ z↓→
    //         p4,9,21 ---------- p5,19,23
    //               | \         | \
    //               |   \       |   \
    //              p6,11,13 --------- p7,15,17
    //               |     |     |     |
    //               |     |     |     |
    //         p0,8,20 ----|---- p1,18,|22
    //                 \   |       \   |
    //                   \ |         \ |
    //              p2,10,12 --------- p3,14,16
    //              p4 ---------- p5
    //               | \         | \
    //               |   \       |   \
    //               |    p6 --------- p7
    //               |     |     |     |
    //               |     |     |     |
    //              p0 ----|---- p1    |
    //                 \   |       \   |
    //                   \ |         \ |
    //                    p2 --------- p3
    pub fn create_cube_indices() -> Vec<IndexType> {
        vec![
            8, 10, 11, // 0
            8, 11, 9, // 3
            23, 20, 21, // 6
            23, 22, 20, // 9
            3, 0, 1, //12
            3, 2, 0, //15
            17, 18, 19, //18
            18, 17, 16, //21
            7, 5, 4, //24
            7, 4, 6, //27
            13, 12, 15, //30
            15, 12, 14, //33
        ]
    }

    pub fn create_cube_positions(rect: &Rect3<f32>) -> Vec<Vec3> {
        vec![
            // face: 0
            vec3(rect.origin.x, rect.origin.y, rect.origin.z), // 0
            vec3(rect.origin.x + rect.size.x, rect.origin.y, rect.origin.z), // 1
            vec3(rect.origin.x, rect.origin.y, rect.origin.z + rect.size.z), // 2
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y,
                rect.origin.z + rect.size.z,
            ), // 3
            // face: 1
            vec3(rect.origin.x, rect.origin.y + rect.size.y, rect.origin.z), // 4
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                rect.origin.z,
            ), // 5
            vec3(
                rect.origin.x,
                rect.origin.y + rect.size.y,
                rect.origin.z + rect.size.z,
            ), // 6
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                rect.origin.z + rect.size.z,
            ), // 7
            // face: 2
            vec3(rect.origin.x, rect.origin.y, rect.origin.z), // 8
            vec3(rect.origin.x, rect.origin.y + rect.size.y, rect.origin.z), // 9
            vec3(rect.origin.x, rect.origin.y, rect.origin.z + rect.size.z), // 10
            vec3(
                rect.origin.x,
                rect.origin.y + rect.size.y,
                rect.origin.z + rect.size.z,
            ), // 11
            // face: 3
            vec3(rect.origin.x, rect.origin.y, rect.origin.z + rect.size.z), // 12
            vec3(
                rect.origin.x,
                rect.origin.y + rect.size.y,
                rect.origin.z + rect.size.z,
            ), // 13
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y,
                rect.origin.z + rect.size.z,
            ), // 14
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                rect.origin.z + rect.size.z,
            ), // 15
            // face: 4
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y,
                rect.origin.z + rect.size.z,
            ), // 16
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                rect.origin.z + rect.size.z,
            ), // 17
            vec3(rect.origin.x + rect.size.x, rect.origin.y, rect.origin.z), // 18
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                rect.origin.z,
            ), // 19
            // face: 5
            vec3(rect.origin.x, rect.origin.y, rect.origin.z), // 20
            vec3(rect.origin.x, rect.origin.y + rect.size.y, rect.origin.z), // 21
            vec3(rect.origin.x + rect.size.x, rect.origin.y, rect.origin.z), // 22
            vec3(
                rect.origin.x + rect.size.x,
                rect.origin.y + rect.size.y,
                rect.origin.z,
            ), // 23
        ]
    }

    pub fn create_cube_texcoords(rect: &Rect2<f32>) -> Vec<Vec2> {
        vec![
            // face: 0
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            // face: 1
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y + rect.size.y),
            // face: 2
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
            // face: 3
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
            // face: 4
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
            // face: 5
            vec2(rect.origin.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x, rect.origin.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y + rect.size.y),
            vec2(rect.origin.x + rect.size.x, rect.origin.y),
        ]
    }

    pub fn create_cube_colors() -> Vec<Vec4> {
        vec![
            vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 1.0f32, 0.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(1.0f32, 0.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
            vec4(0.0f32, 1.0f32, 1.0f32, 1.0f32),
        ]
    }

    pub fn create_cube_normals() -> Vec<Vec3> {
        vec![
            // face: 0
            vec3(0.0f32, -1.0f32, 0.0f32),
            vec3(0.0f32, -1.0f32, 0.0f32),
            vec3(0.0f32, -1.0f32, 0.0f32),
            vec3(0.0f32, -1.0f32, 0.0f32),
            // face: 1
            vec3(0.0f32, 1.0f32, 0.0f32),
            vec3(0.0f32, 1.0f32, 0.0f32),
            vec3(0.0f32, 1.0f32, 0.0f32),
            vec3(0.0f32, 1.0f32, 0.0f32),
            // face: 2
            vec3(-1.0f32, 0.0f32, 0.0f32),
            vec3(-1.0f32, 0.0f32, 0.0f32),
            vec3(-1.0f32, 0.0f32, 0.0f32),
            vec3(-1.0f32, 0.0f32, 0.0f32),
            // face: 3
            vec3(0.0f32, 0.0f32, 1.0f32),
            vec3(0.0f32, 0.0f32, 1.0f32),
            vec3(0.0f32, 0.0f32, 1.0f32),
            vec3(0.0f32, 0.0f32, 1.0f32),
            // face: 4
            vec3(1.0f32, 0.0f32, 0.0f32),
            vec3(1.0f32, 0.0f32, 0.0f32),
            vec3(1.0f32, 0.0f32, 0.0f32),
            vec3(1.0f32, 0.0f32, 0.0f32),
            // face: 5
            vec3(0.0f32, 0.0f32, -1.0f32),
            vec3(0.0f32, 0.0f32, -1.0f32),
            vec3(0.0f32, 0.0f32, -1.0f32),
            vec3(0.0f32, 0.0f32, -1.0f32),
        ]
    }

    #[cfg(test)]
    mod test {
        use crate::mesh::cube::{
            create_cube_colors, create_cube_indices, create_cube_normals, create_cube_positions,
            create_cube_texcoords,
        };
        use crate::rect::{rect2, rect3};

        #[test]
        fn test_len() {
            let indices = create_cube_indices();
            assert_eq!(indices.len(), 36);

            let positions =
                create_cube_positions(&rect3(0.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 1.0f32));
            let colors = create_cube_colors();
            let texcoords = create_cube_texcoords(&rect2(0.0f32, 0.0f32, 1.0f32, 1.0f32));
            let normals = create_cube_normals();

            assert_eq!(positions.len(), 24);
            assert_eq!(positions.len(), colors.len());
            assert_eq!(colors.len(), texcoords.len());
            assert_eq!(texcoords.len(), normals.len());
        }
    }
}

pub mod obj {
    use crate::mesh::{convert_texcoord_into_rect, IndexType};
    use crate::rect::Rect2;
    use nalgebra_glm::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

    pub fn create_bundles_from_mesh(
        positions: &mut Vec<Vec3>,
        colors: &mut Vec<Vec4>,
        texcoords: &mut Vec<Vec2>,
        normals: &mut Vec<Vec3>,
        mesh: &tobj::Mesh,
        texture_rect: &Rect2<f32>,
    ) {
        let mut next_face = 0;
        for f in 0..mesh.num_face_indices.len() {
            let end = next_face + mesh.num_face_indices[f] as usize;
            let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
            for idx in face_indices {
                positions.push(vec3(
                    mesh.positions[(*idx * 3) as usize],
                    mesh.positions[(*idx * 3 + 1) as usize],
                    mesh.positions[(*idx * 3 + 2) as usize],
                ));
                colors.push(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));
                texcoords.push(convert_texcoord_into_rect(
                    mesh.texcoords[(*idx * 2) as usize],
                    1.0f32 - mesh.texcoords[(*idx * 2 + 1) as usize],
                    texture_rect,
                ));
                normals.push(vec3(
                    mesh.normals[(*idx * 3) as usize],
                    mesh.normals[(*idx * 3 + 1) as usize],
                    mesh.normals[(*idx * 3 + 2) as usize],
                ));
            }
            next_face = end;
        }
    }

    pub fn create_elements_from_mesh(
        indices: &mut Vec<IndexType>,
        positions: &mut Vec<Vec3>,
        colors: &mut Vec<Vec4>,
        texcoords: &mut Vec<Vec2>,
        normals: &mut Vec<Vec3>,
        mesh: &tobj::Mesh,
        texture_rect: &Rect2<f32>,
    ) {
        debug_assert!(mesh.positions.len() % 3 == 0);
        debug_assert!(mesh.texcoords.len() % 2 == 0);
        debug_assert!(mesh.normals.len() % 3 == 0);

        let index_offset = positions.len();
        let size = mesh.positions.len() / 3;
        debug_assert_eq!(
            size,
            if mesh.texcoords.is_empty() {
                size
            } else {
                mesh.texcoords.len() / 2
            }
        );
        debug_assert_eq!(size, mesh.normals.len() / 3);

        for i in 0..size {
            positions.push(vec3(
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ));
            colors.push(vec4(1.0f32, 1.0f32, 1.0f32, 1.0f32));
            if mesh.texcoords.is_empty() {
                texcoords.push(vec2(0.0f32, 0.0f32));
            } else {
                texcoords.push(convert_texcoord_into_rect(
                    mesh.texcoords[i * 2],
                    1.0f32 - mesh.texcoords[i * 2 + 1], // Need inverse for OBJ
                    texture_rect,
                ));
            }
            normals.push(vec3(
                mesh.normals[i * 3],
                mesh.normals[i * 3 + 1],
                mesh.normals[i * 3 + 2],
            ));
        }

        let mut next_face = 0;
        for face_num in 0..mesh.num_face_indices.len() {
            let end = next_face + mesh.num_face_indices[face_num] as usize;
            let face_indices: Vec<_> = mesh.indices[next_face..end].iter().collect();
            for idx in face_indices {
                indices.push(*idx + index_offset as IndexType);
            }
            next_face = end;
        }
    }
}

pub fn convert_texcoord_into_rect(u: f32, v: f32, rect: &Rect2<f32>) -> Vec2 {
    vec2(
        rect.origin.x + u * rect.size.x,
        rect.origin.y + v * rect.size.y,
    )
}

#[cfg(test)]
mod test {
    use crate::mesh::MeshBuilder;
    use nalgebra_glm::{vec2, vec3, vec4};

    #[test]
    fn test_manually() {
        let mesh = MeshBuilder::new()
            .indices(vec![0])
            .positions(vec![vec3(0.0f32, 1.0f32, 2.0f32)])
            .colors(vec![vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)])
            .texcoords(vec![vec2(7.0f32, 8.0f32)])
            .normals(vec![vec3(9.0f32, 10.0f32, 11.0f32)])
            .build()
            .unwrap();

        assert_eq!(mesh.indices, [0]);
        assert_eq!(mesh.positions, [vec3(0.0f32, 1.0f32, 2.0f32)]);
        assert_eq!(mesh.colors, [vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)]);
        assert_eq!(mesh.texcoords, [vec2(7.0f32, 8.0f32)]);
        assert_eq!(mesh.normals, [vec3(9.0f32, 10.0f32, 11.0f32)]);
    }

    #[test]
    fn test_manually_failed() {
        let mesh1 = MeshBuilder::new()
            .indices(vec![0])
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
            .indices(vec![0])
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
            .indices(vec![0])
            .positions(vec![vec3(0.0f32, 1.0f32, 2.0f32)])
            .colors(vec![vec4(3.0f32, 4.0f32, 5.0f32, 6.0f32)])
            .texcoords(vec![vec2(7.0f32, 8.0f32), vec2(7.0f32, 8.0f32)])
            .normals(vec![vec3(9.0f32, 10.0f32, 11.0f32)])
            .build();

        assert!(mesh3.is_err());

        let mesh4 = MeshBuilder::new()
            .indices(vec![0])
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

        assert_eq!(mesh.indices, vec![0, 3, 2, 0, 1, 3]);
        assert_eq!(
            mesh.positions,
            [
                vec3(0.0f32, 0.0f32, 0.0f32),
                vec3(32.0f32, 0.0f32, 0.0f32),
                vec3(0.0f32, 32.0f32, 0.0f32),
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
            ]
        );

        assert_eq!(
            mesh.texcoords,
            [
                vec2(0.0f32, 1.0f32),
                vec2(1.0f32, 1.0f32),
                vec2(0.0f32, 0.0f32),
                vec2(1.0f32, 0.0f32),
            ]
        );

        assert_eq!(mesh.normals.len(), 0);
    }
}
