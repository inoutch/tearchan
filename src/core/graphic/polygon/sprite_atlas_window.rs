use crate::core::graphic::polygon::polygon_2d::{
    Polygon2DInterface, Polygon2DProvider, Polygon2DProviderInterface,
};
use crate::core::graphic::polygon::{Polygon, PolygonCore, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::{clone_shared, make_shared, Shared};
use crate::math::mesh::square::{
    create_square_colors, create_square_indices_with_offset, create_square_positions_from_frame,
    create_square_positions_from_frame_with_ratio, create_square_texcoords_from_frame,
    create_square_texcoords_from_frame_with_ratio,
};
use crate::math::mesh::{IndexType, Mesh, MeshBuilder};
use crate::math::vec::{make_vec2_zero, make_vec4_white};
use nalgebra_glm::{vec2, Mat4, Vec2};

pub struct SpriteAtlasWindowProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    provider: T,
    texture_atlas: TextureAtlas,
    texture_indices: Vec<usize>,
}

impl<T> SpriteAtlasWindowProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    pub fn new(
        provider: T,
        texture_atlas: TextureAtlas,
        texture_names: [String; 9],
    ) -> SpriteAtlasWindowProvider<T> {
        let map = texture_atlas.to_frame_map();
        let texture_indices = texture_names.iter().map(|name| map[name]).collect();
        SpriteAtlasWindowProvider {
            provider,
            texture_atlas,
            texture_indices,
        }
    }

    pub fn create_mesh(&self, size: Vec2) -> Mesh {
        let c0 = &self.texture_atlas.frames[self.texture_indices[0]];
        let c1 = &self.texture_atlas.frames[self.texture_indices[2]];
        let c2 = &self.texture_atlas.frames[self.texture_indices[6]];
        let c3 = &self.texture_atlas.frames[self.texture_indices[8]];
        debug_assert_eq!(c0.source.w, c2.source.w);
        debug_assert_eq!(c1.source.w, c3.source.w);
        debug_assert_eq!(c0.source.h, c1.source.h);
        debug_assert_eq!(c2.source.h, c3.source.h);

        let m = &self.texture_atlas.frames[self.texture_indices[4]];

        let e0 = &self.texture_atlas.frames[self.texture_indices[1]];
        let e1 = &self.texture_atlas.frames[self.texture_indices[3]];
        let e2 = &self.texture_atlas.frames[self.texture_indices[5]];
        let e3 = &self.texture_atlas.frames[self.texture_indices[7]];
        debug_assert_eq!(e0.source.w, e3.source.w);
        debug_assert_eq!(e0.source.w, m.source.w);
        debug_assert_eq!(e1.source.h, e2.source.h);
        debug_assert_eq!(e1.source.h, m.source.h);

        debug_assert!(c0.source.w + c1.source.w <= size.x as u32);
        debug_assert!(c0.source.h + c1.source.h <= size.y as u32);

        let m_w = size.x - (e0.source.w + e1.source.w) as f32;
        let m_h = size.y - (e0.source.h + e2.source.h) as f32;

        let mut c0_p =
            create_square_positions_from_frame(&vec2(0.0f32, c2.source.h as f32 + m_h), c0);
        let mut c1_p = create_square_positions_from_frame(
            &vec2(c0.source.w as f32 + m_w, c2.source.h as f32 + m_h),
            c1,
        );
        let mut c2_p = create_square_positions_from_frame(&make_vec2_zero(), c2);
        let mut c3_p =
            create_square_positions_from_frame(&vec2(c0.source.w as f32 + m_w, 0.0f32), c3);

        // let e0_p = create_square_positions_from_frame(e0);
        // let e1_p = create_square_positions_from_frame(e1);
        // let e2_p = create_square_positions_from_frame(e2);
        // let e3_p = create_square_positions_from_frame(e3);

        let mut indices = vec![];
        let mut positions = vec![];
        let mut colors = vec![];
        let mut textures = vec![];

        // Corners
        indices.append(&mut create_square_indices_with_offset(
            positions.len() as IndexType
        ));
        positions.append(&mut c0_p);
        colors.append(&mut create_square_colors(make_vec4_white()));
        textures.append(&mut create_square_texcoords_from_frame(
            self.texture_atlas.size.to_vec2(),
            c0,
        ));

        indices.append(&mut create_square_indices_with_offset(
            positions.len() as IndexType
        ));
        positions.append(&mut c1_p);
        colors.append(&mut create_square_colors(make_vec4_white()));
        textures.append(&mut create_square_texcoords_from_frame(
            self.texture_atlas.size.to_vec2(),
            c1,
        ));

        indices.append(&mut create_square_indices_with_offset(
            positions.len() as IndexType
        ));
        positions.append(&mut c2_p);
        colors.append(&mut create_square_colors(make_vec4_white()));
        textures.append(&mut create_square_texcoords_from_frame(
            self.texture_atlas.size.to_vec2(),
            c2,
        ));

        indices.append(&mut create_square_indices_with_offset(
            positions.len() as IndexType
        ));
        positions.append(&mut c3_p);
        colors.append(&mut create_square_colors(make_vec4_white()));
        textures.append(&mut create_square_texcoords_from_frame(
            self.texture_atlas.size.to_vec2(),
            c3,
        ));

        let m_w_i =
            ((size.x - (e0.source.w + e1.source.w) as f32) / m.source.w as f32).floor() as i32;
        let m_h_i =
            ((size.y - (e0.source.h + e2.source.h) as f32) / m.source.h as f32).floor() as i32;
        let m_w_f = m_w_i as f32 * m.source.w as f32;
        let m_h_f = m_h_i as f32 * m.source.h as f32;
        let ratio = vec2(
            (m_w - m_w_f) / m.source.w as f32,
            (m_h - m_h_f) / m.source.h as f32,
        );

        let c0_offset = vec2(c0.source.w as f32, c0.source.h as f32 + m_h);
        // let c1_offset = vec2(c0.source.w as f32 + m_w, c0.source.h as f32 + m_h);
        let c2_offset = vec2(c2.source.w as f32, c2.source.h as f32);
        let c3_offset = vec2(c2.source.w as f32 + m_w, c2.source.h as f32);

        // Middles
        for y in 0..m_h_i {
            for x in 0..m_w_i {
                let origin = vec2(
                    c2_offset.x + m.source.w as f32 * x as f32,
                    c2_offset.y + m.source.h as f32 * y as f32,
                );
                indices.append(&mut create_square_indices_with_offset(
                    positions.len() as IndexType
                ));
                positions.append(&mut create_square_positions_from_frame(&origin, m));
                colors.append(&mut create_square_colors(make_vec4_white()));
                textures.append(&mut create_square_texcoords_from_frame(
                    self.texture_atlas.size.to_vec2(),
                    m,
                ));
            }
        }
        for y in 0..m_h_i {
            let origin = vec2(
                c0.source.w as f32 + m_w_f,
                c0.source.h as f32 + m.source.h as f32 * y as f32,
            );
            let ratio = vec2(ratio.x, 1.0f32);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, m, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                m,
                &ratio,
            ));
        }
        for x in 0..m_w_i {
            let origin = vec2(
                c0.source.w as f32 + m.source.w as f32 * x as f32,
                c0.source.h as f32 + m_h_f,
            );
            let ratio = vec2(1.0f32, ratio.y);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, m, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                m,
                &ratio,
            ));
        }
        {
            let origin = vec2(c0.source.w as f32 + m_w_f, c0.source.h as f32 + m_h_f);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, m, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                m,
                &ratio,
            ));
        }

        // Right-Left
        for y in 0..m_h_i {
            let origin1 = vec2(0.0f32, c2_offset.y + m.source.h as f32 * y as f32);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame(&origin1, m));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame(
                self.texture_atlas.size.to_vec2(),
                e1,
            ));

            let origin2 = vec2(c3_offset.x, c3_offset.y + m.source.h as f32 * y as f32);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame(&origin2, m));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame(
                self.texture_atlas.size.to_vec2(),
                e2,
            ));
        }

        // Bottom-Top
        for x in 0..m_w_i {
            let origin1 = vec2(c2_offset.x + m.source.w as f32 * x as f32, 0.0f32);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame(&origin1, m));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame(
                self.texture_atlas.size.to_vec2(),
                e3,
            ));

            let origin2 = vec2(c0_offset.x + m.source.w as f32 * x as f32, c0_offset.y);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame(&origin2, m));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame(
                self.texture_atlas.size.to_vec2(),
                e0,
            ));
        }

        {
            let origin = vec2(0.0f32, c0.source.h as f32 + m_h_f);
            let ratio = vec2(1.0f32, ratio.y);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, e1, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                e1,
                &ratio,
            ));
        }
        {
            let origin = vec2(c0.source.w as f32 + m_w_f, c0.source.h as f32 + m_h);
            let ratio = vec2(ratio.x, 1.0f32);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, e0, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                e0,
                &ratio,
            ));
        }

        {
            let origin = vec2(c0.source.w as f32 + m_w, c0.source.h as f32 + m_h_f);
            let ratio = vec2(1.0f32, ratio.y);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, e2, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                e2,
                &ratio,
            ));
        }
        {
            let origin = vec2(c0.source.w as f32 + m_w_f, 0.0f32);
            let ratio = vec2(ratio.x, 1.0f32);
            indices.append(&mut create_square_indices_with_offset(
                positions.len() as IndexType
            ));
            positions.append(&mut create_square_positions_from_frame_with_ratio(
                &origin, e3, &ratio,
            ));
            colors.append(&mut create_square_colors(make_vec4_white()));
            textures.append(&mut create_square_texcoords_from_frame_with_ratio(
                self.texture_atlas.size.to_vec2(),
                e3,
                &ratio,
            ));
        }

        MeshBuilder::new()
            .indices(indices)
            .positions(positions)
            .colors(colors)
            .texcoords(textures)
            .normals(vec![])
            .build()
            .unwrap()
    }

    pub fn inner_provider(&self) -> &T {
        &self.provider
    }

    pub fn inner_provider_mut(&mut self) -> &mut T {
        &mut self.provider
    }
}

