use crate::cube_object::{CubeObject, TransformObject};
use crate::ground_object::GroundObject;
use nalgebra_glm::vec3;
use ncollide3d::shape::{Cuboid, ShapeHandle};
use nphysics3d::algebra::Velocity3;
use nphysics3d::force_generator::DefaultForceGeneratorSet;
use nphysics3d::joint::DefaultJointConstraintSet;
use nphysics3d::object::{
    BodyPartHandle, BodyStatus, ColliderDesc, DefaultBodyHandle, DefaultBodySet,
    DefaultColliderSet, Ground, RigidBodyDesc,
};
use nphysics3d::world::{DefaultGeometricalWorld, DefaultMechanicalWorld};
use std::f32::consts::PI;
use std::rc::Rc;
use tearchan::plugin::object::camera::Camera3DDefaultObject;
use tearchan::plugin::renderer::standard_3d_renderer::Standard3DRenderer;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_core::game::object::{GameObject, GameObjectId};
use tearchan_core::scene::scene_context::SceneContext;
use tearchan_core::scene::scene_factory::SceneFactory;
use tearchan_core::scene::scene_result::SceneResult;
use tearchan_core::scene::Scene;
use tearchan_graphics::camera::camera_3d::Camera3D;
use tearchan_graphics::hal::backend::Texture;
use tearchan_graphics::hal::texture::TextureConfig;
use tearchan_graphics::image::Image;

const PRIMARY_CAMERA: &str = "primary";

pub struct Physics3DScene {
    mechanical_world: DefaultMechanicalWorld<f32>,
    geometrical_world: DefaultGeometricalWorld<f32>,
    bodies: DefaultBodySet<f32>,
    colliders: DefaultColliderSet<f32>,
    joint_constraints: DefaultJointConstraintSet<f32>,
    force_generators: DefaultForceGeneratorSet<f32>,
    cube_handle: DefaultBodyHandle,
    cube_id: GameObjectId,
}

impl Physics3DScene {
    pub fn factory() -> SceneFactory {
        |ctx, _| {
            let image = Image::new_empty();
            let texture = Texture::new(ctx.g.r.render_bundle(), &image, TextureConfig::default());
            let mut render_plugin =
                Standard3DRenderer::from_texture(&mut ctx.g.r, texture, PRIMARY_CAMERA.to_string());
            render_plugin.register_caster_for_render_object(|object| {
                let casted = object.downcast_rc::<CubeObject>().ok()?;
                Some(casted)
            });
            render_plugin.register_caster_for_render_object(|object| {
                let casted = object.downcast_rc::<GroundObject>().ok()?;
                Some(casted)
            });
            render_plugin.register_caster_for_camera_3d(|object| {
                let casted = object.downcast_rc::<Camera3DDefaultObject>().ok()?;
                Some(casted)
            });
            ctx.plugin_manager_mut()
                .add(Box::new(render_plugin), "renderer".to_string(), 0);

            let mechanical_world = DefaultMechanicalWorld::new(vec3(0.0, -9.81, 0.0));
            let geometrical_world = DefaultGeometricalWorld::new();

            let mut bodies = DefaultBodySet::new();
            let mut colliders = DefaultColliderSet::new();
            let joint_constraints = DefaultJointConstraintSet::new();
            let force_generators = DefaultForceGeneratorSet::new();

            // Ground
            let ground_handle = bodies.insert(Ground::new());
            let ground_shape = ShapeHandle::new(Cuboid::new(vec3(1.0f32, 0.1f32, 1.0f32)));
            let ground_collider =
                ColliderDesc::new(ground_shape).build(BodyPartHandle(ground_handle, 0));
            colliders.insert(ground_collider);
            ctx.add(GameObject::new(Rc::new(GroundObject::default())));

            // Cube
            let cube_rigid_body = RigidBodyDesc::new()
                .status(BodyStatus::Dynamic)
                .gravity_enabled(true)
                .rotation(vec3(
                    PI / 180.0f32 * 40.0f32,
                    0.0f32,
                    PI / 180.0f32 * 45.0f32,
                ))
                .translation(vec3(0.0f32, 2.0f32, 0.0f32))
                .velocity(Velocity3::linear(1.0f32, 0.0f32, 1.0f32))
                .build();
            let cube_handle = bodies.insert(cube_rigid_body);
            let cube_shape = ShapeHandle::new(Cuboid::new(vec3(0.1f32, 0.1f32, 0.1f32)));
            let cube_collider = ColliderDesc::new(cube_shape)
                .density(1.0f32)
                .build(BodyPartHandle(cube_handle, 0));
            colliders.insert(cube_collider);
            let cube: GameObject<dyn GameObjectBase> =
                GameObject::new(Rc::new(CubeObject::default()));
            let cube_id = cube.id();
            ctx.add(cube);

            // Camera
            let aspect = ctx.g.r.render_bundle().display_size().logical.x
                / ctx.g.r.render_bundle().display_size().logical.y;
            let mut camera = Camera3D::default_with_aspect(aspect);
            camera.position = vec3(0.0f32, 2.0f32, 4.0f32);
            camera.target_position = vec3(0.0f32, 0.0f32, 0.0f32);
            camera.up = vec3(0.0f32, 1.0f32, 0.0f32);
            camera.update();
            let camera_object = Camera3DDefaultObject::new(camera, PRIMARY_CAMERA.to_string());
            ctx.add(GameObject::new(Rc::new(camera_object)));

            Box::new(Physics3DScene {
                mechanical_world,
                geometrical_world,
                bodies,
                colliders,
                joint_constraints,
                force_generators,
                cube_handle,
                cube_id,
            })
        }
    }
}

impl Scene for Physics3DScene {
    fn update(&mut self, context: &mut SceneContext) -> SceneResult {
        self.mechanical_world.set_timestep(context.g.delta);
        self.mechanical_world.step(
            &mut self.geometrical_world,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joint_constraints,
            &mut self.force_generators,
        );

        let cube_collider = self.colliders.get(self.cube_handle).unwrap();
        let mut cube_object = context
            .find_by_id(self.cube_id)
            .unwrap()
            .cast::<CubeObject>()
            .unwrap();
        cube_object
            .borrow_mut()
            .set_transform(nalgebra_glm::convert(*cube_collider.position()));

        SceneResult::None
    }
}
