use bevy::prelude::*;

//
// PUBLIC TYPES
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
}

//
// CONFIG
//

pub const GRID_WIDTH: i32 = 500;
pub const GRID_HEIGHT: i32 = 500;
pub const FIXED_TIMESTEP: f32 = 1.0 / 120.0;

//
// BOARD (Authoritative occupancy grid)
//

#[derive(Resource)]
pub struct Board {
    /// Maps cell -> Entity occupying it
    cells: Vec<Option<Entity>>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: vec![None; (GRID_WIDTH * GRID_HEIGHT) as usize],
        }
    }

    #[inline]
    fn index(x: i32, y: i32) -> usize {
        (y * GRID_WIDTH + x) as usize
    }

    pub fn clear(&mut self) {
        self.cells.fill(None);
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Entity> {
        if x >= 0 && y >= 0 && x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells[Self::index(x, y)]
        } else {
            None
        }
    }

    pub fn set(&mut self, x: i32, y: i32, entity: Option<Entity>) {
        if x >= 0 && y >= 0 && x < GRID_WIDTH && y < GRID_HEIGHT {
            let idx = Self::index(x, y);
            self.cells[idx] = entity;
        }
    }
}

//
// COMPONENTS
//

#[derive(Component)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Mass(pub f32);

#[derive(Component)]
pub struct Owner(pub PlayerId);

#[derive(Component)]
pub struct Radius(pub f32);

#[derive(Component)]
pub struct Static; // marker

//
// COMMAND API
//

#[derive(Event)]
pub enum GameCommand {
    PlacePiece {
        position: Vec2,
        radius: f32,
        owner: PlayerId,
    },
    Shoot {
        entity: Entity,
        direction: Vec2,
        force: f32,
    },
}

//
// PLUGIN
//

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new())
            .add_event::<GameCommand>()
            .add_systems(
                Update,
                (
                    fixed_step_driver,
                    process_commands,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    integrate_motion,
                    resolve_collisions,
                    rebuild_board.after(resolve_collisions),
                ),
            );
    }
}

//
// FIXED TIMESTEP DRIVER
//

fn fixed_step_driver(mut time: ResMut<Time>) {
    time.set_timestep(FIXED_TIMESTEP);
}

//
// COMMAND HANDLER
//

fn process_commands(
    mut commands: Commands,
    mut events: EventReader<GameCommand>,
    query: Query<&Position>,
) {
    for event in events.read() {
        match event {
            GameCommand::PlacePiece {
                position,
                radius,
                owner,
            } => {
                commands.spawn((
                    Position(*position),
                    Velocity(Vec2::ZERO),
                    Mass(1.0),
                    Radius(*radius),
                    Owner(*owner),
                ));
            }

            GameCommand::Shoot {
                entity,
                direction,
                force,
            } => {
                if let Ok(pos) = query.get(*entity) {
                    let dir = direction.normalize_or_zero();
                    commands.entity(*entity).insert(Velocity(dir * *force));
                }
            }
        }
    }
}

//
// PHYSICS
//

fn integrate_motion(
    mut query: Query<(&mut Position, &mut Velocity), Without<Static>>,
) {
    for (mut pos, mut vel) in &mut query {
        pos.0 += vel.0 * FIXED_TIMESTEP;
        vel.0 *= 0.99; // friction
    }
}

//
// COLLISION (No Overlap Guaranteed)
//

fn resolve_collisions(
    mut query: Query<(Entity, &mut Position, &mut Velocity, &Radius, &Mass)>,
) {
    let mut combinations = query.iter_combinations_mut();

    while let Some([
        (e1, mut p1, mut v1, r1, m1),
        (e2, mut p2, mut v2, r2, m2),
    ]) = combinations.fetch_next()
    {
        let delta = p2.0 - p1.0;
        let dist = delta.length();
        let min_dist = r1.0 + r2.0;

        if dist < min_dist && dist > 0.0 {
            let normal = delta / dist;
            let penetration = min_dist - dist;

            // Positional correction (no overlap)
            p1.0 -= normal * (penetration * 0.5);
            p2.0 += normal * (penetration * 0.5);

            // Elastic impulse
            let relative_velocity = v2.0 - v1.0;
            let vel_along_normal = relative_velocity.dot(normal);

            if vel_along_normal < 0.0 {
                let restitution = 0.9;
                let impulse_mag = -(1.0 + restitution) * vel_along_normal
                    / (1.0 / m1.0 + 1.0 / m2.0);

                let impulse = normal * impulse_mag;

                v1.0 -= impulse / m1.0;
                v2.0 += impulse / m2.0;
            }
        }
    }
}

//
// BOARD REBUILD
//

fn rebuild_board(
    mut board: ResMut<Board>,
    query: Query<(Entity, &Position, &Radius)>,
) {
    board.clear();

    for (entity, pos, radius) in &query {
        let min_x = (pos.0.x - radius.0) as i32;
        let max_x = (pos.0.x + radius.0) as i32;
        let min_y = (pos.0.y - radius.0) as i32;
        let max_y = (pos.0.y + radius.0) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if (Vec2::new(x as f32, y as f32) - pos.0).length_squared()
                    <= radius.0 * radius.0
                {
                    board.set(x, y, Some(entity));
                }
            }
        }
    }
}