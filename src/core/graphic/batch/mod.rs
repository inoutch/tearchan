use crate::core::graphic::batch::batch_change_manager::{BatchChangeManager, BatchChangeNotifier};
use crate::core::graphic::batch::batch_pointer::BatchPointer;
use crate::core::graphic::batch::batch_provider::BatchProvider;
use crate::core::graphic::hal::buffer_interface::BufferInterface;
use crate::extension::shared::{clone_shared, Shared};
use crate::utility::change_notifier::ChangeNotifierObject;
use serde::export::PhantomData;
use std::collections::HashMap;
use std::rc::Rc;
use std::ops::Deref;

pub mod batch2d;
pub mod batch_buffer;
pub mod batch_change_manager;
pub mod batch_pointer;
pub mod batch_provider;
pub mod helpers;

pub type BatchIndex = u32;
pub const DEFAULT_DEFRAGMENTATION_BORDER: usize = 10000;

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
    pub defragmentation_border: usize,
    need_all_copies: bool,
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
            defragmentation_border: DEFAULT_DEFRAGMENTATION_BORDER,
            need_all_copies: false,
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
        self.provider.open();
        if self.need_all_copies {
            let targets = &mut self.contexts;
            if !targets.is_empty() {
                for (_, target) in targets.iter_mut() {
                    self.provider.update(target);
                }
            }
            self.need_all_copies = false;
        } else {
            let mut targets = self.change_manager.targets().borrow_mut();
            if !targets.is_empty() {
                // Receive from client notification
                for (_, target) in targets.iter_mut() {
                    self.provider.update(target);
                }
            }
        }
        self.provider.close();

        self.change_manager.reset();
    }

    pub fn index_size(&self) -> usize {
        self.provider.index_buffer().last()
    }

    pub fn index_buffer(&self) -> &TIndexBuffer {
        self.provider.index_buffer().buffer()
    }

    pub fn vertex_buffers(&self) -> Vec<&TVertexBuffer> {
        self.provider
            .vertex_buffer_contexts()
            .iter()
            .map(|x| x.buffer.buffer())
            .collect()
    }

    fn allocate(
        &mut self,
        object: &Shared<TObject>,
        index_size: usize,
        vertex_size: usize,
        draw_order: BatchIndex,
    ) -> BatchContext<TObject> {
        let vertex_pointers = self
            .provider
            .vertex_buffer_contexts_mut()
            .iter_mut()
            .map(|context| {
                context
                    .buffer
                    .allocate(context.stride as usize * vertex_size)
            })
            .collect::<Vec<_>>();

        let index_pointer = self.provider.index_buffer_mut().allocate(index_size);
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
        let mut fragmentation_size = self.provider.index_buffer().fragmentation_size();
        for (i, vertex_buffer) in self
            .provider
            .vertex_buffer_contexts_mut()
            .iter_mut()
            .enumerate()
        {
            vertex_buffer.buffer.free(&context.vertex_pointers[i]);
            fragmentation_size += vertex_buffer.buffer.fragmentation_size();
        }

        if fragmentation_size >= self.defragmentation_border {
            self.provider.index_buffer_mut().defragmentation();
            for vertex_buffer in self.provider.vertex_buffer_contexts_mut().iter_mut() {
                vertex_buffer.buffer.defragmentation();
            }
            self.need_all_copies = true;
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
    use crate::math::mesh::square::create_square_normals;
    use crate::math::mesh::{IndexType, MeshBuilder};
    use crate::utility::test::func::MockFunc;
    use nalgebra_glm::{vec2, vec4};

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

    #[test]
    fn test_defragmentation() {
        let mock = make_shared(MockFunc::new());
        let mut batch: MockBatch = Batch::new(MockBatchProvider::new(&mock));
        batch.defragmentation_border = 0;

        let mesh1 = MeshBuilder::new()
            .normals(create_square_normals())
            .with_square_and_color(vec2(1.0f32, 2.0f32), vec4(1.0f32, 0.0f32, 0.0f32, 1.0f32))
            .build()
            .unwrap();
        let mesh2 = MeshBuilder::new()
            .normals(create_square_normals())
            .with_square_and_color(vec2(1.0f32, 2.0f32), vec4(0.0f32, 1.0f32, 0.0f32, 1.0f32))
            .build()
            .unwrap();
        let mesh3 = MeshBuilder::new()
            .normals(create_square_normals())
            .with_square_and_color(vec2(1.0f32, 2.0f32), vec4(0.0f32, 0.0f32, 1.0f32, 1.0f32))
            .build()
            .unwrap();

        let p1 = make_shared(Polygon::new(mesh1.clone()));
        let p2 = make_shared(Polygon::new(mesh2.clone()));
        let p3 = make_shared(Polygon::new(mesh3.clone()));
        batch.add(&p1, 0);
        batch.add(&p2, 0);
        batch.add(&p3, 0);

        let indices = mesh1.indices.clone();
        // let colors1 = mesh1
        //     .colors
        //     .iter()
        //     .map(|c| vec![c.x, c.y, c.z, c.w])
        //     .flatten()
        //     .collect::<Vec<f32>>();
        // let colors2 = mesh2
        //     .colors
        //     .iter()
        //     .map(|c| vec![c.x, c.y, c.z, c.w])
        //     .flatten()
        //     .collect::<Vec<f32>>();
        let colors3 = mesh3
            .colors
            .iter()
            .map(|c| vec![c.x, c.y, c.z, c.w])
            .flatten()
            .collect::<Vec<f32>>();

        batch.flush();
        // 6
        // 12 + 16 + 8 + 12 = 48
        batch.defragmentation_border = 55;
        batch.remove(&p1);

        // no defragmentation
        assert_eq!(batch.provider.index_buffer.last(), 18);
        assert_eq!(batch.provider.vertex_buffers[0].buffer.last(), 36);
        assert_eq!(batch.provider.vertex_buffers[1].buffer.last(), 48);
        assert_eq!(batch.provider.vertex_buffers[2].buffer.last(), 24);
        assert_eq!(batch.provider.vertex_buffers[3].buffer.last(), 36);

        // use defragmentation
        batch.defragmentation_border = 108;
        batch.remove(&p2);
        assert_eq!(batch.provider.index_buffer.last(), 6);
        assert_eq!(batch.provider.vertex_buffers[0].buffer.last(), 12);
        assert_eq!(batch.provider.vertex_buffers[1].buffer.last(), 16);
        assert_eq!(batch.provider.vertex_buffers[2].buffer.last(), 8);
        assert_eq!(batch.provider.vertex_buffers[3].buffer.last(), 12);

        batch.flush();
        {
            let mut expect: Vec<IndexType> = vec![];
            expect.extend(&indices);
            assert_eq!(
                batch.provider.index_buffer.buffer().indices.borrow()
                    [0..batch.provider.index_buffer.last()]
                    .to_vec(),
                expect
            );
        }
        {
            let mut expect: Vec<f32> = vec![];
            expect.extend(&colors3);
            assert_nearly_eq!(
                batch.provider.vertex_buffers[1]
                    .buffer
                    .buffer()
                    .vertices
                    .borrow()[0..batch.provider.vertex_buffers[1].buffer.last()]
                    .to_vec(),
                expect
            );
        }
    }
}
