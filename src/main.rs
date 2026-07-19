use std::net::SocketAddr;

use tokio::net::TcpListener;

use tower_lsp_server::{LspService, Server};

use organic_lsp::lsp::lsp_backend::LspBackend;

#[tokio::main]
async fn main() {
  let addr: SocketAddr = "127.0.0.1:9257".parse().unwrap();
  let listener = TcpListener::bind(addr).await.unwrap();

  loop {
    let (stream, _) = listener.accept().await.unwrap();
    let (read, write) = tokio::io::split(stream);
    let (service, socket) = LspService::new(LspBackend::new);
    Server::new(read, write, socket).serve(service).await;
  }
}
