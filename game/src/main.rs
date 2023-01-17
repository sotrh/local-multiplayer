use winit::{event_loop::EventLoop, event::*, window::WindowBuilder};

pub enum NetworkMessage {
    Quit,
}

pub enum GameMessage {

}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_visible(false)
        .build(&event_loop)?;

    let (tx, rx) = std::sync::mpsc::channel();
    
    std::thread::spawn(move || {
        println!("Network: started");
        loop {
            match rx.try_recv() {
                Ok(NetworkMessage::Quit) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    println!("Network: quitting");
                    break;
                },
                _ => (),
            }
        }
    });

    println!("Game: started");
    window.set_visible(true);
    event_loop.run(move |event, _, cf| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => cf.set_exit(),
                WindowEvent::KeyboardInput { input: KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                }, .. } => match (key, state == ElementState::Pressed) {
                    (VirtualKeyCode::Escape, true) => cf.set_exit(),
                    _ => (),
                } 
                _ => ()
            }
            Event::LoopDestroyed => {
                // Stop network thread
                println!("Game: quitting");
                if let Err(e) = tx.send(NetworkMessage::Quit) {
                    eprintln!("Game: unable to notify network thread: {:?}", e);
                }
            }
            _ => ()
        }
    })
}
