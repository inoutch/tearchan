use crate::renderer::Renderer;
use crate::utils::{calc_center_from_scaled_position, calc_position_from_ratio};
use maze_generator::prelude::Direction;
use nalgebra_glm::{vec2, vec3, TVec2, Vec2, Vec3};
use rand::Rng;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use tearchan::util::array_2d::Array2D;
use tearchan::util::thread::ThreadPool;
use tearchan_ecs::component::group_sync::ComponentGroupSync;
use tearchan_ecs::component::resource_sync::ResourceSync;
use tearchan_ecs::component::EntityId;
use tearchan_ecs::entity::manager::EntityManager;
use tearchan_horde::action::manager::TimeMilliseconds;
use tearchan_horde::v2::action::collection::{
    TypedActionAnyMap, TypedAnyActionMapGroupedByEntityId,
};
use tearchan_horde::v2::action::manager::{ActionController, ActionSessionValidator};
use tearchan_horde::v2::action::{ActionType, ArcAction};
use tearchan_horde::v2::job::HordeInterface;
use tearchan_horde::v2::serde::{Deserialize, Serialize};
use tearchan_horde::v2::{calc_ratio_f32_from_ms, define_actions};

enum Command {
    UpdateRenderSpritePosition(EntityId, Vec2),
    UpdateRenderSpriteColor(EntityId, Vec3),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WalkState {
    from: TVec2<i32>,
    to: TVec2<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WaitState;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChangeColorState {
    from: Vec3,
    to: Vec3,
}

define_actions!(
    HordeAction,
    (Walk, WalkState),
    (Wait, WaitState),
    (ChangeColor, ChangeColorState)
);

pub enum HordeJob {
    Wander,
    Wait(TimeMilliseconds),
}

struct PositionData(Vec2);

struct ScaledPositionData(TVec2<i32>);

struct ColorData(Vec3);

struct PathData {
    _from: TVec2<i32>,
    to: TVec2<i32>,
}

trait MapperTrait {
    fn get_cloned<T>(&self) -> Option<Vec<ArcAction<T>>>
    where
        T: 'static;

    fn time(&self) -> TimeMilliseconds {
        0
    }
}

pub struct Game {
    pool: ThreadPool,
    entity_manager: EntityManager,
    // components
    positions: ComponentGroupSync<PositionData>,
    scaled_positions: ComponentGroupSync<ScaledPositionData>,
    colors: ComponentGroupSync<ColorData>,
    paths: ComponentGroupSync<Vec<PathData>>,
    // runtime
    passages: Array2D<EntityId>,
    // renderers
    pub renderer: Renderer,
}

impl Game {
    pub fn new(renderer: Renderer) -> Game {
        Game {
            pool: Default::default(),
            entity_manager: Default::default(),
            positions: Default::default(),
            scaled_positions: Default::default(),
            colors: Default::default(),
            paths: Default::default(),
            passages: Default::default(),
            renderer,
        }
    }

    fn run_action<T>(&mut self, map: T)
    where
        T: MapperTrait,
    {
        // Use for transaction
        let mut rsync = ResourceSync::default();
        let (sender, receiver) = channel::<Command>();

        // Systems
        // Walk system
        if let Some(walk_actions) = map.get_cloned::<WalkState>() {
            let walk_actions = Arc::new(walk_actions);
            {
                let rsync_child = rsync.child();
                let mut positions = self.positions.write();
                let walk_actions = Arc::clone(&walk_actions);
                let sender = Sender::clone(&sender);
                let time = map.time();

                self.pool.execute(move || {
                    rsync_child.begin();
                    let mut positions = positions.get_mut();
                    rsync_child.end();

                    for walk_action in walk_actions.iter() {
                        let state = walk_action.raw();
                        let position = positions.get_mut(walk_action.entity_id());
                        if let Some(position) = position {
                            match walk_action.ty() {
                                ActionType::Start { .. } => {
                                    position.0 = calc_center_from_scaled_position(&state.from);
                                    sender
                                        .send(Command::UpdateRenderSpritePosition(
                                            walk_action.entity_id(),
                                            position.0.clone_owned(),
                                        ))
                                        .unwrap();
                                }
                                ActionType::Update { start, end } => {
                                    let ratio = calc_ratio_f32_from_ms(*start, *end, time);
                                    let from = calc_center_from_scaled_position(&state.from);
                                    let to = calc_center_from_scaled_position(&state.to);
                                    position.0 = calc_position_from_ratio(&from, &to, ratio);
                                    sender
                                        .send(Command::UpdateRenderSpritePosition(
                                            walk_action.entity_id(),
                                            position.0.clone_owned(),
                                        ))
                                        .unwrap();
                                }
                                ActionType::End { .. } => {
                                    position.0 = calc_center_from_scaled_position(&state.to);
                                    sender
                                        .send(Command::UpdateRenderSpritePosition(
                                            walk_action.entity_id(),
                                            position.0.clone_owned(),
                                        ))
                                        .unwrap();
                                }
                            }
                        }
                    }
                });
            }
            {
                let rsync_child = rsync.child();
                let mut scaled_positions = self.scaled_positions.write();
                let walk_actions = Arc::clone(&walk_actions);

                self.pool.execute(move || {
                    rsync_child.begin();
                    let mut scaled_positions = scaled_positions.get_mut();
                    rsync_child.end();

                    for walk_action in walk_actions.iter() {
                        let state = walk_action.raw();
                        let scaled_position = scaled_positions.get_mut(walk_action.entity_id());
                        if let Some(scaled_position) = scaled_position {
                            match walk_action.ty() {
                                ActionType::Start { .. } => {
                                    scaled_position.0 = state.from.clone_owned();
                                }
                                ActionType::Update { .. } => {}
                                ActionType::End { .. } => {
                                    scaled_position.0 = state.to.clone_owned();
                                }
                            }
                        }
                    }
                });
            }
            {
                let rsync_child = rsync.child();
                let mut colors = self.colors.write();
                let sender = Sender::clone(&sender);

                self.pool.execute(move || {
                    rsync_child.begin();
                    let mut colors = colors.get_mut();
                    rsync_child.end();

                    for walk_action in walk_actions.iter() {
                        let _state = walk_action.raw();
                        let color = colors.get_mut(walk_action.entity_id());
                        if let Some(color) = color {
                            match walk_action.ty() {
                                ActionType::Start { .. } => {
                                    color.0 = vec3(0.0f32, 1.0f32, 0.0f32);
                                    sender
                                        .send(Command::UpdateRenderSpriteColor(
                                            walk_action.entity_id(),
                                            color.0.clone_owned(),
                                        ))
                                        .unwrap();
                                }
                                ActionType::Update { .. } => {}
                                ActionType::End { .. } => {}
                            }
                        }
                    }
                });
            }
        }
        if let Some(wait_actions) = map.get_cloned::<WaitState>() {
            let rsync_child = rsync.child();
            let mut colors = self.colors.write();
            let sender = Sender::clone(&sender);

            self.pool.execute(move || {
                rsync_child.begin();
                let mut colors = colors.get_mut();
                rsync_child.end();

                for wait_action in wait_actions.iter() {
                    let _state = wait_action.raw();
                    let color = colors.get_mut(wait_action.entity_id());
                    if let Some(color) = color {
                        match wait_action.ty() {
                            ActionType::Start { .. } => {
                                color.0 = vec3(1.0f32, 0.0f32, 0.0f32);
                                sender
                                    .send(Command::UpdateRenderSpriteColor(
                                        wait_action.entity_id(),
                                        color.0.clone_owned(),
                                    ))
                                    .unwrap();
                            }
                            ActionType::Update { .. } => {}
                            ActionType::End { .. } => {}
                        }
                    }
                }
            });
        }

        rsync.join();
        self.pool.join();

        while let Ok(command) = receiver.try_recv() {
            match command {
                Command::UpdateRenderSpritePosition(entity_id, position) => {
                    self.renderer.update_sprite_position(entity_id, &position);
                }
                Command::UpdateRenderSpriteColor(entity_id, color) => {
                    self.renderer.update_sprite_color(entity_id, &color);
                }
            }
        }
    }
}

impl HordeInterface for Game {
    type Job = HordeJob;

    fn on_change_tick(&mut self, map: &TypedActionAnyMap, validator: &ActionSessionValidator) {
        struct Mapper<'a> {
            map: &'a TypedActionAnyMap,
            validator: &'a ActionSessionValidator<'a>,
        }
        impl<'a> MapperTrait for Mapper<'a> {
            fn get_cloned<T>(&self) -> Option<Vec<ArcAction<T>>>
            where
                T: 'static,
            {
                self.map.get_cloned(self.validator)
            }
        }
        self.run_action(Mapper { map, validator });
    }

    fn on_change_time(&mut self, map: &TypedAnyActionMapGroupedByEntityId, time: TimeMilliseconds) {
        struct Mapper<'a> {
            map: &'a TypedAnyActionMapGroupedByEntityId,
            time: TimeMilliseconds,
        }
        impl<'a> MapperTrait for Mapper<'a> {
            fn get_cloned<T>(&self) -> Option<Vec<ArcAction<T>>>
            where
                T: 'static,
            {
                self.map.get_cloned()
            }
            fn time(&self) -> TimeMilliseconds {
                self.time
            }
        }
        self.run_action(Mapper { map, time });
    }

