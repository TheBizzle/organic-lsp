pub mod analysis;
pub mod diagnostics;
pub mod function;
pub mod organic_type;
pub mod value;

mod builtins;
mod common;
mod expr_analyzer;
mod module_analyzer;
mod scope;

use std::collections::HashMap;

use crate::parser::ast::Module;

use analysis::{Analysis, AnalysisState};
use builtins::{CONSTANTS, NOTES};
use organic_type::OrganicType;

#[must_use]
pub(super) fn analyze(module: Module) -> Analysis {
  let mut state = AnalysisState::default();
  module_analyzer::run(&mut state, module);
  state.analysis
}

pub(super) fn constants() -> HashMap<String, OrganicType> {
  CONSTANTS.iter().map(|cnst| (cnst.name.to_string(), cnst.value.as_type())).collect()
}

pub(super) fn note_names() -> Vec<String> {
  NOTES.keys().cloned().collect()
}
