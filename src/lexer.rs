pub mod doc_loc;
pub mod source_loc;
pub mod token;

use std::ops::Range;

use logos::Logos;

use crate::errors::LexerError;
use crate::lexer::doc_loc::DocLoc;
use crate::lexer::source_loc::SourceLoc;
use crate::lexer::token::{Token, TokenType};

/// # Panics
///
/// When a block comment is detected and its index cannot be retrieved.
/// When 64-bit numbers get lossfully converted to 32-bit numbers.
#[must_use]
pub fn lex(doc_loc: &DocLoc, doc_text: &str) -> (Vec<Token>, Vec<LexerError>) {
  let mut line_num: u32 = 1;
  let mut last_line_offset: u32 = 0;
  let mut errors: Vec<LexerError> = Vec::new();
  let mut token_sequence = Vec::new();

  for (result, span) in TokenType::lexer(doc_text).spanned() {
    match as_u32s(span.clone()) {
      Err(size) => {
        errors.push(LexerError::FileTooBig { size, line_num });
        return (token_sequence, errors);
      },
      Ok((start, length, end)) => {
        let column = 1 + start - last_line_offset;
        let source_loc = SourceLoc { doc_loc: doc_loc.clone(), pos: start, line: line_num, column, length };
        if let Ok(token_type) = result {
          #[allow(clippy::single_match_else)]
          match token_type {
            TokenType::Newline => {
              line_num += 1;
              last_line_offset = end;
            },
            _ => {
              if token_type.clone() == TokenType::BlockComment {
                let comment = &doc_text[span.clone()];
                let num_newlines = as_u32_risky(comment.matches('\n').count());

                if num_newlines > 0 {
                  let start_offset = as_u32_risky(span.start);
                  let last_newline_offset = as_u32_risky(comment.rmatch_indices('\n').next().unwrap().0);
                  line_num += num_newlines;
                  last_line_offset = start_offset + last_newline_offset + 1;
                }
              }

              token_sequence.push(Token { token_type, source_loc });
            },
          }
        } else {
          let error = LexerError::UnknownToken { culprit: doc_text[span].to_string(), source_loc };
          errors.push(error);
        }
      },
    }
  }

  (token_sequence, errors)
}

fn as_u32(value: usize) -> Result<u32, usize> {
  u32::try_from(value).map_err(|_| value)
}

#[allow(clippy::cast_possible_truncation)]
const fn as_u32_risky(value: usize) -> u32 {
  value as u32
}

fn as_u32s(span: Range<usize>) -> Result<(u32, u32, u32), usize> {
  let start = as_u32(span.start)?;
  let len = as_u32(span.len())?;
  let end = as_u32(span.end)?;
  Ok((start, len, end))
}
