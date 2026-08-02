use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::lexer::diagnostics::LexerError::{self, FileTooBig, UnknownToken};
use crate::lexer::source_loc::{MiniLoc, SourceLoc};

use crate::parser::diagnostics::ParserError::{
  self, ExtraToken, FictionalToken, MissingParameterValue, UnexpectedEOF, WrongToken,
};

use crate::analyzer::diagnostics::{AnalyzerError, AnalyzerWarning};

use crate::analyzer::diagnostics::AnalyzerErrorType::{
  BadInternalState, DuplicateParameter, DuplicateVar, ExtraArgument, MissingArgument, NoSuchFn,
  NoSuchVariable, TypeMismatch, VarCannotInitInTermsOfSelf,
};

use crate::analyzer::diagnostics::AnalyzerWarningType::{
  ArgOverridesPrevious, IntermediateCallInFnDef, UselessFnBody,
};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LspError {
  LspLexerError(LexerError),
  LspParserError(ParserError),
  LspAnalyzerError(AnalyzerError),
}
use LspError::{LspAnalyzerError, LspLexerError, LspParserError};

#[must_use]
pub fn error_as_diagnostic(error: LspError) -> Diagnostic {
  let (range, message) = match error {
    LspLexerError(FileTooBig { size, line_num }) => {
      let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: line_num - 1, character: 0 },
      };
      let msg = format!(
        "This file is too large!  organic-lsp can only handle values up to {} characters in size, but this file has at least {size} characters",
        u32::MAX
      );
      (range, msg)
    },
    LspLexerError(UnknownToken { culprit, source_loc }) => {
      (as_range(&source_loc), format!("Unknown token: {culprit}"))
    },

    LspParserError(ExtraToken { token }) => {
      (as_range(&token.source_loc), format!("Token found after EOF: {:?}", token.token_type))
    },
    LspParserError(FictionalToken { location }) => {
      (as_range_mini(location), format!("Unparseable token type at location: {location:?}"))
    },
    LspParserError(MissingParameterValue { token }) => {
      let msg = "Parameter name requires accompanying value";
      (as_range(&token.source_loc), msg.to_string())
    },
    LspParserError(UnexpectedEOF { location, expected }) => {
      (as_range_mini(location), format!("Unexpected EOF at location {location:?}\nExpected: ${expected:?}"))
    },
    LspParserError(WrongToken { token, expected }) => {
      let msg = format!("Wrong token for this context: {token:?}\nExpected: ${expected:?}");
      (as_range(&token.source_loc), msg)
    },

    LspAnalyzerError(AnalyzerError { typ: BadInternalState, offender }) => {
      let msg = format!("Fatal internal error on `{:?}`", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: DuplicateParameter, offender }) => {
      let msg = format!("This function already has a parameter named \"{:?}\"", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: DuplicateVar, offender }) => {
      let msg = format!("Duplicate variable: {:?}", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: ExtraArgument { name }, offender }) => {
      let msg = format!("Unexpected argument to function \"{:?}\": {name}", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: MissingArgument { name, typ }, offender }) => {
      let msg =
        format!("Missing argument of type \"{:?}\" to function \"{:?}\": {name}", typ, offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: NoSuchFn, offender }) => {
      let msg = format!("No such function: {:?}", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: NoSuchVariable, offender }) => {
      let msg = format!("No such variable: {:?}", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: TypeMismatch { expected, got }, offender }) => {
      // TODO: In expected type, if it's a function, print that certain args are optional
      let msg = format!(
        "Could not match expected type `{expected:?}` with actual type `{got:?}`, regarding value `{:?}`.",
        offender.token_type
      );
      (as_range(&offender.source_loc), msg)
    },
    LspAnalyzerError(AnalyzerError { typ: VarCannotInitInTermsOfSelf, offender }) => {
      let msg = format!("`{:?}` cannot be defined in terms of itself", offender.token_type);
      (as_range(&offender.source_loc), msg)
    },
  };

  Diagnostic { range, severity: Some(DiagnosticSeverity::ERROR), message, ..Default::default() }
}

#[must_use]
pub fn warning_as_diagnostic(warning: AnalyzerWarning) -> Diagnostic {
  let (range, message) = match warning {
    AnalyzerWarning { typ: ArgOverridesPrevious, offender } => {
      let msg = "This argument overrides a previous one of the same name";
      (as_range(&offender.source_loc), msg.to_string())
    },
    AnalyzerWarning { typ: IntermediateCallInFnDef, offender } => {
      let msg = "This function call does nothing, since it is not the last statement";
      (as_range(&offender.source_loc), msg.to_string())
    },
    AnalyzerWarning { typ: UselessFnBody, offender } => {
      let msg =
        "This entire function body does nothing, since it does not call a function in its final statement";
      (as_range(&offender.source_loc), msg.to_string())
    },
  };

  Diagnostic { range, severity: Some(DiagnosticSeverity::WARNING), message, ..Default::default() }
}

const fn as_range_mini(mini: MiniLoc) -> Range {
  as_range3(mini.line, mini.column, 1)
}

const fn as_range(source_loc: &SourceLoc) -> Range {
  as_range3(source_loc.line, source_loc.column, source_loc.length)
}

const fn as_range3(line: u32, column: u32, length: u32) -> Range {
  Range {
    start: Position { line: line - 1, character: column - 1 },
    end: Position { line: line - 1, character: column - 1 + length },
  }
}
