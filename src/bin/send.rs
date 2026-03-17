use std::{
    collections::HashSet,
    error::Error,
    io::Read,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use rtp::{Packet, MAX_DATA, TYPE_MSG};

/// Represents a sender that connects to a socket and sends data from stdin
struct Sender {
    socket: UdpSocket,
    smoothed_rtt: Duration,
    rtt_var: Duration,
    window_size: f64,
    ssthresh: f64,
}

impl Sender {
    /// Initializes this sender by connecting to 0.0.0.0 on port 0. smoothed_RTT, rtt variance,
    /// window_size, and ssthresh are all initializes to 0.5s, 0.1s, 2.0, and 64.0 respectively
    fn new(host: Ipv4Addr, port: u16) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(SocketAddr::new(host.into(), port))?;
        socket.set_nonblocking(true)?;
        eprintln!("Sender starting up using port {}", port);

        Ok(Sender {
            socket,
            smoothed_rtt: Duration::from_millis(400),
            rtt_var: Duration::from_millis(100),
            window_size: 3.0,
            ssthresh: 64.0,
        })
    }

    /// Sends the `packet` to this sender's socket
    fn send_packet(&self, packet: &Packet) -> Result<(), Box<dyn Error>> {
        eprintln!("Sending message '{}'", packet);
        self.socket.send(&packet.to_bytes())?;
        Ok(())
    }

    /// Attempts to receive an ack from this sender's socket
    fn recv_ack(&self) -> Option<Packet> {
        let mut buf = [0u8; 1500];
        match self.socket.recv(&mut buf) {
            Ok(len) => {
                let packet = Packet::from_bytes(&buf[..len])?;
                eprintln!("Received message {}", packet);
                Some(packet)
            }
            Err(_) => None,
        }
    }

    /// fills this sender's window by sending data over the socket until the window is filled
    fn fill_window(
        &self,
        seq: &mut u32,
        stdin: &mut impl Read,
        buf: &mut [u8],
        in_flight: &mut Vec<(Packet, Instant)>,
        done: &mut bool,
    ) -> Result<(), Box<dyn Error>> {
        while !*done && in_flight.len() < self.window_size as usize {
            let n = stdin.read(buf)?;
            if n == 0 {
                *done = true;
                break;
            }
            let packet = Packet {
                ptype: TYPE_MSG,
                seq: *seq,
                data: buf[..n].to_vec(),
            };
            self.send_packet(&packet)?;
            in_flight.push((packet, Instant::now()));
            *seq += 1;
        }
        Ok(())
    }

    /// Attempts to receive an ack from this sender's socket and adjust both this sender's expected
    /// smoothed_rtt, window_size, and ssthresh. These take inspiration from TCP Reno and implement
    /// the triple duplicated ACK approach.
    fn handle_ack(
        &mut self,
        in_flight: &mut Vec<(Packet, Instant)>,
        dup_count: &mut u32,
        acked: &mut HashSet<u32>,
    ) {
        while let Some(ack) = self.recv_ack() {
            if let Some(pos) = in_flight.iter().position(|(p, _)| p.seq == ack.seq) {
                let sample = in_flight[pos].1.elapsed(); // The elapsed time of ack'd packet in flight

                // == abs_diff but grader is old vers
                let diff = if sample > self.smoothed_rtt {
                    sample - self.smoothed_rtt
                } else {
                    self.smoothed_rtt - sample
                };

                self.rtt_var = self.rtt_var.mul_f64(0.75) + diff.mul_f64(0.25);
                self.smoothed_rtt = self.smoothed_rtt.mul_f64(0.875) + sample.mul_f64(0.125);
                in_flight.remove(pos);
                acked.insert(ack.seq);

                if self.window_size < self.ssthresh {
                    self.window_size += 1.0;
                } else {
                    self.window_size += 1.0 / self.window_size;
                }
                *dup_count = 0;
                if let Some((packet, sent_at)) = in_flight.first_mut() {
                    let _ = self.send_packet(packet);
                    *sent_at = Instant::now();
                }
            } else if acked.contains(&ack.seq) {
                // Network duplicated, ignore this ack
            } else {
                *dup_count += 1;
                if *dup_count >= 3 {
                    self.ssthresh = (self.window_size / 2.0).max(1.0);
                    self.window_size = self.ssthresh;
                    *dup_count = 0;
                }
            }
        }
    }

    /// Checks all the packets that are currently in this sender's `in_flight` and retransmits them
    /// if they have already timed out
    fn check_timeouts(
        &mut self,
        in_flight: &mut Vec<(Packet, Instant)>,
    ) -> Result<(), Box<dyn Error>> {
        let timeout = self.smoothed_rtt + self.rtt_var * 4;
        let mut did_retransmit = false;
        for (packet, sent_at) in in_flight {
            if sent_at.elapsed() > timeout {
                if !did_retransmit {
                    self.ssthresh = (self.window_size / 2.0).max(1.0);
                    self.window_size = self.ssthresh;
                    did_retransmit = true;
                }
                self.send_packet(packet)?;
                *sent_at = Instant::now();
            }
        }
        Ok(())
    }

    /// Runs all function of this sender and sends all data from stdin to the receiver
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut seq: u32 = 0;
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; MAX_DATA];
        let mut done = false;
        let mut in_flight: Vec<(Packet, Instant)> = Vec::new();
        let mut dup_count: u32 = 0;
        let mut acked = HashSet::new();

        loop {
            self.fill_window(&mut seq, &mut stdin, &mut buf, &mut in_flight, &mut done)?;

            if done && in_flight.is_empty() {
                eprintln!("All done!");
                return Ok(());
            }
            self.handle_ack(&mut in_flight, &mut dup_count, &mut acked);
            self.check_timeouts(&mut in_flight)?;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let host = args[1].parse()?;
    let port: u16 = args[2].parse()?;
    let mut sender = Sender::new(host, port)?;
    sender.run()
}
