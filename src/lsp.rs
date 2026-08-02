mod backend;
mod builtins;
mod common;
mod definition;
mod diagnostics;
mod document;
mod full_analysis;
mod named_arg;
mod pretty_type;
mod semantic_tokens;

use std::collections::HashMap;

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
  CodeActionParams, CodeActionProviderCapability, CodeActionResponse, CompletionOptions, CompletionParams,
  CompletionResponse, DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
  GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
  InitializeParams, InitializeResult, InitializedParams, Location, MarkedString, MessageType, OneOf,
  Position, Range as TowerRange, ReferenceParams, RenameParams, SemanticTokensFullOptions,
  SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
  SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentPositionParams,
  TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, WorkDoneProgressOptions, WorkspaceEdit,
};
use tower_lsp_server::{ClientSocket, LanguageServer, LspService};

use crate::core::doc_loc::DocLoc;

use crate::lsp::backend::LspBackend;
use crate::lsp::common::token_to_location;
use crate::lsp::definition::describe_defn;
use crate::lsp::document::{Entity, LValueInfo};
use crate::lsp::full_analysis::store_and_reanalyze;
use crate::lsp::named_arg::describe_named_arg;
use crate::lsp::semantic_tokens::{TOKEN_TYPES, calc_semantic_tokens};

use Entity::{LValue, NamedArg, NumberLiteral, StringLiteral};

const DEBUG: MessageType = MessageType::ERROR;

pub fn new_lsp() -> (LspService<LspBackend>, ClientSocket) {
  LspService::new(LspBackend::new)
}

impl LanguageServer for LspBackend {
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
    Ok(Some(vec![])) // TODO (e.g. quick-fixes like declaring undeclared variables)
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
      Ok(None)
    }
  }

  async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let TextDocumentPositionParams { text_document, position } = params.text_document_position_params;
    let doc_loc = DocLoc::new(text_document.uri.to_string());

    if let Some(doc) = self.documents.write().await.get(&doc_loc)
      && let Some(line) = doc.entities.get(position.line as usize)
      && let Some((range, entity)) = line.get_key_value(&position.character)
    {
      let str = match entity {
        NamedArg { name, func_addr, func } => describe_named_arg(name, func_addr, func.as_ref()),
        NumberLiteral(value) => format!("value of number literal: `{value}`"),
        StringLiteral(value) => format!("value of string literal: `{value}`"),
        LValue { addr } => doc.infos.get(addr).map_or_else(
          || "Unknown term".to_string(),
          |info_arc| {
            let LValueInfo { definition, defn_info_opt, usages } = info_arc.as_ref();
            let ot_opt = defn_info_opt.as_ref().map(|dinfo| dinfo.organic_type.clone());
            describe_defn(definition, ot_opt, usages.iter().next())
          },
        ),
      };

      let contents = HoverContents::Scalar(MarkedString::String(str));

      let range = TowerRange {
        start: Position { line: position.line, character: range.start },
        end: Position { line: position.line, character: range.end },
      };

      Ok(Some(Hover { contents, range: Some(range) }))
    } else {
      Ok(None)
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
      Ok(None)
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
