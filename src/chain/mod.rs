mod block_index;
mod chain;
pub mod store;

pub use block_index::BlockIndex;
pub use chain::Chain;
pub use chain::ChainStore;
pub use chain::OnMemoryChainStore;

#[derive(Debug)]
pub enum Error {
    RocksDBError(rocksdb::Error),
    EncodeError(bitcoin::consensus::encode::Error),
    BitcoinHashesError(bitcoin_hashes::Error),
}

impl From<rocksdb::Error> for Error {
    fn from(e: rocksdb::Error) -> Error {
        Error::RocksDBError(e)
    }
}

impl From<bitcoin::consensus::encode::Error> for Error {
    fn from(e: bitcoin::consensus::encode::Error) -> Error {
        Error::EncodeError(e)
    }
}

impl From<bitcoin_hashes::Error> for Error {
    fn from(e: bitcoin_hashes::Error) -> Error {
        Error::BitcoinHashesError(e)
    }
}
