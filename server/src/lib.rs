use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::spawn;
use ulci::server::{startup_server, Request, UlciResult};
use ulci::ClientInfo;

const PORT: u16 = 25565;

pub type ConnectionInfo = (Sender<Request>, Receiver<UlciResult>, ClientInfo);

fn run_client(
  connections: &Arc<Sender<ConnectionInfo>>,
  tx: Sender<Request>,
  rx: Receiver<UlciResult>,
) -> Option<()> {
  while let Ok(message) = rx.recv() {
    if let UlciResult::Startup(info) = message {
      connections.send((tx, rx, info)).ok()?;
      break;
    }
  }
  Some(())
}

fn handle_connection(stream: TcpStream, connections: Arc<Sender<ConnectionInfo>>) -> Option<()> {
  let name = if let Ok(ip) = stream.peer_addr() {
    println!("{ip} Connected");
    ip.to_string()
  } else {
    println!("Unknown Connected");
    "Unknown".to_string()
  };
  let stream_2 = stream.try_clone().ok()?;
  let (tx, rx) = channel();
  let (tx_2, rx_2) = channel();
  spawn(move || {
    startup_server(rx, &tx_2, BufReader::new(stream), stream_2, false, || ());
    println!("{name} Disconnected");
  });
  spawn(move || run_client(&connections, tx, rx_2));
  Some(())
}

pub fn handle_connections(connections: Sender<ConnectionInfo>) {
  let connections = Arc::new(connections);
  let listener = TcpListener::bind(format!("0.0.0.0:{PORT}"))
    .unwrap_or_else(|_| panic!("Failed to bind to port {PORT}"));

  for stream in listener.incoming().flatten() {
    let connections = connections.clone();
    if handle_connection(stream, connections).is_none() {
      println!("try_clone broke");
    }
  }
}
