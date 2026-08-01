use std::collections::{HashMap, HashSet};

use crate::core::address::{NamedVarAddress, ScopeAddress};
use crate::core::diagnostics::{AnalyzerError, AnalyzerWarning};

use crate::lexer::token::Token;

use crate::analyzer::builtins::{BuiltIns, INITIAL_SCOPE_ADDRESS, initial_state};
use crate::analyzer::organic_type::OrganicType;
use crate::analyzer::scope::{Env, Scope};
use crate::analyzer::value::TermDefn;

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
pub enum NonVarToken {
  NamedArg(Token),
  Number(Token),
  String(Token),
}

impl NonVarToken {
  #[must_use]
  pub const fn line(&self) -> u32 {
    self.token().source_loc.line
  }

  #[must_use]
  pub const fn token(&self) -> &Token {
    match self {
      Self::NamedArg(token) | Self::Number(token) | Self::String(token) => token,
    }
  }
}

#[derive(Debug)]
pub struct Analysis {
  pub definitions: HashMap<NamedVarAddress, TermDefn>,
  pub defn_infos: HashMap<NamedVarAddress, DefnInfo>,
  pub diagnostics: Diagnostics,
  pub non_var_tokens: Vec<NonVarToken>,
  pub usages: HashMap<NamedVarAddress, HashSet<Token>>,
}

impl Analysis {
  #[must_use]
  pub(super) fn new(definitions: HashMap<NamedVarAddress, TermDefn>) -> Self {
    let usages: HashMap<_, _> = definitions.keys().map(|addr| (addr.clone(), HashSet::new())).collect();
    Self {
      definitions,
      defn_infos: HashMap::new(),
      diagnostics: Diagnostics { errors: Vec::new(), warnings: Vec::new() },
      non_var_tokens: Vec::new(),
      usages,
    }
  }
}

#[derive(Debug)]
pub(super) struct AnalysisState {
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
