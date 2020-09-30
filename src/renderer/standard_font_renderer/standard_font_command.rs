use tearchan_graphics::batch::batch_command::BatchObjectId;

#[derive(Debug)]
pub enum StandardFontCommand {
    SetText { id: BatchObjectId, text: String },
}
