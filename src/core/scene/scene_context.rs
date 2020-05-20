use crate::core::graphic::hal::backend::RendererApi;

pub struct SceneContext<'a, 'b> {
    pub renderer_api: &'a mut RendererApi<'b>,
}

impl<'a, 'b> SceneContext<'a, 'b> {
    pub fn new(renderer_api: &'a mut RendererApi<'b>) -> SceneContext<'a, 'b> {
        SceneContext { renderer_api }
    }
}
