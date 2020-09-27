use crate::person_object::PersonBehavior;
use nalgebra_glm::{vec2, Vec2};
use tearchan_core::game::object::GameObject;
use tearchan_horde::action::action_manager::ActionManager;
use tearchan_horde::action_creator::action_creator_manager::ActionCreatorCommand;
use tearchan_horde::action_creator::action_creator_result::ActionCreatorResult;
use tearchan_horde::horde_plugin::HordePluginProvider;
use tearchan_horde::object::Object;

#[derive(Debug)]
pub enum HordeActionStore {
    Move { from: Vec2, to: Vec2 },
    Rotate { from: f32, to: f32 },
    Wait,
}

#[derive(Debug)]
pub enum HordeActionCreatorStore {
    MoveAndRotate { times: i32 },
    Wait { times: i32 },
}

#[derive(Default)]
pub struct HordeProvider {}

impl HordePluginProvider for HordeProvider {
    type ActionCommonStore = HordeActionStore;
    type ActionCreatorCommonStore = HordeActionCreatorStore;

    fn on_start_action(&mut self, store: &Self::ActionCommonStore, object: GameObject<dyn Object>) {
        let mut person_object = object.cast::<dyn PersonBehavior>().unwrap();
        match store {
            HordeActionStore::Move { from, .. } => {
                person_object.borrow_mut().set_position(from.clone_owned());
                person_object.borrow_mut().update_transform();
            }
            HordeActionStore::Rotate { from, .. } => {
                person_object.borrow_mut().set_rotation(*from);
                person_object.borrow_mut().update_transform();
            }
            _ => {}
        }
    }

    fn on_update_action(
        &mut self,
        store: &Self::ActionCommonStore,
        _current_time: u64,
        ratio: f32,
        object: GameObject<dyn Object>,
    ) {
        let mut person_object = object.cast::<dyn PersonBehavior>().unwrap();
        match store {
            HordeActionStore::Move { from, to } => {
                person_object
                    .borrow_mut()
                    .set_position(calc_position_from_to(from, to, ratio));
                person_object.borrow_mut().update_transform();
            }
            HordeActionStore::Rotate { from, to } => {
                person_object.borrow_mut().set_rotation((to - from) * ratio);
                person_object.borrow_mut().update_transform();
            }
            _ => {}
        }
    }

    fn on_end_action(&mut self, store: &Self::ActionCommonStore, object: GameObject<dyn Object>) {
        let mut person_object = object.cast::<dyn PersonBehavior>().unwrap();
        match store {
            HordeActionStore::Move { to, .. } => {
                person_object.borrow_mut().set_position(to.clone_owned());
                person_object.borrow_mut().update_transform();
            }
            HordeActionStore::Rotate { to, .. } => {
                person_object.borrow_mut().set_rotation(*to);
                person_object.borrow_mut().update_transform();
            }
            _ => {}
        }
    }

    fn run_action_creator(
        &mut self,
        action_manager: &mut ActionManager<Self::ActionCommonStore>,
        store: &mut Self::ActionCreatorCommonStore,
        object: &mut GameObject<dyn Object>,
    ) -> ActionCreatorResult<Self::ActionCreatorCommonStore> {
        match store {
            HordeActionCreatorStore::MoveAndRotate { times } => {
                if *times > 0 {
                    action_manager.run(
                        object.id(),
                        HordeActionStore::Move {
                            from: vec2(0.0f32, 0.0f32),
                            to: vec2(500.0f32, 500.0f32),
                        },
                        3000,
                    );
                    action_manager.run(
                        object.id(),
                        HordeActionStore::Rotate {
                            from: 0.0f32,
                            to: 360.0f32,
                        },
                        3000,
                    );
                    action_manager.run(
                        object.id(),
                        HordeActionStore::Move {
                            from: vec2(500.0f32, 500.0f32),
                            to: vec2(0.0f32, 0.0f32),
                        },
                        3000,
                    );
                    *times -= 1;
                }
            }
            HordeActionCreatorStore::Wait { times } => {
                if *times > 0 {
                    action_manager.run(object.id(), HordeActionStore::Wait, 5000);
                    *times -= 1;
                }
            }
        }
        ActionCreatorResult::Break
    }

    fn create_action_command(
        &mut self,
        priority: u32,
    ) -> ActionCreatorCommand<Self::ActionCreatorCommonStore> {
        match priority {
            0 => ActionCreatorCommand::Run {
                store: HordeActionCreatorStore::MoveAndRotate { times: 3 },
            },
            _ => ActionCreatorCommand::Run {
                store: HordeActionCreatorStore::Wait { times: 1 },
            },
        }
    }
}

pub fn calc_position_from_to(from: &Vec2, to: &Vec2, ratio: f32) -> Vec2 {
    vec2(
        from.x + (to.x - from.x) * ratio,
        from.y + (to.y - from.y) * ratio,
    )
}
