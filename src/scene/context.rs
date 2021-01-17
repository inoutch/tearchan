use std::ops::{Deref, DerefMut};
use tearchan_gfx::context::{GfxContext, GfxRenderContext};

pub struct SceneContext<'a> {
    gfx: GfxContext<'a>,
}

impl<'a> SceneContext<'a> {
    pub fn new(gfx: GfxContext<'a>) -> Self {
        SceneContext { gfx }
    }

    pub fn gfx(&self) -> &GfxContext {
        &self.gfx
    }
}

pub struct SceneRenderContext<'a> {
    scene_context: SceneContext<'a>,
    rendering_context: GfxRenderContext,
}

impl<'a> SceneRenderContext<'a> {
    pub fn new(gfx: (GfxContext<'a>, GfxRenderContext)) -> SceneRenderContext<'a> {
        SceneRenderContext {
            scene_context: SceneContext::new(gfx.0),
            rendering_context: gfx.1,
        }
    }

    pub fn gfx_rendering(&self) -> &GfxRenderContext {
        &self.rendering_context
    }
}

impl<'a> Deref for SceneRenderContext<'a> {
    type Target = SceneContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.scene_context
    }
}

impl<'a> DerefMut for SceneRenderContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scene_context
    }
}
