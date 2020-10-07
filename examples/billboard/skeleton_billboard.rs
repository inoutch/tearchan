use intertrait::cast_to;
use nalgebra_glm::{vec2, vec3};
use serde::export::Option::Some;
use tearchan::plugin::renderer::billboard_renderer::billboard_command_queue::BillboardCommandQueue;
use tearchan::plugin::renderer::billboard_renderer::billboard_render_object::BillboardRenderObject;
use tearchan::plugin::renderer::sprite_renderer::sprite::Sprite;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::EMPTY_ID;
use tearchan_graphics::batch::batch_command::BatchObjectId;
use tearchan_utility::texture::TextureAtlas;

pub struct SkeletonBillboard {
    sprite: Sprite,
    billboard_id: BatchObjectId,
    billboard_queue: Option<BillboardCommandQueue>,
}

impl Default for SkeletonBillboard {
    fn default() -> Self {
        let texture_atlas: TextureAtlas =
            serde_json::from_str(include_str!("../data/sprites/skeleton.json")).unwrap();

        SkeletonBillboard {
            sprite: Sprite::new(texture_atlas),
            billboard_id: EMPTY_ID,
            billboard_queue: None,
        }
    }
}

#[cast_to]
impl GameObjectBase for SkeletonBillboard {}

#[cast_to]
impl BillboardRenderObject for SkeletonBillboard {
    fn attach_queue(&mut self, mut queue: BillboardCommandQueue) {
        self.sprite.set_scale(vec3(0.005f32, 0.005f32, 0.005f32));
        self.sprite.set_anchor_position(vec2(0.5f32, 0.5f32));

        self.billboard_id = queue.create_billboard_with_sprite(&self.sprite);
        self.billboard_queue = Some(queue);

        self.sprite.reset_changes();
    }

    fn detach(&mut self) {
        if let Some(queue) = &mut self.billboard_queue {
            queue.destroy_billboard(self.billboard_id);
        }
    }
}
