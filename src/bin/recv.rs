use rtp::{Packet, TYPE_ACK};
use std::{
    collections::HashMap,
    error::Error,
    io::{stdout, Write},
    net::{SocketAddr, UdpSocket},
};

struct Receiver {
    socket: UdpSocket,
    remote: Option<SocketAddr>,
}

impl Receiver {
    /// Initializes a receiver bound to port 0.0.0.0
    fn new() -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let port = socket.local_addr()?.port();
        eprintln!("Bound to port {}", port);
        Ok(Receiver {
            socket,
            remote: Option::None,
        })
    }

    /// Sends an acknowledgement message to this receiver's remote socket
    fn send_ack(&self, seq: u32) -> Result<(), Box<dyn Error>> {
        let packet = Packet {
            ptype: TYPE_ACK,
            seq,
            data: vec![],
        };

        eprintln!("Sent message {}", packet);
        self.socket
            .send_to(&packet.to_bytes(), self.remote.unwrap())?;
        Ok(())
    }

    /// Receives the message from the given socket. Sets the remote socket if it hasn't been
    /// already for this Receiver
    fn recv(&mut self) -> Result<Option<Packet>, Box<dyn Error>> {
        let mut buf = [0u8; 1500];
        let (len, addr) = self.socket.recv_from(&mut buf)?;

        let remote = *self.remote.get_or_insert(addr);

        if addr != remote {
            eprintln!("Error: Received response from unexpected remote; ignoring");
            return Ok(None);
        }

        let packet = Packet::from_bytes(&buf[..len]);
        if let Some(p) = &packet {
            eprintln!("Received {}", p);
        }
        Ok(packet)
    }

    /// Runs all function of this receiver and sends acknowledgements back to the sender while
    /// writing the data into stdout
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut next_seq: u32 = 0;
        let mut buffer: HashMap<u32, Vec<u8>> = HashMap::new();
        loop {
            let data = self.recv()?;
            match data {
                None => continue,
                Some(packet) => {
                    self.send_ack(packet.seq)?;
                    if packet.seq >= next_seq {
                        buffer.insert(packet.seq, packet.data);
                    }
                    while let Some(data) = buffer.remove(&next_seq) {
                        stdout().write_all(&data)?;
                        stdout().flush()?;
                        next_seq += 1;
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut receiver = Receiver::new()?;
    receiver.run()
}
