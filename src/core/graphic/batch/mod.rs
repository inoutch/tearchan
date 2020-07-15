use crate::core::graphic::batch::batch_change_manager::{BatchChangeManager, BatchChangeNotifier};
use crate::core::graphic::batch::batch_pointer::BatchPointer;
use crate::core::graphic::batch::batch_provider::BatchProvider;
use crate::core::graphic::hal::buffer_interface::BufferInterface;
use crate::extension::shared::{clone_shared, Shared};
use crate::utility::change_notifier::ChangeNotifierObject;
use serde::export::PhantomData;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub mod batch_buffer;
pub mod batch_change_manager;
pub mod batch_pointer;
pub mod batch_provider;

pub type BatchIndex = u32;

pub struct BatchContext<TObject> {
    pub draw_order: BatchIndex,
    pub object: Shared<TObject>,
    pub index_pointer: Shared<BatchPointer>,
    pub vertex_pointers: Vec<Shared<BatchPointer>>, // The same number as the vertex_buffer in the provider
}

pub struct Batch<TObject, TBatchProvider, TIndexBuffer, TVertexBuffer> {
    provider: TBatchProvider,
    contexts: HashMap<*const TObject, Rc<BatchContext<TObject>>>,
    change_manager: BatchChangeManager<TObject>,
    _phantom_data0: PhantomData<TIndexBuffer>,
    _phantom_data1: PhantomData<TVertexBuffer>,
}

impl<TObject, TBatchProvider, TIndexBuffer, TVertexBuffer>
    Batch<TObject, TBatchProvider, TIndexBuffer, TVertexBuffer>
where
    TIndexBuffer: BufferInterface,
    TVertexBuffer: BufferInterface,
    TBatchProvider: BatchProvider<TObject, TIndexBuffer, TVertexBuffer>,
{
    pub fn new(provider: TBatchProvider) -> Self {
        Batch {
            provider,
            contexts: HashMap::new(),
            change_manager: BatchChangeManager::default(),
            _phantom_data0: PhantomData,
            _phantom_data1: PhantomData,
        }
    }

    pub fn add(&mut self, object: &Shared<TObject>, draw_order: BatchIndex)
    where
        TObject: ChangeNotifierObject<BatchChangeNotifier<TObject>>,
    {
        let key = get_object_key(object);
        debug_assert!(
            !self.contexts.contains_key(&key),
            "object has already been added"
        );

        let context = Rc::new(self.allocate(
            object,
            self.provider.index_size(object),
            self.provider.vertex_size(object),
            draw_order,
        ));
        let notifier = self.change_manager.create_notifier(&context);

        self.contexts.insert(key, context);

        object.borrow_mut().set_change_notifier(notifier);
    }

    pub fn remove(&mut self, object: &Shared<TObject>) {
        let context = match self.contexts.remove(&get_object_key(object)) {
            Some(context) => context,
            None => return,
        };
        self.free(&context);
        self.change_manager.remove(&context);
    }

    pub fn provider(&self) -> &TBatchProvider {
        &self.provider
    }

    pub fn provider_mut(&mut self) -> &mut TBatchProvider {
        &mut self.provider
    }

    pub fn flush(&mut self) {
        let mut targets = self.change_manager.targets().borrow_mut();

        self.provider.open();
        if !targets.is_empty() {
            // Receive from client notification
            for (_, target) in targets.iter_mut() {
                self.provider.update(target);
            }
        }
        self.provider.close();
    }

    fn allocate(
        &mut self,
        object: &Shared<TObject>,
        index_size: usize,
        vertex_size: usize,
        draw_order: BatchIndex,
    ) -> BatchContext<TObject> {
        let index_pointer = self.provider.index_buffer_mut().allocate(index_size);
        let vertex_pointers = self
            .provider
            .contexts_mut()
            .iter_mut()
            .map(|context| {
                context
                    .buffer
                    .allocate(context.stride as usize * vertex_size)
            })
            .collect();
        BatchContext {
            draw_order,
            object: clone_shared(object),
            vertex_pointers,
            index_pointer,
        }
    }

    fn free(&mut self, context: &Rc<BatchContext<TObject>>) {
        self.provider
            .index_buffer_mut()
            .free(&context.index_pointer);
        for (i, vertex_buffer) in self.provider.contexts_mut().iter_mut().enumerate() {
            vertex_buffer.buffer.free(&context.vertex_pointers[i]);
        }
    }
}

fn get_object_key<TObject>(object: &Shared<TObject>) -> *const TObject {
    object.borrow().deref() as *const TObject
}

#[cfg(test)]
mod test {
    use crate::core::graphic::batch::batch_provider::test::MockBatchProvider;
    use crate::core::graphic::batch::Batch;
    use crate::core::graphic::hal::index_buffer::test::MockIndexBuffer;
    use crate::core::graphic::hal::vertex_buffer::test::MockVertexBuffer;
    use crate::core::graphic::polygon::Polygon;
    use crate::extension::shared::make_shared;
    use crate::math::mesh::MeshBuilder;
    use crate::utility::test::func::MockFunc;
    use nalgebra_glm::vec2;
    use crate::math::mesh::square::create_square_normals;

    pub type MockBatch = Batch<Polygon, MockBatchProvider, MockIndexBuffer, MockVertexBuffer>;

