use std::collections::{HashMap, HashSet};

use crate::errors::{AnalyzerError, AnalyzerWarning};

use crate::lexer::token::Token;

use crate::analyzer::address::{NamedVarAddress, ScopeAddress};
use crate::analyzer::builtins::{INITIAL_SCOPE_ADDRESS, initial_state};
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
pub struct Analysis<'a> {
  pub definitions: HashMap<NamedVarAddress, TermDefn<'a>>,
  pub defn_infos: HashMap<NamedVarAddress, DefnInfo>,
  pub errors: Vec<AnalyzerError<'a>>,
  pub named_arg_tokens: Vec<Token>,
  pub number_tokens: Vec<Token>,
  pub string_tokens: Vec<Token>,
  pub usages: HashMap<NamedVarAddress, HashSet<Token>>,
  pub warnings: Vec<AnalyzerWarning>,
}

impl<'a> Analysis<'a> {
  #[must_use]
  pub fn new(definitions: HashMap<NamedVarAddress, TermDefn<'a>>) -> Self {
    let usages: HashMap<_, _> = definitions.keys().map(|addr| (addr.clone(), HashSet::new())).collect();
    Self {
      definitions,
      defn_infos: HashMap::new(),
      errors: Vec::new(),
      named_arg_tokens: Vec::new(),
      number_tokens: Vec::new(),
      string_tokens: Vec::new(),
      usages,
      warnings: Vec::new(),
    }
  }
}

#[derive(Debug)]
pub struct AnalysisState<'a> {
  pub analysis: Analysis<'a>,
  pub initting_var_opt: Option<String>,
  pub last_scope_addr: ScopeAddress,
  pub scopes: Vec<Scope>,
  pub vars: HashMap<NamedVarAddress, OrganicType<'a>>,
}

impl Default for AnalysisState<'_> {
  fn default() -> Self {
    let (bindings, vars, defs) = initial_state();
    Self {
      analysis: Analysis::new(defs),
      initting_var_opt: None,
      last_scope_addr: INITIAL_SCOPE_ADDRESS.clone(),
      scopes: vec![Scope { env: Env { bindings }, address: INITIAL_SCOPE_ADDRESS.clone() }],
      vars,
    }
  }
}
