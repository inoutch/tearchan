use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use tearchan_util::id_manager::IdManager;

pub type RegistryId = u64;

pub struct Registry<T> {
    id_manager: IdManager<RegistryId>,
    data: RefCell<HashMap<RegistryId, T>>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Registry {
            id_manager: IdManager::new(0, |id| id + 1),
            data: RefCell::new(HashMap::new()),
        }
    }
}

impl<T> Registry<T> {
    pub fn gen_id(&self) -> RegistryId {
        self.id_manager.create_generator().gen()
    }

    pub fn register(&self, id: RegistryId, value: T) {
        self.data.borrow_mut().insert(id, value);
    }

    pub fn unregister(&self, id: RegistryId) -> Option<T> {
        self.data.borrow_mut().remove(&id)
    }

    pub fn read(&self, id: RegistryId) -> Ref<T> {
        Ref::map(self.data.borrow(), |x| x.get(&id).unwrap())
    }

    pub fn write(&self, id: RegistryId) -> RefMut<T> {
        RefMut::map(self.data.borrow_mut(), |x| x.get_mut(&id).unwrap())
    }
}
