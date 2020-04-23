use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

pub struct Shared<T> {
    v: Rc<RefCell<T>>,
}

impl<T> Shared<T> {
    pub fn new(t: T) -> Shared<T> {
        Shared {
            v: Rc::new(RefCell::new(t)),
        }
    }

    pub fn clone(shared: &Shared<T>) -> Shared<T> {
        Shared {
            v: Rc::clone(&shared.v),
        }
    }
}

impl<T> Shared<T> {
    pub fn borrow(&self) -> Ref<T> {
        self.v.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.v.borrow_mut()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.v.as_ptr()
    }
}

impl<T: fmt::Display> fmt::Display for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl<T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.deref())
    }
}

impl<'a, T> Deref for Shared<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe { self.as_ptr().as_ref().unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use crate::extension::shared::Shared;

    struct Struct {
        value: i32,
    }

    impl Struct {
        pub fn set(&mut self, value: i32) {
            self.value = value;
        }
    }

    #[test]
    fn test_mutation() {
        let s1 = Shared::new(Struct { value: 123 });
        let s2 = Shared::clone(&s1);
        assert_eq!(s1.value, 123);

        {
            let mut s1_mut = s1.borrow_mut();
            s1_mut.set(1);
        }
        assert_eq!(s2.value, 1);

        {
            let mut s2_mut = s1.borrow_mut();
            s2_mut.set(33);
        }
        assert_eq!(s2.value, 33);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_borrows() {
        let s1 = Shared::new(Struct { value: 123 });
        let s2 = Shared::clone(&s1);

        assert_eq!(s1.value, 123);
        let mut s1_mut = s1.borrow_mut();
        s1_mut.set(1);

        assert_eq!(s2.value, 1);
        s2.borrow_mut().set(1);
    }
}
