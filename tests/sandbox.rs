use std::cell::RefCell;
use std::rc::{Rc, Weak};

trait Trait {
    fn foo(&self) -> i32;
}

#[derive(Debug)]
struct Struct {
    value: i32,
}

struct Struct2<'a> {
    value: &'a Struct,
}

impl Trait for Struct {
    fn foo(&self) -> i32 {
        self.value
    }
}

fn new() -> Rc<dyn Trait> {
    Rc::new(Struct { value: 0 })
}

fn println(value: &Struct) {
    println!("{:?}", value);
}

fn increment(value: &mut Struct) {
    value.value += 1;
}

fn return_ref() -> Weak<dyn Trait> {
    let a: Rc<dyn Trait> = Rc::new(Struct { value: 0 });
    Rc::downgrade(&a)
}

pub trait SquareExt {
    fn square(self) -> Self;
}

impl SquareExt for i32 {
    fn square(self) -> Self {
        self * self
    }
}

fn main() {
    println!("{}", 5.square());
}

#[test]
fn sandbox() {
    let data = Rc::new(RefCell::new(vec![1, 2, 3]));
    data.borrow_mut().push(5);
    println!("{:?}", data.borrow());
    // println!("{:?}", data);
    let a = vec![1, 2, 3];
    let b = a.iter()
        .map(|x| *x);
}
