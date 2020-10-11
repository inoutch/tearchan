use crate::game::game_plugin::GamePlugin;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::game_object_manager::GameObjectManager;
use crate::game::object::GameObject;
use crate::ui::ui_object::UIObject;
use crate::ui::ui_touch::UITouch;
use nalgebra_glm::{vec2, TVec2};
use std::collections::HashMap;
use winit::event::{ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent};

pub struct UIManager {
    cursor_location: TVec2<u32>,
    cursor_phase: TouchPhase,
    touch_indices: HashMap<u64, u64>, // touch id, index
    touch_index: u64,
    object_manager: GameObjectManager<dyn UIObject>,
}

impl UIManager {
    pub fn new() -> Self {
        UIManager {
            cursor_location: vec2(0, 0),
            cursor_phase: TouchPhase::Ended,
            touch_indices: HashMap::new(),
            touch_index: 0,
            object_manager: GameObjectManager::new(),
        }
    }
}

impl UIManager {
    fn on_key_up(&mut self, input: &KeyboardInput) {
        self.object_manager.for_each_mut(|object| {
            object.borrow_mut().on_key_up(input);
        });
    }

    fn on_key_down(&mut self, input: &KeyboardInput) {
        self.object_manager.for_each_mut(|object| {
            object.borrow_mut().on_key_down(input);
        });
    }

    fn on_touch_start(&mut self, touch: &UITouch) {
        let index = self.touch_index;
        self.touch_indices.insert(touch.id, self.touch_index);
        self.touch_index += 1;

        self.object_manager.for_each_mut(|object| {
            object.borrow_mut().on_touch_start(index, touch);
        });
    }

    fn on_touch_move(&mut self, touch: &UITouch) {
        let index = match self.touch_indices.get(&touch.id) {
            Some(index) => *index,
            None => return,
        };

        self.object_manager.for_each_mut(|object| {
            object.borrow_mut().on_touch_move(index, touch);
        });
    }

    fn on_touch_end(&mut self, touch: &UITouch) {
        let index = match self.touch_indices.remove(&touch.id) {
            Some(index) => index,
            None => return,
        };
        if self.touch_indices.is_empty() {
            self.touch_index = 0;
        }

        self.object_manager.for_each_mut(|object| {
            object.borrow_mut().on_touch_end(index, touch);
        });
    }

    fn on_touch_cancel(&mut self, touch: &UITouch) {
        let index = match self.touch_indices.remove(&touch.id) {
            Some(index) => index,
            None => return,
        };
        if self.touch_indices.is_empty() {
            self.touch_index = 0;
        }

        self.object_manager.for_each_mut(|object| {
            object.borrow_mut().on_touch_cancel(index, touch);
        });
    }
}

impl GamePlugin for UIManager {
    fn on_add(&mut self, _game_object: &GameObject<dyn GameObjectBase>) {
        // if let Some(ui_object) = GameObject::new(game_object.clone_inner_object()) {
        //     self.object_manager.add(ui_object);
        //     self.object_manager
        //         .sort_by(|a, b| a.borrow().z_index().cmp(&b.borrow().z_index()))
        // }
    }

    fn on_remove(&mut self, game_object: &GameObject<dyn GameObjectBase>) {
        self.object_manager.remove(&game_object.id());
    }

    fn on_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => match input.state {
                ElementState::Pressed => {
                    self.on_key_down(input);
                }
                ElementState::Released => {
                    self.on_key_up(input);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let prev_location = vec2(position.x as u32, position.y as u32);
                if prev_location != self.cursor_location {
                    if let TouchPhase::Started = self.cursor_phase {
                        let touch = UITouch {
                            id: 0,
                            location: prev_location.clone_owned(),
                            phase: TouchPhase::Moved,
                        };
                        self.on_touch_move(&touch);
                    }
                }
                self.cursor_location = prev_location;
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if let MouseButton::Left = button {
                    match state {
                        ElementState::Pressed => {
                            if let TouchPhase::Ended = self.cursor_phase {
                                let touch = UITouch {
                                    id: 0,
                                    location: self.cursor_location.clone_owned(),
                                    phase: TouchPhase::Started,
                                };
                                self.cursor_phase = TouchPhase::Started;
                                self.on_touch_start(&touch);
                            }
                        }
                        ElementState::Released => {
                            let touch = UITouch {
                                id: 0,
                                location: self.cursor_location.clone_owned(),
                                phase: TouchPhase::Ended,
                            };
                            self.cursor_phase = TouchPhase::Ended;
                            self.on_touch_end(&touch);
                        }
                    }
                }
            }
            WindowEvent::Touch(touch) => {
                let touch = UITouch {
                    id: touch.id,
                    location: vec2(touch.location.x as u32, touch.location.y as u32),
                    phase: touch.phase,
                };
                match touch.phase {
                    TouchPhase::Started => {
                        self.on_touch_start(&touch);
                    }
                    TouchPhase::Moved => {
                        self.on_touch_move(&touch);
                    }
                    TouchPhase::Ended => {
                        self.on_touch_end(&touch);
                    }
                    TouchPhase::Cancelled => {
                        self.on_touch_cancel(&touch);
                    }
                }
            }
            _ => {}
        }
    }
}
