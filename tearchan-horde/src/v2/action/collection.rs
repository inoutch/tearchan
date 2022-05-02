use crate::v2::action::manager::ActionSessionValidator;
use crate::v2::action::{ActionSessionId, ArcAction};
use crate::v2::Tick;
use std::any::{Any, TypeId};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use tearchan_ecs::component::EntityId;

pub struct ActionMeta {
    pub type_id: TypeId,
    pub entity_id: EntityId,
    pub session_id: ActionSessionId,
    pub tick: Tick,
}

pub struct ActionAnyVec {
    metas: Vec<ActionMeta>,
    actions: Box<dyn Any>,
}

impl ActionAnyVec {
    pub fn new<T: 'static>() -> Self {
        ActionAnyVec {
            metas: vec![],
            actions: Box::new(Vec::<ArcAction<T>>::new()),
        }
    }
}

impl ActionAnyVec {
    pub fn cast<T: 'static>(&self, validator: &ActionSessionValidator) -> Vec<&ArcAction<T>> {
        self.actions
            .downcast_ref::<Vec<ArcAction<T>>>()
            .expect("Invalid type")
            .iter()
            .zip(self.metas.iter())
            .filter_map(|(action, meta)| {
                if validator.validate(meta) {
                    Some(action)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn cast_cloned<T: 'static>(&self, validator: &ActionSessionValidator) -> Vec<ArcAction<T>> {
        self.actions
            .downcast_ref::<Vec<ArcAction<T>>>()
            .expect("Invalid type")
            .iter()
            .zip(self.metas.iter())
            .filter_map(|(action, meta)| {
                if validator.validate(meta) {
                    Some(action.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn push<T: 'static>(&mut self, action: ArcAction<T>, session_id: ActionSessionId, tick: Tick) {
        let type_id = TypeId::of::<T>();
        self.metas.push(ActionMeta {
            type_id,
            entity_id: action.entity_id,
            session_id,
            tick,
        });
        self.actions
            .downcast_mut::<Vec<ArcAction<T>>>()
            .expect("Invalid type")
            .push(action);
    }
}

#[derive(Default)]
pub struct TypedActionAnyMap {
    map: HashMap<TypeId, ActionAnyVec>,
}

impl TypedActionAnyMap {
    pub fn push<T>(&mut self, action: ArcAction<T>, session_id: ActionSessionId)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        let tick = action.tick().expect("Must have tick");
        self.map
            .entry(type_id)
            .or_insert_with(ActionAnyVec::new::<T>)
            .push(action, session_id, tick);
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

    pub fn iter(&self) -> Iter<'_, TypeId, ActionAnyVec> {
        self.map.iter()
    }
}

#[derive(Default)]
pub struct AnyActionVec {
    vec: Vec<Box<dyn Any>>,
}

impl AnyActionVec {
    pub fn cast<T: 'static>(&self) -> Vec<&ArcAction<T>> {
        self.vec
            .iter()
            .filter_map(|item| item.downcast_ref())
            .collect()
    }

    pub fn cast_cloned<T: 'static>(&self) -> Vec<ArcAction<T>> {
        self.vec
            .iter()
            .filter_map(|item| item.downcast_ref::<ArcAction<T>>())
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn push<T: 'static>(&mut self, item: T) {
        self.vec.push(Box::new(item));
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
    fn replace(&mut self, index: usize, item: Box<dyn Any>) {
        if let Some(x) = self.vec.get_mut(index) {
            *x = item
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<Box<dyn Any>> {
        self.vec.pop()
    }

    #[inline]
    fn push_as_raw(&mut self, raw: Box<dyn Any>) {
        self.vec.push(raw);
    }
}

#[derive(Default)]
pub struct TypedAnyActionMapGroupedByEntityId {
    map: HashMap<TypeId, AnyActionVec>,
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
        let vec = self
            .map
            .entry(type_id)
            .or_insert_with(AnyActionVec::default);
        let index = vec.len();
        vec.push(action);
        self.entities.insert((type_id, index), entity_id);
        self.indices.insert(entity_id, (type_id, index));
    }

    pub fn insert_with_type_id(
        &mut self,
        entity_id: EntityId,
        action: Box<dyn Any>,
        type_id: TypeId,
    ) {
        self.remove(entity_id);

        let collection = self
            .map
            .entry(type_id)
            .or_insert_with(AnyActionVec::default);
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
    fn test_typed_any_vec_grouped_by_entities() {
        let mut collection = TypedAnyActionMapGroupedByEntityId::default();

        collection.insert(
            1,
            Action {
                raw: Arc::new(MoveState),
                entity_id: 1,
                ty: ActionType::Start {
                    tick: 0,
                    start: 0,
                    end: 0,
                },
            },
        );

        collection.insert(
            2,
            Action {
                raw: Arc::new(JumpState),
                entity_id: 2,
                ty: ActionType::Start {
                    tick: 0,
                    start: 0,
                    end: 0,
                },
            },
        );
        collection.insert(
            2,
            Action {
                raw: Arc::new(JumpState),
                entity_id: 2,
                ty: ActionType::Start {
                    tick: 0,
                    start: 0,
                    end: 0,
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
