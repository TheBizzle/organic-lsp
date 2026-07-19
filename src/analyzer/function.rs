use std::borrow::Cow;

use crate::analyzer::organic_type::OrganicType;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Function<'a> {
  pub params: Vec<ParamInfo<'a>>,
  pub return_type: OrganicType<'a>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ParamInfo<'a>(pub Cow<'a, str>, pub OrganicType<'a>, pub bool); // bool: hasDefault

impl Function<'_> {
  #[must_use]
  pub const fn arity(&self) -> u64 {
    self.params.len() as u64
  }
}
