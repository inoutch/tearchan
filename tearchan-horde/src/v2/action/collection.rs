use crate::v2::action::manager::ActionSessionValidator;
use crate::v2::action::{ActionSessionId, ArcAction};
use crate::v2::Tick;
use std::any::{Any, TypeId};
use std::collections::hash_map::Iter;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tearchan_ecs::component::EntityId;

pub struct ActionMeta {
    pub entity_id: EntityId,
    pub session_id: ActionSessionId,
    pub tick: Tick,
}

#[derive(Default)]
pub struct AnyVec {
    vec: Vec<Arc<Box<dyn Any>>>,
}

impl AnyVec {
    pub fn cast<T: 'static>(&self) -> Vec<&T> {
        self.vec
            .iter()
            .filter_map(|item| item.downcast_ref())
            .collect()
    }

    pub fn cast_cloned<T: 'static>(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.vec
            .iter()
            .filter_map(|item| item.downcast_ref::<T>())
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn push<T: 'static>(&mut self, item: T) {
        self.vec.push(Arc::new(Box::new(item)));
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn replace(&mut self, index: usize, item: Arc<Box<dyn Any>>) {
        if let Some(x) = self.vec.get_mut(index) {
            *x = item
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<Arc<Box<dyn Any>>> {
        self.vec.pop()
    }

    #[inline]
    fn push_as_raw(&mut self, raw: Arc<Box<dyn Any>>) {
        self.vec.push(raw);
    }
}

#[derive(Default)]
pub struct TypedAnyActionMapGroupedByEntityId {
    map: HashMap<TypeId, AnyVec>,
    indices: HashMap<EntityId, (TypeId, usize)>,
    entities: HashMap<(TypeId, usize), EntityId>,
}

impl TypedAnyActionMapGroupedByEntityId {
    pub fn get<T>(&self) -> Option<Vec<&ArcAction<T>>>
    where
        T: 'static,
    {
        Some(self.map.get(&TypeId::of::<T>())?.cast())
    }

    pub fn get_cloned<T>(&self) -> Option<Vec<ArcAction<T>>>
    where
        T: 'static,
    {
        Some(self.map.get(&TypeId::of::<T>())?.cast_cloned())
    }

    pub fn insert<T>(&mut self, entity_id: EntityId, action: ArcAction<T>)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();

        self.remove(entity_id);
        let vec = self.map.entry(type_id).or_insert_with(AnyVec::default);
        let index = vec.len();
        vec.push(action);
        self.entities.insert((type_id, index), entity_id);
        self.indices.insert(entity_id, (type_id, index));
    }

    pub fn insert_as_raw(
        &mut self,
        action: Arc<Box<dyn Any>>,
        type_id: TypeId,
        entity_id: EntityId,
    ) {
        self.remove(entity_id);

        let collection = self.map.entry(type_id).or_insert_with(AnyVec::default);
        let index = collection.len();
        collection.push_as_raw(action);
        self.entities.insert((type_id, index), entity_id);
        self.indices.insert(entity_id, (type_id, index));
    }

    pub fn remove(&mut self, entity_id: EntityId) {
        if let Some((type_id, index)) = self.indices.remove(&entity_id) {
            self.entities.remove(&(type_id, index));

            let collection = self.map.get_mut(&type_id).unwrap();
            let last = collection.len() - 1;
            let last_item = collection.pop().unwrap();
            if index != last {
                let last_entity_id = self.entities.remove(&(type_id, last)).unwrap();
                self.indices.remove(&last_entity_id);

                collection.replace(index, last_item);
                self.entities.insert((type_id, index), last_entity_id);
                self.indices.insert(last_entity_id, (type_id, index));
            } else if index == 0 {
                self.map.remove(&type_id);
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, TypeId, AnyVec> {
        self.map.iter()
    }
}

#[derive(Default)]
pub struct AnyActionVec {
    vec: Vec<(ActionMeta, Arc<Box<dyn Any>>)>,
}

impl AnyActionVec {
    pub fn cast<T: 'static>(&self, validator: &ActionSessionValidator) -> Vec<&ArcAction<T>> {
        self.vec
            .iter()
            .filter_map(|(meta, action)| {
                if validator.validate(meta) {
                    action.downcast_ref()
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn cast_cloned<T: 'static>(&self, validator: &ActionSessionValidator) -> Vec<ArcAction<T>> {
        self.vec
            .iter()
            .filter_map(|(meta, item)| {
                if validator.validate(meta) {
                    item.downcast_ref()
                } else {
                    None
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn push<T: 'static>(
        &mut self,
        action: ArcAction<T>,
        session_id: ActionSessionId,
        tick: Tick,
    ) {
        self.vec.push((
            ActionMeta {
                entity_id: action.entity_id,
                session_id,
                tick,
            },
            Arc::new(Box::new(action)),
        ));
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn replace(&mut self, index: usize, item: (ActionMeta, Arc<Box<dyn Any>>)) {
        if let Some(x) = self.vec.get_mut(index) {
            *x = item;
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<(ActionMeta, Arc<Box<dyn Any>>)> {
        self.vec.pop()
    }

    fn push_as_raw(
        &mut self,
        action: Arc<Box<dyn Any>>,
        entity_id: EntityId,
        session_id: ActionSessionId,
        tick: Tick,
    ) {
        self.vec.push((
            ActionMeta {
                entity_id,
                session_id,
                tick,
            },
            action,
        ));
    }
}

#[derive(Default)]
pub struct TypedAnyActionMap {
    map: HashMap<TypeId, AnyActionVec>,
}

impl TypedAnyActionMap {
    pub fn push<T>(&mut self, action: ArcAction<T>, session_id: ActionSessionId)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let tick = action.tick().expect("Must have tick");
        self.map
            .entry(type_id)
            .or_insert_with(AnyActionVec::default)
            .push(action, session_id, tick);
    }

    pub fn push_as_raw(
        &mut self,
        type_id: TypeId,
        action: Arc<Box<dyn Any>>,
        entity_id: EntityId,
        session_id: ActionSessionId,
        tick: Tick,
    ) {
        self.map
            .entry(type_id)
            .or_insert_with(AnyActionVec::default)
            .push_as_raw(action, entity_id, session_id, tick);
    }

    pub fn get<T>(&self, validator: &ActionSessionValidator) -> Option<Vec<&ArcAction<T>>>
    where
        T: 'static,
    {
        let vec = self.map.get(&TypeId::of::<T>())?.cast(validator);
        if vec.is_empty() {
            return None;
        }
        Some(vec)
    }

    pub fn get_cloned<T>(&self, validator: &ActionSessionValidator) -> Option<Vec<ArcAction<T>>>
    where
        T: 'static,
    {
        let vec = self.map.get(&TypeId::of::<T>())?.cast_cloned(validator);
        if vec.is_empty() {
            return None;
        }
        Some(vec)
    }

    pub fn iter(&self) -> Iter<'_, TypeId, AnyActionVec> {
        self.map.iter()
    }
}

#[derive(Default)]
pub struct TypedCloneableAnyActionMapGroupedByEntityId {
    map: HashMap<TypeId, AnyActionVec>,
    indices: BTreeMap<EntityId, (TypeId, usize)>,
    entities: HashMap<(TypeId, usize), EntityId>,
}

impl TypedCloneableAnyActionMapGroupedByEntityId {
    pub fn insert<T>(&mut self, action: ArcAction<T>, session_id: ActionSessionId, tick: Tick)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let entity_id = action.entity_id();

        self.remove(entity_id);
        let vec = self
            .map
            .entry(type_id)
            .or_insert_with(AnyActionVec::default);
        let index = vec.len();
        vec.push(action, session_id, tick);
        self.entities.insert((type_id, index), entity_id);
        self.indices.insert(entity_id, (type_id, index));
    }

    pub fn insert_as_raw(
        &mut self,
        action: Arc<Box<dyn Any>>,
        type_id: TypeId,
        entity_id: EntityId,
        session_id: ActionSessionId,
        tick: Tick,
    ) {
        self.remove(entity_id);

        let collection = self
            .map
            .entry(type_id)
            .or_insert_with(AnyActionVec::default);
        let index = collection.len();
        collection.push_as_raw(action, entity_id, session_id, tick);
        self.entities.insert((type_id, index), entity_id);
        self.indices.insert(entity_id, (type_id, index));
    }

    pub fn remove(&mut self, entity_id: EntityId) {
        if let Some((type_id, index)) = self.indices.remove(&entity_id) {
            self.entities.remove(&(type_id, index));

            let collection = self.map.get_mut(&type_id).unwrap();
            let last = collection.len() - 1;
            let last_item = collection.pop().unwrap();
            if index != last {
                let last_entity_id = self.entities.remove(&(type_id, last)).unwrap();
                self.indices.remove(&last_entity_id);

                collection.replace(index, last_item);
                self.entities.insert((type_id, index), last_entity_id);
                self.indices.insert(last_entity_id, (type_id, index));
            } else if index == 0 {
                self.map.remove(&type_id);
            }
        }
    }

    pub fn push_actions_to(&self, map: &mut TypedAnyActionMap, current_tick: Tick) {
        for (_entity_id, (type_id, index)) in self.indices.iter() {
            let vec = match self.map.get(type_id) {
                None => continue,
                Some(vec) => vec,
            };
            let (meta, action) = &vec.vec[*index];
            if meta.tick != current_tick {
                map.push_as_raw(
                    *type_id,
                    Arc::clone(action),
                    meta.entity_id,
                    meta.session_id,
                    meta.tick,
                );
            }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, TypeId, AnyActionVec> {
        self.map.iter()
    }
}

#[cfg(test)]
mod test {
    use crate::v2::action::collection::TypedAnyActionMapGroupedByEntityId;
    use crate::v2::action::{Action, ActionType};
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct MoveState;

    #[derive(Clone, Debug)]
    struct JumpState;

    #[test]
    fn test_typed_any_map_grouped_by_entities() {
        let mut collection = TypedAnyActionMapGroupedByEntityId::default();

        collection.insert(
            1,
            Action {
                raw: Arc::new(MoveState),
                entity_id: 1,
                ty: ActionType::Start {
                    start: 0,
                    end: 0,
                    each: false,
                },
            },
        );

        collection.insert(
            2,
            Action {
                raw: Arc::new(JumpState),
                entity_id: 2,
                ty: ActionType::Start {
                    start: 0,
                    end: 0,
                    each: false,
                },
            },
        );
        collection.insert(
            2,
            Action {
                raw: Arc::new(JumpState),
                entity_id: 2,
                ty: ActionType::Start {
                    start: 0,
                    end: 0,
                    each: false,
                },
            },
        );

        insta::assert_debug_snapshot!(collection.get::<MoveState>());
        insta::assert_debug_snapshot!(collection.get::<JumpState>());

        collection.remove(2);

        insta::assert_debug_snapshot!(collection.get::<MoveState>()); // should have one
        insta::assert_debug_snapshot!(collection.get::<JumpState>()); // should be empty

        collection.remove(1);

        insta::assert_debug_snapshot!(collection.get::<MoveState>()); // should be empty

        assert_eq!(collection.entities.len(), 0);
        assert_eq!(collection.indices.len(), 0);
        assert_eq!(collection.map.len(), 0);
    }
}
