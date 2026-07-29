use std::ops::Range;

use tower_lsp_server::ls_types::{Location, Position, Range as TowerRange};

use crate::lexer::source_loc::SourceLoc;
use crate::lexer::token::Token;

pub(super) fn token_to_location(token: &Token) -> Location {
  let uri = token.source_loc.doc_loc.as_str().parse().unwrap();
  let Range { start, end } = source_loc_to_range(&token.source_loc.clone());
  let range = TowerRange {
    start: Position { line: token.source_loc.line - 1, character: start },
    end: Position { line: token.source_loc.line - 1, character: end },
  };
  Location { uri, range }
}

pub(super) fn source_loc_to_range(source_loc: &SourceLoc) -> Range<u32> {
  let SourceLoc { column, length, .. } = source_loc;
  (column - 1)..(column - 1 + length)
}
