use std::collections::HashMap;
use std::sync::Arc;

use rangemap::RangeMap;

use tower_lsp_server::ls_types::Uri;

use crate::core::address::NamedVarAddress;
use crate::core::diagnostics::{LspError, error_as_diagnostic, warning_as_diagnostic};
use crate::core::doc_loc::DocLoc;

use crate::lexer::lex;
use crate::lexer::source_loc::SourceLoc;

use crate::parser::ast::Module;
use crate::parser::parse;

use crate::analyzer::analysis::{Analysis, Diagnostics, NonVarToken};
use crate::analyzer::analyze;

use crate::lsp::backend::LspBackend;
use crate::lsp::common::source_loc_to_range;
use crate::lsp::document::{Document, Entity, LValueInfo};

use LspError::{LspAnalyzerError, LspLexerError, LspParserError};

pub(super) async fn store_and_reanalyze(this: &LspBackend, uri: Uri, text: String) {
  let doc_loc = DocLoc::new(uri.to_string());

  let (
    Analysis {
      definitions,
      mut defn_infos,
      diagnostics: Diagnostics { errors, warnings },
      non_var_tokens,
      usages,
    },
    pre_errors,
  ) = {
    let (tokens, lerrors) = lex(&doc_loc, &text);
    let lsp_lerrors: Vec<_> = lerrors.into_iter().map(LspLexerError).collect();

    match parse(tokens) {
      Ok(module) => (analyze(module), lsp_lerrors),
      Err(error) => {
        let lsp_all_errors = vec![LspParserError(error)].into_iter().chain(lsp_lerrors).collect();
        let dummy_module = Module { includes: Vec::new(), statements: Vec::new() };
        (analyze(dummy_module), lsp_all_errors)
      },
    }
  };

  let diagnostics: Vec<_> = errors
    .into_iter()
    .map(LspAnalyzerError)
    .chain(pre_errors)
    .map(error_as_diagnostic)
    .chain(warnings.into_iter().map(warning_as_diagnostic))
    .collect();
  this.client.publish_diagnostics(uri, diagnostics.clone(), None).await;

  let definits: Vec<_> = definitions
    .into_iter()
    .flat_map(|(addr, definition)| {
      usages.get(&addr).map_or_else(Vec::new, |token_set| {
        let defn_info_opt = defn_infos.remove(&addr);
        let info_arc = Arc::new(LValueInfo { definition, defn_info_opt, usages: token_set.clone() });
        token_set.iter().map(|t| (t.source_loc.clone(), addr.clone(), info_arc.clone())).collect()
      })
    })
    .collect();

  let (mut entities, infos) = definits
    .iter()
    .map(|(source_loc, _, _)| source_loc.line)
    .chain(non_var_tokens.iter().map(NonVarToken::line))
    .max()
    .map_or_else(
      || (Vec::<RangeMap<u32, Entity>>::new(), HashMap::<NamedVarAddress, Arc<LValueInfo>>::new()),
      |max| {
        let mut addrs = vec![RangeMap::new(); max as usize];
        let mut lv_infos = HashMap::new();
        for (loc, addr, info_arc) in definits {
          insert_entity(&mut addrs, &loc, Entity::LValue { addr: addr.clone() });
          lv_infos.insert(addr, info_arc); // Kind of clumsy and redundant
        }
        (addrs, lv_infos)
      },
    );

  for ref nvt in non_var_tokens {
    let entity_type = match nvt {
      NonVarToken::NamedArg(_) => Entity::NamedArg,
      NonVarToken::Number(_) => Entity::NumberLiteral,
      NonVarToken::String(_) => Entity::StringLiteral,
    };

    insert_entity(&mut entities, &nvt.token().source_loc, entity_type);
  }

  let doc = Document { contents: text.clone(), diagnostics, entities, infos };
  this.documents.write().await.insert(doc_loc.clone(), doc);
}

fn insert_entity(entities: &mut [RangeMap<u32, Entity>], loc: &SourceLoc, entity: Entity) {
  entities.get_mut((loc.line - 1) as usize).unwrap().insert(source_loc_to_range(loc), entity);
}
