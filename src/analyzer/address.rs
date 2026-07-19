#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct NamedVarAddress {
  pub name: String,
  pub scope_addr: ScopeAddress,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd)]
pub struct ScopeAddress {
  pub n: u64,
}
