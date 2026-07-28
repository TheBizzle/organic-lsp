use crate::analyzer::organic_type::OrganicType;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Function {
  pub params: Vec<ParamInfo>,
  pub return_type: OrganicType,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ParamInfo(pub String, pub OrganicType, pub bool); // bool: hasDefault

impl Function {
  #[must_use]
  pub const fn arity(&self) -> u64 {
    self.params.len() as u64
  }
}
