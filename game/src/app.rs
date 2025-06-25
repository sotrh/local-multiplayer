use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId as WinitDeviceId, RawKeyEvent, WindowEvent},
    event_loop::{self, ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};

use crate::{
    game::{Game, Input, InputEvent, PlayerId},
    render::Renderer,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    fullscreen: bool,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DeviceId {
    Winit(WinitDeviceId),
    Gamepad(gilrs::GamepadId),
}

pub enum AppEvent {
    RendererCreated(Renderer),
    RendererFailed,
}

impl std::fmt::Debug for AppEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RendererCreated(_) => f.debug_tuple("RendererCreated").field(&"...").finish(),
            Self::RendererFailed => write!(f, "RendererFailed"),
        }
    }
}

pub struct App {
    renderer: Option<Renderer>,
    game: Game,
    proxy: event_loop::EventLoopProxy<AppEvent>,
    gamepads: gilrs::Gilrs,
    accumulator: Duration,
    game_timer: Instant,
    players: HashMap<DeviceId, PlayerId>,
    wasd: [f32; 4],
}

impl App {
    pub fn new(event_loop: &EventLoop<AppEvent>) -> Self {
        let proxy = event_loop.create_proxy();
        let gamepads = gilrs::GilrsBuilder::new().build().unwrap();
        Self {
            gamepads,
            renderer: None,
            proxy,
            game: Game::new(),
            accumulator: Duration::ZERO,
            game_timer: Instant::now(),
            players: HashMap::new(),
            wasd: [0.0; 4],
        }
    }

    fn spawn_task<F, Fut>(&self, task: F)
    where
        F: Send + 'static + FnOnce() -> Fut,
        Fut: Future<Output = AppEvent>,
    {
        let proxy = self.proxy.clone();
        std::thread::spawn(move || {
            let event = pollster::block_on(task());
            proxy.send_event(event).unwrap();
        });
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.spawn_task(move || async {
            match Renderer::new(window).await {
                Ok(renderer) => AppEvent::RendererCreated(renderer),
                Err(e) => {
                    log::error!("Failed to create renderer {}", e);
                    AppEvent::RendererFailed
                }
            }
        });
        self.game_timer = Instant::now();
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // let mut
        while let Some(event) = self.gamepads.next_event() {
            let player = *self
                .players
                .entry(DeviceId::Gamepad(event.id))
                .or_insert_with(|| self.game.spawn_player());

            match event.event {
                gilrs::EventType::AxisChanged(axis, mut amount, ..) => {
                    if amount.abs() < 0.1 {
                        amount = 0.0;
                    }
                    match axis {
                        gilrs::Axis::LeftStickX | gilrs::Axis::DPadX => {
                            self.game.handle_input(InputEvent {
                                id: player,
                                input: Input::X(amount),
                            });
                        }
                        gilrs::Axis::LeftStickY | gilrs::Axis::DPadY => {
                            self.game.handle_input(InputEvent {
                                id: player,
                                input: Input::Y(amount),
                            });
                        }
                        _ => {}
                    }
                }
                gilrs::EventType::Connected => {}
                gilrs::EventType::Disconnected => {}
                _ => {}
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::RendererCreated(renderer) => {
                renderer.window.request_redraw();
                self.game.resize(
                    renderer.window.inner_size().width,
                    renderer.window.inner_size().height,
                );
                self.renderer = Some(renderer);
            }
            AppEvent::RendererFailed => event_loop.exit(),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: WinitDeviceId,
        event: DeviceEvent,
    ) {
        let id = *self
            .players
            .entry(DeviceId::Winit(_device_id))
            .or_insert_with(|| self.game.spawn_player());

        match event {
            DeviceEvent::Key(RawKeyEvent {
                physical_key: PhysicalKey::Code(key),
                state,
            }) => {
                const W: usize = 0;
                const A: usize = 1;
                const S: usize = 2;
                const D: usize = 3;

                let amount = if state.is_pressed() { 1.0 } else { 0.0 };
                match key {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.wasd[W] = amount;
                        self.game.handle_input(InputEvent {
                            id,
                            input: Input::Y(self.wasd[W] - self.wasd[S]),
                        });
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.wasd[A] = amount;
                        self.game.handle_input(InputEvent {
                            id,
                            input: Input::X(self.wasd[D] - self.wasd[A]),
                        });
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.wasd[S] = amount;
                        self.game.handle_input(InputEvent {
                            id,
                            input: Input::Y(self.wasd[W] - self.wasd[S]),
                        });
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.wasd[D] = amount;
                        self.game.handle_input(InputEvent {
                            id,
                            input: Input::X(self.wasd[D] - self.wasd[A]),
                        });
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let renderer = if let Some(renderer) = &mut self.renderer {
            renderer
        } else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                self.game.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                renderer.window.request_redraw();

                let dt = self.game_timer.elapsed();
                self.accumulator += dt;
                self.game_timer = Instant::now();

                const TICK_RATE: Duration = Duration::from_millis(16);
                while self.accumulator > TICK_RATE {
                    self.accumulator -= TICK_RATE;
                    self.game.tick(TICK_RATE);
                }

                if !renderer.render(&self.game) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
