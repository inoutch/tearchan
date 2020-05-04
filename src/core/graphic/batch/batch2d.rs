use crate::core::graphic::batch::batch_base::BatchBase;
use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_buffer_f32::BatchBufferF32;
use crate::core::graphic::batch::batch_bundle::BatchBundle;
use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::core::graphic::batch::default::Batch;
use crate::core::graphic::hal::backend::FixedApi;
use crate::core::graphic::hal::vertex_buffer::VertexBuffer;
use crate::core::graphic::polygon::default::Polygon;
use crate::core::graphic::polygon::polygon_base_buffer::PolygonBaseBuffer;
use crate::extension::shared::Shared;
use crate::utility::buffer_interface::BufferInterface;
use std::rc::Rc;

pub struct Batch2D<TBatchBuffer: BatchBuffer> {
    bundles: Vec<BatchBundle<TBatchBuffer>>,
}

impl<TObject: PolygonBaseBuffer<TBatchBuffer>, TBatchBuffer> BatchBase<TObject, TBatchBuffer>
    for Batch2D<TBatchBuffer>
where
    TBatchBuffer: BatchBuffer + BufferInterface<f32>,
{
    fn update(&mut self, object_bundle: &mut Rc<BatchObjectBundle<TObject>>) {
        debug_assert_eq!(self.bundles.len(), 3, "Invalid bundles length");
        debug_assert_eq!(
            object_bundle.pointers.len(),
            3,
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
}

impl<TObject: PolygonBaseBuffer<TBatchBuffer>, TBatchBuffer>
    Batch<TObject, TBatchBuffer, Batch2D<TBatchBuffer>>
where
    TBatchBuffer: BatchBuffer + BufferInterface<f32>,
{
    pub fn new_batch2d(
        position_buffer: TBatchBuffer,
        color_buffer: TBatchBuffer,
        texcoord_buffer: TBatchBuffer,
    ) -> Batch<TObject, TBatchBuffer, Batch2D<TBatchBuffer>> {
        Batch::new(Batch2D {
            bundles: vec![
                BatchBundle {
                    stride: 3,
                    batch_buffer: position_buffer,
                },
                BatchBundle {
                    stride: 3,
                    batch_buffer: color_buffer,
                },
                BatchBundle {
                    stride: 3,
                    batch_buffer: texcoord_buffer,
                },
            ],
        })
    }
}

impl Batch2D<BatchBufferF32> {
    pub fn new(api: &FixedApi) -> Batch<Polygon, BatchBufferF32, Batch2D<BatchBufferF32>> {
        Batch::new_batch2d(
            BatchBufferF32::new(api),
            BatchBufferF32::new(api),
            BatchBufferF32::new(api),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::core::graphic::batch::batch2d::Batch2D;
    use crate::core::graphic::batch::batch_buffer::tests::MockBatchBuffer;
    use crate::core::graphic::batch::default::Batch;
    use crate::core::graphic::polygon::default::Polygon;
    use crate::extension::shared::Shared;
    use crate::math::mesh::MeshBuilder;
    use crate::utility::test::func::MockFunc;
    use nalgebra_glm::vec2;

    impl Batch2D<MockBatchBuffer> {
        pub fn new_batch2d_with_mock(
            mock_func: &Shared<MockFunc>,
        ) -> Batch<Polygon, MockBatchBuffer, Batch2D<MockBatchBuffer>> {
            Batch::new_batch2d(
                MockBatchBuffer::new(mock_func),
                MockBatchBuffer::new(mock_func),
                MockBatchBuffer::new(mock_func),
            )
        }
    }

    #[test]
    fn test_batch2d() {
        let mock_func = Shared::new(MockFunc::new());
        let mesh = MeshBuilder::new()
            .with_square(vec2(32.0f32, 32.0f32))
            .build()
            .unwrap();

        let polygon = Shared::new(Polygon::new(mesh));
        let mut batch2d = Batch2D::new_batch2d_with_mock(&mock_func);
        batch2d.add(&polygon, 0);
        batch2d.render();

        mock_func.print_logs();
    }
}
