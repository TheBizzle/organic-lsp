use std::collections::HashSet;

use crate::core::doc_loc::DocLoc;

use crate::lexer::token::{Token, TokenType::Identifier};

use crate::analyzer::{constants, note_names};

use crate::lsp::backend::LspBackend;
use crate::lsp::builtins::DOCS;

use tower_lsp_server::ls_types::{
  CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, TextDocumentPositionParams,
};

// TODO: Allow immediately after whitespace
// TODO: Turn off in strings and comments
// TODO: Give sorting priority
// TODO: Have context/scope/type awareness

pub(super) async fn completion(this: &LspBackend, params: CompletionParams) -> Option<CompletionResponse> {
  let TextDocumentPositionParams { text_document, position } = params.text_document_position;
  let doc_loc = DocLoc::new(text_document.uri.to_string());

  let line = position.line as usize;
  let column = position.character;

  if let Some(doc) = this.documents.read().await.get(&doc_loc)
    && column > 0
    && let Some(token) = doc.tokens[line].get(&(column - 1))
    && let Token { token_type: Identifier(wip), .. } = token
  {
    let user_vars = doc.last_var_names.iter().filter_map(|var_name| {
      if var_name.starts_with(wip) {
        Some(CompletionItem {
          label: var_name.clone(),
          kind: Some(CompletionItemKind::VARIABLE),
          detail: Some("User variable".to_string()),
          ..Default::default()
        })
      } else {
        None
      }
    });

    let note_names = note_names();

    let notes = note_names.iter().filter_map(|note| {
      if note.starts_with(wip) {
        Some(CompletionItem {
          label: note.clone(),
          kind: Some(CompletionItemKind::CONSTANT),
          detail: Some("Pitch".to_string()),
          ..Default::default()
        })
      } else {
        None
      }
    });

    let constants = constants();

    let consts = constants.keys().filter_map(|const_name| {
      if const_name.starts_with(wip) {
        Some(CompletionItem {
          label: const_name.clone(),
          kind: Some(CompletionItemKind::CONSTANT),
          detail: Some("Built-in".to_string()),
          ..Default::default()
        })
      } else {
        None
      }
    });

    let mut known_params = HashSet::new();

    let funcs_and_fields = DOCS.values().flat_map(|builtin| {
      let func_opt = if builtin.name.starts_with(wip) {
        Some(CompletionItem {
          label: builtin.name.to_string(),
          kind: Some(CompletionItemKind::FUNCTION),
          detail: Some(builtin.description.to_string()),
          ..Default::default()
        })
      } else {
        None
      };

      let params = builtin.parameters.iter().filter_map(|(param_name, desc)| {
        if param_name.starts_with(wip) && !known_params.contains(param_name) {
          known_params.insert(param_name);
          Some(CompletionItem {
            label: param_name.to_string(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some(desc.to_string()),
            ..Default::default()
          })
        } else {
          None
        }
      });

      params.into_iter().chain(func_opt).collect::<Vec<_>>()
    });

    let items = user_vars.chain(notes).chain(consts).chain(funcs_and_fields).collect();

    Some(CompletionResponse::Array(items))
  } else {
    None
  }
}
