use crate::game::{CreatePassageParams, CreatePlayerParams, Game};
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
use winit::event::WindowEvent;
use winit::window::WindowBuilder;

mod game;
mod renderer;
mod utils;

struct HordeScene {
    game: Game,
    job_manager: JobManager,
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

            for _ in 0..500 {
                game.create_player(CreatePlayerParams {
                    controller: &mut job_manager.action_controller(),
                    initial_position: vec2(
                        rng.gen_range(0..maze_width),
                        rng.gen_range(0..maze_height),
                    ),
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
            _ => {}
        }
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        let delta = (context.delta * 1000.0f32) as TimeMilliseconds;
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
