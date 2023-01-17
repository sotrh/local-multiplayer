use std::{net::UdpSocket, marker::PhantomData};

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Could not parse packet")]
    InvalidPacket,
    #[error("Packet is too large")]
    PacketTooLarge,
    #[error("Buffer is too small")]
    BufferTooSmall,
    #[error("No connection")]
    NoConnection,
    #[error("Unable to connect to {0}")]
    UnableToConnect(String),
    #[error("A network error occured")]
    Io(#[from] std::io::Error),
}


pub trait Packet {
    type Output;
    const MAX_SIZE: usize;
    fn to_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> Option<Self::Output>;
}

impl Packet for String {
    type Output = Self;

    const MAX_SIZE: usize = 1080;

    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self::Output> {
        String::from_utf8(bytes.to_owned()).ok()
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
    }
}
