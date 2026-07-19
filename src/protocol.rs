use std::io::{self, Read, Write};
use thiserror::Error;

pub enum Frame {
    Chat { id: u32, text: Vec<u8> },
    Dropped { id: u32 },
}

#[repr(u8)]
enum FrameType {
    Chat = 0,
    Dropped = 1,
}

impl TryFrom<u8> for FrameType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FrameType::Chat),
            1 => Ok(FrameType::Dropped),
            _ => Err(ProtocolError::UnknownFrameType(value)),
        }
    }
}
const MAX_PAYLOAD_SIZE: u16 = u16::MAX;

pub fn encode(frame: &Frame, stream: &mut impl Write) -> Result<(), ProtocolError> {
    let (type_byte, payload) = match frame {
        Frame::Chat { id, text } => (
            FrameType::Chat as u8,
            [&id.to_be_bytes()[..], text].concat(),
        ),
        Frame::Dropped { id } => (FrameType::Dropped as u8, id.to_be_bytes().to_vec()),
    };
    let payload_len = payload.len();
    if payload_len > MAX_PAYLOAD_SIZE as usize {
        return Err(ProtocolError::PayloadIsTooLong(payload_len));
    }
    let len = payload_len as u16;
    stream.write_all(&[type_byte])?;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&payload)?;
    Ok(())
}

pub fn decode(stream: &mut impl Read) -> Result<Frame, ProtocolError> {
    let mut type_buf = [0u8; 1];
    let mut header_buf = [0u8; 2];

    match stream.read_exact(&mut header_buf) {
        Ok(_) => {}
        Err(e) => {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return Err(ProtocolError::Disconnect);
            }
            return Err(ProtocolError::IO(e));
        }
    };
    let len = u16::from_be_bytes(header_buf);

    let mut payload = vec![0u8; len as usize];
    match stream.read_exact(&mut payload) {
        Ok(_) => Ok(payload),
        Err(e) => {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return Err(ProtocolError::Disconnect);
            }
            Err(ProtocolError::IO(e))
        }
    }
}

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("payload is too large: {0}")]
    PayloadIsTooLong(usize),
    #[error("couldn't write the frame")]
    IO(#[from] io::Error),
    #[error("disconnect")]
    Disconnect,
    #[error("Unknown frame type {0}")]
    UnknownFrameType(u8),
}
