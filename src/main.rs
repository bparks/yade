use hex::FromHex;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Listening!");
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        println!("Connection established!");

        let welcome = Vec::from_hex("590000000a352e352e352d31302e342e31312d4d61726961444200090000005b462f554126367d00fef7210200ff8115000000000000070000003631334d6a4f2d2f454d355d006d7973716c5f6e61746976655f70617373776f726400").unwrap();
        stream.write(&welcome).unwrap();

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
    }
}