use crate::core::graphic::batch::batch_buffer_pointer::BatchBufferPointer;
use crate::extension::shared::Shared;

pub struct BatchObjectBundle<TObject> {
    pub index: i32,
    pub object: Shared<TObject>,
    pub pointers: Vec<BatchBufferPointer>,
}

impl<TObject> BatchObjectBundle<TObject> {
    pub fn new(
        index: i32,
        object: Shared<TObject>,
        pointers: Vec<BatchBufferPointer>,
    ) -> BatchObjectBundle<TObject> {
        BatchObjectBundle {
            index,
            object,
            pointers,
        }
    }
}
