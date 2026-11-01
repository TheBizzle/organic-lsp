use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ordered_float::NotNan;

use crate::core::address::{NamedVarAddress, ScopeAddress};

use crate::lexer::token::Token;

use crate::analyzer::builtins::{BuiltIns, INITIAL_SCOPE_ADDRESS, initial_state};
use crate::analyzer::diagnostics::AnalyzerDiagnostic;
use crate::analyzer::function::Function;
use crate::analyzer::organic_type::OrganicType as OT;
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
  pub organic_type: OT,
  pub token: Token,
}

#[derive(Debug)]
pub enum NonVarToken {
  NamedArg(Token, NamedVarAddress, Arc<Function>),
  Number(Token, NotNan<f64>),
  String(Token, String),
}

impl NonVarToken {
  #[must_use]
  pub const fn line(&self) -> u32 {
    self.token().source_loc.line
  }

  #[must_use]
  pub const fn token(&self) -> &Token {
    match self {
      Self::NamedArg(token, _, _) | Self::Number(token, _) | Self::String(token, _) => token,
    }
  }
}

#[derive(Debug)]
pub struct Analysis {
  pub definitions: HashMap<NamedVarAddress, TermDefn>,
  pub defn_infos: HashMap<NamedVarAddress, DefnInfo>,
  pub diagnostics: Vec<AnalyzerDiagnostic>,
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
      diagnostics: Vec::new(),
      non_var_tokens: Vec::new(),
      usages,
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum FillableState {
  Absent,
  InFunction,
  InArg,
}

#[derive(Debug)]
pub(super) struct AnalysisState {
  pub analysis: Analysis,
  pub generic_bindings: HashMap<String, OT>,
  pub granulate_fillable_state: FillableState,
  pub map_fillable_state: FillableState,
  pub oscillator_fillable_state: FillableState,
  pub initting_var_opt: Option<String>,
  pub last_scope_addr: ScopeAddress,
  pub scopes: Vec<Scope>,
  pub vars: HashMap<NamedVarAddress, OT>,
}

impl Default for AnalysisState {
  fn default() -> Self {
    let BuiltIns { bindings, defs, vars } = initial_state();
    Self {
      analysis: Analysis::new(defs),
      generic_bindings: HashMap::new(),
      granulate_fillable_state: FillableState::Absent,
      map_fillable_state: FillableState::Absent,
      oscillator_fillable_state: FillableState::Absent,
      initting_var_opt: None,
      last_scope_addr: INITIAL_SCOPE_ADDRESS.clone(),
      scopes: vec![Scope { env: Env { bindings }, address: INITIAL_SCOPE_ADDRESS.clone() }],
      vars,
    }
  }
}

impl AnalysisState {
  pub fn bind_generic(&mut self, base: OT, got: &OT) -> OT {
    match (base, got) {
      (OT::Generic(name), _) if let Some(existing_binding) = self.generic_bindings.get(&name) => {
        existing_binding.clone()
      },
      (OT::Generic(name), other) => {
        self.generic_bindings.insert(name, other.clone());
        got.clone()
      },
      (other, OT::Generic(name)) => {
        self.generic_bindings.insert(name.clone(), other);
        got.clone()
      },
      (OT::List(inner), _) if let OT::List(inner_got) = got => {
        OT::List(Box::new(self.bind_generic(*inner, inner_got)))
      },
      (x, _) => x,
    }
  }

  pub fn consume_generic_binding(&mut self, typ: &OT) -> Option<OT> {
    match typ {
      OT::Generic(name) => self.generic_bindings.remove(name),
      OT::List(inner) => self.consume_generic_binding(inner).map(|t| OT::List(Box::new(t))),
      _ => Some(typ.clone()),
    }
  }
}
