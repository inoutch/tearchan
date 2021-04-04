use crate::engine::Spawner;
use std::any::Any;
use std::ops::{Deref, DerefMut};
use tearchan_gfx::context::{GfxContext, GfxRenderContext};
use tearchan_util::any::OptAnyBox;

pub struct SceneContext<'a, 'b, 'c, 'd> {
    gfx: GfxContext<'a>,
    spawner: &'b Spawner<'c>,
    custom: &'d mut OptAnyBox,
}

impl<'a, 'b, 'c, 'd> SceneContext<'a, 'b, 'c, 'd> {
    pub fn new(gfx: GfxContext<'a>, spawner: &'b Spawner<'c>, custom: &'d mut OptAnyBox) -> Self {
        SceneContext {
            gfx,
            spawner,
            custom,
        }
    }

    pub fn gfx(&self) -> &GfxContext {
        &self.gfx
    }

    pub fn spawner(&self) -> &'b Spawner<'c> {
        &self.spawner
    }

    pub fn custom<T: Any>(&self) -> &T {
        self.custom.get().unwrap()
    }

    pub fn custom_mut<T: Any>(&mut self) -> &mut T {
        self.custom.get_mut().unwrap()
    }
}

pub struct SceneRenderContext<'a, 'b, 'c, 'd> {
    scene_context: SceneContext<'a, 'b, 'c, 'd>,
    rendering_context: GfxRenderContext,
    pub delta: f32, // seconds each frame
}

impl<'a, 'b, 'c, 'd> SceneRenderContext<'a, 'b, 'c, 'd> {
    pub fn new(
        gfx: (GfxContext<'a>, GfxRenderContext),
        spawner: &'b Spawner<'c>,
        custom: &'d mut OptAnyBox,
        delta: f32,
    ) -> SceneRenderContext<'a, 'b, 'c, 'd> {
        SceneRenderContext {
            scene_context: SceneContext::new(gfx.0, spawner, custom),
            rendering_context: gfx.1,
            delta,
        }
    }

    pub fn gfx_rendering(&self) -> &GfxRenderContext {
        &self.rendering_context
    }
}

impl<'a, 'b, 'c, 'd> Deref for SceneRenderContext<'a, 'b, 'c, 'd> {
    type Target = SceneContext<'a, 'b, 'c, 'd>;

    fn deref(&self) -> &Self::Target {
        &self.scene_context
    }
}

impl<'a, 'b, 'c, 'd> DerefMut for SceneRenderContext<'a, 'b, 'c, 'd> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene_context
    }
}
