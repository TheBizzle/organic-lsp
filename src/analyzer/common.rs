use crate::core::address::NamedVarAddress;

use crate::lexer::token::Token;

use crate::analyzer::analysis::AnalysisState;
use crate::analyzer::diagnostics::AnalyzerDiagnosticType::{AnalyzerError, AnalyzerWarning};
use crate::analyzer::diagnostics::{AnalyzerDiagnostic, AnalyzerErrorType, AnalyzerWarningType};
use crate::analyzer::organic_type::OrganicType;

pub(super) fn resolve_addr(state: &mut AnalysisState, name: &str) -> Option<NamedVarAddress> {
  for scope in state.scopes.iter_mut().rev() {
    if let Some(addr) = scope.env.bindings.get(name) {
      return Some(addr.clone());
    }
  }
  None
}

pub(super) fn resolve_type(state: &AnalysisState, addr: &NamedVarAddress) -> OrganicType {
  state
    .vars
    .get(addr)
    .map_or_else(|| panic!("Invalid LSP state!  Known `{addr:?}` lacked a binding!"), Clone::clone)
}

pub(super) fn push_warning(state: &mut AnalysisState, token: Token, aw_typ: AnalyzerWarningType) {
  state.analysis.diagnostics.push(AnalyzerDiagnostic { typ: AnalyzerWarning(aw_typ), offender: token });
}

pub(super) fn push_error(state: &mut AnalysisState, token: Token, ae_typ: AnalyzerErrorType) {
  state.analysis.diagnostics.push(AnalyzerDiagnostic { typ: AnalyzerError(ae_typ), offender: token });
}
