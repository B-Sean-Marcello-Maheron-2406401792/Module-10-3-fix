use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub sender: String,
    pub content: String,
    pub avatar: Option<String>,
    pub addr: Option<String>,
    pub timestamp: Option<String>,
}

type ClientMap = Arc<Mutex<HashMap<SocketAddr, String>>>;

#[tokio::main]
async fn main() {
    // let addr = "127.0.0.1:8080";
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    let (tx, _rx) = broadcast::channel::<String>(64);
    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));

    println!("[Server] Listening on ws://{}", addr);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        let tx = tx.clone();
        let clients = Arc::clone(&clients);
        tokio::spawn(handle_connection(stream, peer_addr, tx, clients));
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    tx: broadcast::Sender<String>,
    clients: ClientMap,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[Server] Handshake error from {}: {}", addr, e);
            return;
        }
    };

    println!("[Server] New connection: {}", addr);
    clients.lock().unwrap().insert(addr, addr.to_string());
    println!("[Server] Total clients: {}", clients.lock().unwrap().len());

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut rx = tx.subscribe();

    // Kirim welcome hanya ke klien ini
    let welcome = ChatMessage {
        msg_type: "system".into(),
        sender: "Server".into(),
        content: format!("Welcome to YewChat! You are connected from {}", addr),
        avatar: None,
        addr: None,
        timestamp: Some(current_time()),
    };
    let _ = ws_sender
        .send(Message::Text(serde_json::to_string(&welcome).unwrap()))
        .await;

    // Beritahu klien lain ada yang join
    let join_msg = ChatMessage {
        msg_type: "system".into(),
        sender: "Server".into(),
        content: format!("{} joined the chat", addr),
        avatar: None,
        addr: Some(addr.to_string()),
        timestamp: Some(current_time()),
    };
    let _ = tx.send(serde_json::to_string(&join_msg).unwrap());

    // Task: broadcast channel → kirim ke WebSocket klien ini
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if ws_sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });

    // Task utama: terima pesan dari klien → broadcast ke semua
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                println!("[Server] From {}: {}", addr, text);

                let enriched = if let Ok(mut chat) =
                    serde_json::from_str::<ChatMessage>(&text)
                {
                    chat.addr = Some(addr.to_string());
                    chat.timestamp = Some(current_time());
                    serde_json::to_string(&chat).unwrap_or(text)
                } else {
                    let fallback = ChatMessage {
                        msg_type: "chat".into(),
                        sender: addr.to_string(),
                        content: text,
                        avatar: None,
                        addr: Some(addr.to_string()),
                        timestamp: Some(current_time()),
                    };
                    serde_json::to_string(&fallback).unwrap()
                };

                let _ = tx.send(enriched);
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    clients.lock().unwrap().remove(&addr);
    println!("[Server] {} disconnected. Total: {}", addr, clients.lock().unwrap().len());

    let leave_msg = ChatMessage {
        msg_type: "system".into(),
        sender: "Server".into(),
        content: format!("{} left the chat", addr),
        avatar: None,
        addr: Some(addr.to_string()),
        timestamp: Some(current_time()),
    };
    let _ = tx.send(serde_json::to_string(&leave_msg).unwrap());
}

fn current_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}