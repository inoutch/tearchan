use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use tearchan_util::id_manager::IdManager;

pub type RegistryId = u64;

pub struct Registry<T> {
    id_manager: IdManager<RegistryId>,
    storage: RefCell<Storage<T>>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Registry {
            id_manager: IdManager::new(0, |id| id + 1),
            storage: RefCell::new(Storage {
                data: HashMap::new(),
            }),
        }
    }
}

impl<T> Registry<T> {
    pub fn gen_id(&self) -> RegistryId {
        self.id_manager.create_generator().gen()
    }

    pub fn register(&self, id: RegistryId, value: T) {
        self.storage.borrow_mut().register(id, value);
    }

    pub fn unregister(&self, id: RegistryId) -> Option<T> {
        self.storage.borrow_mut().unregister(id)
    }

    pub fn read(&self, id: RegistryId) -> Ref<T> {
        Ref::map(self.storage.borrow(), |x| x.read(id))
    }

    pub fn write(&self, id: RegistryId) -> RefMut<T> {
        RefMut::map(self.storage.borrow_mut(), |x| x.write(id))
    }

    pub fn read_storage(&self) -> Ref<Storage<T>> {
        self.storage.borrow()
    }

    pub fn write_storage(&self) -> RefMut<Storage<T>> {
        self.storage.borrow_mut()
    }
}

pub struct Storage<T> {
    data: HashMap<RegistryId, T>,
}

impl<T> Storage<T> {
    pub fn read(&self, id: RegistryId) -> &T {
        self.data.get(&id).unwrap()
    }

    pub fn write(&mut self, id: RegistryId) -> &mut T {
        self.data.get_mut(&id).unwrap()
    }

    pub fn register(&mut self, id: RegistryId, value: T) {
        self.data.insert(id, value);
    }

    pub fn unregister(&mut self, id: RegistryId) -> Option<T> {
        self.data.remove(&id)
    }
}
