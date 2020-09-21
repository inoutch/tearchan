use crate::game::object::game_object_operator::GameObjectOperator;
use crate::game::object::{GameObject, GameObjectId};
use intertrait::CastFrom;
use std::cmp::Ordering;
use std::collections::HashMap;
use tearchan_utility::shared::Shared;

pub struct GameObjectManager<T: ?Sized>
where
    T: CastFrom,
{
    objects: Shared<HashMap<GameObjectId, GameObject<T>>>,
    sorted_object_ids: Shared<Vec<GameObjectId>>,
}

impl<T: ?Sized> GameObjectManager<T>
where
    T: CastFrom,
{
    pub fn new() -> GameObjectManager<T> {
        GameObjectManager {
            objects: Shared::new(HashMap::new()),
            sorted_object_ids: Shared::new(vec![]),
        }
    }

    #[inline]
    pub fn add(&mut self, object: GameObject<T>) {
        self.sorted_object_ids.borrow_mut().push(object.id());
        self.objects.borrow_mut().insert(object.id(), object);
    }

    #[inline]
    pub fn remove(&mut self, id: &GameObjectId) -> Option<GameObject<T>> {
        let p = self
            .sorted_object_ids
            .borrow()
            .iter()
            .position(|x| x == id)
            .expect("This id is not added");
        self.sorted_object_ids.borrow_mut().remove(p);
        self.objects.borrow_mut().remove(id)
    }

    pub fn find_by_id(&self, id: GameObjectId) -> Option<GameObject<T>> {
        self.objects.borrow().get(&id).map(|obj| obj.clone())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.objects.borrow().len()
    }

    #[inline]
    pub fn for_each_mut<F>(&mut self, callback: F)
    where
        F: Fn(&mut GameObject<T>),
    {
        let objects = self.sorted_object_ids.borrow_mut();
        for id in objects.iter() {
            callback(self.objects.borrow_mut().get_mut(&id).unwrap());
        }
    }

    pub fn create_operator(&self) -> GameObjectOperator<T> {
        GameObjectOperator::new(
            Shared::clone(&self.objects),
            Shared::clone(&self.sorted_object_ids),
        )
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&GameObject<T>, &GameObject<T>) -> Ordering,
    {
        let objects = self.objects.borrow_mut();
        self.sorted_object_ids.borrow_mut().sort_by(|a, b| {
            let obj_a = objects.get(a).unwrap();
            let obj_b = objects.get(b).unwrap();
            compare(obj_a, obj_b)
        });
    }
}
