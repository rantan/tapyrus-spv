//! `network` module
//!
//! The `network` module contains p2p communication functionality.

mod peer;
pub use peer::connect;
pub use peer::Peer;

mod handshake;
pub use handshake::Handshake;

mod block_header_download;
use crate::network::peer::PeerID;
pub use block_header_download::BlockHeaderDownload;

pub mod bytes;
pub mod codec;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    CodecError(codec::Error),
    UnboundedSendError(tokio::sync::mpsc::error::UnboundedSendError),
    UnboundedRecvError(tokio::sync::mpsc::error::UnboundedRecvError),
    MaliciousPeer(PeerID),
    WrongMagicBytes,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<codec::Error> for Error {
    fn from(e: codec::Error) -> Error {
        Error::CodecError(e)
    }
}

impl From<tokio::sync::mpsc::error::UnboundedSendError> for Error {
    fn from(e: tokio::sync::mpsc::error::UnboundedSendError) -> Error {
        Error::UnboundedSendError(e)
    }
}

impl From<tokio::sync::mpsc::error::UnboundedRecvError> for Error {
    fn from(e: tokio::sync::mpsc::error::UnboundedRecvError) -> Error {
        Error::UnboundedRecvError(e)
    }
}
