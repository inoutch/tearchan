use intertrait::cast_to;
use tearchan::renderer::standard_font_renderer::standard_font_command_queue::StandardFontCommandQueue;
use tearchan::renderer::standard_font_renderer::standard_font_render_object::StandardFontRenderObject;
use tearchan_core::game::object::game_object_base::GameObjectBase;
use tearchan_graphics::batch::batch_command::BatchObjectId;

pub struct TextObject {
    text: String,
    batch_object_id: Option<BatchObjectId>,
    font_queue: Option<StandardFontCommandQueue>,
}

#[cast_to]
impl GameObjectBase for TextObject {}

impl TextObject {
    pub fn new(text: String) -> Self {
        TextObject {
            text,
            batch_object_id: None,
            font_queue: None,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text.clone();
        if let Some(queue) = &mut self.font_queue {
            queue.update_text(self.batch_object_id.unwrap(), text);
        }
    }
}

#[cast_to]
impl StandardFontRenderObject for TextObject {
    fn attach_queue(&mut self, mut queue: StandardFontCommandQueue) {
        self.batch_object_id = Some(queue.create_text(self.text.to_string(), None));
        self.font_queue = Some(queue);
    }
}
