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

use crate::analyzer;
use crate::analyzer::analysis::Analysis;
use crate::errors::LspError;
use crate::errors::ParserError::{self, ExtraToken, FictionalToken, UnexpectedEOF, WrongToken};
use crate::lexer::doc_loc::DocLoc;
use crate::lexer::lex;
use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::Module;
use crate::parser::grammar::ModuleParser;
use LspError::{LspLexerError, LspParserError};

pub fn analyze<'a>(doc_loc: &DocLoc, doc_text: &str) -> (Analysis<'a>, Vec<LspError<'a>>) {
  let (tokens, lerrors) = lex(doc_loc, doc_text);
  let lsp_lerrors = lerrors.into_iter().map(LspLexerError).collect();

  match parse(tokens) {
    Ok(module) => (analyzer::analyze(module), lsp_lerrors),
    Err(error) => {
      let lsp_all_errors = vec![LspParserError(error)].into_iter().chain(lsp_lerrors).collect();
      let dummy_module = Module { includes: Vec::new(), statements: Vec::new() };
      (analyzer::analyze(dummy_module), lsp_all_errors)
    },
  }
}

fn parse(tokens: Vec<Token>) -> Result<Module, ParserError> {
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
