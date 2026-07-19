use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rangemap::RangeMap;

use tokio::sync::RwLock;

use tower_lsp_server::Client;
use tower_lsp_server::ls_types::Diagnostic;

use crate::analyzer::address::NamedVarAddress;
use crate::analyzer::analysis::DefnInfo;
use crate::analyzer::value::TermDefn;
use crate::lexer::doc_loc::DocLoc;
use crate::lexer::token::Token;

type Diagnostics = Vec<Diagnostic>;
type Documents<'a> = Arc<RwLock<HashMap<DocLoc, Document<'a>>>>;

#[derive(Debug, PartialEq)]
pub struct LValueInfo<'a> {
  pub definition: TermDefn<'a>,
  pub defn_info_opt: Option<DefnInfo>,
  pub usages: HashSet<Token>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Entity {
  LValue { addr: NamedVarAddress },
  NamedArg,
  NumberLiteral,
  StringLiteral,
}

#[derive(Debug)]
pub struct Document<'a> {
  pub contents: String,
  pub diagnostics: Diagnostics,
  pub entities: Vec<RangeMap<u32, Entity>>,
  pub infos: HashMap<NamedVarAddress, Arc<LValueInfo<'a>>>,
}

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
