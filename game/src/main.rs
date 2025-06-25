use winit::event_loop::EventLoop;

use crate::app::App;

mod game;
mod app;
mod render;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let event_loop = EventLoop::with_user_event().build()?;

    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)?;

    Ok(())
}