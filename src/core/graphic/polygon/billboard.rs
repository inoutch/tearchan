use crate::core::graphic::batch::batch_change_manager::BatchChangeNotifier;
use crate::core::graphic::hal::buffer_interface::BufferMappedMemoryInterface;
use crate::core::graphic::polygon::polygon_2d::{
    Polygon2DInterface, Polygon2DProvider, Polygon2DProviderInterface,
};
use crate::core::graphic::polygon::sprite_atlas::{SpriteAtlasInterface, SpriteAtlasProvider};
use crate::core::graphic::polygon::{Polygon, PolygonCommon, PolygonCore, PolygonProvider};
use crate::core::graphic::texture::TextureAtlas;
use crate::extension::shared::{clone_shared, make_shared, Shared};
use crate::math::change_range::ChangeRange;
use crate::math::mesh::MeshBuilder;
use crate::utility::change_notifier::{ChangeNotifier, ChangeNotifierObject};
use nalgebra_glm::{rotate, scale, translate, vec2, Mat4, Vec2, Vec3};

pub trait BillboardInterface {
    fn copy_origins_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    );
}

pub struct BillboardProvider {
    provider: SpriteAtlasProvider<Polygon2DProvider>,
    origin_change_range: ChangeRange,
    notifier: Option<BatchChangeNotifier<Billboard>>,
}

impl PolygonProvider for BillboardProvider {
    fn set_position(&mut self, core: &mut PolygonCore, position: Vec3) -> bool {
        if core.position != position {
            core.position = position;
            self.origin_change_range.update_all();
            self.request_change(core);
            true
        } else {
            false
        }
    }

    fn transform(&self, core: &PolygonCore) -> Mat4 {
        translate(
            &self.billboard_transform(core),
            &self.provider.inner_provider().transform_anchor_point(),
        )
    }

    fn transform_for_child(&self, core: &PolygonCore) -> Mat4 {
        translate(
            &Mat4::identity(),
            &self
                .provider
                .inner_provider()
                .transform_anchor_point_for_child(),
        ) * self.transform(core)
    }

    fn request_change(&mut self, _core: &mut PolygonCore) {
        if let Some(notifier) = &mut self.notifier {
            notifier.request_change();
        }
    }
}

impl BillboardProvider {
    fn billboard_transform(&self, core: &PolygonCore) -> Mat4 {
        let current = scale(
            &rotate(
                &Mat4::identity(),
                self.rotation_radian(core),
                self.rotation_axis(core),
            ),
            self.scale(core),
        );
        if let Some(x) = core.parent() {
            return x.borrow().transform_for_child() * current;
        }
        current
    }

    fn copy_origins_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        core: &mut PolygonCore,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    ) {
        println!("copy_origins_into");
        let position = core.position();
        let range = match force {
            true => self.origin_change_range.get_range_or_full(),
            false => match self.origin_change_range.get_range() {
                Some(range) => range,
                None => return,
            },
        };
        for i in range {
            buffer.set(position.x, offset + i * 3);
            buffer.set(position.y, offset + i * 3 + 1);
            buffer.set(position.z, offset + i * 3 + 2);
        }

        self.origin_change_range.reset();
    }
}

pub struct Billboard {
    polygon: Shared<Polygon>,
    provider: Shared<BillboardProvider>,
}

impl Billboard {
    pub fn new(texture_atlas: TextureAtlas) -> Self {
        let rect_size = 4;
        let frame = texture_atlas
            .frames
            .first()
            .expect("There must be at least one or more frames");
        let mesh = MeshBuilder::new()
            .with_frame(texture_atlas.size.to_vec2(), frame)
            .build()
            .unwrap();

        let polygon_2d_provider =
            Polygon2DProvider::new(vec2(frame.source.w as f32, frame.source.h as f32));
        let sprite_atlas_provider = SpriteAtlasProvider::new(polygon_2d_provider, texture_atlas);
        let provider = make_shared(BillboardProvider {
            provider: sprite_atlas_provider,
            origin_change_range: ChangeRange::new(rect_size),
            notifier: None,
        });
        let cloned_provider = clone_shared(&provider);

        Billboard {
            polygon: make_shared(Polygon::new_with_provider(provider, mesh)),
            provider: cloned_provider,
        }
    }

    pub fn copy_origins_into<TBuffer: BufferMappedMemoryInterface<f32>>(
        &mut self,
        buffer: &mut TBuffer,
        offset: usize,
        force: bool,
    ) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .copy_origins_into(&mut polygon.core, buffer, offset, force);
    }

    #[inline]
    pub fn polygon(&self) -> &Shared<Polygon> {
        &self.polygon
    }
}

impl Polygon2DInterface for Billboard {
    fn set_anchor_point(&mut self, anchor_point: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .provider
            .inner_provider_mut()
            .set_anchor_point(&mut polygon.core, anchor_point);
    }

    fn set_size(&mut self, size: Vec2) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .provider
            .inner_provider_mut()
            .set_size(&mut polygon.core, size);
    }

    fn size(&self) -> Vec2 {
        let polygon = self.polygon.borrow();
        self.provider
            .borrow()
            .provider
            .inner_provider()
            .size(&polygon.core)
            .clone_owned()
    }
}

impl SpriteAtlasInterface for Billboard {
    fn set_atlas(&mut self, index: usize) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .provider
            .set_atlas(&mut polygon.core, index);
    }

    fn set_atlas_with_key(&mut self, key: &str) {
        let mut polygon = self.polygon.borrow_mut();
        self.provider
            .borrow_mut()
            .provider
            .set_atlas_with_key(&mut polygon.core, key);
    }
}

impl ChangeNotifierObject<BatchChangeNotifier<Billboard>> for Billboard {
    fn set_change_notifier(&mut self, notifier: BatchChangeNotifier<Billboard>) {
        let mut polygon = self.polygon.borrow_mut();
        let mut provider = self.provider.borrow_mut();
        provider.notifier = Some(notifier);
        provider.request_change(&mut polygon.core);
    }
}
