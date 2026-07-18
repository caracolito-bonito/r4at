use std::io::{self, Read, Write};
use thiserror::Error;

const MAX_PAYLOAD_SIZE: u16 = u16::MAX;

pub fn encode(payload: &[u8], stream: &mut impl Write) -> Result<(), ProtocolError> {
    let payload_len = payload.len();
    if payload_len > MAX_PAYLOAD_SIZE as usize {
        return Err(ProtocolError::PayloadIsTooLong(payload_len));
    }
    let len = payload_len as u16;

    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(payload)?;
    Ok(())
}

pub fn decode(stream: &mut impl Read) -> Result<Vec<u8>, ProtocolError> {
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
}
