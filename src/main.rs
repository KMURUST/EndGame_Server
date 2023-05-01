use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct TetrisData {
    map: [[usize; 10]; 20], // 10x20 map
    score: usize, // current score
}

#[tokio::main]
async fn main() {
    let mut tetris_map = HashMap::<String, TetrisData>::new();
    let shared_map = Arc::new(Mutex::new(tetris_map));

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server Listening on: {addr}");
    
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let addr_string = addr.to_string();
        println!("{addr_string}");
        
        let shared_map = shared_map.clone();
        tokio::spawn(async move {

            let mut buf = [0; 4096];
            loop {
                let mut map = shared_map.lock().await;
                println!("{addr_string} READ");
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => { return; },
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket: {e}");
                        return;
                    }
                };
                let serialized_data = &buf[..n];
                let data: TetrisData = serde_json::from_slice(serialized_data).unwrap();
                // println!("{}: {:?}",addr_string, data);
                map.insert(addr_string.clone(), data.clone());

                let empty_data = TetrisData {
                    map: [[0; 10]; 20],
                    score: 0
                };

                let mut serialized_data = serde_json::to_string(&empty_data).unwrap();
                for (key, value) in map.clone().into_iter() {
                    if *key != addr_string {
                        serialized_data = serde_json::to_string(&value).unwrap();
                        break;
                    }
                }
                println!("{addr_string} WRITE");
                socket.write_all(serialized_data.as_bytes()).await.unwrap();
                println!("{}: {}",addr_string, data.score);

            }
            // socket.shutdown().await.unwrap();

        });
    }
}
