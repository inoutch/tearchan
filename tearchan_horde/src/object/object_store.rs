use intertrait::cast::CastRc;
use intertrait::CastFrom;
use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;
use tearchan_core::game::object::{
    BorrowError, BorrowFlag, BorrowMutError, BorrowRef, BorrowRefMut, GameObjectId, Ref, RefMut,
    EMPTY_ID, UNUSED,
};

pub trait ObjectStoreBase: CastFrom {}

pub struct ObjectStore<T: ?Sized> {
    id: GameObjectId,
    parent_id: Option<GameObjectId>,
    kind: String,
    borrow: Rc<Cell<BorrowFlag>>,
    store: Rc<T>,
}

impl<T: ?Sized> ObjectStore<T>
where
    T: CastFrom,
{
    pub fn new(kind: String, store: Rc<T>) -> ObjectStore<T> {
        ObjectStore {
            id: EMPTY_ID,
            parent_id: None,
            kind,
            borrow: Rc::new(Cell::new(UNUSED)),
            store,
        }
    }

    pub fn id(&self) -> GameObjectId {
        self.id
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn parent_id(&self) -> Option<GameObjectId> {
        self.parent_id
    }

    pub fn cast<U>(&self) -> Option<ObjectStore<U>>
    where
        U: ?Sized + CastFrom,
    {
        self.store
            .clone()
            .cast::<U>()
            .ok()
            .map(|store| ObjectStore::new(self.kind.to_string(), store))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(&self) -> Self {
        ObjectStore {
            id: self.id,
            parent_id: self.parent_id,
            kind: self.kind.to_string(),
            borrow: Rc::clone(&self.borrow),
            store: Rc::clone(&self.store),
        }
    }

    pub fn set_id(&mut self, new_id: GameObjectId) {
        debug_assert_eq!(self.id, EMPTY_ID);
        self.id = new_id;
    }

    pub fn set_parent_id(&mut self, parent_id: GameObjectId) {
        debug_assert!(self.parent_id.is_none() || Some(parent_id) == self.parent_id);
        self.parent_id = Some(parent_id);
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        match BorrowRef::new(&self.borrow) {
            Some(b) => Ok(Ref::new(self.store.deref(), b)),
            None => Err(BorrowError),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    pub fn try_borrow_mut(&mut self) -> Result<RefMut<'_, T>, BorrowMutError> {
        match BorrowRefMut::new(&self.borrow) {
            Some(b) => Ok(RefMut::new(
                unsafe { Rc::get_mut_unchecked(&mut self.store) },
                b,
            )),
            None => Err(BorrowMutError),
        }
    }
}
