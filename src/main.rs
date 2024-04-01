use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};
use std::collections::HashMap;
use std::sync::Arc;

type ClientMap = Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>;

async fn handle_connection(mut stream: TcpStream, clients: ClientMap) {
    let addr = stream.peer_addr().unwrap().to_string();
    let (tx, mut rx) = mpsc::channel(10);

    clients.lock().await.insert(addr.clone(), tx);

    let mut buffer = vec![0; 1024];

    loop {
        tokio::select! {
            size = stream.read(&mut buffer) => {
                let size = size.expect("failed to read data from socket");
                if size == 0 {
                    break;
                }
                let msg = String::from_utf8_lossy(&buffer[..size]).to_string();
                for (_, client_tx) in clients.lock().await.iter() {
                    let _ = client_tx.send(msg.clone()).await;
                }
            },
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    if stream.write_all(msg.as_bytes()).await.is_err() {
                        break;
                    }
                }
            },
        }
    }

    clients.lock().await.remove(&addr);
}

async fn broadcast_task(clients: ClientMap) {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let clients_clone = clients.clone();
        tokio::spawn(async move {
            handle_connection(stream, clients_clone).await;
        });
    }
}

#[tokio::main]
async fn main() {
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    broadcast_task(clients).await;
}