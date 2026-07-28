use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::lexer::token::Token;

use crate::parser::ast::{Arg, Expr, Formal, FuncCall, FuncLiteral, Operator, Statement, Symbol};
use Operator::{Divide, GreaterOrEquals, GreaterThan, LessOrEquals, LessThan, Minus, Plus, Times};

use crate::core::diagnostics::AnalyzerErrorType::{
  DuplicateParameter, ExtraArgument, MissingArgument, NoSuchFn, NoSuchVariable, TypeMismatch,
  VarCannotInitInTermsOfSelf,
};

use crate::core::diagnostics::AnalyzerWarningType::{
  ArgOverridesPrevious, IntermediateCallInFnDef, UselessFnBody,
};

use crate::analyzer::address::{NamedVarAddress, ScopeAddress};
use crate::analyzer::analysis::{AnalysisState, DefnInfo, HighlightingType as HLT};
use crate::analyzer::common::{push_error, push_warning, resolve_addr, resolve_type};
use crate::analyzer::function::{Function, ParamInfo};
use crate::analyzer::module_analyzer::crawl_statement;
use crate::analyzer::organic_type::OrganicType as OT;
use crate::analyzer::scope::{Env, Scope};
use crate::analyzer::value::TermDefn::UserDefined;

pub(super) fn crawl_expr<'a>(state: &mut AnalysisState<'a>, expr: Expr) -> Option<OT<'a>> {
  match expr {
    Expr::Call { call, .. } => crawl_function_call(state, call),
    Expr::Function { value, .. } => crawl_function_def(state, value),
    Expr::Grouping { value, .. } => crawl_expr(state, *value),
    Expr::List { values, .. } => crawl_list(state, values),
    Expr::LValue { name, token } => crawl_lvalue(state, &name, token),
    Expr::Negated { value, token } => crawl_negated(state, *value, token),
    Expr::Number { token, .. } => {
      state.analysis.number_tokens.push(token);
      Some(OT::Number)
    },
    Expr::Op { left, operator, right, .. } => Some(crawl_op(state, *left, &operator, *right)),
    Expr::String { token, .. } => {
      state.analysis.string_tokens.push(token);
      Some(OT::String)
    },
  }
}

pub(super) fn crawl_function_call<'a>(state: &mut AnalysisState<'a>, fn_call: FuncCall) -> Option<OT<'a>> {
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
          let pram = ParamInfo(Cow::Owned(name.name), crawl_expr(state, value).unwrap_or(OT::Unknown), false);
          (mapping, pram)
        })
        .collect();

      match resolve_type(state, &addr) {
        OT::Function(func) => {
          state.analysis.usages.entry(addr).or_default().insert(token.clone());
          state.analysis.named_arg_tokens.extend(actual_tokens.values().cloned());

          let mut expecteds: HashMap<_, _> = func
            .params
            .clone()
            .into_iter()
            .map(|ParamInfo(name, typ, is_optional)| (name.into_owned(), (typ, is_optional)))
            .collect();

          let mut actuals: HashMap<_, _> = actual_args
            .into_iter()
            .map(|ParamInfo(name, typ, is_optional)| (name.into_owned(), (typ, is_optional)))
            .collect();

          let keys: HashSet<_> = expecteds.keys().cloned().chain(actuals.keys().cloned()).collect();

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
              (Some((expected, _)), Some((got, _))) => {
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

          Some(func.return_type.clone())
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

fn crawl_function_def<'a>(state: &mut AnalysisState<'a>, func: FuncLiteral) -> Option<OT<'a>> {
  let FuncLiteral { formals, body, .. } = func;

  let param_pairs = {
    let mut known_names = HashSet::new();

    formals
      .into_iter()
      .map(|Formal { name, default }| {
        if known_names.contains(&name.name) {
          push_error(state, name.token.clone(), DuplicateParameter);
        } else {
          known_names.insert(name.name.clone());
        }
        let typ = crawl_expr(state, default)?;
        Some((name.token, ParamInfo(Cow::Owned(name.name), typ, true)))
      })
      .collect::<Option<Vec<_>>>()?
  };

  let return_type = crawl_fn_body(state, body, &param_pairs);
  let params = param_pairs.into_iter().map(|(_, param)| param).collect();

  Some(OT::Function(Arc::new(Function { params, return_type })))
}

fn crawl_fn_body<'a>(
  state: &mut AnalysisState<'a>, mut body: Vec<Statement>, params: &Vec<(Token, ParamInfo<'a>)>,
) -> OT<'a> {
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

      for (token, ParamInfo(name_cow, typ, _)) in params {
        let name = name_cow.to_string();
        let my_addr = NamedVarAddress { name: name.clone(), scope_addr: address.clone() };
        let defn_info = DefnInfo { hl_type: HLT::Parameter, token: token.clone() };

        state.analysis.definitions.insert(my_addr.clone(), UserDefined { token: token.clone() });
        state.analysis.defn_infos.insert(my_addr.clone(), defn_info);
        state.analysis.usages.insert(my_addr.clone(), HashSet::from([token.clone()]));
        state.vars.insert(my_addr.clone(), typ.clone());
        bindings.insert(name, my_addr);
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

fn crawl_list<'a>(state: &mut AnalysisState<'a>, values: Vec<Expr>) -> Option<OT<'a>> {
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

fn crawl_lvalue<'a>(state: &mut AnalysisState<'a>, name: &Symbol, token: Token) -> Option<OT<'a>> {
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

fn crawl_negated<'a>(state: &mut AnalysisState<'a>, expr: Expr, token: Token) -> Option<OT<'a>> {
  let typ = crawl_expr(state, expr)?;
  if typ == OT::Number {
    Some(typ)
  } else {
    push_error(state, token, TypeMismatch { expected: OT::Number, got: typ });
    None
  }
}

fn crawl_op<'a>(state: &mut AnalysisState<'a>, left: Expr, op: &Operator, right: Expr) -> OT<'a> {
  let left_token = left.get_token();
  if let Some(got) = crawl_expr(state, left)
    && got != OT::Number
  {
    push_error(state, left_token, TypeMismatch { expected: OT::Number, got });
  }

  let right_token = right.get_token();
  if let Some(got) = crawl_expr(state, right)
    && got != OT::Number
  {
    push_error(state, right_token, TypeMismatch { expected: OT::Number, got });
  }

  match op {
    Plus | Minus | Times | Divide => OT::Number,
    LessThan | LessOrEquals | GreaterThan | GreaterOrEquals => OT::Boolean,
  }
}
