pub mod camera;

use std::{
    collections::HashSet,
    time::Duration,
};

use crate::game::camera::Camera2d;

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
    pub(crate) position: glam::Vec2,
    pub(crate) score: i32,
    joystick: glam::Vec2,
    speed: f32,
}

pub struct Pickup {
    pub(crate) position: glam::Vec2,
    value: i32,
}

pub struct Game {
    players: Vec<Player>,
    pickups: Vec<Pickup>,
    camera: Camera2d,
    ui_camera: Camera2d,
    pickup_timer: Duration,
    pickup_accumulator: Duration,
}

impl Game {
    pub fn new(pickup_timer: Duration) -> Self {
        Self {
            players: Vec::new(),
            pickups: Vec::new(),
            camera: Camera2d::new(1.0, 1.0, glam::Vec2::ZERO),
            ui_camera: Camera2d::new(1.0, 1.0, glam::Vec2::ZERO),
            pickup_timer,
            pickup_accumulator: Duration::ZERO,
        }
    }

    pub fn spawn_player(&mut self) -> PlayerId {
        let id = PlayerId(self.players.len());
        self.players.push(Player {
            size: 10.0,
            position: glam::vec2(0.0, 0.0),
            joystick: glam::vec2(0.0, 0.0),
            score: 0,
            speed: 100.0,
        });
        id
    }

    pub fn tick(&mut self, dt: Duration) {
        self.handle_spawn(dt);

        let dt = dt.as_secs_f32();

        self.handle_physics(dt);
    }

    fn handle_spawn(&mut self, dt: Duration) {
        if self.players.is_empty() {
            return;
        }

        self.pickup_accumulator += dt;

        while self.pickup_accumulator >= self.pickup_timer {
            self.pickup_accumulator -= self.pickup_timer;
            self.pickups.push(Pickup {
                position: glam::vec2(rand::random(), rand::random()) * 200.0 - 100.0,
                value: 1,
            });
        }
    }

    fn handle_physics(&mut self, dt: f32) {
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
                a.distance_sq.total_cmp(&b.distance_sq)
            } else {
                // reverse pickup order to make removing pickups safe
                b.pickup.cmp(&a.pickup)
            }
        });

        let mut handled_pickups = HashSet::new();

        for collision in collisions.into_iter() {
            if handled_pickups.contains(&collision.pickup) {
                continue;
            }

            handled_pickups.insert(collision.pickup);

            self.players[collision.player].score += self.pickups[collision.pickup].value;

            let _ = self.pickups.swap_remove(collision.pickup);
        }
    }

    pub(crate) fn handle_input(&mut self, event: InputEvent) {
        let player = &mut self.players[event.id.0];
        match event.input {
            Input::X(amount) => player.joystick.x = amount,
            Input::Y(amount) => player.joystick.y = amount,
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let hw = width as f32 * 0.5;
        let hh = height as f32 * 0.5;
        self.camera.width = hw;
        self.camera.height = hh;
        self.ui_camera.width = width as f32;
        self.ui_camera.height = -(height as f32);
        self.ui_camera.position.x = self.ui_camera.width * 0.25;
        self.ui_camera.position.y = self.ui_camera.height * -0.25;
    }

    pub(crate) fn players(&self) -> &[Player] {
        &self.players
    }

    pub(crate) fn pickups(&self) -> &[Pickup] {
        &self.pickups
    }

    pub(crate) fn active_camera(&self) -> &Camera2d {
        &self.camera
    }

    pub(crate) fn ui_camera(&self) -> &Camera2d {
        &self.ui_camera
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
    if distance_sq <= radius * radius {
        Some(Collision {
            player,
            pickup,
            distance_sq,
        })
    } else {
        None
    }
}
