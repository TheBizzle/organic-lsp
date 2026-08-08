use std::sync::Arc;

use crate::lexer::token::{Token, TokenType::Identifier};

use crate::analyzer::organic_type::OrganicType as OT;
use crate::analyzer::value::{Accidental, Note, PitchClass as PC, TermDefn};

use crate::lsp::pretty_type::pretty_type;

#[must_use]
pub(super) fn describe_defn(defn: &TermDefn, ot_opt: Option<OT>, token_opt: Option<&Token>) -> String {
  match defn {
    TermDefn::BuiltinConstant { value } => {
      format!(
        "Built-in constant `{}` of type `{}`",
        ident_name(token_opt.expect("Built-in constant must surely have a token")),
        pretty_type(&value.as_type())
      )
    },
    TermDefn::BuiltinFn { value } => {
      format!(
        "Built-in function `{}` of type `{}`",
        ident_name(token_opt.expect("Built-in function must surely have a token")),
        pretty_type(&OT::Function(Arc::new(value.clone())))
      )
    },
    TermDefn::BuiltinNote { note } => {
      let hertz = calculate_note(note);
      let hertz_3_decimals = (hertz * 1000.0).round() / 1000.0;
      format!("Built-in note `{}` (`{}` Hz)", describe_note(note), hertz_3_decimals)
    },
    TermDefn::UserDefined { token, .. } => format!(
      "User-defined value
```scala
{}: {}
```",
      ident_name(token),
      pretty_type(&ot_opt.unwrap_or(OT::Unknown))
    ),
  }
}

fn describe_note(note: &Note) -> String {
  let Note { pitch_class, accidental, octave } = note;

  let pc = match pitch_class {
    PC::A => "A",
    PC::B => "B",
    PC::C => "C",
    PC::D => "D",
    PC::E => "E",
    PC::F => "F",
    PC::G => "G",
  };

  let acci = match accidental {
    Accidental::Flat => "b",
    Accidental::Natural => "",
    Accidental::Sharp => "#",
  };

  format!("{pc}{acci}{octave}")
}

fn calculate_note(note: &Note) -> f64 {
  let Note { pitch_class, accidental, octave } = note;

  let semitones_above_c: i32 = match pitch_class {
    PC::C => 0,
    PC::D => 2,
    PC::E => 4,
    PC::F => 5,
    PC::G => 7,
    PC::A => 9,
    PC::B => 11,
  };

  let acci: i32 = match accidental {
    Accidental::Flat => -1,
    Accidental::Natural => 0,
    Accidental::Sharp => 1,
  };

  let semis = 12 * (i32::from(*octave) + 1) + semitones_above_c + acci;
  440.0 * ((f64::from(semis) - 69.0) / 12.0).exp2()
}

fn ident_name(token: &Token) -> &str {
  if let Identifier(name) = &token.token_type {
    name
  } else {
    panic!("You shouldn't be able to get here with something that isn't an identifier: {token:?}")
  }
}
