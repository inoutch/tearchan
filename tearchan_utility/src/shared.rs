use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

pub struct Shared<T> {
    v: Rc<RefCell<T>>,
}

impl<T> Shared<T> {
    pub fn new(t: T) -> Shared<T> {
        Shared {
            v: Rc::new(RefCell::new(t)),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(&self) -> Shared<T> {
        Shared {
            v: Rc::clone(&self.v),
        }
    }

    pub fn borrow(&self) -> Ref<T> {
        self.v.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.v.borrow_mut()
    }

    pub fn downgrade(&self) -> WeakShared<T> {
        WeakShared::new(self)
    }
}

#[derive(Clone)]
pub struct WeakShared<T> {
    v: Weak<RefCell<T>>,
}

impl<T> WeakShared<T> {
    pub fn new(t: &Shared<T>) -> WeakShared<T> {
        WeakShared {
            v: Rc::downgrade(&t.v),
        }
    }

    pub fn upgrade(&self) -> Option<Shared<T>> {
        self.v.upgrade().map(|v| Shared { v })
    }
}

#[cfg(test)]
mod test {
    use crate::shared::Shared;
    use std::ops::Deref;

    #[test]
    fn test() {
        let a = Shared::new(123);
        let b = Shared::clone(&a);

        assert_eq!(a.borrow().deref(), &123);
        assert_eq!(b.borrow().deref(), &123);

        *b.borrow_mut() = 256;

        assert_eq!(a.borrow().deref(), &256);
        assert_eq!(b.borrow().deref(), &256);
    }
}
