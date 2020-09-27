use crate::action_creator::action_creator_manager::ActionCreatorCommand;

#[derive(Debug)]
pub enum ActionCreatorResult<TActionCreatorCommonStore> {
    Continue {
        command: ActionCreatorCommand<TActionCreatorCommonStore>,
    },
    Break,
}