    #[test]
    fn test_add_and_remove() {
        let mock = make_shared(MockFunc::new());
        let mut batch: MockBatch = Batch::new(MockBatchProvider::new(&mock));

        let mesh = MeshBuilder::new()
            .normals(create_square_normals())
            .with_square(vec2(1.0f32, 2.0f32))
            .build()
            .unwrap();
        let index_size1 = mesh.indices.len();

        let polygon1 = make_shared(Polygon::new(mesh));
        batch.add(&polygon1, 0);

        assert_eq!(batch.provider.index_buffer.last(), index_size1);
        assert_eq!(batch.provider.index_buffer.last(), 6);

        assert_eq!(batch.provider.vertex_buffers[0].buffer.last(), 12);
        assert_eq!(batch.provider.vertex_buffers[1].buffer.last(), 16);
        assert_eq!(batch.provider.vertex_buffers[2].buffer.last(), 8);
        assert_eq!(batch.provider.vertex_buffers[3].buffer.last(), 12);

        let mesh = MeshBuilder::new()
            .normals(create_square_normals())
            .with_square(vec2(1.0f32, 2.0f32))
            .build()
            .unwrap();
        let positions = mesh
            .positions
            .iter()
            .map(|p| vec![p.x, p.y, p.z])
            .flatten()
            .collect::<Vec<f32>>();
        let colors = mesh
            .colors
            .iter()
            .map(|c| vec![c.x, c.y, c.z, c.w])
            .flatten()
            .collect::<Vec<f32>>();
        let texcoords = mesh
            .texcoords
            .iter()
            .map(|t| vec![t.x, t.y])
            .flatten()
            .collect::<Vec<f32>>();
        let normals = mesh
            .normals
            .iter()
            .map(|n| vec![n.x, n.y, n.z])
            .flatten()
            .collect::<Vec<f32>>();

        let index_size2 = mesh.indices.len();
        let polygon2 = make_shared(Polygon::new(mesh));
        batch.add(&polygon2, 0);

        assert_eq!(
            batch.provider.index_buffer.last(),
            index_size1 + index_size2
        );
        assert_eq!(batch.provider.index_buffer.last(), 12);

        assert_eq!(batch.provider.vertex_buffers[0].buffer.last(), 24);
        assert_eq!(batch.provider.vertex_buffers[1].buffer.last(), 32);
        assert_eq!(batch.provider.vertex_buffers[2].buffer.last(), 16);
        assert_eq!(batch.provider.vertex_buffers[3].buffer.last(), 24);

        assert_nearly_eq!(
            batch.provider.vertex_buffers[0]
                .buffer
                .buffer()
                .vertices
                .borrow()[0..24]
                .to_vec(),
            vec![0.0f32; 24]
        );
        assert_nearly_eq!(
            batch.provider.vertex_buffers[1]
                .buffer
                .buffer()
                .vertices
                .borrow()[0..32]
                .to_vec(),
            vec![0.0f32; 32]
        );
        assert_nearly_eq!(
            batch.provider.vertex_buffers[2]
                .buffer
                .buffer()
                .vertices
                .borrow()[0..16]
                .to_vec(),
            vec![0.0f32; 16]
        );
        assert_nearly_eq!(
            batch.provider.vertex_buffers[3]
                .buffer
                .buffer()
                .vertices
                .borrow()[0..24]
                .to_vec(),
            vec![0.0f32; 24]
        );

        batch.flush();
        {
            let mut expect: Vec<f32> = vec![];
            expect.extend(&positions);
            expect.extend(&positions);
            assert_nearly_eq!(
                batch.provider.vertex_buffers[0]
                    .buffer
                    .buffer()
                    .vertices
                    .borrow()[0..24]
                    .to_vec(),
                expect
            );
        }
        {
            let mut expect: Vec<f32> = vec![];
            expect.extend(&colors);
            expect.extend(&colors);
            assert_nearly_eq!(
                batch.provider.vertex_buffers[1]
                    .buffer
                    .buffer()
                    .vertices
                    .borrow()[0..32]
                    .to_vec(),
                expect
            );
        }
        {
            let mut expect: Vec<f32> = vec![];
            expect.extend(&texcoords);
            expect.extend(&texcoords);
            assert_nearly_eq!(
                batch.provider.vertex_buffers[2]
                    .buffer
                    .buffer()
                    .vertices
                    .borrow()[0..16]
                    .to_vec(),
                expect
            );
        }
        {
            let mut expect: Vec<f32> = vec![];
            expect.extend(&normals);
            expect.extend(&normals);
            assert_nearly_eq!(
                batch.provider.vertex_buffers[3]
                    .buffer
                    .buffer()
                    .vertices
                    .borrow()[0..24]
                    .to_vec(),
                expect
            );
        }

        batch.remove(&polygon1);

        assert_eq!(batch.provider.vertex_buffers[0].buffer.last(), 24);
        assert_eq!(batch.provider.vertex_buffers[1].buffer.last(), 32);
        assert_eq!(batch.provider.vertex_buffers[2].buffer.last(), 16);
        assert_eq!(batch.provider.vertex_buffers[3].buffer.last(), 24);
    }

    #[test]
    #[should_panic(expected = "object has already been added")]
    fn test_duplicated_addition() {
        let mock = make_shared(MockFunc::new());
        let mut batch: MockBatch = Batch::new(MockBatchProvider::new(&mock));

        let mesh = MeshBuilder::new()
            .with_square(vec2(1.0f32, 2.0f32))
            .build()
            .unwrap();

        let polygon = make_shared(Polygon::new(mesh));
        batch.add(&polygon, 0);
        batch.add(&polygon, 0);
    }
}
