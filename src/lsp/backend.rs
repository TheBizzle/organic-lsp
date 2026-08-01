use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use tower_lsp_server::Client;

use crate::core::doc_loc::DocLoc;

use crate::lsp::document::Document;

type Documents = Arc<RwLock<HashMap<DocLoc, Document>>>;

#[derive(Debug)]
pub struct LspBackend {
  pub client: Client,
  pub documents: Documents,
}

impl LspBackend {
  #[must_use]
  pub fn new(client: Client) -> Self {
    let documents = Arc::new(RwLock::new(HashMap::new()));
    Self { client, documents }
  }
}
