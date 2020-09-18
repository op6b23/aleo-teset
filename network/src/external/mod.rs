// Copyright (C) 2019-2020 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

pub mod channel;
pub use channel::*;

pub mod message;
pub use message::*;

/// Messages are serialized into bytes for transmission, and deserialized into a message payload when received.
pub mod message_types;
#[doc(inline)]
pub use message_types::*;

pub mod protocol;
pub use protocol::*;

use crate::server::Context;
use snarkos_errors::network::SendError;

use std::{net::SocketAddr, sync::Arc};

/// Broadcast transaction to connected peers
pub async fn propagate_transaction(
    context: Arc<Context>,
    transaction_bytes: Vec<u8>,
    transaction_sender: SocketAddr,
) -> Result<(), SendError> {
    debug!("Propagating a transaction to peers");

    let peer_book = context.peer_book.read().await;
    let local_address = *context.local_address.read().await;
    let connections = context.connections.read().await;
    let mut num_peers = 0;

    for (socket, _) in &peer_book.get_connected() {
        if *socket != transaction_sender && *socket != local_address {
            if let Some(channel) = connections.get(socket) {
                match channel.write(&Transaction::new(transaction_bytes.clone())).await {
                    Ok(_) => num_peers += 1,
                    Err(error) => warn!(
                        "Failed to propagate transaction to peer {}. (error message: {})",
                        channel.address, error
                    ),
                }
            }
        }
    }

    debug!("Transaction propagated to {} peers", num_peers);

    Ok(())
}

/// Broadcast block to connected peers
pub async fn propagate_block(
    context: Arc<Context>,
    block_bytes: Vec<u8>,
    block_miner: SocketAddr,
) -> Result<(), SendError> {
    debug!("Propagating a block to peers");

    let peer_book = context.peer_book.read().await;
    let local_address = *context.local_address.read().await;
    let connections = context.connections.read().await;
    let mut num_peers = 0;

    for (socket, _) in &peer_book.get_connected() {
        if *socket != block_miner && *socket != local_address {
            if let Some(channel) = connections.get(socket) {
                match channel.write(&Block::new(block_bytes.clone())).await {
                    Ok(_) => num_peers += 1,
                    Err(error) => warn!(
                        "Failed to propagate block to peer {}. (error message: {})",
                        channel.address, error
                    ),
                }
            }
        }
    }

    debug!("Block propagated to {} peers", num_peers);

    Ok(())
}
