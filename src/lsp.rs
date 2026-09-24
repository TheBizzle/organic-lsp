pub mod diagnostics;
pub mod document;
pub mod miniformat;

mod backend;
mod builtins;
mod code_action;
mod common;
mod completion;
mod definition;
mod full_analysis;
mod kebab_cased;
mod named_arg;
mod pretty_type;
mod semantic_tokens;

use std::borrow::Cow;
use std::collections::HashMap;
use std::future::ready;
use std::iter::once;

use tower_lsp_server::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp_server::ls_types::{
  CodeActionParams, CodeActionProviderCapability, CodeActionResponse, CompletionOptions, CompletionParams,
  CompletionResponse, DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
  DocumentRangeFormattingParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
  HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, Location,
  MarkedString, MessageType, OneOf, Position, PrepareRenameResponse, Range as TowerRange, ReferenceParams,
  RenameOptions, RenameParams, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
  SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
  TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Uri,
  WorkDoneProgressOptions, WorkspaceEdit,
};
use tower_lsp_server::{ClientSocket, LanguageServer, LspService};

use crate::core::doc_loc::DocLoc;

use crate::lexer::token::{Token, TokenType::Identifier};

use crate::parser::parse;

use crate::lsp::backend::LspBackend;
use crate::lsp::code_action::{actions_in_diagnostics, actions_in_selection, actions_under_cursor};
use crate::lsp::common::token_to_location;
use crate::lsp::completion::completion;
use crate::lsp::definition::describe_defn;
use crate::lsp::diagnostics::{LspError::LspParserError, error_as_diagnostic};
use crate::lsp::document::{Entity, LValueInfo};
use crate::lsp::full_analysis::store_and_reanalyze;
use crate::lsp::miniformat::miniformat;
use crate::lsp::named_arg::describe_named_arg;
use crate::lsp::semantic_tokens::{TOKEN_TYPES, calc_semantic_tokens};

use Entity::{LValue, NamedArg, NumberLiteral, StringLiteral};

const DEBUG: MessageType = MessageType::ERROR;

pub fn new_lsp() -> (LspService<LspBackend>, ClientSocket) {
  LspService::new(LspBackend::new)
}

