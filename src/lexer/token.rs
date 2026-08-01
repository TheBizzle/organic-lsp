use logos::Logos;

use crate::lexer::source_loc::SourceLoc;
use ordered_float::NotNan;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Token {
  pub token_type: TokenType,
  pub source_loc: SourceLoc,
}

#[derive(Clone, Debug, Eq, Hash, Logos, Ord, PartialEq, PartialOrd)]
pub enum TokenType {
  #[regex(r"(?s)/\*.*?\*/")]
  BlockComment,

  #[token(":")]
  Colon,
  #[token(",")]
  Comma,
  #[regex(r"//[^\r\n]*", allow_greedy = true)]
  Comment,

  #[token("/")]
  Divide,

  #[token("=")]
  Equals,

  #[token(">")]
  GreaterThan,
  #[token(">=")]
  GreaterThanEquals,

  #[regex(r"[A-Za-z_][A-Za-z0-9_\-]*", |lex| lex.slice().to_owned())]
  Identifier(String),
  #[token("include")]
  Include,

  #[token("{")]
  LeftBrace,
  #[token("[")]
  LeftBracket,
  #[token("(")]
  LeftParen,
  #[token("<")]
  LessThan,
  #[token("<=")]
  LessThanEquals,

  #[token("\n")]
  Newline,
  #[regex(r"[0-9]+(\.[0-9]+)?", |lex| NotNan::new(lex.slice().parse::<f64>().unwrap()).unwrap())]
  Number(NotNan<f64>),

  #[token("-")]
  Minus,
  #[token("*")]
  Multiply,

  #[token("+")]
  Plus,

  #[token("}")]
  RightBrace,
  #[token("]")]
  RightBracket,
  #[token(")")]
  RightParen,

  #[regex(r#""([^"\\]|\\.)*""#, |lex| {
    let s = lex.slice();
    s[1..s.len() - 1].to_string()
  })]
  String(String),

  #[regex(r#""([^"\\]|\\.)*$"#, |lex| {
    let s = lex.slice();
    s[1..s.len() - 1].to_string()
  })]
  UnterminatedString(String),

  #[regex(r"[ \t\f]+", logos::skip)] // Ignore most whitespace
  Whitespace,
}
