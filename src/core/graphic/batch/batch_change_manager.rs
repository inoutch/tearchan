use crate::core::graphic::batch::batch_object_bundle::BatchObjectBundle;
use crate::extension::shared::{make_shared, Shared, WeakShared};
use crate::utility::change_notifier::ChangeNotifier;
use serde::export::Option::Some;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub struct BatchChangeManager<TObject> {
    targets: Shared<HashMap<*const BatchObjectBundle<TObject>, Rc<BatchObjectBundle<TObject>>>>,
}

impl<TObject> BatchChangeManager<TObject> {
    pub fn remove(&mut self, bundle: &Rc<BatchObjectBundle<TObject>>) {
        let key: *const BatchObjectBundle<TObject> = bundle.deref();
        self.targets.borrow_mut().remove(&key);
    }

    pub fn reset(&mut self) {
        self.targets.borrow_mut().clear();
    }

    pub fn create_notifier(
        &mut self,
        bundle: &Rc<BatchObjectBundle<TObject>>,
    ) -> BatchChangeNotifier<TObject> {
        BatchChangeNotifier {
            targets: Rc::downgrade(&self.targets),
            bundle: Rc::clone(bundle),
        }
    }

    pub fn targets(
        &mut self,
    ) -> &Shared<HashMap<*const BatchObjectBundle<TObject>, Rc<BatchObjectBundle<TObject>>>> {
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
    targets: WeakShared<HashMap<*const BatchObjectBundle<TObject>, Rc<BatchObjectBundle<TObject>>>>,
    bundle: Rc<BatchObjectBundle<TObject>>,
}

impl<TObject> ChangeNotifier for BatchChangeNotifier<TObject> {
    fn request_change(&mut self) {
        if let Some(targets) = self.targets.upgrade() {
            let key: *const BatchObjectBundle<TObject> = self.bundle.deref();
            if !targets.borrow().contains_key(&key) {
                targets.borrow_mut().insert(key, Rc::clone(&self.bundle));
            }
        }
    }
}
