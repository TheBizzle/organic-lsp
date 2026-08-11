use std::env;
use std::net::SocketAddr;

use tokio::io::{stdin, stdout};
use tokio::net::TcpListener;

use tower_lsp_server::Server;

use organic_lsp::lsp::new_lsp;

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();

  if args.iter().any(|str| str == "debug") {
    debug_server().await;
  } else {
    prod_server().await;
  }
}

async fn prod_server() {
  let stdin = stdin();
  let stdout = stdout();

  let (service, socket) = new_lsp();

  Server::new(stdin, stdout, socket).serve(service).await;
}

async fn debug_server() {
  let addr: SocketAddr = "127.0.0.1:9257".parse().unwrap();
  let listener = TcpListener::bind(addr).await.unwrap();

  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let (read, write) = tokio::io::split(stream);
    let (service, socket) = new_lsp();
    Server::new(read, write, socket).serve(service).await;
  }
}
