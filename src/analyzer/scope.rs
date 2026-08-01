use std::collections::HashMap;

use crate::core::address::{NamedVarAddress, ScopeAddress};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Env {
  pub bindings: HashMap<String, NamedVarAddress>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Scope {
  pub env: Env,
  pub address: ScopeAddress,
}
