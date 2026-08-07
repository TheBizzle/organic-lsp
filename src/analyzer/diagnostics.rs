use crate::lexer::token::Token;

use crate::analyzer::organic_type::OrganicType;

#[derive(Debug)]
pub struct AnalyzerDiagnostic {
  pub typ: AnalyzerDiagnosticType,
  pub offender: Token,
}

#[derive(Debug)]
pub enum AnalyzerDiagnosticType {
  AnalyzerError(AnalyzerErrorType),
  AnalyzerLint(AnalyzerLintType),
  AnalyzerWarning(AnalyzerWarningType),
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

#[derive(Debug)]
pub enum AnalyzerLintType {
  CamelCase,
  SnakeCase,
}

#[derive(Debug)]
pub enum AnalyzerWarningType {
  ArgOverridesPrevious,
  IntermediateCallInFnDef,
  UselessFnBody,
}
