use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

type Outbox = mpsc::UnboundedSender<String>;
type Clients = Arc<Mutex<HashMap<String, Outbox>>>;

enum Cmd { Chat(String), Quit, Unkown, Who }

fn parse(line: &str) -> Cmd {
    let mut parts = line.trim().splitn(2, ' ');
    match (parts.next(), parts.next()) {
        (Some("CHAT"), Some(text)) => Cmd::Chat(text.to_string()),
        (Some("QUIT"), _) => Cmd::Quit,
        (Some("WHO"), _) => Cmd::Who, 
        _ => Cmd::Unkown
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000").await?;
    println!("listening on 127.0.0.1:4000");
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("{addr} connected");
        let clients = clients.clone();
        tokio::spawn(async move {
            handle(socket, clients).await;
            println!("{addr} disconnected");
        });
    }
}
async fn handle(socket: TcpStream, clients: Clients) {
    let (read, mut write) = socket.into_split();
    let mut lines = BufReader::new(read).lines();

    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
    tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if write.write_all(format!("{msg}\n").as_bytes()).await.is_err()
            {
                break;
            }
        }
    });

    let _ = out_tx.send("Enter your name:".into());
    let name = loop {
        let n = match lines.next_line().await {
            Ok(Some(n)) => n.trim().to_string(),
            _ => return
        };
        let mut map = clients.lock().await;
        if n.is_empty() || map.contains_key(&n) {
            let _ = out_tx.send("ERR name taken".into());
            continue;
        }
        map.insert(n.clone(), out_tx.clone());
        break n;
    };

    while let Ok(Some(line)) = lines.next_line().await {
        match parse(&line) {
            Cmd::Chat(text) => {
                let _ = out_tx.send("OK".into());
                for (_n, tx) in clients.lock().await.iter() {
                        let _ = tx.send(format!("EVT CHAT {name}: {text}"));
                }
            }
            Cmd::Quit => {
                let _ = out_tx.send("OK bye".into());
                break
            }
            Cmd::Unkown => {
                let _ = out_tx.send("ERR unkown command".into());
            }
            Cmd::Who => {
                let names = clients.lock().await.keys().cloned().collect::<Vec<_>>().join(", ");
                let _ = out_tx.send(format!("OK {names}"));
            }
        }
    }
    let mut map = clients.lock().await;
    map.remove(&name);
    for tx in map.values() {
        let _ = tx.send(format!("EVT leave {name}"));
    }
}