use crate::game::game_plugin_command::GamePluginCommand;
use std::collections::VecDeque;
use tearchan_utility::shared::Shared;

pub struct GamePluginOperator {
    commands: Shared<VecDeque<GamePluginCommand>>,
}

impl GamePluginOperator {
    pub fn new(commands: Shared<VecDeque<GamePluginCommand>>) -> GamePluginOperator {
        GamePluginOperator { commands }
    }

    pub fn queue(&mut self, command: GamePluginCommand) {
        self.commands.borrow_mut().push_back(command);
    }
}
