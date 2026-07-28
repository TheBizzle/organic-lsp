use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::lexer::doc_loc::DocLoc;
use crate::lsp::document::Document;
use tower_lsp_server::Client;

type Documents<'a> = Arc<RwLock<HashMap<DocLoc, Document<'a>>>>;

#[derive(Debug)]
pub struct LspBackend<'a> {
  pub client: Client,
  pub documents: Documents<'a>,
}

impl LspBackend<'_> {
  #[must_use]
  pub fn new(client: Client) -> Self {
    let documents = Arc::new(RwLock::new(HashMap::new()));
    Self { client, documents }
  }
}
