use tearchan_core::game::object::game_object_base::GameObjectBase;

pub mod object_error;
pub mod object_factory;
pub mod object_manager;
pub mod object_store;

pub trait Object: GameObjectBase {}
