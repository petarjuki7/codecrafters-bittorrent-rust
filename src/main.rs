use bittorrent_starter_rust::peers::*;
use bittorrent_starter_rust::torrent::Torrent;
use bittorrent_starter_rust::tracker::{TrackerRequest, TrackerResponse};
use clap::*;
use futures_util::{SinkExt, StreamExt};
use serde_bencode;
use serde_json::{self, Map};
use sha1::Digest;
use sha1::Sha1;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BLOCK_MAX: usize = 1 << 14;

// Available if you need it!
// use serde_bencode

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
enum Command {
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf,
    },
    Peers {
        torrent: PathBuf,
    },
    Handshake {
        torrent: PathBuf,
        ip: SocketAddrV4,
    },
    DownloadPiece {
        #[arg(short)]
        output: PathBuf,
        torrent: PathBuf,
        piece: usize,
    },
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => {
            let decoded_value = decode_bencoded_value(&value).0;
            println!("{}", decoded_value.to_string());
        }
        Command::Info { torrent } => {
            let torrent_file = std::fs::read(torrent).unwrap();
            let t: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();
            let hash = t.calc_info_hash();
            println!("Tracker URL: {}", t.announce);
            println!("Length: {}", t.info.length);
            println!("Info Hash: {}", hex::encode(hash));
            println!("Piece Length: {}", t.info.piece_length);
            println!("Piece Hashes:");
            let mut hash_iterator = t.info.pieces.chunks_exact(20);
            while let Some(chunk) = hash_iterator.next() {
                println!("{}", hex::encode(chunk));
            }
        }
        Command::Peers { torrent } => {
            let torrent_file = std::fs::read(torrent).unwrap();
            let t: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();
            let info_hash = t.calc_info_hash();

            let request = TrackerRequest {
                peer_id: "00112233445566778899".to_string(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: t.info.length,
                compact: 1,
            };

            let url_params = serde_urlencoded::to_string(&request).unwrap();

            let tracker_url = format!(
                "{}?{}&info_hash={}",
                t.announce,
                url_params,
                urlencode(&info_hash)
            );

            let response = reqwest::blocking::get(tracker_url).unwrap();
            let response = response.bytes().unwrap();

            let response: TrackerResponse =
                serde_bencode::from_bytes(&response).expect("parse tracker response");

            let nes = response.peers;

            let ips: Vec<SocketAddrV4> = nes
                .chunks_exact(6)
                .map(|slice_6| {
                    SocketAddrV4::new(
                        Ipv4Addr::new(slice_6[0], slice_6[1], slice_6[2], slice_6[3]),
                        u16::from_be_bytes([slice_6[4], slice_6[5]]),
                    )
                })
                .collect();

            for peer in ips {
                println!("{}:{}", peer.ip(), peer.port());
            }
        }
        Command::Handshake { torrent, ip } => {
            let torrent_file = std::fs::read(torrent).unwrap();
            let t: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();
            let info_hash = t.calc_info_hash();

            let handshake = Handshake {
                len: 19,
                string: *b"BitTorrent protocol",
                reserved: [0, 0, 0, 0, 0, 0, 0, 0],
                sha1_infohash: info_hash,
                peer_id: [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9],
            };

            let handshake_bytes = unsafe { handshake.as_u8_slice() };

            let mut stream = TcpStream::connect(ip).unwrap();

            stream.write_all(handshake_bytes).unwrap();

            let mut buffer = [0; 1 + 19 + 8 + 20 + 20];
            stream.read_exact(&mut buffer).unwrap();

            println!("Peer ID: {}", hex::encode(&buffer[48..]));
        }
        Command::DownloadPiece {
            output,
            torrent,
            piece,
        } => {
            let torrent_file = std::fs::read(torrent).unwrap();
            let t: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();
            let info_hash = t.calc_info_hash();

            let request = TrackerRequest {
                peer_id: "00112233445566778899".to_string(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: t.info.length,
                compact: 1,
            };

            let url_params = serde_urlencoded::to_string(&request).unwrap();

            let tracker_url = format!(
                "{}?{}&info_hash={}",
                t.announce,
                url_params,
                urlencode(&info_hash)
            );

            let response = reqwest::blocking::get(tracker_url).unwrap();
            let response = response.bytes().unwrap();

            let response: TrackerResponse =
                serde_bencode::from_bytes(&response).expect("parse tracker response");

            let nes = response.peers;

            let ips: Vec<SocketAddrV4> = nes
                .chunks_exact(6)
                .map(|slice_6| {
                    SocketAddrV4::new(
                        Ipv4Addr::new(slice_6[0], slice_6[1], slice_6[2], slice_6[3]),
                        u16::from_be_bytes([slice_6[4], slice_6[5]]),
                    )
                })
                .collect();

            let handshake = Handshake {
                len: 19,
                string: *b"BitTorrent protocol",
                reserved: [0, 0, 0, 0, 0, 0, 0, 0],
                sha1_infohash: info_hash,
                peer_id: [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9],
            };

            let handshake_bytes = unsafe { handshake.as_u8_slice() };

            let mut stream = tokio::net::TcpStream::connect(ips[0]).await.unwrap();

            stream.write_all(handshake_bytes).await.unwrap();

            let mut buffer = [0; 1 + 19 + 8 + 20 + 20];
            stream.read_exact(&mut buffer).await.unwrap();

            let mut peer = tokio_util::codec::Framed::new(stream, MessageFramer);

            let bitfield = peer.next().await.unwrap().unwrap();
            assert_eq!(bitfield.tag, MessageTag::Bitfield);

            peer.send(Message {
                tag: MessageTag::Interested,
                payload: Vec::new(),
            })
            .await
            .unwrap();

            let unchoke = peer.next().await.unwrap().unwrap();
            eprintln!("{:?}", unchoke.tag);

            let piece_hash = t.info.pieces.chunks_exact(20).next().unwrap();
            let piece_size = t.info.piece_length;
            eprintln!("{}", piece_size);
            eprintln!("{:?}", piece_hash);

            let nblocks = (piece_size + (BLOCK_MAX - 1)) / BLOCK_MAX;
            let mut all_blocks: Vec<u8> = Vec::with_capacity(piece_size as usize);
            for block in 0..nblocks {
                let block_size = if block == nblocks - 1 {
                    let md = piece_size % BLOCK_MAX;
                    if md == 0 {
                        BLOCK_MAX
                    } else {
                        md
                    }
                } else {
                    BLOCK_MAX
                };
                let mut request =
                    Request::new(piece as u32, (block * BLOCK_MAX) as u32, block_size as u32);
                let request_bytes = Vec::from(request.as_bytes_mut());
                peer.send(Message {
                    tag: MessageTag::Request,
                    payload: request_bytes,
                })
                .await
                .unwrap();

                let piece = peer
                    .next()
                    .await
                    .expect("peer always sends a piece")
                    .unwrap();
                assert_eq!(piece.tag, MessageTag::Piece);
                assert!(!piece.payload.is_empty());

                let piece = Piece::ref_from_bytes(&piece.payload[..])
                    .expect("always get all Piece response fields from peer");
                assert_eq!(piece.begin() as usize, block * BLOCK_MAX);
                assert_eq!(piece.block().len(), block_size);
                all_blocks.extend(piece.block());
            }
            assert_eq!(all_blocks.len(), piece_size);

            let mut hasher = Sha1::new();
            hasher.update(&all_blocks);
            let hash: [u8; 20] = hasher
                .finalize()
                .try_into()
                .expect("GenericArray<_, 20> == [_; 20]");
            assert_eq!(&hash, piece_hash);

            tokio::fs::write(&output, all_blocks).await.unwrap();
            println!("Piece {piece} downloaded to {}.", output.display());
        }
    }
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((num, rest)) = encoded_value.split_at(1).1.split_once('e') {
                return (
                    serde_json::Value::Number(num.parse::<i64>().unwrap().into()),
                    rest,
                );
            }
        }
        Some('l') => {
            let mut lista: Vec<serde_json::Value> = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (value, remainder) = decode_bencoded_value(rest);
                lista.push(value);
                rest = remainder;
            }

            return (serde_json::Value::Array(lista), &rest[1..]);
        }

        Some('d') => {
            let mut dict = Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (k, remainder) = decode_bencoded_value(rest);
                let k = match k {
                    serde_json::Value::String(k) => k,
                    k => {
                        panic!("dict keys must be strings, not {k:?}");
                    }
                };
                let (value, remainder) = decode_bencoded_value(remainder);
                rest = remainder;
                dict.insert(k, value);
            }
            return (dict.into(), &rest[1..]);
        }

        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                let length = len.parse::<usize>().unwrap();
                return (rest[..length].to_string().into(), &rest[length..]);
            }
        }
        _ => {}
    };

    panic!("Unhandled encoded value: {}", encoded_value)
}

fn urlencode(t: &[u8; 20]) -> String {
    let mut encoded = String::with_capacity(3 * t.len());
    for &byte in t {
        encoded.push('%');
        encoded.push_str(&hex::encode(&[byte]));
    }
    encoded
}
