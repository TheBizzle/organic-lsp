use std::collections::HashSet;

use crate::core::address::NamedVarAddress;

use crate::parser::ast::{Include, Module, Statement, VarDecl};

use crate::analyzer::analysis::{AnalysisState, DefnInfo, HighlightingType as HLT};
use crate::analyzer::common::{push_error, push_lint, resolve_addr};
use crate::analyzer::diagnostics::AnalyzerErrorType::DuplicateVar;
use crate::analyzer::diagnostics::AnalyzerLintType::{CamelCase, SnakeCase};
use crate::analyzer::expr_analyzer::{crawl_expr, crawl_function_call};
use crate::analyzer::organic_type::OrganicType as OT;
use crate::analyzer::value::TermDefn::UserDefined;

pub(super) fn run(state: &mut AnalysisState, module: Module) {
  module.includes.into_iter().for_each(|include| crawl_include(state, &include));
  module.statements.into_iter().for_each(|statement| crawl_statement(state, statement));
}

const fn crawl_include(_state: &mut AnalysisState, include: &Include) {
  let Include { path: _ } = include;
  // TODO: Import foreign terms into namespace
}

pub(super) fn crawl_statement(state: &mut AnalysisState, statement: Statement) {
  match statement {
    Statement::FunctionCall(fn_call) => {
      crawl_function_call(state, *fn_call);
    },
    Statement::VariableDecl(var_decl) => crawl_var_decl(state, var_decl),
  }
}

fn crawl_var_decl(state: &mut AnalysisState, var_decl: VarDecl) {
  let my_addr =
    NamedVarAddress { name: var_decl.name.name.clone(), scope_addr: state.last_scope_addr.clone() };

  let name = var_decl.name.name.clone();
  if name.chars().skip(1).take(name.chars().count().saturating_sub(2)).any(|c| c == '_') {
    push_lint(state, var_decl.name.token.clone(), SnakeCase);
  } else if name.chars().any(char::is_uppercase) {
    push_lint(state, var_decl.name.token.clone(), CamelCase);
  }

  if resolve_addr(state, var_decl.name.name.as_str()).is_some() {
    push_error(state, var_decl.name.token.clone(), DuplicateVar);
  } else {
    state.analysis.definitions.insert(my_addr.clone(), UserDefined { token: var_decl.name.token.clone() });
    state.analysis.usages.insert(my_addr.clone(), HashSet::from([var_decl.name.token.clone()]));
  }

  let prev = Option::replace(&mut state.initting_var_opt, var_decl.name.name.clone());
  if let Some(typ) = crawl_expr(state, var_decl.init) {
    let hl_type = match typ {
      OT::Function(_) => HLT::Function,
      _ => HLT::Variable,
    };
    let defn_info = DefnInfo { hl_type, organic_type: typ.clone(), token: var_decl.name.token };
    state.scopes.last_mut().unwrap().env.bindings.insert(var_decl.name.name.clone(), my_addr.clone());
    state.analysis.defn_infos.insert(my_addr.clone(), defn_info);
    state.vars.insert(my_addr, typ);
  }
  state.initting_var_opt = prev;
}
