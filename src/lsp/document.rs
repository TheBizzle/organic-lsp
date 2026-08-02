use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ordered_float::NotNan;

use rangemap::RangeMap;

use tower_lsp_server::ls_types::Diagnostic;

use crate::core::address::NamedVarAddress;

use crate::lexer::token::Token;

use crate::analyzer::analysis::DefnInfo;
use crate::analyzer::function::Function;
use crate::analyzer::value::TermDefn;

#[derive(Debug, PartialEq)]
pub struct LValueInfo {
  pub definition: TermDefn,
  pub defn_info_opt: Option<DefnInfo>,
  pub usages: HashSet<Token>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Entity {
  LValue { addr: NamedVarAddress },
  NamedArg { name: String, func_addr: NamedVarAddress, func: Arc<Function> },
  NumberLiteral(NotNan<f64>),
  StringLiteral(String),
}

#[derive(Debug)]
pub struct Document {
  pub contents: String,
  pub diagnostics: Vec<Diagnostic>,
  pub entities: Vec<RangeMap<u32, Entity>>,
  pub infos: HashMap<NamedVarAddress, Arc<LValueInfo>>,
}
