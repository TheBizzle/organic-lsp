use crate::analyzer::function::Function;
use crate::analyzer::value::PitchClass as PC;
use crate::analyzer::value::{Accidental, ConstantValue, Note, TermDefn};
use ConstantValue as CV;

#[must_use]
pub fn describe_defn(defn: &TermDefn) -> String {
  match defn {
    TermDefn::BuiltinConstant { value } => format!("Built-in constant: {}", describe_constant(value)),
    TermDefn::BuiltinFn { value } => format!("Built-in function: {}", describe_function(value)),
    TermDefn::BuiltinNote { note } => {
      let hertz = calculate_note(note);
      let hertz_3_decimals = (hertz * 1000.0).round() / 1000.0;
      format!("Built-in note: {} ({})", describe_note(note), hertz_3_decimals)
    },
    TermDefn::UserDefined { token } => format!("User-defined value: {token:?}"), // TODO: Get
                                                                                 // type info
                                                                                 // in here
  }
}

fn describe_constant(value: &ConstantValue) -> String {
  match value {
    CV::AudioEffect => "AudioEffect".to_string(),
    CV::Boolean(_) => "Boolean".to_string(),
    CV::List(inner) => format!("List[{}]", describe_constant(inner)),
    CV::Number(_) => "Number".to_string(),
    CV::RandomArg => "RandomnessArgument".to_string(), // TODO
    CV::RoundArg => "RoundArgument".to_string(),       // TODO
    CV::SequenceArg => "SequenceArgument".to_string(), // TODO
    CV::String(_) => "String".to_string(),
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

fn describe_function(func: &Function) -> String {
  let Function { params, return_type } = func;
  format!("({params:?}): {return_type:?}") // TODO: Don't just dump debugging output
}