    fn on_first(&self, _entity_id: EntityId, priority: u32) -> Self::Job {
        match priority {
            0 => HordeJob::Wander,
            _ => HordeJob::Wait(3000),
        }
    }

    fn on_next(
        &self,
        entity_id: EntityId,
        job: Self::Job,
        controller: &mut ActionController,
    ) -> Option<Self::Job> {
        match job {
            HordeJob::Wander => return run_wander_job(self, entity_id, controller),
            HordeJob::Wait(duration) => {
                controller.enqueue(entity_id, Arc::new(WaitState), duration);
            }
        }
        None
    }
}

fn run_wander_job(
    game: &Game,
    entity_id: EntityId,
    controller: &mut ActionController,
) -> Option<HordeJob> {
    let mut rng: rand::rngs::StdRng =
        rand::SeedableRng::seed_from_u64(entity_id * (controller.current_tick() + 1));

    let scaled_positions = game.scaled_positions.read();
    let scaled_positions = scaled_positions.get();
    let scaled_position = scaled_positions.get(entity_id)?;

    let steps: usize = rng.gen_range(5..10);
    let mut first = scaled_position.0.clone_owned();
    for _ in 0..steps {
        let passage_entity_id = *game.passages.get(&first)?;
        let paths = game.paths.read();
        let paths = paths.get();
        let passage_paths = paths.get(passage_entity_id)?;

        if passage_paths.is_empty() {
            break;
        }
        let next = &passage_paths[rng.gen_range(0..passage_paths.len())];
        controller.enqueue(
            entity_id,
            Arc::new(WalkState {
                from: first.clone_owned(),
                to: next.to.clone_owned(),
            }),
            rng.gen_range(1..3) * 500,
        );
        first = next.to.clone_owned();
    }
    controller.enqueue(entity_id, Arc::new(WaitState), rng.gen_range(2000..3000));
    None
}