impl<T> PolygonProvider for SpriteAtlasWindowProvider<T>
where
    T: 'static + PolygonProvider + Polygon2DProviderInterface,
{
    fn transform(&self, core: &PolygonCore<Polygon>) -> Mat4 {
        self.provider.transform(core)
    }

    fn transform_for_child(&self, core: &PolygonCore<Polygon>) -> Mat4 {
        self.provider.transform_for_child(core)
    }
}

pub struct SpriteAtlasWindow {
    polygon: Shared<Polygon>,
    provider: Shared<SpriteAtlasWindowProvider<Polygon2DProvider>>,
}

impl SpriteAtlasWindow {
    pub fn new(texture_atlas: TextureAtlas, texture_names: [String; 9], size: Vec2) -> Self {
        let provider = make_shared(SpriteAtlasWindowProvider::new(
            Polygon2DProvider::new(size.clone_owned()),
            texture_atlas,
            texture_names,
        ));
        let cloned_provider = clone_shared(&provider);
        let mesh = provider.borrow().create_mesh(size);
        let polygon = make_shared(Polygon::new_with_provider(cloned_provider, mesh));
        SpriteAtlasWindow { polygon, provider }
    }

    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}

impl Polygon2DInterface for SpriteAtlasWindow {
    fn set_anchor_point(&mut self, anchor_point: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .inner_provider_mut()
            .set_anchor_point(&mut polygon.core, anchor_point);
    }

    fn set_size(&mut self, size: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .inner_provider_mut()
            .set_anchor_point(&mut polygon.core, size);
    }

    fn size(&self) -> Vec2 {
        let polygon = self.polygon.borrow();
        self.provider
            .borrow_mut()
            .inner_provider()
            .size(&polygon.core)
            .clone_owned()
    }
}
