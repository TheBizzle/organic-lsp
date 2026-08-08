use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::address::{NamedVarAddress, ScopeAddress};

use crate::analyzer::diagnostics::AnalyzerErrorType::{
  DuplicateParameter, ExtraArgument, MissingArgument, NoSuchFn, NoSuchVariable, TypeMismatch,
  VarCannotInitInTermsOfSelf,
};

use crate::analyzer::diagnostics::AnalyzerWarningType::{
  ArgOverridesPrevious, IntermediateCallInFnDef, UselessFnBody,
};

use crate::lexer::token::Token;

use crate::parser::ast::{Arg, Expr, Formal, FuncCall, FuncLiteral, Operator, Statement, Symbol};
use Operator::{Divide, Equals, GreaterOrEquals, GreaterThan, LessOrEquals, LessThan, Minus, Plus, Times};

use crate::analyzer::analysis::{AnalysisState, DefnInfo, HighlightingType as HLT, NonVarToken};
use crate::analyzer::common::{push_error, push_warning, resolve_addr, resolve_type};
use crate::analyzer::function::{Function, ParamInfo};
use crate::analyzer::module_analyzer::crawl_statement;
use crate::analyzer::organic_type::OrganicType as OT;
use crate::analyzer::scope::{Env, Scope};
use crate::analyzer::value::TermDefn::UserDefined;

pub(super) fn crawl_expr(state: &mut AnalysisState, expr: Expr) -> Option<OT> {
  match expr {
    Expr::Call { call, .. } => crawl_function_call(state, call),
    Expr::Function { value, .. } => crawl_function_def(state, value),
    Expr::Grouping { value, .. } => crawl_expr(state, *value),
    Expr::List { values, .. } => crawl_list(state, values),
    Expr::LValue { name, token, .. } => crawl_lvalue(state, &name, token),
    Expr::Negated { value, token, .. } => crawl_negated(state, *value, token),
    Expr::Number { value, token, .. } => {
      state.analysis.non_var_tokens.push(NonVarToken::Number(token, value));
      Some(OT::Number)
    },
    Expr::Op { left, operator, right, .. } => Some(crawl_op(state, *left, &operator, *right)),
    Expr::String { value, token, .. } => {
      state.analysis.non_var_tokens.push(NonVarToken::String(token, value));
      Some(OT::String)
    },
  }
}

pub(super) fn crawl_function_call(state: &mut AnalysisState, fn_call: FuncCall) -> Option<OT> {
  let FuncCall { func: Symbol { name, token }, args } = fn_call;

  match resolve_addr(state, &name) {
    None => {
      push_error(state, token, NoSuchFn);
      None
    },

    Some(addr) => {
      {
        let mut seen_names = HashSet::<String>::new();
        for arg in &args {
          if seen_names.contains(&arg.name.name) {
            push_warning(state, arg.name.token.clone(), ArgOverridesPrevious);
          } else {
            seen_names.insert(arg.name.name.clone());
          }
        }
      }

      let (mut actual_tokens, actual_args): (HashMap<_, _>, Vec<_>) = args
        .into_iter()
        .map(|Arg { name, value }| {
          let mapping = (name.name.clone(), name.token.clone());
          let pram = ParamInfo(name.name, crawl_expr(state, value).unwrap_or(OT::Unknown), false);
          (mapping, pram)
        })
        .collect();

      match resolve_type(state, &addr) {
        OT::Function(func) => {
          crawl_verified_function_call(state, &token, &addr, &mut actual_tokens, actual_args, &func)
        },
        expected => {
          let func = Function { params: actual_args, return_type: OT::Unknown };
          let got = OT::Function(Arc::new(func));
          push_error(state, token, TypeMismatch { expected, got });
          None
        },
      }
    },
  }
}

