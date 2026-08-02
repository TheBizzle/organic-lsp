use crate::lexer::token::Token;

use crate::analyzer::organic_type::OrganicType;

#[derive(Debug)]
pub struct AnalyzerError {
  pub typ: AnalyzerErrorType,
  pub offender: Token,
}

#[derive(Debug)]
pub struct AnalyzerWarning {
  pub typ: AnalyzerWarningType,
  pub offender: Token,
}

#[derive(Debug)]
pub enum AnalyzerWarningType {
  ArgOverridesPrevious,
  IntermediateCallInFnDef,
  UselessFnBody,
}

#[derive(Debug)]
pub enum AnalyzerErrorType {
  BadInternalState,
  DuplicateParameter,
  DuplicateVar,
  ExtraArgument { name: String },
  MissingArgument { name: String, typ: OrganicType },
  NoSuchFn,
  NoSuchVariable,
  TypeMismatch { expected: OrganicType, got: OrganicType },
  VarCannotInitInTermsOfSelf,
}
