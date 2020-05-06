use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_bundle::BatchBundle;
use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::extension::shared::Shared;
use std::rc::Rc;

pub trait BatchBase<TObject, TBatchBuffer: BatchBuffer> {
    fn update(&mut self, object_bundle: &mut Rc<BatchObjectBundle<TObject>>);
    fn size(&self, object: &Shared<TObject>) -> usize;
    fn bundles_mut(&mut self) -> &mut Vec<BatchBundle<TBatchBuffer>>;
    fn bundles(&self) -> &Vec<BatchBundle<TBatchBuffer>>;
    fn triangle_count(&self) -> usize;
}

#[cfg(test)]
pub mod tests {
    use crate::core::graphic::batch::batch_base::BatchBase;
    use crate::core::graphic::batch::batch_buffer::tests::MockBatchBuffer;
    use crate::core::graphic::batch::batch_bundle::BatchBundle;
    use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
    use crate::extension::shared::Shared;
    use crate::utility::test::func::MockFunc;
    use std::rc::Rc;

    pub struct MockBatchBase {
        mock_func: Shared<MockFunc>,
        bundles: Vec<BatchBundle<MockBatchBuffer>>,
    }

    impl MockBatchBase {
        pub fn new(mock_func: &Shared<MockFunc>) -> MockBatchBase {
            MockBatchBase {
                mock_func: Shared::clone(mock_func),
                bundles: vec![BatchBundle {
                    stride: 0,
                    batch_buffer: MockBatchBuffer::new(mock_func),
                }],
            }
        }
    }

    impl<TObject> BatchBase<TObject, MockBatchBuffer> for MockBatchBase {
        fn update(&mut self, object_bundle: &mut Rc<BatchObjectBundle<TObject>>) {
            self.mock_func.borrow_mut().call(vec![
                "MockBatchBase.update".to_string(),
                object_bundle.index.to_string(),
            ]);
        }

        fn size(&self, object: &Shared<TObject>) -> usize {
            0
        }

        fn bundles_mut(&mut self) -> &mut Vec<BatchBundle<MockBatchBuffer>> {
            &mut self.bundles
        }

        fn bundles(&self) -> &Vec<BatchBundle<MockBatchBuffer>> {
            &self.bundles
        }

        fn triangle_count(&self) -> usize {
            0
        }
    }
}
