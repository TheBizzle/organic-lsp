use std::collections::HashMap;

use tower_lsp_server::ls_types::{
  CodeAction, CodeActionKind, CodeActionOrCommand as CAoCo, CodeActionResponse, Diagnostic, NumberOrString,
  Position, Range, TextEdit, Uri, WorkspaceEdit,
};

use crate::lexer::token::{Token, TokenType::Identifier};

use crate::analyzer::value::TermDefn::UserDefined;

use crate::lsp::common::source_loc_to_tower_range;
use crate::lsp::diagnostics::DiagnosticCode;
use crate::lsp::document::{Document, Entity::LValue};
use crate::lsp::kebab_cased::kebab_cased;

pub(super) fn actions_under_cursor(uri: &Uri, doc: &Document, range: Range) -> Option<CodeActionResponse> {
  let line = range.start.line as usize;
  let column = range.start.character;

  if range.start == range.end
    && let Some(LValue { addr }) = doc.entities[line].get(&column)
    && let Some(info_arc) = doc.infos.get(addr)
    && let UserDefined { token, start, end } = &info_arc.as_ref().definition
    && let usages = info_arc.as_ref().usages.iter().filter(|t| t != &token).collect::<Vec<_>>()
    && !usages.is_empty()
  {
    let start_index = start.source_loc.pos as usize;
    let end_index = (end.source_loc.pos + end.source_loc.length) as usize;
    let rhs_text = doc.contents[start_index..end_index].to_string();

    let deletion = TextEdit {
      new_text: String::new(),
      range: Range {
        start: Position { line: token.source_loc.line - 1, character: token.source_loc.column - 1 },
        end: Position {
          line: end.source_loc.line - 1,
          character: token.source_loc.column - 1 + token.source_loc.length,
        },
      },
    };

    let edits = usages
      .iter()
      .map(|token| TextEdit {
        range: source_loc_to_tower_range(&token.source_loc),
        new_text: rhs_text.clone(),
      })
      .chain(std::iter::once(deletion))
      .collect();

    let edit =
      Some(WorkspaceEdit { changes: Some(HashMap::from([(uri.clone(), edits)])), ..Default::default() });

    let action = CodeAction {
      title: "Inline variable".to_string(),
      kind: Some(CodeActionKind::REFACTOR_INLINE),
      diagnostics: None,
      edit,
      command: None,
      is_preferred: Some(false),
      disabled: None,
      data: None,
    };

    Some(vec![CAoCo::CodeAction(action)])
  } else {
    None
  }
}

pub(super) fn actions_in_selection(_uri: &Uri, range: Range) -> Option<CodeActionResponse> {
  if range.start == range.end {
    None
  } else {
    Some(Vec::new()) // TODO: Extract variable ::REFACTOR_EXTRACT
  }
}

pub(super) fn actions_in_diagnostics(
  uri: &Uri, doc: &Document, diagnostics: Vec<Diagnostic>,
) -> Option<CodeActionResponse> {
  let mut actions = Vec::new();

  for diagnostic in diagnostics {
    let line = diagnostic.range.start.line as usize;
    let column = diagnostic.range.start.character;

    if let Some(NumberOrString::Number(code)) = diagnostic.code
      && let Some(diag_code) = DiagnosticCode::from_repr(code)
      && (diag_code == DiagnosticCode::Analyzer_Lint_CamelCase
        || diag_code == DiagnosticCode::Analyzer_Lint_SnakeCase)
      && let Some(LValue { addr }) = doc.entities[line].get(&column)
      && let Some(info_arc) = doc.infos.get(addr)
      && let UserDefined { token: Token { token_type, .. }, .. } = &info_arc.as_ref().definition
      && let Identifier(name) = token_type
    {
      let kebabed = kebab_cased(name);

      let edits = info_arc
        .as_ref()
        .usages
        .iter()
        .map(|token| TextEdit {
          range: source_loc_to_tower_range(&token.source_loc),
          new_text: kebabed.clone(),
        })
        .collect();

      let edit =
        Some(WorkspaceEdit { changes: Some(HashMap::from([(uri.clone(), edits)])), ..Default::default() });

      let action = CodeAction {
        title: "Use kebab-case name".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit,
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
      };

      actions.push(CAoCo::CodeAction(action));
    }
  }

  match actions.as_slice() {
    [] => None,
    _ => Some(actions),
  }
}
