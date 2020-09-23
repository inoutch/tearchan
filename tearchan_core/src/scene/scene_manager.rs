use crate::game::game_context::GameContext;
use crate::game::object::game_object_base::GameObjectBase;
use crate::game::object::game_object_manager::GameObjectManager;
use crate::scene::scene_context::SceneContext;
use crate::scene::scene_factory::{SceneFactory, SceneOption};
use crate::scene::scene_result::SceneResult;
use crate::scene::Scene;
use std::option::Option::Some;
use winit::event::WindowEvent;
use crate::game::game_plugin_manager::GamePluginManager;

pub struct SceneManager {
    current_scene: Box<dyn Scene>,
    scene_factory: Option<(SceneFactory, Option<Box<dyn SceneOption>>)>,
    object_manager: GameObjectManager<dyn GameObjectBase>,
}

impl SceneManager {
    pub fn new(scene_creator: SceneFactory) -> SceneManager {
        SceneManager {
            current_scene: Box::new(DummyScene {}),
            scene_factory: Some((scene_creator, None)),
            object_manager: GameObjectManager::new(),
        }
    }

    pub fn event(&mut self, _event: &WindowEvent) {
        /*match event {
            WindowEvent::Resized(_) => {}
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {}
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::ReceivedCharacter(character) => {
                self.current_scene.on_character(character);
            }
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput { input, .. } => match input.state {
                ElementState::Pressed => {
                    self.current_scene.on_key_down(input);
                }
                ElementState::Released => {
                    self.current_scene.on_key_up(input);
                }
            },
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::CursorMoved { position, .. } => {
                let prev_location = vec2(position.x as u32, position.y as u32);
                if prev_location != self.cursor_location {
                    if let TouchPhase::Started = self.cursor_phase {
                        let touch = Touch {
                            id: 0,
                            location: prev_location.to_owned(),
                            phase: TouchPhase::Moved,
                        };
                        self.current_scene.on_touch_move(&touch);
                    }
                }
                self.cursor_location = prev_location;
            }
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::MouseInput { button, state, .. } => {
                if let MouseButton::Left = button {
                    match state {
                        ElementState::Pressed => {
                            if let TouchPhase::Ended = self.cursor_phase {
                                let touch = Touch {
                                    id: 0,
                                    location: self.cursor_location.to_owned(),
                                    phase: TouchPhase::Started,
                                };
                                self.cursor_phase = TouchPhase::Started;
                                self.current_scene.on_touch_start(&touch);
                            }
                        }
                        ElementState::Released => {
                            let touch = Touch {
                                id: 0,
                                location: self.cursor_location.to_owned(),
                                phase: TouchPhase::Ended,
                            };
                            self.cursor_phase = TouchPhase::Ended;
                            self.current_scene.on_touch_end(&touch);
                        }
                    }
                }
            }
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(touch) => {
                let touch = Touch {
                    id: touch.id,
                    location: vec2(touch.location.x as u32, touch.location.y as u32),
                    phase: touch.phase,
                };
                match touch.phase {
                    TouchPhase::Started => {
                        self.current_scene.on_touch_start(&touch);
                    }
                    TouchPhase::Moved => {
                        self.current_scene.on_touch_move(&touch);
                    }
                    TouchPhase::Ended => {
                        self.current_scene.on_touch_end(&touch);
                    }
                    TouchPhase::Cancelled => {
                        self.current_scene.on_touch_cancel(&touch);
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {}
            WindowEvent::ThemeChanged(_) => {}
        }*/
    }

    pub fn on_update(&mut self, context: &mut GameContext, plugin_manager: &mut GamePluginManager) {
        let mut scene_context = SceneContext::new(context, plugin_manager, &mut self.object_manager);
        if let Some((scene_factory, options)) = std::mem::replace(&mut self.scene_factory, None) {
            self.current_scene = scene_factory(&mut scene_context, options);
            self.scene_factory = None;
        }

        match self.current_scene.update(&mut scene_context) {
            SceneResult::Exit => {}
            SceneResult::TransitScene {
                scene_factory,
                option,
            } => {
                self.scene_factory = Some((scene_factory, option));
            }
            _ => {}
        }
    }
}

pub struct DummyScene;

impl Scene for DummyScene {
    fn update(&mut self, _scene_context: &mut SceneContext) -> SceneResult {
        SceneResult::None
    }
}
