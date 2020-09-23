use tearchan_graphics::hal::backend::RendererContext;

pub struct GameContext<'a> {
    pub delta: f32,
    pub r: RendererContext<'a>,
}

impl<'a> GameContext<'a> {
    pub fn new(delta: f32, renderer_context: RendererContext<'a>) -> GameContext {
        GameContext {
            delta,
            r: renderer_context,
        }
    }
}