pub struct CreatePlayerParams<'a> {
    pub controller: &'a mut ActionController<'a>,
    pub initial_position: TVec2<i32>,
}

fn create_player(game: &mut Game, params: CreatePlayerParams) {
    let entity_id = game.entity_manager.gen();
    let position = calc_center_from_scaled_position(&params.initial_position);
    game.positions
        .write()
        .get_mut()
        .push(entity_id, PositionData(position.clone_owned()));
    game.scaled_positions.write().get_mut().push(
        entity_id,
        ScaledPositionData(params.initial_position.clone_owned()),
    );
    game.colors
        .write()
        .get_mut()
        .push(entity_id, ColorData(vec3(1.0f32, 0.0f32, 0.0f32)));
    params.controller.attach(entity_id);

    game.renderer.add_sprite(entity_id, &position);
}

pub struct CreatePassageParams {
    pub initial_position: TVec2<i32>,
    pub directions: Vec<Direction>,
}

fn create_passage(game: &mut Game, params: CreatePassageParams) {
    let entity_id = game.entity_manager.gen();
    game.scaled_positions.write().get_mut().push(
        entity_id,
        ScaledPositionData(params.initial_position.clone_owned()),
    );

    let mut paths = Vec::new();
    let mut lines = Vec::new();
    let from = calc_center_from_scaled_position(&params.initial_position);
    for direction in params.directions {
        let scaled_to = match direction {
            Direction::North => vec2(0, -1),
            Direction::South => vec2(0, 1),
            Direction::East => vec2(1, 0),
            Direction::West => vec2(-1, 0),
        } + params.initial_position;
        let to = calc_center_from_scaled_position(&scaled_to);
        lines.push((from, to));
        paths.push(PathData {
            _from: params.initial_position.clone_owned(),
            to: scaled_to,
        });
    }

    game.paths.write().get_mut().push(entity_id, paths);

    game.passages.set(&params.initial_position, entity_id);

    game.renderer.add_line(entity_id, &lines);
}

impl Game {
    pub fn create_player(&mut self, params: CreatePlayerParams) {
        create_player(self, params);
    }

    pub fn create_passage(&mut self, params: CreatePassageParams) {
        create_passage(self, params);
    }
}
