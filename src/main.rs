use bittorrent_starter_rust::torrent::Torrent;
use bittorrent_starter_rust::tracker::{TrackerRequest, TrackerResponse};
use clap::*;
use serde_bencode;
use serde_json::{self, Map};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

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
    Decode { value: String },
    Info { torrent: PathBuf },
    Peers { torrent: PathBuf },
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args = Args::parse();

    match args.command {
        Command::Decode { value } => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            //println!("Logs from your program will appear here!");

            // Uncomment this block to pass the first stage

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
