use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Debug, Clone, Serialize)]
pub struct TrackerRequest {
    pub peer_id: String,

    pub port: u16,

    pub uploaded: usize,

    pub downloaded: usize,

    pub left: i64,

    pub compact: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    pub interval: usize,

    pub peers: ByteBuf,
}
