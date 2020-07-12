use crate::core::graphic::batch::BatchContext;
use crate::extension::shared::{make_shared, Shared, WeakShared};
use crate::utility::change_notifier::ChangeNotifier;
use serde::export::Option::Some;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub struct BatchChangeManager<TObject> {
    targets: Shared<HashMap<*const BatchContext<TObject>, Rc<BatchContext<TObject>>>>,
}

impl<TObject> BatchChangeManager<TObject> {
    pub fn remove(&mut self, context: &Rc<BatchContext<TObject>>) {
        let key: *const BatchContext<TObject> = context.deref();
        self.targets.borrow_mut().remove(&key);
    }

    pub fn reset(&mut self) {
        self.targets.borrow_mut().clear();
    }

    pub fn create_notifier(
        &mut self,
        context: &Rc<BatchContext<TObject>>,
    ) -> BatchChangeNotifier<TObject> {
        BatchChangeNotifier {
            targets: Rc::downgrade(&self.targets),
            context: Rc::clone(context),
        }
    }

    pub fn targets(
        &mut self,
    ) -> &Shared<HashMap<*const BatchContext<TObject>, Rc<BatchContext<TObject>>>> {
        &self.targets
    }
}

impl<TObject> Default for BatchChangeManager<TObject> {
    fn default() -> Self {
        BatchChangeManager {
            targets: make_shared(HashMap::new()),
        }
    }
}

pub struct BatchChangeNotifier<TObject> {
    targets: WeakShared<HashMap<*const BatchContext<TObject>, Rc<BatchContext<TObject>>>>,
    context: Rc<BatchContext<TObject>>,
}

impl<TObject> ChangeNotifier for BatchChangeNotifier<TObject> {
    fn request_change(&mut self) {
        if let Some(targets) = self.targets.upgrade() {
            let key: *const BatchContext<TObject> = self.context.deref();
            if !targets.borrow().contains_key(&key) {
                targets.borrow_mut().insert(key, Rc::clone(&self.context));
            }
        }
    }
}
