use crate::core::graphic::hal::backend::FixedApi;

pub struct SceneContext<'a> {
    renderer_api: &'a FixedApi<'a>,
}

impl<'a> SceneContext<'a> {
    pub fn new(renderer_api: &'a FixedApi<'a>) -> SceneContext<'a> {
        SceneContext { renderer_api }
    }

    pub fn borrow_renderer_api(&self) -> &FixedApi {
        self.renderer_api
    }
}
