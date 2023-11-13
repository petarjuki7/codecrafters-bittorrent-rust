use serde::Deserialize;
use serde::Serialize;
use serde_bytes::ByteBuf;
use sha1::Digest;
use sha1::Sha1;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct Torrent {
    pub info: Info,
    #[serde(default)]
    pub announce: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub length: i64,
}

impl Torrent {
    pub fn calc_info_hash(&self) -> [u8; 20] {
        let mut hasher = Sha1::new();
        let info = serde_bencode::to_bytes(&self.info).unwrap();
        hasher.update(&info);
        return hasher.finalize().into();
    }
}
