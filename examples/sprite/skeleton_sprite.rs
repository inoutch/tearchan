use nalgebra_glm::vec4;
use serde::export::Option::Some;
use std::collections::HashMap;
use tearchan::plugin::animation::animation_object::AnimationObject;
use tearchan::plugin::animation::animator::{AnimationData, AnimationGroup, Animator};
use tearchan::plugin::renderer::sprite_renderer::sprite::Sprite;
use tearchan::plugin::renderer::sprite_renderer::sprite_command_queue::SpriteCommandQueue;
use tearchan::plugin::renderer::sprite_renderer::sprite_render_object::SpriteRenderObject;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_graphics::batch::batch_command::BatchObjectId;
use tearchan_utility::texture::TextureAtlas;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum SkeletonState {
    Stand,
    Walk,
}

pub struct SkeletonSprite {
    sprite: Sprite,
    sprite_id: Option<BatchObjectId>,
    sprite_queue: Option<SpriteCommandQueue>,
    animator: Animator<SkeletonState, &'static str>,
    prev_animation: u64,
}

impl Default for SkeletonSprite {
    fn default() -> Self {
        let texture_atlas: TextureAtlas =
            serde_json::from_str(include_str!("../data/sprites/skeleton.json")).unwrap();
        let mut groups: HashMap<SkeletonState, AnimationGroup<&'static str>> = HashMap::new();
        groups.insert(
            SkeletonState::Stand,
            AnimationGroup {
                frames: vec!["go_2.png"],
                duration_sec: 1.0,
            },
        );
        groups.insert(
            SkeletonState::Walk,
            AnimationGroup {
                frames: vec![
                    "go_1.png", "go_2.png", "go_3.png", "go_4.png", "go_5.png", "go_6.png",
                    "go_7.png", "go_8.png",
                ],
                duration_sec: 0.08,
            },
        );

        let mut sprite = Sprite::new(texture_atlas);
        sprite.set_color(vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32));
        SkeletonSprite {
            sprite,
            sprite_id: None,
            sprite_queue: None,
            animator: Animator::new(AnimationData { groups }, SkeletonState::Walk),
            prev_animation: std::u64::MAX,
        }
    }
}

impl GameObjectBase for SkeletonSprite {}

impl SpriteRenderObject for SkeletonSprite {
    fn attach_sprite_queue(&mut self, mut queue: SpriteCommandQueue) {
        self.sprite_id = Some(queue.create_sprite(&self.sprite, None));
        self.sprite_queue = Some(queue);
    }
}

impl AnimationObject for SkeletonSprite {
    fn update(&mut self, delta: f32) {
        self.animator.update(delta);
        if let Some(queue) = &mut self.sprite_queue {
            let (key, next) = self.animator.animation();
            if self.prev_animation != next {
                self.prev_animation = next;
                self.sprite.set_atlas(key.to_string());
                queue.update_sprite(self.sprite_id.unwrap(), &self.sprite);
            }
        }
    }
}
