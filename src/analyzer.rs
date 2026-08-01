pub mod analysis;
pub mod function;
pub mod organic_type;
pub mod value;

mod builtins;
mod common;
mod expr_analyzer;
mod module_analyzer;
mod scope;

use crate::parser::ast::Module;

use analysis::{Analysis, AnalysisState};

#[must_use]
pub(super) fn analyze(module: Module) -> Analysis {
  let mut state = AnalysisState::default();
  module_analyzer::run(&mut state, module);
  state.analysis
}
