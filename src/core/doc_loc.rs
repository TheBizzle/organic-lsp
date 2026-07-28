#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocLoc(String);

impl DocLoc {
  pub fn new(value: impl Into<String>) -> Self {
    Self(value.into())
  }

  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}
