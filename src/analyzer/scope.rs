use std::collections::HashMap;

use crate::analyzer::address::{NamedVarAddress, ScopeAddress};

#[derive(Debug, Eq, PartialEq)]
pub struct Env {
  pub bindings: HashMap<String, NamedVarAddress>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Scope {
  pub env: Env,
  pub address: ScopeAddress,
}
