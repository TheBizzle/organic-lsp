use lalrpop_util::ParseError as LALR;

pub mod ast;
pub mod lalrpop;

lalrpop_util::lalrpop_mod!(
  #[ allow( clippy::cast_sign_loss
          , clippy::cloned_instead_of_copied
          , clippy::default_trait_access
          , clippy::implicit_clone
          , clippy::match_same_arms
          , clippy::missing_const_for_fn
          , clippy::missing_errors_doc
          , clippy::must_use_candidate
          , clippy::option_if_let_else
          , clippy::redundant_pub_crate
          , clippy::too_many_lines
          , clippy::trivially_copy_pass_by_ref
          , clippy::unnecessary_wraps
          , clippy::unused_self
          , clippy::use_self
          )
  ]
  pub grammar, "/parser/grammar.rs"
);

use crate::core::diagnostics::ParserError::{self, ExtraToken, FictionalToken, UnexpectedEOF, WrongToken};
use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::Module;
use crate::parser::grammar::ModuleParser;

/// # Errors
///
/// When tokens are encountered that do not form valid Organic code.
pub fn parse(tokens: Vec<Token>) -> Result<Module, ParserError> {
  let parser = ModuleParser::new();
  let triples: Vec<_> = tokens
    .into_iter()
    .filter_map(|token| match token.token_type {
      TokenType::BlockComment | TokenType::Comment => None,
      _ => {
        let (start, end) = token.source_loc.as_minis();
        Some((start, token, end))
      },
    })
    .collect();

  parser.parse(triples).map_err(|err| match err {
    LALR::InvalidToken { location } => FictionalToken { location },
    LALR::UnrecognizedToken { token: (_start, token, _end), expected } => WrongToken { token, expected },
    LALR::UnrecognizedEof { location, expected } => UnexpectedEOF { location, expected },
    LALR::User { error } => error,
    LALR::ExtraToken { token: (_start, token, _end) } => ExtraToken { token },
  })
}
