use crate::core::file::File;
use crate::core::graphic::hal::backend::Graphics;
use crate::core::graphic::hal::renderer::ResizeContext;
use crate::core::scene::scene_base::SceneBase;
use crate::core::scene::scene_context::{SceneContext, SceneContextCommand, SceneOption};
use crate::core::scene::scene_creator::SceneCreator;
use crate::core::scene::touch::Touch;
use nalgebra_glm::{vec2, TVec2};
use winit::event::{ElementState, KeyboardInput, MouseButton, TouchPhase, WindowEvent};

pub struct SceneManager {
    current_scene: Box<dyn SceneBase>,
    scene_creator: Option<(SceneCreator, Option<Box<dyn SceneOption>>)>,
    commands: Vec<SceneContextCommand>,
    cursor_location: TVec2<u32>,
    cursor_phase: TouchPhase,
}

impl SceneManager {
    pub fn new(scene_creator: SceneCreator) -> SceneManager {
        SceneManager {
            current_scene: Box::new(DummyScene {}),
            scene_creator: Some((scene_creator, None)),
            commands: vec![],
            cursor_location: vec2(0, 0),
            cursor_phase: TouchPhase::Ended,
        }
    }

    pub fn render(&mut self, delta: f32, graphics: &mut Graphics, file: &mut File) {
        loop {
            let command = match self.commands.pop() {
                None => break,
                Some(command) => command,
            };
            match command {
                SceneContextCommand::TransitScene {
                    scene_creator,
                    option,
                } => {
                    self.scene_creator = Some((scene_creator, option));
                }
            }
        }

        let mut scene_context = SceneContext::new(graphics, file, &mut self.commands);
        if let Some((scene_creator, options)) = std::mem::replace(&mut self.scene_creator, None) {
            self.current_scene = scene_creator(&mut scene_context, options);
            self.scene_creator = None;
        }

        self.current_scene.update(&mut scene_context, delta);
    }

    pub fn resize(&mut self, context: &mut ResizeContext) {
        self.current_scene.on_resize(context);
    }

    pub fn event(&mut self, event: &WindowEvent) {
        match event {
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
        }
    }
}

pub struct DummyScene;

impl SceneBase for DummyScene {
    fn update(&mut self, _scene_context: &mut SceneContext, _delta: f32) {}

    fn on_touch_start(&mut self, _touch: &Touch) {}

    fn on_touch_end(&mut self, _touch: &Touch) {}

    fn on_touch_move(&mut self, _touch: &Touch) {}

    fn on_touch_cancel(&mut self, _touch: &Touch) {}

    fn on_key_down(&mut self, _input: &KeyboardInput) {}

    fn on_key_up(&mut self, _input: &KeyboardInput) {}

    fn on_resize(&mut self, _context: &mut ResizeContext) {}
}
