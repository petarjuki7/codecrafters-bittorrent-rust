use serde_json::{self, Map};
use std::env;

// Available if you need it!
// use serde_bencode

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
                let (key, remainder) = decode_bencoded_value(rest);
                let (value, remainder) = decode_bencoded_value(remainder);
                rest = remainder;
                println!("{key}");
                dict.insert(key.to_string(), value);
                println!("{key}");
            }
            return (serde_json::Value::Object(dict), &rest[1..]);
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
    } else {
        println!("unknown commands: {}", args[1])
    }
}
