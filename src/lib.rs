use std::fmt::{Display, Formatter, Result};

use crc32fast::Hasher;

pub const HEADER_SIZE: usize = 9;
pub const MAX_DATA: usize = 1491;

pub const TYPE_MSG: u8 = 0;
pub const TYPE_ACK: u8 = 1;

/// Represents a packet sent over this transfer protocol. The data is stored in bytes and packets
/// are differentiated by their `ptype`
pub struct Packet {
    pub ptype: u8,
    pub seq: u32,
    pub data: Vec<u8>,
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let type_str = match self.ptype {
            TYPE_MSG => "MSG",
            TYPE_ACK => "ACK",
            _ => "UNK",
        };
        write!(f, "[{} seq={} len={}]", type_str, self.seq, self.data.len())
    }
}

impl Packet {
    /// Computes the checksum for this packet to ensure the data has not been corrupted
    fn compute_checksum(ptype: u8, seq: u32, data: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&[ptype]);
        hasher.update(&seq.to_be_bytes());
        hasher.update(data);
        hasher.finalize()
    }

    /// Converts this packet into a byte representation
    pub fn to_bytes(&self) -> Vec<u8> {
        let checksum = Self::compute_checksum(self.ptype, self.seq, &self.data);
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.data.len());
        buf.push(self.ptype);
        buf.extend_from_slice(&self.seq.to_be_bytes());
        buf.extend_from_slice(&checksum.to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    /// Converts bytes into a Packet. If not valid, a `None` is returned
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < HEADER_SIZE {
            return None;
        }
        let ptype = buf[0];
        let seq = u32::from_be_bytes(buf[1..5].try_into().ok()?);
        let checksum = u32::from_be_bytes(buf[5..9].try_into().ok()?);
        let data = buf[9..].to_vec();

        if checksum != Self::compute_checksum(ptype, seq, &data) {
            return None;
        }

        Some(Packet { ptype, seq, data })
    }
}
