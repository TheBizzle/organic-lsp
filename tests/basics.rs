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
  use organic_lsp::lsp::diagnostics::DiagnosticCode as DC;
  use organic_lsp::lsp::document::Document;
  use organic_lsp::lsp::miniformat::miniformat;
  use organic_lsp::lsp::new_lsp;

  #[tokio::test]
  async fn can_open_basic_chord() {
    test_no_problem("./Organic/examples/basic/chord").await;
  }

  #[tokio::test]
  async fn can_open_basic_hello_world() {
    test_no_problem("./Organic/examples/basic/hello-world").await;
  }

  #[tokio::test]
  async fn can_open_basic_mountain() {
    test_no_problem("./Organic/examples/basic/mountain").await;
  }

  #[tokio::test]
  async fn can_open_music_pluck() {
    test_no_problem("./Organic/examples/basic/pluck").await;
  }

  #[tokio::test]
  async fn can_open_music_scale() {
    test_no_problem("./Organic/examples/basic/scale").await;
  }

  #[tokio::test]
  async fn can_open_music_arps() {
    test_no_problem("./Organic/examples/music/arps").await;
  }

  #[tokio::test]
  async fn can_open_music_cascade() {
    test_no_problem("./Organic/examples/music/cascade").await;
  }

  #[tokio::test]
  async fn can_open_music_long_chords() {
    test_no_problem("./Organic/examples/music/long-chords").await;
  }

  #[tokio::test]
  async fn can_open_music_partials() {
    test_no_problem("./Organic/examples/music/partials").await;
  }

  #[tokio::test]
  async fn can_open_music_random_swell() {
    test_no_problem("./Organic/examples/music/random-swell").await;
  }

  #[tokio::test]
  async fn can_open_music_wind_chimes() {
    test_no_problem("./Organic/examples/music/wind-chimes").await;
  }

  #[tokio::test]
  async fn can_open_sounds_crossing() {
    test_no_problem("./Organic/examples/sounds/crossing").await;
  }

  #[tokio::test]
  async fn can_open_sounds_cascade() {
    test_no_problem("./Organic/examples/music/cascade").await;
  }

  #[tokio::test]
  async fn can_open_sounds_dial_tone() {
    test_no_problem("./Organic/examples/sounds/dial-tone").await;
  }

  #[tokio::test]
  async fn can_open_sounds_flange() {
    test_no_problem("./Organic/examples/sounds/flange").await;
  }

  #[tokio::test]
  async fn can_open_sounds_groovy_bass() {
    test_no_problem("./Organic/examples/sounds/groovy-bass").await;
  }

  #[tokio::test]
  async fn can_open_sounds_ping() {
    test_no_problem("./Organic/examples/sounds/ping").await;
  }

  #[tokio::test]
  async fn can_open_sounds_probe() {
    test_no_problem("./Organic/examples/sounds/probe").await;
  }

  #[tokio::test]
  async fn can_open_sounds_siren() {
    test_no_problem("./Organic/examples/sounds/siren").await;
  }

  #[tokio::test]
  async fn can_open_sounds_spread_phase() {
    test_no_problem("./Organic/examples/sounds/spread-phase").await;
  }

  #[tokio::test]
  async fn can_open_sounds_wub() {
    test_no_problem("./Organic/examples/sounds/wub").await;
  }

  #[tokio::test]
  async fn can_open_crlf_lex() {
    test_no_problem("./tests/crlf_lex").await;
  }

  #[tokio::test]
  async fn can_open_generics() {
    test_no_problem("./tests/generics").await;
  }

  #[tokio::test]
  async fn can_open_old_cascade() {
    let severity = Some(DiagnosticSeverity::HINT);

    let message1 =
      "Variable `note_attack` should have a `kebab-case` name (e.g. `note-attack`), but it's in `snake_case`"
        .to_string();

    let start1 = Position { line: 0, character: 0 };
    let end1 = Position { line: 0, character: 11 };
    let range1 = Range { start: start1, end: end1 };
    let diagnostic1 = Diagnostic {
      range: range1,
      severity,
      code: code(DC::Analyzer_Lint_SnakeCase),
      message: message1,
      ..Default::default()
    };

    let message2 =
      "Variable `note_length` should have a `kebab-case` name (e.g. `note-length`), but it's in `snake_case`"
        .to_string();

    let start2 = Position { line: 1, character: 0 };
    let end2 = Position { line: 1, character: 11 };
    let range2 = Range { start: start2, end: end2 };
    let diagnostic2 = Diagnostic {
      range: range2,
      severity,
      code: code(DC::Analyzer_Lint_SnakeCase),
      message: message2,
      ..Default::default()
    };

    test_errors("./tests/old_cascade", vec![diagnostic1, diagnostic2]).await;
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
    let diagnostic1 = Diagnostic {
      range: range1,
      severity,
      code: code(DC::Parser_Error_UnexpectedEOF),
      message: eof_message,
      ..Default::default()
    };

    let message = "Unknown token: '".to_string();

    let start2 = Position { line: 0, character: 9 };
    let end2 = Position { line: 0, character: 10 };
    let range2 = Range { start: start2, end: end2 };
    let diagnostic2 = Diagnostic {
      range: range2,
      severity,
      code: code(DC::Lexer_Error_UnknownToken),
      message: message.clone(),
      ..Default::default()
    };

    let start3 = Position { line: 0, character: 20 };
    let end3 = Position { line: 0, character: 21 };
    let range3 = Range { start: start3, end: end3 };
    let diagnostic3 = Diagnostic {
      range: range3,
      severity,
      code: code(DC::Lexer_Error_UnknownToken),
      message,
      ..Default::default()
    };

    test_errors("./tests/invalid_lex", vec![diagnostic1, diagnostic2, diagnostic3]).await;
  }

  #[tokio::test]
  async fn opens_and_errors_on_invalid_parse() {
    let severity = Some(DiagnosticSeverity::ERROR);
    let message = "Wrong token for this context: Token { token_type: Identifier(\"apples\"), source_loc: SourceLoc { doc_loc: DocLoc(\"file://./tests/invalid_parse.organic\"), pos: 4, line: 1, column: 5, length: 6 } }\nExpected: $[\"\\\"=\\\"\", \"\\\"(\\\"\"]".to_string();

    let start1 = Position { line: 0, character: 4 };
    let end1 = Position { line: 0, character: 10 };
    let range1 = Range { start: start1, end: end1 };
    let diagnostic1 = Diagnostic {
      range: range1,
      severity,
      code: code(DC::Parser_Error_WrongToken),
      message: message.clone(),
      ..Default::default()
    };

    test_errors("./tests/invalid_parse", vec![diagnostic1]).await;
  }

  #[tokio::test]
  async fn opens_and_errors_on_invalid_analysis() {
    let severity = Some(DiagnosticSeverity::ERROR);

    let message1 = "No such variable: Identifier(\"note_length\")".to_string();
    let start1 = Position { line: 2, character: 76 };
    let end1 = Position { line: 2, character: 87 };
    let range1 = Range { start: start1, end: end1 };
    let diagnostic1 = Diagnostic {
      range: range1,
      severity,
      code: code(DC::Analyzer_Error_NoSuchVariable),
      message: message1,
      ..Default::default()
    };

    let message2 = "Could not match expected type `number` with actual type `???`, regarding value `Identifier(\"length\")`.".to_string();
    let start2 = Position { line: 2, character: 68 };
    let end2 = Position { line: 2, character: 74 };
    let range2 = Range { start: start2, end: end2 };
    let diagnostic2 = Diagnostic {
      range: range2,
      severity,
      code: code(DC::Analyzer_Error_TypeMismatch),
      message: message2,
      ..Default::default()
    };

    test_errors("./tests/invalid_analysis", vec![diagnostic1, diagnostic2]).await;
  }

  #[tokio::test]
  async fn can_open_formatted() {
    test_no_problem("./tests/formatted_harmonics").await;
  }

  #[tokio::test]
  async fn can_format_formatted_idempotent() {
    let file_path = "./tests/formatted_harmonics";
    let path = PathBuf::from(format!("{file_path}.organic"));
    let original = read_to_string(&path).await.unwrap();
    let original_no_eof_nl = &original[..original.len() - 1];

    open(file_path, |document| {
      let result = miniformat(&document.ast, 110);
      assert_eq!(result, original_no_eof_nl);
    })
    .await;
  }

  #[tokio::test]
  async fn can_open_unformatted() {
    test_no_problem("./tests/unformatted_harmonics").await;
  }

  #[tokio::test]
  async fn can_format_unformatted_changes() {
    let formatted_reference_text = {
      let original = read_to_string(&PathBuf::from("./tests/formatted_harmonics.organic")).await.unwrap();
      original[..original.len() - 1].to_string()
    };

    let file_path = "./tests/unformatted_harmonics";
    let unformatted_reference_text = {
      let original = read_to_string(&PathBuf::from(format!("{file_path}.organic"))).await.unwrap();
      original[..original.len() - 1].to_string()
    };

    open(file_path, |document| {
      let result = miniformat(&document.ast, 110);
      assert_ne!(result, unformatted_reference_text);
      assert_eq!(result, formatted_reference_text);
    })
    .await;
  }

  async fn test_no_problem(path: &str) {
    let diagnostics = open_and_diagnose(path).await;
    assert!(diagnostics.is_empty(), "Expected {path} to have no errors, got: {diagnostics:?}");
  }

  async fn test_errors(path: &str, expected: Vec<Diagnostic>) {
    let actual = open_and_diagnose(path).await;
    assert_eq!(actual, expected);
  }

  #[allow(clippy::significant_drop_tightening)]
  async fn open<T, F: Fn(&Document) -> T>(file_path: &str, callback: F) -> T {
    let path = PathBuf::from(format!("{file_path}.organic"));
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

    let document_store = backend.documents.read().await;
    let document = document_store.get(&DocLoc::new(mini_uri)).unwrap();

    callback(document)
  }

  async fn open_and_diagnose(path: &str) -> Vec<Diagnostic> {
    open(path, |document| document.diagnostics.clone()).await
  }

  #[allow(clippy::unnecessary_wraps)]
  fn code(n: DC) -> Option<NumberOrString> {
    Some(NumberOrString::Number(n as i32))
  }
}
