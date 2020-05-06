use crate::core::graphic::batch::batch_base::BatchBase;
use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::core::graphic::hal::backend::{FixedApi, FixedGraphicPipeline};
use crate::core::graphic::hal::graphic_pipeline::GraphicPipeline;
use crate::extension::collection::VecExt;
use crate::extension::shared::Shared;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

pub struct Batch<TObject, TBatchBuffer: BatchBuffer, TBatchBase: BatchBase<TObject, TBatchBuffer>> {
    base: TBatchBase,
    material: i32,
    // TODO: Change to actual material
    object_bundles: Vec<Rc<BatchObjectBundle<TObject>>>,
    object_bundles_cache: HashMap<*const TObject, Rc<BatchObjectBundle<TObject>>>,
    _marker: PhantomData<fn() -> TBatchBuffer>,
}

impl<TObject, TBatchBuffer: BatchBuffer, TBatchBase: BatchBase<TObject, TBatchBuffer>>
    Batch<TObject, TBatchBuffer, TBatchBase>
{
    pub fn new(batch_base: TBatchBase) -> Batch<TObject, TBatchBuffer, TBatchBase> {
        Batch {
            base: batch_base,
            material: 0,
            object_bundles: vec![],
            object_bundles_cache: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn add(&mut self, object: &Shared<TObject>, draw_order: i32) {
        let key: *const TObject = object.borrow().deref();
        debug_assert!(
            !self.object_bundles_cache.contains_key(&key),
            "This object already has been added"
        );

        let object_bundle = Rc::new(self.allocate(object, self.base.size(object), draw_order));
        self.object_bundles_cache
            .insert(key, Rc::clone(&object_bundle));
        self.object_bundles.push(object_bundle);
    }

    pub fn remove(&mut self, object: &Shared<TObject>) {
        let key: *const TObject = object.borrow().deref();
        if let Some(x) = self.object_bundles_cache.get(&key) {
            {
                self.object_bundles.remove_item_is(move |y| {
                    std::ptr::eq(x.borrow(), y.borrow() as *const BatchObjectBundle<TObject>)
                });
            }
            let pointers = &x.pointers;
            for (i, bundle) in self.base.bundles_mut().iter_mut().enumerate() {
                let pointer = &pointers[i];
                bundle.batch_buffer.free(pointer);
            }
            self.object_bundles_cache.remove(&key);
        }
    }

    pub fn flush(&mut self) {
        for x in self.object_bundles.iter_mut() {
            self.base.update(x);
        }
        self.flush_all();
    }

    pub fn sort_by_render_order(&mut self) {}

    pub fn triangle_count(&self) -> usize {
        match self.base.bundles().first() {
            Some(x) => x.batch_buffer.size() / x.stride as usize,
            None => 0,
        }
    }

    pub fn batch_buffers(&self) -> Vec<&TBatchBuffer> {
        self.base
            .bundles()
            .iter()
            .map(|x| &x.batch_buffer)
            .collect()
    }

    fn allocate(
        &mut self,
        object: &Shared<TObject>,
        size: usize,
        index: i32,
    ) -> BatchObjectBundle<TObject> {
        let bundles = self.base.bundles_mut();
        let data = bundles
            .iter_mut()
            .map(|x| x.batch_buffer.allocate(x.stride as usize * size))
            .collect::<Vec<_>>();
        BatchObjectBundle::new(index, Shared::clone(object), data)
    }

    fn flush_all(&mut self) {
        let bundles = self.base.bundles_mut();
        for x in bundles.iter_mut() {
            x.batch_buffer.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::graphic::batch::batch_base::tests::MockBatchBase;
    use crate::core::graphic::batch::batch_buffer::tests::MockBatchBuffer;
    use crate::core::graphic::batch::default::Batch;
    use crate::extension::shared::Shared;
    use crate::utility::test::func::MockFunc;

    #[test]
    fn test_add_and_remove() {
        let mock_func = Shared::new(MockFunc::new());
        let mut batch: Batch<i32, MockBatchBuffer, MockBatchBase> =
            Batch::new(MockBatchBase::new(&mock_func));

        let object1 = Shared::new(1234);
        let object2 = Shared::new(2345);

        batch.add(&object1, 0);
        assert_eq!(batch.object_bundles.len(), 1);
        assert_eq!(batch.object_bundles_cache.len(), 1);

        batch.add(&object2, 1);
        assert_eq!(batch.object_bundles.len(), 2);
        assert_eq!(batch.object_bundles_cache.len(), 2);

        batch.remove(&object1);
        assert_eq!(batch.object_bundles.len(), 1);
        assert_eq!(batch.object_bundles_cache.len(), 1);
    }

    #[test]
    #[should_panic(expected = "This object already has been added")]
    fn test_duplicated_add() {
        let mock_func = Shared::new(MockFunc::new());
        let mut batch: Batch<i32, MockBatchBuffer, MockBatchBase> =
            Batch::new(MockBatchBase::new(&mock_func));

        let object1 = Shared::new(1234);
        batch.add(&object1, 0);
        batch.add(&object1, 1);
    }

    #[test]
    fn test_render() {}
}
