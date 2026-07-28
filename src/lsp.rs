use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use rangemap::RangeMap;

use tower_lsp_server::LanguageServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
  CodeActionParams, CodeActionProviderCapability, CodeActionResponse, CompletionOptions, CompletionParams,
  CompletionResponse, DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
  GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
  InitializeParams, InitializeResult, InitializedParams, Location, MarkedString, MessageType, OneOf,
  Position, Range as TowerRange, ReferenceParams, RenameParams, SemanticTokensFullOptions,
  SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
  SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentPositionParams,
  TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Uri, WorkDoneProgressOptions, WorkspaceEdit,
};

pub mod definition;
pub mod document;
pub mod lsp_backend;
pub mod semantic_tokens;

use crate::analyzer::address::NamedVarAddress;
use crate::analyzer::analysis::Analysis;
use crate::core::diagnostics::LspError::LspAnalyzerError;
use crate::core::diagnostics::{error_as_diagnostic, warning_as_diagnostic};
use crate::lexer::doc_loc::DocLoc;
use crate::lexer::source_loc::SourceLoc;
use crate::lexer::token::Token;
use crate::lsp::definition::describe_defn;
use crate::lsp::document::{Document, Entity, LValueInfo};
use crate::lsp::lsp_backend::LspBackend;
use crate::lsp::semantic_tokens::{TOKEN_TYPES, calc_semantic_tokens};
use crate::parser::analyze;

use Entity::LValue;

const DEBUG: MessageType = MessageType::ERROR;

impl LanguageServer for LspBackend<'static> {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        // TODO: TextDocumentSyncKind::INCREMENTAL
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),

        hover_provider: Some(HoverProviderCapability::Simple(true)),

        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
          SemanticTokensOptions {
            legend: SemanticTokensLegend { token_types: TOKEN_TYPES.to_vec(), token_modifiers: vec![] },
            range: Some(false),
            full: Some(SemanticTokensFullOptions::Bool(true)),
            work_done_progress_options: WorkDoneProgressOptions::default(),
          },
        )),

        completion_provider: Some(CompletionOptions::default()),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        document_formatting_provider: Some(OneOf::Left(true)),

        ..ServerCapabilities::default()
      },

      ..InitializeResult::default()
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    self.client.log_message(DEBUG, "Organic LSP initialized!").await;
  }

  async fn shutdown(&self) -> Result<()> {
    Ok(())
  }

  async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
    Ok(Some(vec![])) // TODO
  }

  async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
    Ok(None) // TODO
  }

  async fn did_open(&self, params: DidOpenTextDocumentParams) {
    store_and_reanalyze(self, params.text_document.uri, params.text_document.text).await;
  }

  async fn did_change(&self, params: DidChangeTextDocumentParams) {
    if let Some(change) = params.content_changes.into_iter().next() {
      let uri = params.text_document.uri;
      let doc_loc = DocLoc::new(uri.to_string());
      if let Some(value) = self.documents.write().await.get_mut(&doc_loc) {
        value.contents.clone_from(&change.text);
      }
      store_and_reanalyze(self, uri, change.text).await;
    }
  }

  async fn formatting(&self, _params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
    Ok(Some(vec![])) // TODO
  }

  async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
    let doc_loc = DocLoc::new(text_document.uri.to_string());

    if let Some(doc) = self.documents.write().await.get(&doc_loc)
      && let Some(line) = doc.entities.get(position.line as usize)
      && let Some(LValue { addr }) = line.get(&position.character)
      && let Some(info_arc) = doc.infos.get(addr)
      && let Some(defn_info) = info_arc.as_ref().defn_info_opt.as_ref()
    {
      Ok(Some(GotoDefinitionResponse::Scalar(token_to_location(&defn_info.token))))
    } else {
      Ok(None) // TODO: Probably not actually OK
    }
  }

  async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
    let doc_loc = DocLoc::new(text_document.uri.to_string());

    if let Some(doc) = self.documents.write().await.get(&doc_loc)
      && let Some(line) = doc.entities.get(position.line as usize)
      && let Some((range, LValue { addr })) = line.get_key_value(&position.character)
      && let Some(info_arc) = doc.infos.get(addr)
    {
      let range = TowerRange {
        start: Position { line: position.line, character: range.start },
        end: Position { line: position.line, character: range.end },
      };

      let str = describe_defn(&info_arc.as_ref().definition);
      let contents = HoverContents::Scalar(MarkedString::String(str));

      Ok(Some(Hover { contents, range: Some(range) }))
    } else {
      Ok(None) // TODO: Probably not actually OK
    }
  }

  async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
    let TextDocumentPositionParams { text_document, position } = params.text_document_position;
    let doc_loc = DocLoc::new(text_document.uri.to_string());

    if let Some(doc) = self.documents.write().await.get(&doc_loc)
      && let Some(line) = doc.entities.get(position.line as usize)
      && let Some(LValue { addr }) = line.get(&position.character)
      && let Some(info_arc) = doc.infos.get(addr)
    {
      let locs = info_arc.as_ref().usages.iter().map(token_to_location).collect();
      Ok(Some(locs))
    } else {
      Ok(None) // TODO: Probably not actually OK
    }
  }

  async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
    let pos = params.text_document_position;
    Ok(Some(WorkspaceEdit {
      changes: Some(HashMap::from([(
        pos.text_document.uri,
        vec![TextEdit {
          range: TowerRange {
            // TODO
            start: Position::new(0, 0),
            end: Position::new(0, 0),
          },
          new_text: params.new_name,
        }],
      )])),
      document_changes: None,
      change_annotations: None,
    }))
  }

  async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
    let uri = DocLoc::new(params.text_document.uri.to_string());

    if let Some(doc) = self.documents.read().await.get(&uri) {
      Ok(calc_semantic_tokens(&uri, doc).await)
    } else {
      let msg = format!("No known document for URI: {uri:?}");
      self.client.log_message(DEBUG, msg).await;
      Result::Ok(None)
    }
  }
}

