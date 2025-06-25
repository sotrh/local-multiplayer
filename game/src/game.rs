use std::{collections::HashSet, time::Duration};

pub struct InputEvent {
    pub(crate) id: PlayerId,
    pub(crate) input: Input,
}

pub enum Input {
    X(f32),
    Y(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerId(usize);

#[derive(Debug)]
pub struct Player {
    size: f32,
    position: glam::Vec2,
    joystick: glam::Vec2,
    score: f32,
    speed: f32,
}

pub struct Pickup {
    position: glam::Vec2,
    value: f32,
}

pub struct Game {
    players: Vec<Player>,
    pickups: Vec<Pickup>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            pickups: Vec::new(),
        }
    }

    pub fn spawn_player(&mut self) -> PlayerId {
        let id = PlayerId(self.players.len());
        self.players.push(Player {
            size: 10.0,
            position: glam::vec2(0.0, 0.0),
            joystick: glam::vec2(0.0, 0.0),
            score: 0.0,
            speed: 10.0,
        });
        id
    }

    pub fn tick(&mut self, duration: Duration) {
        let dt = duration.as_secs_f32();

        let mut collisions = Vec::new();

        for (id, player) in self.players.iter_mut().enumerate() {
            let velocity = player.joystick * player.speed;
            player.position += velocity * dt;

            for (i, pickup) in self.pickups.iter().enumerate() {
                if let Some(collision) =
                    circle_point(id, player.position, player.size, i, pickup.position)
                {
                    collisions.push(collision);
                }
            }
        }

        collisions.sort_by(|a, b| {
            if a.pickup == b.pickup {
                // reverse pickup order to make removing pickups safe
                b.distance_sq.total_cmp(&a.distance_sq)
            } else {
                a.pickup.cmp(&b.pickup)
            }
        });

        let mut handled_pickups = HashSet::new();

        for collision in collisions.into_iter() {
            if handled_pickups.contains(&collision.pickup) {
                continue;
            }

            handled_pickups.insert(collision.pickup);

            self.players[collision.player].score += self.pickups[collision.pickup].value;
        }

        log::debug!("{:?}", self.players.iter().map(|p| p.joystick).collect::<Vec<_>>());
    }
    
    pub(crate) fn handle_input(&mut self, event: InputEvent) {
        let player = &mut self.players[event.id.0];
        match event.input {
            Input::X(amount) => player.joystick.x = amount,
            Input::Y(amount) => player.joystick.y = amount,
        }
    }
}

struct Collision {
    player: usize,
    pickup: usize,
    distance_sq: f32,
}

fn circle_point(
    player: usize,
    center: glam::Vec2,
    radius: f32,
    pickup: usize,
    point: glam::Vec2,
) -> Option<Collision> {
    let distance_sq = center.distance_squared(point);
    if radius * radius <= distance_sq {
        Some(Collision {
            player,
            pickup,
            distance_sq,
        })
    } else {
        None
    }
}