fn crawl_verified_function_call(
  state: &mut AnalysisState, token: &Token, addr: &NamedVarAddress,
  actual_tokens: &mut HashMap<String, Token>, actual_args: Vec<ParamInfo>, func: &Arc<Function>,
) -> Option<OT> {
  state.analysis.usages.entry(addr.clone()).or_default().insert(token.clone());
  let named_nvts: Vec<_> = actual_tokens
    .values()
    .cloned()
    .map(|token| NonVarToken::NamedArg(token, addr.clone(), func.clone()))
    .collect();
  state.analysis.non_var_tokens.extend(named_nvts);

  let mut expecteds: HashMap<_, _> = func
    .params
    .clone()
    .into_iter()
    .map(|ParamInfo(name, typ, is_optional)| (name, (typ, is_optional)))
    .collect();

  let mut actuals: HashMap<_, _> =
    actual_args.into_iter().map(|ParamInfo(name, typ, is_optional)| (name, (typ, is_optional))).collect();

  let keys: HashSet<_> = expecteds.keys().cloned().chain(actuals.keys().cloned()).collect();

  let mut generic_bindings: HashMap<String, OT> = HashMap::new();

  for key in keys {
    match (expecteds.remove(&key), actuals.remove(&key)) {
      (None, None) => panic!("Invalid state!  Neither \"expected\" nor \"got\" had key \"{key}\"!"),
      (Some((_, true)), None) => { /* Optional and not supplied */ },
      (Some((typ, false)), None) => {
        push_error(state, token.clone(), MissingArgument { name: key, typ });
      },
      (None, Some(_)) => {
        push_error(state, token.clone(), ExtraArgument { name: key });
      },
      (Some((base_expected, _)), Some((got, _))) => {
        let expected = find_generic_bindings(&mut generic_bindings, base_expected, &got);

        if expected != got {
          if let Some(actual_token) = actual_tokens.remove(&key) {
            push_error(state, actual_token, TypeMismatch { expected, got });
          } else {
            panic!("Invalid state!  Tried to look up a token that didn't exist: \"{key}\"")
          }
        }
      },
    }
  }

  consume_generic_binding(&mut generic_bindings, &func.return_type)
}

fn find_generic_bindings(bindings: &mut HashMap<String, OT>, base: OT, got: &OT) -> OT {
  match base {
    OT::Generic(name) if let Some(existing_binding) = bindings.get(&name) => existing_binding.clone(),
    OT::Generic(name) => {
      bindings.insert(name, got.clone());
      got.clone()
    },
    OT::List(inner) if let OT::List(inner_got) = got => {
      OT::List(Box::new(find_generic_bindings(bindings, *inner, inner_got)))
    },
    _ => base,
  }
}

fn consume_generic_binding(bindings: &mut HashMap<String, OT>, typ: &OT) -> Option<OT> {
  match typ {
    OT::Generic(name) => bindings.remove(name),
    OT::List(inner) => consume_generic_binding(bindings, inner).map(|t| OT::List(Box::new(t))),
    _ => Some(typ.clone()),
  }
}

fn crawl_function_def(state: &mut AnalysisState, func: FuncLiteral) -> Option<OT> {
  let FuncLiteral { formals, body, .. } = func;

  let param_quartets = {
    let mut known_names = HashSet::new();

    formals
      .into_iter()
      .map(|Formal { name, default }| {
        if known_names.contains(&name.name) {
          push_error(state, name.token.clone(), DuplicateParameter);
        } else {
          known_names.insert(name.name.clone());
        }
        let start = default.get_start();
        let end = default.get_end();
        let typ = crawl_expr(state, default)?;
        Some((name.token, start, end, ParamInfo(name.name, typ, true)))
      })
      .collect::<Option<Vec<_>>>()?
  };

  let return_type = crawl_fn_body(state, body, &param_quartets);
  let params = param_quartets.into_iter().map(|(_, _, _, param)| param).collect();

  Some(OT::Function(Arc::new(Function { params, return_type })))
}