async fn store_and_reanalyze(this: &LspBackend<'_>, uri: Uri, text: String) {
  let doc_loc = DocLoc::new(uri.to_string());

  let (
    Analysis {
      definitions,
      mut defn_infos,
      errors,
      named_arg_tokens,
      number_tokens,
      string_tokens,
      usages,
      warnings,
    },
    pre_errors,
  ) = analyze(&doc_loc, &text);

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
    .chain(named_arg_tokens.iter().map(|token| token.source_loc.line))
    .chain(number_tokens.iter().map(|token| token.source_loc.line))
    .chain(string_tokens.iter().map(|token| token.source_loc.line))
    .max()
    .map_or_else(
      || (Vec::<RangeMap<u32, Entity>>::new(), HashMap::<NamedVarAddress, Arc<LValueInfo>>::new()),
      |max| {
        let mut addrs = vec![RangeMap::new(); max as usize];
        let mut lv_infos = HashMap::new();
        for (loc, addr, info_arc) in definits {
          insert_entity(&mut addrs, &loc, LValue { addr: addr.clone() });
          lv_infos.insert(addr, info_arc); // Kind of clumsy and redundant
        }
        (addrs, lv_infos)
      },
    );

  for Token { source_loc, .. } in named_arg_tokens {
    insert_entity(&mut entities, &source_loc, Entity::NamedArg);
  }

  for Token { source_loc, .. } in number_tokens {
    insert_entity(&mut entities, &source_loc, Entity::NumberLiteral);
  }

  for Token { source_loc, .. } in string_tokens {
    insert_entity(&mut entities, &source_loc, Entity::StringLiteral);
  }

  let doc = Document { contents: text.clone(), diagnostics, entities, infos };
  this.documents.write().await.insert(doc_loc.clone(), doc);
}

fn insert_entity(entities: &mut [RangeMap<u32, Entity>], loc: &SourceLoc, entity: Entity) {
  entities.get_mut((loc.line - 1) as usize).unwrap().insert(source_loc_to_range(loc), entity);
}

fn source_loc_to_range(source_loc: &SourceLoc) -> Range<u32> {
  let SourceLoc { column, length, .. } = source_loc;
  (column - 1)..(column - 1 + length)
}

fn token_to_location(token: &Token) -> Location {
  let uri = token.source_loc.doc_loc.as_str().parse().unwrap();
  let Range { start, end } = source_loc_to_range(&token.source_loc.clone());
  let range = TowerRange {
    start: Position { line: token.source_loc.line - 1, character: start },
    end: Position { line: token.source_loc.line - 1, character: end },
  };
  Location { uri, range }
}
