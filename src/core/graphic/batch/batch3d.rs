use crate::core::graphic::batch::batch_base::BatchBase;
use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use crate::core::graphic::batch::batch_bundle::BatchBundle;
use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::core::graphic::batch::default::Batch;
use crate::core::graphic::hal::backend::FixedApi;
use crate::core::graphic::polygon::default::Polygon;
use crate::core::graphic::polygon::polygon_base_buffer::PolygonBaseBuffer;
use crate::extension::shared::Shared;
use crate::utility::buffer_interface::BufferInterface;
use std::rc::Rc;

pub struct Batch3D<TBatchBuffer: BatchBuffer> {
    bundles: Vec<BatchBundle<TBatchBuffer>>,
}

impl<TObject: PolygonBaseBuffer<TBatchBuffer>, TBatchBuffer> BatchBase<TObject, TBatchBuffer>
    for Batch3D<TBatchBuffer>
where
    TBatchBuffer: BatchBuffer + BufferInterface<f32>,
{
    fn update(&mut self, object_bundle: &mut Rc<BatchObjectBundle<TObject>>) {
        debug_assert_eq!(self.bundles.len(), 4, "Invalid bundles length");
        debug_assert_eq!(
            object_bundle.pointers.len(),
            4,
            "Invalid object pointers length"
        );

        let mut object = object_bundle.object.borrow_mut();
        object.copy_positions_into(
            &mut self.bundles[0].batch_buffer,
            object_bundle.pointers[0].start,
        );
        object.copy_colors_into(
            &mut self.bundles[1].batch_buffer,
            object_bundle.pointers[1].start,
        );
        object.copy_texcoords_into(
            &mut self.bundles[2].batch_buffer,
            object_bundle.pointers[2].start,
        );
        /*object.copy_normals_into(
            &mut self.bundles[3].batch_buffer,
            object_bundle.pointers[3].start,
        );*/
    }

    fn size(&self, object: &Shared<TObject>) -> usize {
        object.borrow_mut().mesh().size()
    }

    fn bundles_mut(&mut self) -> &mut Vec<BatchBundle<TBatchBuffer>> {
        &mut self.bundles
    }

    fn bundles(&self) -> &Vec<BatchBundle<TBatchBuffer>> {
        &self.bundles
    }

    fn triangle_count(&self) -> usize {
        let bundle = &self.bundles[0];
        bundle.batch_buffer.size() / bundle.stride as usize
    }
}

impl<TObject: PolygonBaseBuffer<TBatchBuffer>, TBatchBuffer>
    Batch<TObject, TBatchBuffer, Batch3D<TBatchBuffer>>
where
    TBatchBuffer: BatchBuffer + BufferInterface<f32>,
{
    pub fn new_batch3d(
        position_buffer: TBatchBuffer,
        color_buffer: TBatchBuffer,
        texcoord_buffer: TBatchBuffer,
        normal_buffer: TBatchBuffer,
    ) -> Batch<TObject, TBatchBuffer, Batch3D<TBatchBuffer>> {
        Batch::new(Batch3D {
            bundles: vec![
                BatchBundle {
                    stride: 3,
                    batch_buffer: position_buffer,
                },
                BatchBundle {
                    stride: 4,
                    batch_buffer: color_buffer,
                },
                BatchBundle {
                    stride: 2,
                    batch_buffer: texcoord_buffer,
                },
                BatchBundle {
                    stride: 3,
                    batch_buffer: normal_buffer,
                },
            ],
        })
    }
}

impl Batch3D<BatchBufferF32> {
    pub fn new(api: &FixedApi) -> Batch<Polygon, BatchBufferF32, Batch3D<BatchBufferF32>> {
        Batch::new_batch3d(
            BatchBufferF32::new(api),
            BatchBufferF32::new(api),
            BatchBufferF32::new(api),
            BatchBufferF32::new(api),
        )
    }
}
