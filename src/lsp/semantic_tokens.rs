use tower_lsp_server::ls_types::{SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensResult};

use crate::analyzer::analysis::{DefnInfo, HighlightingType as HLT};
use crate::analyzer::value::TermDefn;
use crate::core::doc_loc::DocLoc;
use crate::lexer::lex;
use crate::lexer::source_loc::SourceLoc;
use crate::lexer::token::Token;
use crate::lexer::token::TokenType::{
  BlockComment, Colon, Comma, Comment, Divide, Equals, GreaterThan, GreaterThanEquals, Identifier, Include,
  LeftBrace, LeftBracket, LeftParen, LessThan, LessThanEquals, Minus, Multiply, Newline, Number, Plus,
  RightBrace, RightBracket, RightParen, String, Whitespace,
};
use crate::lsp::document::{Document, Entity};

const LANGUAGE_CONSTANT: SemanticTokenType = SemanticTokenType::new("language_constant");

pub const TOKEN_TYPES: &[SemanticTokenType] = &[
  SemanticTokenType::FUNCTION,
  LANGUAGE_CONSTANT,
  SemanticTokenType::NUMBER,
  SemanticTokenType::OPERATOR,
  SemanticTokenType::PARAMETER,
  SemanticTokenType::STRING,
  SemanticTokenType::VARIABLE,
  SemanticTokenType::PROPERTY,
  SemanticTokenType::COMMENT,
];

struct Semantic {
  line: u32,
  start: u32,
  length: u32,
  token_type: SemanticTokenType,
  modifiers: Vec<Modifier>,
}

enum Modifier {
  _Mod1,
  _Mod2,
  _Mod3,
}

pub async fn calc_semantic_tokens(doc_loc: &DocLoc, document: &Document) -> Option<SemanticTokensResult> {
  let (tokens, _) = lex(doc_loc, &document.contents);

  let mut last_loc = SourceLoc { doc_loc: doc_loc.clone(), pos: 0, line: 1, column: 1, length: 0 };

  let mut semantics = Vec::new();
  for token in tokens {
    if let Some(converted) = convert_token(&token, &last_loc, document) {
      semantics.push(converted);
      last_loc = token.source_loc;
    }
  }

  let data = semantics.iter().map(as_lsp_token).collect();
  Some(SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data }))
}

fn convert_token(token: &Token, last_loc: &SourceLoc, document: &Document) -> Option<Semantic> {
  let Token { source_loc, token_type } = token;

  #[allow(clippy::match_same_arms)]
  let opt = match token_type {
    BlockComment => Some(SemanticTokenType::COMMENT),
    Colon => None,
    Comma => None,
    Comment => Some(SemanticTokenType::COMMENT),
    Divide => Some(SemanticTokenType::OPERATOR),
    Equals => None,
    GreaterThan => Some(SemanticTokenType::OPERATOR),
    GreaterThanEquals => Some(SemanticTokenType::OPERATOR),
    Identifier(_) => Some(calc_highlighting_for_ident(source_loc, document)),
    Include => Some(SemanticTokenType::KEYWORD),
    LeftBrace => None,
    LeftBracket => None,
    LeftParen => None,
    LessThan => Some(SemanticTokenType::OPERATOR),
    LessThanEquals => Some(SemanticTokenType::OPERATOR),
    Newline => panic!("Lexer leaking newlines should not be possible"),
    Number(_) => Some(SemanticTokenType::NUMBER),
    Minus => Some(SemanticTokenType::OPERATOR),
    Multiply => Some(SemanticTokenType::OPERATOR),
    Plus => Some(SemanticTokenType::OPERATOR),
    RightBrace => None,
    RightBracket => None,
    RightParen => None,
    String(_) => Some(SemanticTokenType::STRING),
    Whitespace => panic!("Lexer leaking whitespace should not be possible"),
  };

  opt.map(|token_type| {
    let line = token.source_loc.line - last_loc.line;
    let start = if line == 0 {
      token.source_loc.column - last_loc.column
    } else {
      token.source_loc.column - 1
    };
    let length = token.source_loc.length;

    Semantic { line, start, length, token_type, modifiers: vec![] }
  })
}

fn calc_highlighting_for_ident(source_loc: &SourceLoc, document: &Document) -> SemanticTokenType {
  let SourceLoc { line, column, .. } = source_loc;
  if let Some(line_ranges) = document.entities.get((line - 1) as usize)
    && let Some(entity) = line_ranges.get(&(column - 1))
  {
    match entity {
      Entity::LValue { addr } => document.infos.get(addr).map_or(SemanticTokenType::VARIABLE, |info_arc| {
        #[allow(clippy::match_same_arms)]
        match info_arc.as_ref().definition {
          TermDefn::BuiltinConstant { .. } => LANGUAGE_CONSTANT,
          TermDefn::BuiltinFn { .. } => SemanticTokenType::FUNCTION,
          TermDefn::BuiltinNote { .. } => LANGUAGE_CONSTANT,
          TermDefn::UserDefined { .. } => match info_arc.as_ref().defn_info_opt.as_ref() {
            Some(DefnInfo { hl_type: HLT::Function, .. }) => SemanticTokenType::FUNCTION,
            Some(DefnInfo { hl_type: HLT::Parameter, .. }) => SemanticTokenType::PARAMETER,
            Some(DefnInfo { hl_type: HLT::Variable, .. }) => SemanticTokenType::VARIABLE,
            None => SemanticTokenType::VARIABLE,
          },
        }
      }),
      Entity::NamedArg => SemanticTokenType::PROPERTY,
      Entity::NumberLiteral | Entity::StringLiteral => panic!("These tokens are not legal here."),
    }
  } else {
    SemanticTokenType::VARIABLE
  }
}

fn as_lsp_token(token: &Semantic) -> SemanticToken {
  let token_type = as_type_int(&token.token_type);
  let modifiers = token.modifiers.iter().map(as_modifier_int).sum();
  SemanticToken {
    delta_line: token.line,
    delta_start: token.start,
    length: token.length,
    token_type,
    token_modifiers_bitset: modifiers,
  }
}

fn as_type_int(token_type: &SemanticTokenType) -> u32 {
  // The token types are not structural/matchable. --Jason B. (7/18/26)
  if token_type == &SemanticTokenType::FUNCTION {
    0
  } else if token_type == &LANGUAGE_CONSTANT {
    1
  } else if token_type == &SemanticTokenType::NUMBER {
    2
  } else if token_type == &SemanticTokenType::OPERATOR {
    3
  } else if token_type == &SemanticTokenType::PARAMETER {
    4
  } else if token_type == &SemanticTokenType::STRING {
    5
  } else if token_type == &SemanticTokenType::VARIABLE {
    6
  } else if token_type == &SemanticTokenType::PROPERTY {
    7
  } else if token_type == &SemanticTokenType::COMMENT {
    8
  } else {
    eprintln!("Warning!  Unknown token type: {token_type:?}");
    100
  }
}

const fn as_modifier_int(modifier: &Modifier) -> u32 {
  match modifier {
    Modifier::_Mod1 => 1,
    Modifier::_Mod2 => 2,
    Modifier::_Mod3 => 4,
  }
}
