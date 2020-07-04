use crate::core::graphic::batch::batch_base::BatchBase;
use crate::core::graphic::batch::batch_buffer::BatchBuffer;
use crate::core::graphic::batch::batch_change_manager::{BatchChangeManager, BatchChangeNotifier};
use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::extension::collection::VecExt;
use crate::extension::shared::Shared;
use crate::utility::change_notifier::ChangeNotifierObject;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

pub mod batch2d;
pub mod batch3d;
pub mod batch_base;
pub mod batch_billboard;
pub mod batch_buffer;
pub mod batch_buffer_f32;
pub mod batch_buffer_pointer;
pub mod batch_bundle;
pub mod batch_change_manager;
pub mod batch_line;
pub mod batch_object_bundle;

pub struct Batch<TObject, TBatchBuffer: BatchBuffer, TBatchBase: BatchBase<TObject, TBatchBuffer>> {
    base: TBatchBase,
    object_bundles: Vec<Rc<BatchObjectBundle<TObject>>>,
    object_bundles_cache: HashMap<*const TObject, Rc<BatchObjectBundle<TObject>>>,
    change_manager: BatchChangeManager<TObject>,
    _marker: PhantomData<fn() -> TBatchBuffer>,
}

impl<TObject, TBatchBuffer: BatchBuffer, TBatchBase: BatchBase<TObject, TBatchBuffer>>
    Batch<TObject, TBatchBuffer, TBatchBase>
{
    pub fn new(batch_base: TBatchBase) -> Batch<TObject, TBatchBuffer, TBatchBase> {
        Batch {
            base: batch_base,
            object_bundles: vec![],
            object_bundles_cache: HashMap::new(),
            change_manager: BatchChangeManager::default(),
            _marker: PhantomData,
        }
    }

    pub fn add(&mut self, object: &Shared<TObject>, draw_order: i32)
    where
        TObject: ChangeNotifierObject<BatchChangeNotifier<TObject>>,
    {
        let key: *const TObject = object.borrow().deref();
        debug_assert!(
            !self.object_bundles_cache.contains_key(&key),
            "This object already has been added"
        );

        let object_bundle = Rc::new(self.allocate(object, self.base.size(object), draw_order));
        self.object_bundles_cache
            .insert(key, Rc::clone(&object_bundle));
        self.object_bundles.push(Rc::clone(&object_bundle));
        let notifier = self.change_manager.create_notifier(&object_bundle);
        object.borrow_mut().set_change_notifier(notifier);
    }

    pub fn remove(&mut self, object: &Shared<TObject>) {
        let key: *const TObject = object.borrow().deref();
        if let Some(x) = self.object_bundles_cache.get(&key) {
            {
                self.object_bundles.remove_item_is(move |y| {
                    std::ptr::eq(x.deref(), y.deref() as *const BatchObjectBundle<TObject>)
                });
            }
            let pointers = &x.pointers;
            for (i, bundle) in self.base.bundles_mut().iter_mut().enumerate() {
                let pointer = &pointers[i];
                bundle.batch_buffer.free(pointer);
            }
            self.change_manager.remove(x);
            self.object_bundles_cache.remove(&key);
        }
    }

    pub fn flush(&mut self) {
        if !self.change_manager.targets().borrow().is_empty() {
            {
                let mut targets = self.change_manager.targets().borrow_mut();
                for (_, x) in targets.iter_mut() {
                    self.base.update(x);
                }
            }
            self.change_manager.reset();
            self.flush_all();
        }
    }

    pub fn sort_by_render_order(&mut self) {}

    pub fn vertex_count(&self) -> usize {
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
    use crate::core::graphic::batch::batch_change_manager::BatchChangeNotifier;
    use crate::core::graphic::batch::Batch;
    use crate::extension::shared::make_shared;
    use crate::utility::change_notifier::ChangeNotifierObject;
    use crate::utility::test::func::MockFunc;

    struct Object {
        pub id: i32,
    }

    impl ChangeNotifierObject<BatchChangeNotifier<Object>> for Object {
        fn set_change_notifier(&mut self, _notifier: BatchChangeNotifier<Object>) {}
    }

    #[test]
    fn test_add_and_remove() {
        let mock_func = make_shared(MockFunc::new());
        let mut batch: Batch<Object, MockBatchBuffer, MockBatchBase> =
            Batch::new(MockBatchBase::new(&mock_func));

        let object1 = make_shared(Object { id: 1234 });
        let object2 = make_shared(Object { id: 2345 });

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
        let mock_func = make_shared(MockFunc::new());
        let mut batch: Batch<Object, MockBatchBuffer, MockBatchBase> =
            Batch::new(MockBatchBase::new(&mock_func));

        let object1 = make_shared(Object { id: 1234 });
        batch.add(&object1, 0);
        batch.add(&object1, 1);
    }

    #[test]
    fn test_render() {}
}
