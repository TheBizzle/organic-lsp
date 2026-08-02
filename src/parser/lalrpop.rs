use crate::lexer::token::Token;
use crate::lexer::token::TokenType::Identifier;

use crate::parser::ast::{Arg, Expr, Formal, Symbol};
use crate::parser::diagnostics::ParserError;

#[derive(Debug)]
pub(super) struct Param {
  pub symbol: Symbol,
  pub value_option: Option<Expr>,
}

/// # Errors
/// When a parameter does not have an accompanying value
pub(super) fn as_args(params: Vec<Param>) -> Result<Vec<Arg>, ParserError> {
  params
    .into_iter()
    .map(|param| {
      if let Some(value) = param.value_option {
        Ok(Arg { name: param.symbol, value })
      } else {
        Err(ParserError::MissingParameterValue { token: param.symbol.token })
      }
    })
    .collect()
}

/// # Errors
/// When a parameter does not have an accompanying value
pub(super) fn as_formals(params: Vec<Param>) -> Result<Vec<Formal>, ParserError> {
  params
    .into_iter()
    .map(|param| {
      if let Some(default) = param.value_option {
        Ok(Formal { name: param.symbol, default })
      } else {
        Err(ParserError::MissingParameterValue { token: param.symbol.token })
      }
    })
    .collect()
}

/// # Panics
/// When given a token that isn't an identifier
#[must_use]
pub(super) fn as_symbol(ident: &Token, descriptor: &str) -> Symbol {
  match &ident.token_type {
    Identifier(name) => Symbol { name: name.clone(), token: ident.clone() },
    x => panic!("Impossible {descriptor}: {x:?}"),
  }
}

pub(super) fn sequence<T>(head: T, tail: Vec<T>) -> Vec<T> {
  std::iter::once(head).chain(tail).collect()
}
