use std::sync::Arc;

use crate::analyzer::function::Function as Func;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OrganicType {
  AudioEffect,
  Boolean,
  Generic(String),
  Function(Arc<Func>),
  List(Box<Self>),
  Number,
  RandomArg,
  RoundArg,
  SequenceArg,
  String,
  Unknown,
}