impl LanguageServer for LspBackend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    ready(Ok(InitializeResult {
      capabilities: ServerCapabilities {
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
        rename_provider: Some(OneOf::Right(RenameOptions {
          prepare_provider: Some(true),
          work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        document_range_formatting_provider: Some(OneOf::Left(true)),

        ..ServerCapabilities::default()
      },

      ..InitializeResult::default()
    }))
    .await
  }

  async fn initialized(&self, _: InitializedParams) {
    self.client.log_message(DEBUG, "Organic LSP initialized!").await;
  }

  async fn shutdown(&self) -> Result<()> {
    ready(Ok(())).await
  }

  async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
    let doc_loc = DocLoc::new(params.text_document.uri.to_string());
    if let Some(doc) = self.documents.read().await.get(&doc_loc) {
      let uri = params.text_document.uri;
      let range = params.range;
      let diagnostics = params.context.diagnostics;

      let res1 = actions_under_cursor(&uri, doc, range);
      let res2 = actions_in_selection(&uri, doc, range);
      let res3 = actions_in_diagnostics(&uri, doc, diagnostics);

      let results: Vec<_> = res1.into_iter().chain(res2).chain(res3).flatten().collect();

      let result_opt = match results.as_slice() {
        [] => None,
        _ => Some(results),
      };

      Ok(result_opt)
    } else {
      let msg = format!("No known document for URI: {doc_loc:?}");
      self.client.log_message(DEBUG, msg).await;
      Ok(None)
    }
  }

  async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    Ok(completion(self, params).await)
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

  async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
    fn formatter_run(str: &str) -> String {
      str.to_string()
    }

    let uri = DocLoc::new(params.text_document.uri.to_string());

    let res_opt = self.documents.read().await.get(&uri).and_then(|doc| {
      let text = &doc.contents;
      let new_text = formatter_run(text);

      if &new_text == text {
        None
      } else {
        let num_lines = u32::try_from(text.lines().count()).expect("Document can't have that many lines");
        let num_columns = u32::try_from(text.lines().count()).expect("Last line can't have that many chars");

        let edit = TextEdit {
          range: TowerRange {
            start: Position { line: 0, character: 0 },
            end: Position { line: num_lines, character: num_columns },
          },
          new_text,
        };

        Some(vec![edit])
      }
    });

    Ok(res_opt)
  }

  async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> Result<Option<Vec<TextEdit>>> {
    let uri = DocLoc::new(params.text_document.uri.to_string());

    self.documents.read().await.get(&uri).map_or_else(
      || {
        Result::Err(Error {
          code: ErrorCode::InvalidRequest,
          message: Cow::Owned(format!("Invalid document: {uri:?}")),
          data: None,
        })
      },
      |doc| {
        let TowerRange { start, end } = params.range;

        // Intention: Avoid exploding on `0 - 1` when selecting an empty line --Jason B. (8/14/26)
        let mut end_pos = end;
        while end_pos.line > start.line && end_pos.character == 0 {
          end_pos.line -= 1;
          end_pos.character = doc
            .tokens
            .get(end_pos.line as usize)
            .unwrap()
            .iter()
            .next_back()
            .map_or(0, |(range, _)| range.end);
        }

        if let Some(start_line) = doc.tokens.get(start.line as usize)
          && let Some(start_token) = start_line.get(&start.character)
          && let Some(end_line) = doc.tokens.get(end_pos.line as usize)
          && let Some(end_token) = end_line.get(&(end_pos.character - 1))
        {
          let mut tokens = Vec::new();

          for line_num in start.line..=end_pos.line {
            if let Some(line) = doc.tokens.get(line_num as usize) {
              let rightmost_column = line.iter().next_back().map_or(0, |(range, _)| range.end);
              #[rustfmt::skip]
              let beginning_column = if line_num == start.line { start.character } else { 0 };
              #[rustfmt::skip]
              let final_column = if line_num == end_pos.line { end_pos.character } else { rightmost_column };
              for (_, token) in line.overlapping(beginning_column..final_column) {
                tokens.push(token.clone());
              }
            } else {
              let message = Cow::Borrowed("Range includes non-existent line");
              return Result::Err(Error { code: ErrorCode::InvalidParams, message, data: None });
            }
          }

          match parse(tokens) {
            Err(perror) => {
              let message = Cow::Owned(format!(
                "Unparsable selection: {}",
                error_as_diagnostic(LspParserError(perror)).message
              ));
              Result::Err(Error { code: ErrorCode::InvalidParams, message, data: None })
            },
            Ok(mini_ast) => {
              let indent_depth = start_token.source_loc.column - 1;
              let indent = " ".repeat(indent_depth as usize);
              let max_width = 110 - indent_depth;
              let formatted = miniformat(&mini_ast, max_width);
              let new_text = formatted
                .split_once('\n')
                .map(|(head, tail)| {
                  once(head.to_string())
                    .chain(tail.split('\n').map(|line| {
                      if line.trim().is_empty() {
                        String::new()
                      } else {
                        format!("{indent}{line}")
                      }
                    }))
                    .collect::<Vec<_>>()
                    .join("\n")
                })
                .unwrap_or(formatted);

              let real_range = TowerRange {
                start: Position { line: start.line, character: start_token.source_loc.column - 1 },
                end: Position {
                  line: end_pos.line,
                  character: end_token.source_loc.column + end_token.source_loc.length - 1,
                },
              };

              let edit = TextEdit { range: real_range, new_text };
              Result::Ok(Some(vec![edit]))
            },
          }
        } else {
          let message = Cow::Borrowed("Range can only include complete statements");
          Result::Err(Error { code: ErrorCode::InvalidParams, message, data: None })
        }
      },
    )
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
        NumberLiteral(value) => format!("value of `number` literal: `{value}`"),
        StringLiteral(value) => format!("value of `string` literal: `{value}`"),
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

  async fn prepare_rename(
    &self, params: TextDocumentPositionParams,
  ) -> Result<Option<PrepareRenameResponse>> {
    let doc_loc = DocLoc::new(params.text_document.uri.to_string());
    if let Some(doc) = self.documents.write().await.get(&doc_loc)
      && let Some(line) = doc.tokens.get(params.position.line as usize)
      && let Some(token) = line.get(&params.position.character)
      && let Token { token_type: Identifier(name), .. } = token
    {
      let Location { range, .. } = token_to_location(token);
      Ok(Some(PrepareRenameResponse::RangeWithPlaceholder { range, placeholder: name.clone() }))
    } else {
      Ok(None)
    }
  }

  async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
    let TextDocumentPositionParams { text_document, position } = params.text_document_position;
    let doc_loc = DocLoc::new(text_document.uri.to_string());

    if let Some(doc) = self.documents.write().await.get(&doc_loc)
      && let Some(line) = doc.entities.get(position.line as usize)
      && let Some(LValue { addr }) = line.get(&position.character)
      && let Some(info_arc) = doc.infos.get(addr)
    {
      let mut text_edits: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
      for x in &info_arc.as_ref().usages {
        let Location { range, uri } = token_to_location(x);
        text_edits.entry(uri).or_default().push(TextEdit { range, new_text: params.new_name.clone() });
      }

      let changes = Some(text_edits);
      let ws_edit = WorkspaceEdit { changes, document_changes: None, change_annotations: None };

      Ok(Some(ws_edit))
    } else {
      Ok(None)
    }
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
