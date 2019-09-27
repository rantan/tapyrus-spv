//! # Tapyrus SPV Library
//!
//!
//!

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(missing_docs)]
#![deny(unused_must_use)]
#![forbid(unsafe_code)]
#![feature(async_await)]

extern crate bitcoin;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate bytes;

use crate::chain::store::DBChainStore;
use crate::chain::{Chain, ChainStore, OnMemoryChainStore};
use crate::network::{connect, BlockHeaderDownload, Handshake};
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::network::constants::Network;
use std::sync::{Arc, Mutex};
use tokio::prelude::Future;

mod chain;
mod network;

#[cfg(test)]
mod test_helper;

/// SPV
#[derive(Clone)]
pub struct SPV {
    network: Network,
}

impl SPV {
    /// returns SPV instance.
    pub fn new(network: Network) -> SPV {
        SPV { network }
    }

    /// run spv node.
    pub fn run(&self) {
        info!("start SPV node.");

        // initialize chain_state
        let datadir_path = "/tmp/spv";
        let db = rocksdb::DB::open_default(&datadir_path).unwrap();
        let mut chain_store = DBChainStore::new(db);
        chain_store.initialize(genesis_block(Network::Regtest));
        let chain_active = Chain::new(chain_store);
        let chain_state = Arc::new(Mutex::new(ChainState::new(chain_active)));

        let chain_state_for_block_header_download = chain_state.clone();

        let connection = connect("127.0.0.1:18444", self.network)
            .and_then(|peer| Handshake::new(peer))
            .and_then(move |peer| {
                BlockHeaderDownload::new(peer, chain_state_for_block_header_download)
            })
            .map(move |_peer| {
                let chain_state = chain_state.lock().unwrap();
                let chain_active = chain_state.borrow_chain_active();
                info!("current block height: {}", chain_active.height());
            })
            .map_err(|e| error!("Error: {:?}", e));
        tokio::run(connection);
    }
}

/// Manage blockchain status
pub struct ChainState<T: ChainStore> {
    chain_active: Chain<T>,
}

impl ChainState<DBChainStore> {
    /// create ChainState instance
    pub fn new<T: ChainStore>(chain_active: Chain<T>) -> ChainState<T> {
        ChainState { chain_active }
    }
}

impl<T: ChainStore> ChainState<T> {
    /// borrow chain_active
    pub fn borrow_chain_active(&self) -> &Chain<T> {
        &self.chain_active
    }

    /// borrow mutable chain_active
    pub fn borrow_mut_chain_active(&mut self) -> &mut Chain<T> {
        &mut self.chain_active
    }
}
