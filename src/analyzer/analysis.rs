use std::collections::{HashMap, HashSet};

use crate::core::diagnostics::{AnalyzerError, AnalyzerWarning};

use crate::lexer::token::Token;

use crate::analyzer::builtins::{BuiltIns, INITIAL_SCOPE_ADDRESS, initial_state};
use crate::analyzer::organic_type::OrganicType;
use crate::analyzer::scope::{Env, Scope};
use crate::analyzer::value::TermDefn;
use crate::core::address::{NamedVarAddress, ScopeAddress};

#[derive(Debug, Eq, PartialEq)]
pub enum HighlightingType {
  Function,
  Parameter,
  Variable,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DefnInfo {
  pub hl_type: HighlightingType,
  pub token: Token,
}

#[derive(Debug)]
pub struct Diagnostics {
  pub errors: Vec<AnalyzerError>,
  pub warnings: Vec<AnalyzerWarning>,
}

#[derive(Debug)]
pub struct Analysis {
  pub definitions: HashMap<NamedVarAddress, TermDefn>,
  pub defn_infos: HashMap<NamedVarAddress, DefnInfo>,
  pub diagnostics: Diagnostics,
  pub named_arg_tokens: Vec<Token>,
  pub number_tokens: Vec<Token>,
  pub string_tokens: Vec<Token>,
  pub usages: HashMap<NamedVarAddress, HashSet<Token>>,
}

impl Analysis {
  #[must_use]
  pub fn new(definitions: HashMap<NamedVarAddress, TermDefn>) -> Self {
    let usages: HashMap<_, _> = definitions.keys().map(|addr| (addr.clone(), HashSet::new())).collect();
    Self {
      definitions,
      defn_infos: HashMap::new(),
      diagnostics: Diagnostics { errors: Vec::new(), warnings: Vec::new() },
      named_arg_tokens: Vec::new(),
      number_tokens: Vec::new(),
      string_tokens: Vec::new(),
      usages,
    }
  }
}

#[derive(Debug)]
pub struct AnalysisState {
  pub analysis: Analysis,
  pub initting_var_opt: Option<String>,
  pub last_scope_addr: ScopeAddress,
  pub scopes: Vec<Scope>,
  pub vars: HashMap<NamedVarAddress, OrganicType>,
}

impl Default for AnalysisState {
  fn default() -> Self {
    let BuiltIns { bindings, defs, vars } = initial_state();
    Self {
      analysis: Analysis::new(defs),
      initting_var_opt: None,
      last_scope_addr: INITIAL_SCOPE_ADDRESS.clone(),
      scopes: vec![Scope { env: Env { bindings }, address: INITIAL_SCOPE_ADDRESS.clone() }],
      vars,
    }
  }
}
