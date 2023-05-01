use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct TetrisData {
    map: [[i32; 10]; 20], // 10x20 map
    score: i32, // current score
}

#[tokio::main]
async fn main() {
    let mut tetris_map = HashMap::<String, TetrisData>::new();

    let addr = "0.0.0.0:8888";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server Listening on: {addr}");
    
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let addr_string = addr.to_string();
        
        
        let mut buf = [0; 4096];
        
        let n = match socket.read(&mut buf).await {
            Ok(n) if n == 0 => { return; },
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket: {e}");
                return;
            }
        }
        let serialized_data = &buf[..n];
        let data = serde_json::from_slice(serialized_data).unwrap();
        tetris_map.insert(addr_string, data);

        let empty_data = TetrisData {
            map: [[0; 10]; 20],
            score: 0
        };

        let mut serialized_data = serde_json::to_string(&empty_data).unwrap();
        for (key, value) in &tetris_map {
            if *key != addr_string {
                serialized_data = serde_json::to_string(value).unwrap();
                break;
            }
        }
        socket.write_all(serialized_data.as_bytes()).await.unwrap();
        socket.shutdown().await.unwrap();

        println!("{}: {}",addr_string, data.score);
    }
}
