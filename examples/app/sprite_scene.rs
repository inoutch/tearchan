use crate::app::texture_bundle::generate_texture_bundle;
use nalgebra_glm::{vec3, Vec2, Vec3};
use std::collections::HashMap;
use tearchan::core::graphic::animation::animator::{AnimationData, AnimationGroup, Animator};
use tearchan::core::graphic::batch::batch2d::Batch2D;
use tearchan::core::graphic::camera_2d::Camera2D;
use tearchan::core::graphic::hal::backend::{GraphicPipeline, Texture};
use tearchan::core::graphic::hal::graphic_pipeline::GraphicPipelineConfig;
use tearchan::core::graphic::hal::renderer::ResizeContext;
use tearchan::core::graphic::hal::texture::TextureConfig;
use tearchan::core::graphic::polygon::sprite_atlas::SpriteAtlas;
use tearchan::core::graphic::polygon::PolygonCommon;
use tearchan::core::graphic::shader::standard_2d_shader_program::Standard2DShaderProgram;
use tearchan::core::scene::scene_base::SceneBase;
use tearchan::core::scene::scene_context::SceneContext;
use tearchan::core::scene::scene_creator::SceneCreator;
use tearchan::core::scene::touch::Touch;
use tearchan::math::vec::make_vec3_zero;
use winit::event::{KeyboardInput, VirtualKeyCode};

#[derive(Eq, PartialEq, Hash, Debug)]
enum AnimationState {
    Stand,
    Walk,
}

pub struct SpriteScene {
    camera: Camera2D,
    batch: Batch2D,
    shader_program: Standard2DShaderProgram,
    graphic_pipeline: GraphicPipeline,
    texture: Texture,
    sprite1: SpriteAtlas,
    animator1: Animator<AnimationState, &'static str>,
    sprite1_velocity: Vec3,
}

impl SpriteScene {
    pub fn creator() -> SceneCreator {
        |ctx, _| {
            let screen_size: Vec2 = ctx.graphics.display_size().logical.clone_owned();
            let camera = Camera2D::new(screen_size.clone_owned());

            let mut batch = Batch2D::new_batch2d(ctx.graphics);
            let (texture_atlas, image) = generate_texture_bundle();
            let texture = ctx
                .graphics
                .create_texture(&image, TextureConfig::default());
            let sprite1 = SpriteAtlas::new(texture_atlas.clone());
            sprite1.polygon().borrow_mut().set_position(vec3(
                screen_size.x / 2.0f32 - 100.0f32,
                screen_size.y / 2.0f32,
                0.0f32,
            ));
            let mut groups: HashMap<AnimationState, AnimationGroup<&'static str>> = HashMap::new();
            groups.insert(
                AnimationState::Stand,
                AnimationGroup {
                    frames: vec!["go_2.png"],
                    duration_sec: 1.0,
                },
            );
            groups.insert(
                AnimationState::Walk,
                AnimationGroup {
                    frames: vec![
                        "go_1.png", "go_2.png", "go_3.png", "go_4.png", "go_5.png", "go_6.png",
                        "go_7.png", "go_8.png",
                    ],
                    duration_sec: 0.08,
                },
            );
            let animator1 = Animator::new(AnimationData { groups }, AnimationState::Walk);

            let sprite2 = SpriteAtlas::new(texture_atlas);
            sprite2.polygon().borrow_mut().set_position(vec3(
                screen_size.x / 2.0f32 + 100.0f32,
                screen_size.y / 2.0f32,
                0.0f32,
            ));

            batch.add(sprite2.polygon(), 0);
            batch.add(sprite1.polygon(), 0);

            let shader_program = Standard2DShaderProgram::new(ctx.graphics, camera.base());
            let graphic_pipeline = ctx
                .graphics
                .create_graphic_pipeline(shader_program.shader(), GraphicPipelineConfig::default());
            Box::new(SpriteScene {
                camera,
                batch,
                shader_program,
                graphic_pipeline,
                texture,
                sprite1,
                animator1,
                sprite1_velocity: make_vec3_zero(),
            })
        }
    }
}

impl SceneBase for SpriteScene {
    fn update(&mut self, ctx: &mut SceneContext, delta: f32) {
        self.animator1.update(delta);
        self.animator1
            .set_state(if self.sprite1_velocity.x == 0.0f32 {
                AnimationState::Stand
            } else {
                AnimationState::Walk
            });
        let (animation, _) = self.animator1.animation();
        self.sprite1.set_atlas_with_key(animation);

        self.camera.update();
        self.batch.flush();

        self.shader_program.prepare(self.camera.combine());

        let descriptor_set = self.graphic_pipeline.descriptor_set();
        let write_descriptor_sets = self
            .shader_program
            .create_write_descriptor_sets(descriptor_set, &self.texture);

        ctx.graphics
            .write_descriptor_sets(write_descriptor_sets);
        ctx.graphics.draw_elements(
            &self.graphic_pipeline,
            self.batch.index_size(),
            self.batch.index_buffer(),
            &self.batch.vertex_buffers(),
        );

        let v = self.sprite1_velocity.to_owned() * delta * 60.0f32;
        let p = self.sprite1.polygon().borrow().position() + v;
        self.sprite1.polygon().borrow_mut().set_position(p);
    }

    fn on_touch_start(&mut self, _touch: &Touch) {}

    fn on_touch_end(&mut self, _touch: &Touch) {}

    fn on_touch_move(&mut self, _touch: &Touch) {}

    fn on_touch_cancel(&mut self, _touch: &Touch) {}

    fn on_key_down(&mut self, input: &KeyboardInput) {
        match input.virtual_keycode {
            None => {}
            Some(code) => match code {
                VirtualKeyCode::Right => {
                    self.sprite1_velocity = vec3(3.0f32, 0.0f32, 0.0f32);
                }
                VirtualKeyCode::Left => {
                    self.sprite1_velocity = vec3(-3.0f32, 0.0f32, 0.0f32);
                }
                _ => {}
            },
        }
    }

    fn on_key_up(&mut self, input: &KeyboardInput) {
        match input.virtual_keycode {
            None => {}
            Some(code) => match code {
                VirtualKeyCode::Right | VirtualKeyCode::Left => {
                    self.sprite1_velocity = make_vec3_zero();
                }
                _ => {}
            },
        }
    }

    fn on_resize(&mut self, _context: &mut ResizeContext) {}
}
