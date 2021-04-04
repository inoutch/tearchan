use std::any::Any;

pub struct OptAnyBox {
    inner: Option<Box<dyn Any>>,
}

impl OptAnyBox {
    pub fn new(inner: Option<Box<dyn Any>>) -> Self {
        OptAnyBox { inner }
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        let inner = match &self.inner {
            None => return None,
            Some(x) => x,
        };
        let inner: &T = match inner.downcast_ref() {
            None => return None,
            Some(x) => x,
        };
        Some(inner)
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let inner = match &mut self.inner {
            None => return None,
            Some(x) => x,
        };
        let inner: &mut T = match inner.downcast_mut() {
            None => return None,
            Some(x) => x,
        };
        Some(inner)
    }
}

#[cfg(test)]
mod test {
    use crate::any::OptAnyBox;

    struct Custom {
        value: i32,
    }
    #[test]
    fn test() {
        let any_box = OptAnyBox::new(Some(Box::new(Custom { value: 1 })));
        assert_eq!(any_box.get::<Custom>().unwrap().value, 1);
    }
}
