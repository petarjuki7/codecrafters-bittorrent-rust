use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    // If encoded_value starts with a digit, it's a number
    if let Some((len, string)) = encoded_value.split_once(':') {
        // Example: "5:hello" -> "hello"
        if let Ok(len) = len.parse::<usize>() {
            return serde_json::Value::String(string[..len].to_string());
        }
    } 
    if let Some('i') = encoded_value.chars().next() {
        if let Some(e_pos) = encoded_value.find('e'){
            return serde_json::Value::Number(encoded_value[1..e_pos].parse().unwrap())
        }
    }
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
         let decoded_value = decode_bencoded_value(encoded_value);
         println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
