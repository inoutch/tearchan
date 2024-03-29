use crate::game::{CreatePassageParams, CreatePlayerParams, DirectionState, EntityType, Game};
use crate::renderer::Renderer;
use crate::utils::create_texture_view;
use maze_generator::prelude::{Coordinates, Direction, Generator};
use maze_generator::recursive_backtracking::RbGenerator;
use nalgebra_glm::vec2;
use rand::Rng;
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan_horde::action::manager::TimeMilliseconds;
use tearchan_horde::v2::job::manager::JobManager;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};
use winit::window::WindowBuilder;

mod game;
mod renderer;
mod utils;

struct HordeScene {
    game: Game,
    job_manager: JobManager<Game>,
}

impl HordeScene {
    fn factory() -> SceneFactory {
        |context, _| {
            let mut job_manager = JobManager::default();

            let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(0);
            let mut generator = RbGenerator::new(Some([1; 32]));
            let maze_width = 50;
            let maze_height = 50;
            let maze = generator.generate(maze_width, maze_height).unwrap();
            println!("[maze]:\n{:?}", maze);

            let mut game = Game::new(Renderer::new(context));

            game.player_id = game.create_cell(CreatePlayerParams {
                job_manager: &mut job_manager,
                initial_position: vec2(rng.gen_range(0..maze_width), rng.gen_range(0..maze_height)),
                entity_type: EntityType::Player,
            });

            for _ in 0..500 {
                game.create_cell(CreatePlayerParams {
                    job_manager: &mut job_manager,
                    initial_position: vec2(
                        rng.gen_range(0..maze_width),
                        rng.gen_range(0..maze_height),
                    ),
                    entity_type: EntityType::Enemy,
                });
            }

            for y in 0..maze_height {
                for x in 0..maze_width {
                    let field = maze.get_field(&Coordinates { x, y }).unwrap();
                    game.create_passage(CreatePassageParams {
                        initial_position: vec2(x, y),
                        directions: Direction::all()
                            .iter()
                            .filter(|direction| field.has_passage(*direction))
                            .cloned()
                            .collect(),
                    });
                }
            }

            Box::new(HordeScene { game, job_manager })
        }
    }
}

impl Scene for HordeScene {
    fn update(&mut self, context: &mut SceneContext, event: WindowEvent) -> SceneControlFlow {
        #[allow(clippy::single_match)]
        match event {
            WindowEvent::Resized(_size) => {
                self.game.renderer.resize(context);
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => match keycode {
                            VirtualKeyCode::W => self
                                .game
                                .go_player(&mut self.job_manager, DirectionState::Up),
                            VirtualKeyCode::A => self
                                .game
                                .go_player(&mut self.job_manager, DirectionState::Left),
                            VirtualKeyCode::S => self
                                .game
                                .go_player(&mut self.job_manager, DirectionState::Down),
                            VirtualKeyCode::D => self
                                .game
                                .go_player(&mut self.job_manager, DirectionState::Right),
                            VirtualKeyCode::Key0 => self.game.speed = 0.0f32,
                            VirtualKeyCode::Key1 => self.game.speed = 1.0f32,
                            VirtualKeyCode::Key2 => self.game.speed = 2.0f32,
                            VirtualKeyCode::Key3 => self.game.speed = 3.0f32,
                            VirtualKeyCode::Z => self.game.save_world(&self.job_manager),
                            VirtualKeyCode::X => self.game.load_world(&mut self.job_manager),
                            _ => {}
                        },
                        ElementState::Released => {}
                    }
                }
            }
            _ => {}
        }
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        self.game.restore(self.job_manager.current_tick());

        let delta = (context.delta * 1000.0f32 * self.game.speed) as TimeMilliseconds;
        self.job_manager.run(&mut self.game, delta);

        self.game.renderer.render(context);

        SceneControlFlow::None
    }
}

fn main() {
    let window_builder = WindowBuilder::new().with_title("Tearchan Horde Example");
    let startup_config = EngineStartupConfigBuilder::new()
        .window_builder(window_builder)
        .scene_factory(HordeScene::factory())
        .build();
    let engine = Engine::new(startup_config);
    engine.run();
}
