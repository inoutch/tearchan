use crate::renderer::Renderer;
use crate::utils::{calc_center_from_scaled_position, calc_position_from_ratio, CELL_SCALE_SIZE};
use maze_generator::prelude::Direction;
use nalgebra_glm::{distance, vec2, vec3, TVec2, Vec2, Vec3};
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
use tearchan_horde::v2::job::manager::JobManager;
use tearchan_horde::v2::job::HordeInterface;
use tearchan_horde::v2::serde::{Deserialize, Serialize};
use tearchan_horde::v2::{calc_ratio_f32_from_ms, calc_ratio_f32_from_tick, define_actions, Tick};

const PLAYER_SPEED: f32 = 500.0f32; // ms/cell

enum Command {
    UpdateRenderSpritePosition(EntityId, Vec2),
    UpdateRenderSpriteColor(EntityId, Vec3),
}

pub enum CellType {
    Player,
    Enemy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalkState {
    from_scaled: TVec2<i32>,
    to_scaled: TVec2<i32>,
    from: Vec2,
    to: Vec2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaitState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeColorState {
    from: Vec3,
    to: Vec3,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum DirectionState {
    Right,
    Left,
    Up,
    Down,
    None,
}

impl DirectionState {
    pub fn get_vec2(&self) -> TVec2<i32> {
        match self {
            DirectionState::Right => vec2(1, 0),
            DirectionState::Left => vec2(-1, 0),
            DirectionState::Up => vec2(0, -1),
            DirectionState::Down => vec2(0, 1),
            DirectionState::None => vec2(0, 0),
        }
    }
}

impl From<TVec2<i32>> for DirectionState {
    fn from(diff: TVec2<i32>) -> Self {
        if diff.x > 0 {
            DirectionState::Right
        } else if diff.y > 0 {
            DirectionState::Down
        } else if diff.x < 0 {
            DirectionState::Left
        } else {
            DirectionState::Up
        }
    }
}

define_actions!(
    HordeAction,
    (Walk, WalkState),
    (Wait, WaitState),
    (ChangeColor, ChangeColorState),
    (Direction, DirectionState)
);

pub enum HordeJob {
    Wander,
    Wait(TimeMilliseconds),
    GoDestination,
}

struct PositionData {
    from: (Vec2, Tick),
    to: (Vec2, Tick),
}

struct ScaledPositionData(TVec2<i32>);

struct ColorData(Vec3);

struct PathData {
    _from: TVec2<i32>,
    to: TVec2<i32>,
}

struct DirectionData(DirectionState);

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
    pub player_id: EntityId,
    // components
    positions: ComponentGroupSync<PositionData>,
    scaled_positions: ComponentGroupSync<ScaledPositionData>,
    colors: ComponentGroupSync<ColorData>,
    paths: ComponentGroupSync<Vec<PathData>>,
    cell_types: ComponentGroupSync<CellType>,
    directions: ComponentGroupSync<DirectionData>,
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
            player_id: 0, // Invalid EntityId
            positions: Default::default(),
            scaled_positions: Default::default(),
            colors: Default::default(),
            paths: Default::default(),
            cell_types: Default::default(),
            directions: Default::default(),
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
                                ActionType::Start { start, end } => {
                                    position.from = (state.from.clone_owned(), *start);
                                    position.to = (state.to.clone_owned(), *end);
                                    sender
                                        .send(Command::UpdateRenderSpritePosition(
                                            walk_action.entity_id(),
                                            state.from.clone_owned(),
                                        ))
                                        .unwrap();
                                }
                                ActionType::Update { start, end } => {
                                    let ratio = calc_ratio_f32_from_ms(*start, *end, time);
                                    let position =
                                        calc_position_from_ratio(&state.from, &state.to, ratio);
                                    sender
                                        .send(Command::UpdateRenderSpritePosition(
                                            walk_action.entity_id(),
                                            position,
                                        ))
                                        .unwrap();
                                }
                                ActionType::End { .. } => {
                                    sender
                                        .send(Command::UpdateRenderSpritePosition(
                                            walk_action.entity_id(),
                                            state.to.clone_owned(),
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
                                    scaled_position.0 = state.from_scaled.clone_owned();
                                }
                                ActionType::Update { .. } => {}
                                ActionType::End { .. } => {
                                    scaled_position.0 = state.to_scaled.clone_owned();
                                }
                            }
                        }
                    }
                });
            }
            {
                let rsync_child = rsync.child();
                let cell_types = self.cell_types.read();
                let mut colors = self.colors.write();
                let sender = Sender::clone(&sender);
                let walk_actions = Arc::clone(&walk_actions);

                self.pool.execute(move || {
                    rsync_child.begin();
                    let cell_types = cell_types.get();
                    let mut colors = colors.get_mut();
                    rsync_child.end();

                    for walk_action in walk_actions.iter() {
                        let _state = walk_action.raw();
                        let color = colors.get_mut(walk_action.entity_id());
                        let cell_type = cell_types.get(walk_action.entity_id());
                        if let Some(color) = color {
                            if let Some(cell_type) = cell_type {
                                match walk_action.ty() {
                                    ActionType::Start { .. } => {
                                        color.0 = match cell_type {
                                            CellType::Player => vec3(0.0f32, 0.0f32, 1.0f32),
                                            CellType::Enemy => vec3(0.0f32, 1.0f32, 0.0f32),
                                        };
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
                    }
                });
            }
            {
                let rsync_child = rsync.child();
                let mut directions = self.directions.write();

                self.pool.execute(move || {
                    rsync_child.begin();
                    let mut directions = directions.get_mut();
                    rsync_child.end();

                    for walk_action in walk_actions.iter() {
                        let state = walk_action.raw();
                        let direction = directions.get_mut(walk_action.entity_id());
                        match walk_action.ty() {
                            ActionType::Start { .. } => {
                                if let Some(direction) = direction {
                                    direction.0 =
                                        DirectionState::from(state.to_scaled - state.from_scaled);
                                }
                            }
                            ActionType::Update { .. } => {}
                            ActionType::End { .. } => {
                                if let Some(direction) = direction {
                                    direction.0 = DirectionState::None;
                                }
                            }
                        }
                    }
                });
            }
        }
        if let Some(wait_actions) = map.get_cloned::<WaitState>() {
            let wait_actions = Arc::new(wait_actions);
            {
                let rsync_child = rsync.child();
                let cell_types = self.cell_types.read();
                let mut colors = self.colors.write();
                let sender = Sender::clone(&sender);
                let wait_actions = Arc::clone(&wait_actions);

                self.pool.execute(move || {
                    rsync_child.begin();
                    let cell_types = cell_types.get();
                    let mut colors = colors.get_mut();
                    rsync_child.end();

                    for wait_action in wait_actions.iter() {
                        let _state = wait_action.raw();
                        let color = colors.get_mut(wait_action.entity_id());
                        let cell_type = cell_types.get(wait_action.entity_id());
                        if let Some(cell_type) = cell_type {
                            if let Some(color) = color {
                                match wait_action.ty() {
                                    ActionType::Start { .. } => {
                                        color.0 = match cell_type {
                                            CellType::Player => vec3(0.0f32, 0.0f32, 1.0f32),
                                            CellType::Enemy => vec3(1.0f32, 0.0f32, 0.0f32),
                                        };
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
                    }
                });
            }
            {
                let rsync_child = rsync.child();
                let mut directions = self.directions.write();
                self.pool.execute(move || {
                    rsync_child.begin();
                    let mut directions = directions.get_mut();
                    rsync_child.end();

                    for wait_action in wait_actions.iter() {
                        match wait_action.ty() {
                            ActionType::Start { .. } => {
                                let _state = wait_action.raw();
                                let direction = directions.get_mut(wait_action.entity_id());
                                if let Some(direction) = direction {
                                    direction.0 = DirectionState::None;
                                }
                            }
                            ActionType::Update { .. } => {}
                            ActionType::End { .. } => {}
                        }
                    }
                });
            }
        }
        if let Some(direction_actions) = map.get_cloned::<DirectionState>() {
            let rsync_child = rsync.child();
            let mut directions = self.directions.write();

            self.pool.execute(move || {
                rsync_child.begin();
                let mut directions = directions.get_mut();
                rsync_child.end();

                for direction_action in direction_actions.iter() {
                    let state = direction_action.raw();
                    match direction_action.ty() {
                        ActionType::Start { .. } => {
                            let direction = directions.get_mut(direction_action.entity_id());
                            if let Some(direction) = direction {
                                direction.0 = *state.as_ref();
                            }
                        }
                        ActionType::Update { .. } => {}
                        ActionType::End { .. } => {}
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
    type Job = Arc<HordeJob>;

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

    fn on_cancel_job(&mut self, _entity_id: EntityId, mut jobs: Vec<Self::Job>) {
        while let Some(job) = jobs.pop() {
            match job.as_ref() {
                HordeJob::Wander => {}
                HordeJob::Wait(_) => {}
                HordeJob::GoDestination => {}
            }
        }
    }

    fn on_first(&self, entity_id: EntityId, priority: u32) -> Self::Job {
        let cell_types = self.cell_types.read();
        let cell_types = cell_types.get();
        let cell_type = cell_types.get(entity_id).unwrap();
        match cell_type {
            CellType::Player => match priority {
                0 => Arc::new(HordeJob::GoDestination),
                _ => Arc::new(HordeJob::Wait(3000)),
            },
            CellType::Enemy => match priority {
                0 => Arc::new(HordeJob::Wander),
                _ => Arc::new(HordeJob::Wait(3000)),
            },
        }
    }

    fn on_next(
        &self,
        entity_id: EntityId,
        job: Self::Job,
        controller: &mut ActionController,
    ) -> Option<Self::Job> {
        match job.as_ref() {
            HordeJob::Wander => return run_wander_job(self, entity_id, controller),
            HordeJob::Wait(duration) => {
                controller.enqueue(entity_id, Arc::new(WaitState), *duration);
            }
            HordeJob::GoDestination => return run_go_destination_job(self, entity_id, controller),
        }
        None
    }
}

fn run_wander_job(
    game: &Game,
    entity_id: EntityId,
    controller: &mut ActionController,
) -> Option<Arc<HordeJob>> {
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
                from_scaled: first.clone_owned(),
                to_scaled: next.to.clone_owned(),
                from: calc_center_from_scaled_position(&first),
                to: calc_center_from_scaled_position(&next.to),
            }),
            rng.gen_range(1..3) * 500,
        );
        first = next.to.clone_owned();
    }
    controller.enqueue(entity_id, Arc::new(WaitState), rng.gen_range(2000..3000));
    None
}

fn run_go_destination_job(
    game: &Game,
    entity_id: EntityId,
    controller: &mut ActionController,
) -> Option<Arc<HordeJob>> {
    let directions = game.directions.read();
    let directions = directions.get();
    let direction = directions.get(entity_id)?;

    if let DirectionState::None = direction.0 {
        return Some(Arc::new(HordeJob::Wait(TimeMilliseconds::MAX)));
    }

    let scaled_positions = game.scaled_positions.read();
    let scaled_positions = scaled_positions.get();
    let scaled_position = scaled_positions.get(entity_id)?;

    let positions = game.positions.read();
    let positions = positions.get();
    let position = positions.get(entity_id)?;

    let position = calc_position_from_ratio(
        &position.from.0,
        &position.to.0,
        calc_ratio_f32_from_tick(position.from.1, position.to.1, controller.current_tick()),
    );

    let center_of_scaled_position = calc_center_from_scaled_position(&scaled_position.0);
    let to_scaled_position = if position.x != center_of_scaled_position.x {
        if position.x < center_of_scaled_position.x {
            match direction.0 {
                DirectionState::Right => vec2(scaled_position.0.x, scaled_position.0.y),
                DirectionState::Left => vec2(scaled_position.0.x - 1, scaled_position.0.y),
                _ => scaled_position.0.clone_owned(),
            }
        } else {
            match direction.0 {
                DirectionState::Right => vec2(scaled_position.0.x + 1, scaled_position.0.y),
                DirectionState::Left => vec2(scaled_position.0.x, scaled_position.0.y),
                _ => scaled_position.0.clone_owned(),
            }
        }
    } else if position.y != center_of_scaled_position.y {
        if position.y < center_of_scaled_position.y {
            match direction.0 {
                DirectionState::Up => vec2(scaled_position.0.x, scaled_position.0.y - 1),
                DirectionState::Down => vec2(scaled_position.0.x, scaled_position.0.y),
                _ => scaled_position.0.clone_owned(),
            }
        } else {
            match direction.0 {
                DirectionState::Up => vec2(scaled_position.0.x, scaled_position.0.y),
                DirectionState::Down => vec2(scaled_position.0.x, scaled_position.0.y + 1),
                _ => scaled_position.0.clone_owned(),
            }
        }
    } else {
        scaled_position.0 + direction.0.get_vec2()
    };

    let passage_entity_id = game.passages.get(&scaled_position.0)?;
    let paths = game.paths.read();
    let paths = paths.get();
    let paths = paths.get(*passage_entity_id)?;

    let has_path = paths.iter().any(|path| path.to == to_scaled_position);
    if !has_path {
        return Some(Arc::new(HordeJob::Wait(TimeMilliseconds::MAX)));
    }

    let to_position = calc_center_from_scaled_position(&to_scaled_position);
    let cell_distance = distance(&to_position, &position) / CELL_SCALE_SIZE;

    controller.enqueue(
        entity_id,
        Arc::new(WalkState {
            from_scaled: scaled_position.0.clone_owned(),
            to_scaled: to_scaled_position.clone_owned(),
            from: position.clone_owned(),
            to: to_position,
        }),
        (cell_distance * PLAYER_SPEED) as TimeMilliseconds,
    );

    Some(Arc::new(HordeJob::Wait(TimeMilliseconds::MAX)))
}

pub struct CreatePlayerParams<'a> {
    pub job_manager: &'a mut JobManager<Game>,
    pub initial_position: TVec2<i32>,
    pub cell_type: CellType,
}

fn create_cell(game: &mut Game, params: CreatePlayerParams) -> EntityId {
    let entity_id = game.entity_manager.gen();
    let position = calc_center_from_scaled_position(&params.initial_position);
    game.positions.write().get_mut().push(
        entity_id,
        PositionData {
            from: (position.clone_owned(), 0),
            to: (position.clone_owned(), 0),
        },
    );
    game.scaled_positions.write().get_mut().push(
        entity_id,
        ScaledPositionData(params.initial_position.clone_owned()),
    );
    game.colors
        .write()
        .get_mut()
        .push(entity_id, ColorData(vec3(1.0f32, 0.0f32, 0.0f32)));
    game.cell_types
        .write()
        .get_mut()
        .push(entity_id, params.cell_type);
    game.directions
        .write()
        .get_mut()
        .push(entity_id, DirectionData(DirectionState::None));

    params.job_manager.attach(entity_id);

    game.renderer.add_sprite(entity_id, &position);

    entity_id
}

pub struct CreatePassageParams {
    pub initial_position: TVec2<i32>,
    pub directions: Vec<Direction>,
}

fn create_passage(game: &mut Game, params: CreatePassageParams) -> EntityId {
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

    entity_id
}

impl Game {
    pub fn create_cell(&mut self, params: CreatePlayerParams) -> EntityId {
        create_cell(self, params)
    }

    pub fn create_passage(&mut self, params: CreatePassageParams) -> EntityId {
        create_passage(self, params)
    }

    pub fn go_player(&mut self, job_manager: &mut JobManager<Game>, direction: DirectionState) {
        assert_ne!(self.player_id, 0);
        let directions = self.directions.read();
        let directions = directions.get();
        if directions
            .get(self.player_id)
            .map(|d| d.0 == DirectionState::None)
            .unwrap_or(false)
        {
            job_manager.interrupt(self.player_id, Arc::new(direction), 0);
        }
    }
}
