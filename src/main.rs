use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread::sleep;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time;
use server::ThreadPool;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to port");
    let clients: Arc<Mutex<HashMap<String, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));

    let pool = ThreadPool::new(4);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let clients_clone = Arc::clone(&clients);
                pool.execute(move || {
                    handle_connection(stream, clients_clone);
                });
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}

fn handle_connection(mut stream: TcpStream, clients: Arc<Mutex<HashMap<String, TcpStream>>>){
    println!("New connection: {}", stream.peer_addr().unwrap());
    let stream_clone = stream.try_clone().expect("Failed to clone client stream");
    let addr = stream.peer_addr().unwrap().to_string();
    clients.lock().unwrap().insert(addr.clone(), stream_clone);
    
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 { // Connection closed
                    break;
                }
                let msg = String::from_utf8_lossy(&buffer[..size]).to_string();
                println!("{}",msg);
                let mut clients_lock = clients.lock().unwrap();
                
                // Iterate over the HashMap keys to avoid direct mutable borrow
                let client_keys: Vec<String> = clients_lock.keys().cloned().collect();
                for key in client_keys {
                    if let Some(client_stream) = clients_lock.get_mut(&key) {
                        if client_stream.peer_addr().unwrap() != stream.peer_addr().unwrap() {
                            match client_stream.write_all(msg.as_bytes()) {
                                Ok(_) => {},
                                Err(e) => {
                                    println!("Failed to send message: {:?}", e);
                                    clients_lock.remove(&key);// 연결 종료 처리 로직 (클라이언트 목록에서 제거 등)
                                    break; // 또는 적절한 조치
                                },
                            }
                        }
                    }
                }
            },
            Err(_) => {
                println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                break;
            }
        }
        let one_millis = time::Duration::from_millis(1);
        sleep(one_millis);
    }
}