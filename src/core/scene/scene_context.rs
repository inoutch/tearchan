use crate::core::graphic::hal::backend::FixedApi;

pub struct SceneContext<'a, 'b> {
    pub renderer_api: &'a mut FixedApi<'b>,
}

impl<'a, 'b> SceneContext<'a, 'b> {
    pub fn new(renderer_api: &'a mut FixedApi<'b>) -> SceneContext<'a, 'b> {
        SceneContext { renderer_api }
    }
}
