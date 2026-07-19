use strum::EnumIter;

use crate::lexer::token::Token;

use crate::analyzer::function::Function;

#[derive(Clone, Debug, EnumIter, Eq, PartialEq)]
pub enum Accidental {
  Flat,
  Natural,
  Sharp,
}

// Do not think of adding a `Function` constructor here.  Functions go in `FUNCTIONS`, not `CONSTANTS`.
// --Jason B. (7/25/26)
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

#[derive(Clone, Debug, PartialEq)]
pub enum TermDefn<'a> {
  BuiltinConstant { value: ConstantValue },
  BuiltinFn { value: Function<'a> },
  BuiltinNote { note: Note },
  UserDefined { token: Token },
}
