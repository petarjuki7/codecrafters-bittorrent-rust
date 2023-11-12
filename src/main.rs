use serde::Deserialize;
use serde_bencode;
use serde_bytes::ByteBuf;
use serde_json::{self, Map};
use std::env;

// Available if you need it!
// use serde_bencode

#[derive(Debug, Deserialize)]
struct Node(String, i64);

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Info {
    pub name: String,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Torrent {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
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

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        //println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value).0;
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let torrent_path = &args[2];
        let torrent_file = std::fs::read(torrent_path).unwrap();
        let t: Torrent = serde_bencode::from_bytes(&torrent_file).unwrap();
        println!("Tracker URL: {}", t.announce.unwrap());
        println!("Length: {}", t.info.length.unwrap());
    } else {
        println!("unknown commands: {}", args[1])
    }
}
