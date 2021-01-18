use crate::engine::Spawner;
use std::ops::{Deref, DerefMut};
use tearchan_gfx::context::{GfxContext, GfxRenderContext};

pub struct SceneContext<'a, 'b, 'c> {
    gfx: GfxContext<'a>,
    spawner: &'b Spawner<'c>,
}

impl<'a, 'b, 'c> SceneContext<'a, 'b, 'c> {
    pub fn new(gfx: GfxContext<'a>, spawner: &'b Spawner<'c>) -> Self {
        SceneContext { gfx, spawner }
    }

    pub fn gfx(&self) -> &GfxContext {
        &self.gfx
    }
}

pub struct SceneRenderContext<'a, 'b, 'c> {
    scene_context: SceneContext<'a, 'b, 'c>,
    rendering_context: GfxRenderContext,
}

impl<'a, 'b, 'c> SceneRenderContext<'a, 'b, 'c> {
    pub fn new(
        gfx: (GfxContext<'a>, GfxRenderContext),
        spawner: &'b Spawner<'c>,
    ) -> SceneRenderContext<'a, 'b, 'c> {
        SceneRenderContext {
            scene_context: SceneContext::new(gfx.0, spawner),
            rendering_context: gfx.1,
        }
    }

    pub fn gfx_rendering(&self) -> &GfxRenderContext {
        &self.rendering_context
    }
}

impl<'a, 'b, 'c> Deref for SceneRenderContext<'a, 'b, 'c> {
    type Target = SceneContext<'a, 'b, 'c>;

    fn deref(&self) -> &Self::Target {
        &self.scene_context
    }
}

impl<'a, 'b, 'c> DerefMut for SceneRenderContext<'a, 'b, 'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene_context
    }
}
