use strum::EnumIter;

use crate::lexer::token::Token;

use crate::analyzer::function::Function;
use crate::analyzer::organic_type::OrganicType as OT;

#[derive(Clone, Debug, EnumIter, Eq, PartialEq)]
pub enum Accidental {
  Flat,
  Natural,
  Sharp,
}

// Don't even think of adding a `Function` constructor here.  Functions go in `FUNCTIONS`, not `CONSTANTS`.
// --Jason B. (7/28/26)
#[derive(Clone, Debug, PartialEq)]
pub enum ConstantValue {
  AudioEffect,
  Boolean(bool),
  List(Box<Self>),
  Number(f64),
  RandomArg,
  RoundArg,
  SequenceArg,
  String(String),
}

impl ConstantValue {
  #[must_use]
  pub fn as_type(&self) -> OT {
    match self {
      Self::AudioEffect => OT::AudioEffect,
      Self::Boolean(_) => OT::Boolean,
      Self::List(inner) => OT::List(Box::new(inner.as_type())),
      Self::Number(_) => OT::Number,
      Self::RandomArg => OT::RandomArg,
      Self::RoundArg => OT::RoundArg,
      Self::SequenceArg => OT::SequenceArg,
      Self::String(_) => OT::String,
    }
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Note {
  pub pitch_class: PitchClass,
  pub accidental: Accidental,
  pub octave: u8,
}

#[derive(Clone, Debug, EnumIter, Eq, PartialEq)]
pub enum PitchClass {
  A,
  B,
  C,
  D,
  E,
  F,
  G,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum TermDefn {
  BuiltinConstant { value: ConstantValue },
  BuiltinFn { value: Function },
  BuiltinNote { note: Note },
  UserDefined { token: Token, start: Token, end: Token },
}
