#[cfg(test)]
mod tests {

  use std::path::PathBuf;

  use tokio::fs::read_to_string;

  use tower_lsp_server::LanguageServer;
  use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidOpenTextDocumentParams, NumberOrString, Position, Range,
    TextDocumentItem,
  };

  use organic_lsp::core::doc_loc::DocLoc;
  use organic_lsp::lsp::new_lsp;

  #[tokio::test]
  async fn can_open_cascade() {
    test_no_problem("./Organic/examples/cascade").await;
  }

  #[tokio::test]
  async fn can_open_chord_swell() {
    test_no_problem("./Organic/examples/chord_swell").await;
  }

  #[tokio::test]
  async fn can_open_groovy_bass() {
    test_no_problem("./Organic/examples/groovy_bass").await;
  }

  #[tokio::test]
  async fn can_open_harmonics() {
    test_no_problem("./Organic/examples/harmonics").await;
  }

  #[tokio::test]
  async fn can_open_siren() {
    test_no_problem("./Organic/examples/siren").await;
  }

  #[tokio::test]
  async fn can_open_spread_phase() {
    test_no_problem("./Organic/examples/spread_phase").await;
  }

  #[tokio::test]
  #[should_panic(expected = "No such file or directory")]
  async fn fails_to_open_nonexistent() {
    test_errors("./Organic/examples/doopy", Vec::new()).await;
  }

  #[tokio::test]
  async fn opens_and_errors_on_invalid_lex() {
    let severity = Some(DiagnosticSeverity::ERROR);

    let eof_message =
      "Unexpected EOF at location MiniLoc { line: 1, column: 21 }\nExpected: $[\"\\\"=\\\"\", \"\\\"(\\\"\"]"
        .to_string();

    let start1 = Position { line: 0, character: 20 };
    let end1 = Position { line: 0, character: 21 };
    let range1 = Range { start: start1, end: end1 };
    let diagnostic1 =
      Diagnostic { range: range1, severity, code: code(5), message: eof_message, ..Default::default() };

    let message = "Unknown token: '".to_string();

    let start2 = Position { line: 0, character: 9 };
    let end2 = Position { line: 0, character: 10 };
    let range2 = Range { start: start2, end: end2 };
    let diagnostic2 =
      Diagnostic { range: range2, severity, code: code(1), message: message.clone(), ..Default::default() };

    let start3 = Position { line: 0, character: 20 };
    let end3 = Position { line: 0, character: 21 };
    let range3 = Range { start: start3, end: end3 };
    let diagnostic3 = Diagnostic { range: range3, severity, code: code(1), message, ..Default::default() };

    test_errors("./tests/invalid_lex", vec![diagnostic1, diagnostic2, diagnostic3]).await;
  }

  #[tokio::test]
  async fn opens_and_errors_on_invalid_parse() {
    let severity = Some(DiagnosticSeverity::ERROR);
    let message = "Wrong token for this context: Token { token_type: Identifier(\"apples\"), source_loc: SourceLoc { doc_loc: DocLoc(\"file://./tests/invalid_parse.organic\"), pos: 4, line: 1, column: 5, length: 6 } }\nExpected: $[\"\\\"=\\\"\", \"\\\"(\\\"\"]".to_string();

    let start1 = Position { line: 0, character: 4 };
    let end1 = Position { line: 0, character: 10 };
    let range1 = Range { start: start1, end: end1 };
    let diagnostic1 =
      Diagnostic { range: range1, severity, code: code(6), message: message.clone(), ..Default::default() };

    test_errors("./tests/invalid_parse", vec![diagnostic1]).await;
  }

  #[tokio::test]
  async fn opens_and_errors_on_invalid_analysis() {
    let severity = Some(DiagnosticSeverity::ERROR);

    let message1 = "No such variable: Identifier(\"note_length\")".to_string();
    let start1 = Position { line: 2, character: 76 };
    let end1 = Position { line: 2, character: 87 };
    let range1 = Range { start: start1, end: end1 };
    let diagnostic1 =
      Diagnostic { range: range1, severity, code: code(13), message: message1, ..Default::default() };

    let message2 = "Could not match expected type `number` with actual type `???`, regarding value `Identifier(\"length\")`.".to_string();
    let start2 = Position { line: 2, character: 68 };
    let end2 = Position { line: 2, character: 74 };
    let range2 = Range { start: start2, end: end2 };
    let diagnostic2 =
      Diagnostic { range: range2, severity, code: code(14), message: message2, ..Default::default() };

    test_errors("./tests/invalid_analysis", vec![diagnostic1, diagnostic2]).await;
  }

  async fn test_no_problem(path: &str) {
    let diagnostics = open(path).await;
    assert!(diagnostics.is_empty(), "Expected {path} to have no errors, got: {diagnostics:?}");
  }

  async fn test_errors(path: &str, expected: Vec<Diagnostic>) {
    let actual = open(path).await;
    assert_eq!(actual, expected);
  }

  async fn open(path: &str) -> Vec<Diagnostic> {
    let path = PathBuf::from(format!("{path}.organic"));
    let text = read_to_string(&path).await.unwrap();

    let mini_uri = format!("file://{}", path.display());
    let uri = mini_uri.clone().parse().unwrap();

    let backend_service = {
      let (service, _socket) = new_lsp();
      service
    };
    let backend = backend_service.inner();

    backend
      .did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem { uri, language_id: "organic".into(), version: 1, text },
      })
      .await;

    backend.documents.read().await.get(&DocLoc::new(mini_uri)).unwrap().diagnostics.clone()
  }

  #[allow(clippy::unnecessary_wraps)]
  fn code(n: i32) -> Option<NumberOrString> {
    Some(NumberOrString::Number(n))
  }
}
