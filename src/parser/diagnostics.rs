use crate::lexer::source_loc::MiniLoc;
use crate::lexer::token::Token;

#[derive(Debug)]
pub enum ParserError {
  ExtraToken { token: Token },
  FictionalToken { location: MiniLoc },
  MissingParameterValue { token: Token },
  UnexpectedEOF { location: MiniLoc, expected: Vec<String> },
  WrongToken { token: Token, expected: Vec<String> },
}
