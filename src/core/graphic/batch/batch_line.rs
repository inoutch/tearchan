use crate::core::graphic::batch::batch_base::BatchBase;
use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use crate::core::graphic::batch::batch_bundle::BatchBundle;
use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::core::graphic::batch::Batch;
use crate::core::graphic::hal::backend::RendererApi;
use crate::core::graphic::polygon::Polygon;
use crate::extension::shared::Shared;
use crate::utility::buffer_interface::BufferInterface;
use std::rc::Rc;

pub struct BatchLine<TBatchBuffer: BatchBuffer> {
    bundles: Vec<BatchBundle<TBatchBuffer>>,
}

impl<TBatchBuffer> BatchBase<Polygon, TBatchBuffer> for BatchLine<TBatchBuffer>
where
    TBatchBuffer: BatchBuffer + BufferInterface<f32>,
{
    fn update(&mut self, object_bundle: &mut Rc<BatchObjectBundle<Polygon>>) {
        debug_assert_eq!(self.bundles.len(), 2, "Invalid bundles length");
        debug_assert_eq!(
            object_bundle.pointers.len(),
            2,
            "Invalid object pointers length"
        );

        let mut object = object_bundle.object.borrow_mut();
        object.copy_positions_into(
            &mut self.bundles[0].batch_buffer,
            object_bundle.pointers[0].borrow().start,
        );
        object.copy_colors_into(
            &mut self.bundles[1].batch_buffer,
            object_bundle.pointers[1].borrow().start,
        );
    }

    fn size(&self, object: &Shared<Polygon>) -> usize {
        object.borrow().mesh_size()
    }

    fn bundles_mut(&mut self) -> &mut Vec<BatchBundle<TBatchBuffer>> {
        &mut self.bundles
    }

    fn bundles(&self) -> &Vec<BatchBundle<TBatchBuffer>> {
        &self.bundles
    }
}

impl<TBatchBuffer> Batch<Polygon, TBatchBuffer, BatchLine<TBatchBuffer>>
where
    TBatchBuffer: BatchBuffer + BufferInterface<f32>,
{
    pub fn new_batch_line(
        position_buffer: TBatchBuffer,
        color_buffer: TBatchBuffer,
    ) -> Batch<Polygon, TBatchBuffer, BatchLine<TBatchBuffer>> {
        Batch::new(BatchLine {
            bundles: vec![
                BatchBundle {
                    stride: 3,
                    batch_buffer: position_buffer,
                },
                BatchBundle {
                    stride: 4,
                    batch_buffer: color_buffer,
                },
            ],
        })
    }
}

impl BatchLine<BatchBufferF32> {
    pub fn new(api: &RendererApi) -> Batch<Polygon, BatchBufferF32, BatchLine<BatchBufferF32>> {
        Batch::new_batch_line(BatchBufferF32::new(api), BatchBufferF32::new(api))
    }
}
