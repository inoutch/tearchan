use crate::component::group::ComponentGroup;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Clone)]
pub struct ComponentGroupSync<T>
where
    T: Sync,
{
    inner: Arc<RwLock<ComponentGroup<T>>>,
}

impl<T> Default for ComponentGroupSync<T>
where
    T: Sync + Send,
{
    fn default() -> ComponentGroupSync<T> {
        ComponentGroupSync {
            inner: Arc::new(RwLock::new(ComponentGroup::default())),
        }
    }
}

impl<T> ComponentGroupSync<T>
where
    T: Sync + Send,
{
    pub fn new(component_group: ComponentGroup<T>) -> ComponentGroupSync<T> {
        ComponentGroupSync {
            inner: Arc::new(RwLock::new(component_group)),
        }
    }

    pub fn read(&self) -> ComponentGroupSyncReader<T> {
        ComponentGroupSyncReader {
            inner: Arc::clone(&self.inner),
        }
    }

    pub fn write(&mut self) -> ComponentGroupSyncWriter<T> {
        ComponentGroupSyncWriter {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct ComponentGroupSyncReader<T>
where
    T: Sync,
{
    inner: Arc<RwLock<ComponentGroup<T>>>,
}

impl<T> Clone for ComponentGroupSyncReader<T>
where
    T: Sync,
{
    fn clone(&self) -> ComponentGroupSyncReader<T> {
        ComponentGroupSyncReader {
            inner: Arc::clone(&self.inner),
        }
    }
}

unsafe impl<T> Send for ComponentGroupSyncReader<T> where T: Sync {}

impl<T> ComponentGroupSyncReader<T>
where
    T: Sync,
{
    pub fn get(&self) -> RwLockReadGuard<ComponentGroup<T>> {
        self.inner.read().unwrap()
    }
}
pub struct ComponentGroupSyncWriter<T>
where
    T: Sync,
{
    inner: Arc<RwLock<ComponentGroup<T>>>,
}
unsafe impl<T> Send for ComponentGroupSyncWriter<T> where T: Sync {}

impl<T> ComponentGroupSyncWriter<T>
where
    T: Sync,
{
    pub fn get_mut(&mut self) -> RwLockWriteGuard<ComponentGroup<T>> {
        self.inner.write().unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::component::group::{ComponentGroup, ComponentGroupDeserializableData};
    use crate::component::group_sync::ComponentGroupSync;
    use crate::component::zip::ZipEntity1;
    use crate::entity::manager::ENTITY_REMAPPER;
    use tearchan_util::thread::ThreadPool;

    #[test]
    fn test_default() {
        let _: ComponentGroupSync<i32> = ComponentGroupSync::default();
    }

    #[test]
    fn test_zip() {
        let thread_pool = ThreadPool::new(4);

        let mut group1: ComponentGroupSync<i32> = ComponentGroupSync::default();
        let mut group2: ComponentGroupSync<i32> = ComponentGroupSync::default();

        {
            let mut writer1 = group1.write();
            let mut writer2 = group2.write();

            thread_pool.execute(move || {
                writer1.get_mut().push(1, 32);
                writer2.get_mut().push(1, 64);

                writer1.get_mut().push(2, 12);
                writer2.get_mut().push(2, 24);

                writer1.get_mut().push(3, 45);
                writer2.get_mut().push(3, 90);

                writer1.get_mut().push(4, 67);
                writer2.get_mut().push(4, 134);
            });
        }
        thread_pool.join();

        // Read x Read
        {
            let read1 = group1.read();
            let read2 = group2.read();
            thread_pool.execute(move || {
                read1
                    .get()
                    .iter()
                    .zip_entities(&ZipEntity1::new(&read2.get()))
                    .for_each(|(_id, entity1, entity2)| {
                        assert_eq!(entity1 * 2, *entity2);
                    });
            });
        }
        thread_pool.join();

        // Write x Read
        {
            let mut write1 = group1.write();
            let read2 = group2.read();
            thread_pool.execute(move || {
                write1
                    .get_mut()
                    .iter_mut()
                    .zip_entities_mut(&ZipEntity1::new(&read2.get()))
                    .for_each(|(_id, entity1, entity2)| {
                        *entity1 = entity2 * 2;
                    });
            });
        }
        thread_pool.join();

        let read1 = group1.read();
        assert_eq!(read1.get().get(1).unwrap(), &128);
        assert_eq!(read1.get().get(2).unwrap(), &48);
        assert_eq!(read1.get().get(3).unwrap(), &180);
        assert_eq!(read1.get().get(4).unwrap(), &268);
    }

    #[test]
    fn test_serialization() {
        let _lock = ENTITY_REMAPPER.lock();
        let mut group: ComponentGroupSync<i32> = ComponentGroupSync::default();
        group.write().get_mut().push(0, 0);
        group.write().get_mut().push(1, 11);
        group.write().get_mut().push(2, 22);

        let read = group.read();
        let value: &ComponentGroup<i32> = &read.get();
        let str = serde_json::to_string(&value.to_data()).unwrap();
        let data: ComponentGroupDeserializableData<i32> = serde_json::from_str(&str).unwrap();
        let component_group = ComponentGroup::default();
        let mut component_group_sync = ComponentGroupSync::new(component_group);
        component_group_sync.write().get_mut().load_data(data);
        assert_eq!(*component_group_sync.read().get().get(0).unwrap(), 0);
        assert_eq!(*component_group_sync.read().get().get(1).unwrap(), 11);
        assert_eq!(*component_group_sync.read().get().get(2).unwrap(), 22);
    }
}