fn crawl_fn_body(
  state: &mut AnalysisState, mut body: Vec<Statement>, params: &Vec<(Token, Token, Token, ParamInfo)>,
) -> OT {
  if let Some(last) = body.pop() {
    let statements = body;
    let (decls, fn_calls): (Vec<_>, Vec<_>) =
      statements.into_iter().fold((Vec::new(), Vec::new()), |(mut ds, mut fcs), stmt| {
        match stmt {
          Statement::FunctionCall(fn_call) => fcs.push(*fn_call),
          Statement::VariableDecl(decl) => ds.push(decl),
        }
        (ds, fcs)
      });

    for fn_call in fn_calls {
      push_warning(state, fn_call.func.token.clone(), IntermediateCallInFnDef);
    }

    let address = ScopeAddress { n: state.last_scope_addr.n + 1 };

    let env = {
      let mut bindings = HashMap::new();

      for (token, start, end, ParamInfo(name, typ, _)) in params {
        let my_addr = NamedVarAddress { name: name.clone(), scope_addr: address.clone() };
        let defn_info = DefnInfo { hl_type: HLT::Parameter, organic_type: typ.clone(), token: token.clone() };
        state.analysis.defn_infos.insert(my_addr.clone(), defn_info);

        let defn = UserDefined { token: token.clone(), start: start.clone(), end: end.clone() };
        state.analysis.definitions.insert(my_addr.clone(), defn);

        state.analysis.usages.insert(my_addr.clone(), HashSet::from([token.clone()]));
        state.vars.insert(my_addr.clone(), typ.clone());
        bindings.insert(name.clone(), my_addr);
      }

      Env { bindings }
    };

    let new_scope = Scope { env, address: address.clone() };
    state.scopes.push(new_scope);
    state.last_scope_addr = address;

    for decl in decls {
      crawl_statement(state, Statement::VariableDecl(decl));
    }

    let out = match last {
      Statement::FunctionCall(fn_call) => crawl_function_call(state, *fn_call).unwrap_or(OT::Unknown),
      Statement::VariableDecl(decl) => {
        push_warning(state, decl.name.token.clone(), UselessFnBody);
        crawl_statement(state, Statement::VariableDecl(decl));
        OT::Unknown
      },
    };

    assert!(!state.scopes.is_empty(), "Critical error!  You should never be able to pop the global scope!");
    state.scopes.pop();

    out
  } else {
    OT::Unknown
  }
}

fn crawl_list(state: &mut AnalysisState, values: Vec<Expr>) -> Option<OT> {
  let mut types = values.into_iter().map(|value| crawl_expr(state, value)).collect::<Option<Vec<_>>>()?;
  if types.is_empty() {
    Some(OT::List(Box::new(OT::Unknown)))
  } else {
    let first = types.remove(0);
    if types.iter().all(|x| &first == x) {
      Some(OT::List(Box::new(first)))
    } else {
      None
    }
  }
}

fn crawl_lvalue(state: &mut AnalysisState, name: &Symbol, token: Token) -> Option<OT> {
  if Some(name.name.clone()) == state.initting_var_opt {
    push_error(state, token.clone(), VarCannotInitInTermsOfSelf);
  }

  if let Some(addr) = resolve_addr(state, &name.name) {
    state
      .analysis
      .usages
      .get_mut(&addr)
      .unwrap_or_else(|| panic!("Usages must get initialized for: {}", name.name))
      .insert(token);
    Some(resolve_type(state, &addr))
  } else {
    push_error(state, token, NoSuchVariable);
    None
  }
}

fn crawl_negated(state: &mut AnalysisState, expr: Expr, token: Token) -> Option<OT> {
  let typ = crawl_expr(state, expr)?;
  if typ == OT::Number {
    Some(typ)
  } else {
    push_error(state, token, TypeMismatch { expected: OT::Number, got: typ });
    None
  }
}

#[rustfmt::skip]
fn crawl_op(state: &mut AnalysisState, left: Expr, op: &Operator, right: Expr) -> OT {
  let left_token = left.get_token();
  let right_token = right.get_token();

  let ltype_opt = crawl_expr(state, left);
  let rtype_opt = crawl_expr(state, right);

  if op == &Equals {
    if let Some(expected) = ltype_opt && let Some(got) = rtype_opt && expected != got {
      push_error(state, right_token, TypeMismatch { expected, got });
    }
  } else {
    if let Some(got) = ltype_opt && got != OT::Number {
      push_error(state, left_token, TypeMismatch { expected: OT::Number, got });
    }
    if let Some(got) = rtype_opt && got != OT::Number {
      push_error(state, right_token, TypeMismatch { expected: OT::Number, got });
    }
  }

  match op {
    Plus | Minus | Times | Divide => OT::Number,
    Equals | GreaterThan | GreaterOrEquals | LessThan | LessOrEquals => OT::Boolean,
  }
}
