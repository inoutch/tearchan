use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type Shared<T> = Rc<RefCell<T>>;

pub type WeakShared<T> = Weak<RefCell<T>>;

pub fn make_shared<T>(v: T) -> Shared<T> {
    Rc::new(RefCell::new(v))
}

pub fn clone_shared<T>(v: &Shared<T>) -> Shared<T> {
    Rc::clone(v)
}
