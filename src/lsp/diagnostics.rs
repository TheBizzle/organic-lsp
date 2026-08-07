use strum::{EnumCount, FromRepr};

use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

use crate::lexer::diagnostics::LexerError::{self, FileTooBig, UnknownToken};
use crate::lexer::source_loc::{MiniLoc, SourceLoc};
use crate::lexer::token::{Token, TokenType::Identifier};

use crate::parser::diagnostics::ParserError::{
  self, ExtraToken, FictionalToken, MissingParameterValue, UnexpectedEOF, WrongToken,
};

use crate::analyzer::diagnostics::{AnalyzerErrorType, AnalyzerWarningType};

use crate::analyzer::diagnostics::AnalyzerErrorType::{
  BadInternalState, DuplicateParameter, DuplicateVar, ExtraArgument, MissingArgument, NoSuchFn,
  NoSuchVariable, TypeMismatch, VarCannotInitInTermsOfSelf,
};

use crate::analyzer::diagnostics::AnalyzerWarningType::{
  ArgOverridesPrevious, CamelCase, IntermediateCallInFnDef, SnakeCase, UselessFnBody,
};

use crate::lsp::kebab_cased::kebab_cased;
use crate::lsp::pretty_type::pretty_type;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LspError {
  LspLexerError(LexerError),
  LspParserError(ParserError),
  LspAnalyzerError { typ: AnalyzerErrorType, offender: Token },
}
use LspError::{LspAnalyzerError, LspLexerError, LspParserError};

#[derive(FromRepr, EnumCount, Eq, PartialEq)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub(super) enum DiagnosticCode {
  Lexer_Error_FileTooBig,
  Lexer_Error_UnknownToken,
  Parser_Error_ExtraToken,
  Parser_Error_FictionalToken,
  Parser_Error_MissingParameterValue,
  Parser_Error_UnexpectedEOF,
  Parser_Error_WrongToken,
  Analyzer_Error_BadInternalState,
  Analyzer_Error_DuplicateParameter,
  Analyzer_Error_DuplicateVar,
  Analyzer_Error_ExtraArgument,
  Analyzer_Error_MissingArgument,
  Analyzer_Error_NoSuchFn,
  Analyzer_Error_NoSuchVariable,
  Analyzer_Error_TypeMismatch,
  Analyzer_Error_VarCannotInitInTermsOfSelf,
  Analyzer_Warning_ArgOverridesPrevious,
  Analyzer_Warning_CamelCase,
  Analyzer_Warning_IntermediateCallInFnDef,
  Analyzer_Warning_SnakeCase,
  Analyzer_Warning_UselessFnBody,
}
use DiagnosticCode as DC;

