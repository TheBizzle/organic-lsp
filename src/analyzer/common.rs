use crate::lexer::token::Token;

use crate::core::diagnostics::{AnalyzerError, AnalyzerErrorType, AnalyzerWarning, AnalyzerWarningType};

use crate::analyzer::address::NamedVarAddress;
use crate::analyzer::analysis::AnalysisState;
use crate::analyzer::organic_type::OrganicType as OT;

pub(super) fn resolve_addr(state: &mut AnalysisState, name: &str) -> Option<NamedVarAddress> {
  for scope in state.scopes.iter_mut().rev() {
    if let Some(addr) = scope.env.bindings.get(name) {
      return Some(addr.clone());
    }
  }
  None
}

pub(super) fn resolve_type<'a>(state: &AnalysisState<'a>, addr: &NamedVarAddress) -> OT<'a> {
  state
    .vars
    .get(addr)
    .map_or_else(|| panic!("Invalid LSP state!  Known `{addr:?}` lacked a binding!"), Clone::clone)
}

pub(super) fn push_warning(state: &mut AnalysisState, token: Token, typ: AnalyzerWarningType) {
  state.analysis.warnings.push(AnalyzerWarning { typ, offender: token });
}

pub(super) fn push_error<'a>(state: &mut AnalysisState<'a>, token: Token, typ: AnalyzerErrorType<'a>) {
  state.analysis.errors.push(AnalyzerError { typ, offender: token });
}
