use crate::lexer::source_loc::SourceLoc;

#[derive(Debug)]
pub enum LexerError {
  FileTooBig { size: usize, line_num: u32 },
  UnknownToken { culprit: String, source_loc: SourceLoc },
}