#[must_use]
pub fn error_as_diagnostic(error: LspError) -> Diagnostic {
  let (range, message, diag_code) = match error {
    LspLexerError(FileTooBig { size, line_num }) => {
      let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: line_num - 1, character: 0 },
      };
      let msg = format!(
        "This file is too large!  organic-lsp can only handle values up to {} characters in size, but this file has at least {size} characters",
        u32::MAX
      );
      (range, msg, DC::Lexer_Error_FileTooBig)
    },
    LspLexerError(UnknownToken { culprit, source_loc }) => {
      (as_range(&source_loc), format!("Unknown token: {culprit}"), DC::Lexer_Error_UnknownToken)
    },

    LspParserError(ExtraToken { token }) => {
      let dc = DC::Parser_Error_ExtraToken;
      (as_range(&token.source_loc), format!("Token found after EOF: {:?}", token.token_type), dc)
    },
    LspParserError(FictionalToken { location }) => {
      let dc = DC::Parser_Error_FictionalToken;
      (as_range_mini(location), format!("Unparseable token type at location: {location:?}"), dc)
    },
    LspParserError(MissingParameterValue { token }) => {
      let msg = "Parameter name requires accompanying value";
      (as_range(&token.source_loc), msg.to_string(), DC::Parser_Error_MissingParameterValue)
    },
    LspParserError(UnexpectedEOF { location, expected }) => {
      let msg = format!("Unexpected EOF at location {location:?}\nExpected: ${expected:?}");
      (as_range_mini(location), msg, DC::Parser_Error_UnexpectedEOF)
    },
    LspParserError(WrongToken { token, expected }) => {
      let msg = format!("Wrong token for this context: {token:?}\nExpected: ${expected:?}");
      (as_range(&token.source_loc), msg, DC::Parser_Error_WrongToken)
    },

    LspAnalyzerError { typ: BadInternalState, offender } => {
      let msg = format!("Fatal internal error on `{:?}`", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_BadInternalState)
    },
    LspAnalyzerError { typ: DuplicateParameter, offender } => {
      let msg = format!("This function already has a parameter named \"{:?}\"", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_DuplicateParameter)
    },
    LspAnalyzerError { typ: DuplicateVar, offender } => {
      let msg = format!("Duplicate variable: {:?}", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_DuplicateVar)
    },
    LspAnalyzerError { typ: ExtraArgument { name }, offender } => {
      let msg = format!("Unexpected argument to function \"{:?}\": {name}", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_ExtraArgument)
    },
    LspAnalyzerError { typ: MissingArgument { name, typ }, offender } => {
      let msg =
        format!("Missing argument of type \"{:?}\" to function \"{:?}\": {name}", typ, offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_MissingArgument)
    },
    LspAnalyzerError { typ: NoSuchFn, offender } => {
      let msg = format!("No such function: {:?}", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_NoSuchFn)
    },
    LspAnalyzerError { typ: NoSuchVariable, offender } => {
      let msg = format!("No such variable: {:?}", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_NoSuchVariable)
    },
    LspAnalyzerError { typ: TypeMismatch { expected, got }, offender } => {
      let msg = format!(
        "Could not match expected type `{}` with actual type `{}`, regarding value `{:?}`.",
        pretty_type(&expected),
        pretty_type(&got),
        offender.token_type
      );
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_TypeMismatch)
    },
    LspAnalyzerError { typ: VarCannotInitInTermsOfSelf, offender } => {
      let msg = format!("`{:?}` cannot be defined in terms of itself", offender.token_type);
      (as_range(&offender.source_loc), msg, DC::Analyzer_Error_VarCannotInitInTermsOfSelf)
    },
  };

  Diagnostic {
    range,
    severity: Some(DiagnosticSeverity::ERROR),
    message,
    code: Some(NumberOrString::Number(diag_code as i32)),
    ..Default::default()
  }
}

#[must_use]
pub fn warning_as_diagnostic(warning: &AnalyzerWarningType, offender: Token) -> Diagnostic {
  let (range, message, diag_code) = match warning {
    ArgOverridesPrevious => {
      let msg = "This argument overrides a previous one of the same name";
      (as_range(&offender.source_loc), msg.to_string(), DC::Analyzer_Warning_ArgOverridesPrevious)
    },
    CamelCase => {
      if let Identifier(name) = offender.token_type {
        let msg = format!(
          "Variable `{name}` should have a `kebab-case` name (e.g. `{}`), but it's in `camelCase`",
          kebab_cased(&name)
        );
        (as_range(&offender.source_loc), msg, DC::Analyzer_Warning_CamelCase)
      } else {
        panic!("Impossible snake-cased non-identifer identifier: {offender:?}")
      }
    },
    IntermediateCallInFnDef => {
      let msg = "This function call does nothing, since it is not the last statement";
      (as_range(&offender.source_loc), msg.to_string(), DC::Analyzer_Warning_IntermediateCallInFnDef)
    },
    SnakeCase => {
      if let Identifier(name) = offender.token_type {
        let msg = format!(
          "Variable `{name}` should have a `kebab-case` name (e.g. `{}`), but it's in `snake_case`",
          kebab_cased(&name)
        );
        (as_range(&offender.source_loc), msg, DC::Analyzer_Warning_SnakeCase)
      } else {
        panic!("Impossible snake-cased non-identifer identifier: {offender:?}")
      }
    },
    UselessFnBody => {
      let msg =
        "This entire function body does nothing, since it does not call a function in its final statement";
      (as_range(&offender.source_loc), msg.to_string(), DC::Analyzer_Warning_UselessFnBody)
    },
  };

  Diagnostic {
    range,
    severity: Some(DiagnosticSeverity::WARNING),
    message,
    code: Some(NumberOrString::Number(diag_code as i32)),
    ..Default::default()
  }
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
