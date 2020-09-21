use intertrait::cast::CastRc;
use intertrait::CastFrom;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::rc::Rc;

pub mod game_object_base;
pub mod game_object_manager;
pub mod game_object_operator;

pub type GameObjectId = *const bool;

type BorrowFlag = isize;
const UNUSED: BorrowFlag = 0;

#[derive(Debug)]
pub struct BorrowError;

#[derive(Debug)]
pub struct BorrowMutError;

#[inline(always)]
fn is_writing(x: BorrowFlag) -> bool {
    x < UNUSED
}

#[inline(always)]
fn is_reading(x: BorrowFlag) -> bool {
    x > UNUSED
}

pub struct GameObject<T: ?Sized>
where
    T: CastFrom,
{
    id_marker: Rc<Pin<Box<bool>>>,
    borrow: Rc<Cell<BorrowFlag>>,
    object: Rc<T>,
}

impl<T: ?Sized> GameObject<T>
where
    T: CastFrom,
{
    pub fn new(object: Rc<T>) -> GameObject<T> {
        GameObject {
            id_marker: Rc::new(Box::pin(true)),
            borrow: Rc::new(Cell::new(UNUSED)),
            object,
        }
    }

    pub fn from_inner_properties(
        object: Rc<T>,
        id_marker: Rc<Pin<Box<bool>>>,
        borrow: Rc<Cell<BorrowFlag>>,
    ) -> GameObject<T> {
        GameObject {
            id_marker,
            borrow,
            object,
        }
    }

    pub fn clone_inner_borrow(&self) -> Rc<Cell<BorrowFlag>> {
        self.borrow.clone()
    }

    pub fn clone_inner_id_marker(&self) -> Rc<Pin<Box<bool>>> {
        self.id_marker.clone()
    }

    pub fn clone_inner_object(&self) -> Rc<T> {
        self.object.clone()
    }

    pub fn id(&self) -> GameObjectId {
        self.id_marker.deref().deref()
    }

    pub fn cast<U>(&self) -> Option<GameObject<U>>
    where
        U: ?Sized + CastFrom,
    {
        self.object
            .clone()
            .cast::<U>()
            .ok()
            .map(|x| GameObject::new(x))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        match BorrowRef::new(&self.borrow) {
            Some(b) => Ok(Ref {
                value: self.object.deref(),
                _borrow: b,
            }),
            None => Err(BorrowError),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    pub fn try_borrow_mut(&mut self) -> Result<RefMut<'_, T>, BorrowMutError> {
        match BorrowRefMut::new(&self.borrow) {
            Some(b) => Ok(RefMut {
                value: unsafe { Rc::get_mut_unchecked(&mut self.object) },
                _borrow: b,
            }),
            None => Err(BorrowMutError),
        }
    }

    pub fn clone(&self) -> Self {
        GameObject {
            id_marker: Rc::clone(&self.id_marker),
            borrow: Rc::clone(&self.borrow),
            object: Rc::clone(&self.object),
        }
    }
}

struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    pub fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        let b = borrow.get().wrapping_add(1);
        if !is_reading(b) {
            None
        } else {
            borrow.set(b);
            Some(BorrowRef { borrow })
        }
    }
}

impl Drop for BorrowRef<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(is_reading(borrow));
        self.borrow.set(borrow - 1);
    }
}

impl Clone for BorrowRef<'_> {
    #[inline]
    fn clone(&self) -> Self {
        let borrow = self.borrow.get();
        debug_assert!(is_reading(borrow));

        assert_ne!(borrow, isize::MAX);
        self.borrow.set(borrow + 1);
        BorrowRef {
            borrow: self.borrow,
        }
    }
}

pub struct Ref<'a, T>
where
    T: ?Sized,
{
    value: &'a T,
    _borrow: BorrowRef<'a>,
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

struct BorrowRefMut<'a> {
    borrow: &'a Cell<BorrowFlag>,
}

impl Drop for BorrowRefMut<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(is_writing(borrow));
        self.borrow.set(borrow + 1);
    }
}

impl<'a> BorrowRefMut<'a> {
    #[inline]
    fn new(borrow: &'a Cell<BorrowFlag>) -> Option<BorrowRefMut<'a>> {
        match borrow.get() {
            UNUSED => {
                borrow.set(UNUSED - 1);
                Some(BorrowRefMut { borrow })
            }
            _ => None,
        }
    }
}

impl<'a> Clone for BorrowRefMut<'a> {
    fn clone(&self) -> BorrowRefMut<'a> {
        let borrow = self.borrow.get();
        debug_assert!(is_writing(borrow));

        assert_ne!(borrow, isize::MIN);
        self.borrow.set(borrow - 1);
        BorrowRefMut {
            borrow: self.borrow,
        }
    }
}

pub struct RefMut<'a, T: ?Sized + 'a> {
    value: &'a mut T,
    _borrow: BorrowRefMut<'a>,
}

impl<T: ?Sized> Deref for RefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<T: ?Sized> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

#[cfg(test)]
mod test {
    use crate::game::object::game_object_manager::GameObjectManager;
    use crate::game::object::GameObject;
    use intertrait::{cast_to, CastFrom};
    use std::rc::Rc;

    trait RenderObject: CastFrom {
        fn render(&self);

        fn render_mut(&mut self);
    }

    trait UpdateObject: CastFrom {
        fn update(&self, delta: f32);
    }

    struct Object {
        id: i32,
    }

    #[cast_to]
    impl RenderObject for Object {
        fn render(&self) {}

        fn render_mut(&mut self) {}
    }

    #[cast_to]
    impl UpdateObject for Object {
        fn update(&self, _delta: f32) {}
    }

    #[test]
    fn test_type() {
        let mut game_object_manager_1: GameObjectManager<dyn RenderObject> =
            GameObjectManager::new();
        let mut game_object_manager_2: GameObjectManager<dyn UpdateObject> =
            GameObjectManager::new();

        for _ in 0..10 {
            let game_object = GameObject::new(Rc::new(Object { id: 33 }));
            game_object_manager_1.add(GameObject::from_inner_properties(
                game_object.clone_inner_object(),
                game_object.clone_inner_id_marker(),
                game_object.clone_inner_borrow(),
            ));
            game_object_manager_2.add(GameObject::from_inner_properties(
                game_object.clone_inner_object(),
                game_object.clone_inner_id_marker(),
                game_object.clone_inner_borrow(),
            ));
        }
        assert_eq!(game_object_manager_1.len(), 10);
        assert_eq!(game_object_manager_2.len(), 10);
    }

    #[test]
    fn test_inheritance() {
        let original = GameObject::new(Rc::new(Object { id: 24 }));
        let mut casted_clone = original.cast::<dyn RenderObject>().unwrap();
        let force_mut = unsafe { Rc::get_mut_unchecked(&mut casted_clone.object) };
        force_mut.render_mut();
    }

    #[test]
    fn test_borrow() {
        let original = GameObject::new(Rc::new(Object { id: 99 }));

        let mut cloned_1 = original.clone();
        let mut cloned_2 = original.clone();

        {
            let b1 = cloned_1.borrow();
            {
                let b2 = cloned_2.borrow();

                assert_eq!(b1.id, 99);
                assert_eq!(b2.id, 99);
            }

            assert!(cloned_2.try_borrow_mut().is_err());
        }

        let _b1 = cloned_1.borrow_mut();
        assert!(cloned_2.try_borrow_mut().is_err());
    }
}
