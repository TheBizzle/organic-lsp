pub mod address;
pub mod analysis;
pub mod builtins;
pub mod common;
pub mod expr_analyzer;
pub mod function;
pub mod module_analyzer;
pub mod organic_type;
pub mod scope;
pub mod value;

use crate::parser::ast::Module;

use analysis::{Analysis, AnalysisState};

#[must_use]
pub fn analyze(module: Module) -> Analysis {
  let mut state = AnalysisState::default();
  module_analyzer::run(&mut state, module);
  state.analysis
}
