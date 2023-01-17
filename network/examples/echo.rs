use std::net::UdpSocket;

fn main() -> anyhow::Result<()> {
    let server_addr = "0.0.0.0:8000";

    let server = UdpSocket::bind(server_addr)?;

    let mut buffer = [0; 64];
    
    loop {
        let (amt, src) = server.recv_from(&mut buffer)?;
        let bytes = &buffer[..amt];
        let msg = std::str::from_utf8(bytes)?.trim();

        println!("Server: received message: {}", msg);

        server.send_to(bytes, &src)?;

        match msg {
            "exit" => {
                println!("Server: exiting");
                break;
            },
            _ => (),
        }
    }

    Ok(())
}