pub trait VecExt<T> {
    // TODO: Replace to https://github.com/rust-lang/rust/issues/40062
    fn remove_item_ext(&mut self, item: &T) -> Option<T>
    where
        T: PartialEq;
    fn remove_item_is<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnMut(&T) -> bool;
}

impl<T> VecExt<T> for Vec<T> {
    fn remove_item_ext(&mut self, item: &T) -> Option<T>
    where
        T: PartialEq,
    {
        let pos = match self.iter().position(|x| *x == *item) {
            Some(x) => x,
            None => return None,
        };
        Some(self.remove(pos))
    }

    fn remove_item_is<P>(&mut self, predicate: P) -> Option<T>
    where
        P: FnMut(&T) -> bool,
    {
        let pos = match self.iter().position(predicate) {
            Some(x) => x,
            None => return None,
        };
        Some(self.remove(pos))
    }
}

#[cfg(test)]
mod tests {
    use crate::extension::collection::VecExt;
    use std::borrow::Borrow;
    use std::rc::Rc;

    struct Struct {
        pub value: i32,
    }

    #[test]
    fn test_remove_item() {
        let mut v = vec![1, 3, 4, 6, 10];
        v.remove_item_ext(&3);
        v.remove_item_ext(&10);

        assert_eq!(v, vec![1, 4, 6]);
    }

    #[test]
    fn test_remove_item_is() {
        let i1 = Rc::new(Struct { value: 1 });
        let i2 = Rc::new(Struct { value: 3 });
        let i3 = Rc::new(Struct { value: 6 });
        let i4 = Rc::new(Struct { value: 7 });
        let target = Rc::clone(&i2);
        let mut v = vec![i1, i2, i3, i4];

        let t1 = target.borrow() as *const Struct;
        v.remove_item_is(move |x| {
            let y = x.borrow() as *const Struct;
            std::ptr::eq(y, t1)
        });
    }
}
