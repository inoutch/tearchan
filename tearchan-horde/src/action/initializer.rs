use crate::action::manager::ActionManagerData;
use crate::ActionController;
use tearchan_ecs::component::EntityId;

pub enum ActionInitializer<'a, T> {
    Controller(ActionController<'a, T>),
    Data(ActionManagerData<T>),
}

impl<'a, T> ActionInitializer<'a, T> {
    pub fn attach(&mut self, entity_id: EntityId) {
        match self {
            ActionInitializer::Controller(controller) => {
                controller.attach(entity_id);
            }
            ActionInitializer::Data(data) => {
                data.entity_ids.insert(entity_id);
            }
        }
    }

    pub fn detach(&mut self, entity_id: EntityId) {
        match self {
            ActionInitializer::Controller(controller) => {
                controller.detach(entity_id);
            }
            ActionInitializer::Data(data) => {
                data.entity_ids.remove(&entity_id);
            }
        }
    }
}

impl<'a, T> From<ActionController<'a, T>> for ActionInitializer<'a, T> {
    fn from(controller: ActionController<'a, T>) -> Self {
        ActionInitializer::Controller(controller)
    }
}
