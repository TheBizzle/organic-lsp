use std::collections::HashMap;
use std::f64::consts;
use std::sync::{Arc, LazyLock};

use strum::IntoEnumIterator;

use crate::core::address::{NamedVarAddress, ScopeAddress};

use crate::analyzer::function::{Function, ParamInfo as PI};
use crate::analyzer::organic_type::OrganicType;
use crate::analyzer::value::TermDefn::{self, BuiltinConstant, BuiltinFn, BuiltinNote};
use crate::analyzer::value::{Accidental, ConstantValue, Note, PitchClass};

use ConstantValue as CV;
use OrganicType as OT;

pub(super) struct Constant {
  name: &'static str,
  value: ConstantValue,
}

pub(super) struct StdLibFn {
  name: &'static str,
  func: Function,
}

#[derive(Clone)]
pub(super) struct BuiltIns {
  pub bindings: HashMap<String, NamedVarAddress>,
  pub defs: HashMap<NamedVarAddress, TermDefn>,
  pub vars: HashMap<NamedVarAddress, OrganicType>,
}

#[must_use]
pub(super) fn initial_state() -> BuiltIns {
  (*INITIAL_STATE).clone()
}

#[allow(clippy::type_complexity)]
static INITIAL_STATE: LazyLock<BuiltIns> = LazyLock::new(|| {
  let scope_addr = INITIAL_SCOPE_ADDRESS;

  let (const_defs, constants): (HashMap<_, _>, HashMap<_, _>) = CONSTANTS
    .iter()
    .map(|Constant { name, value }| {
      let addr = NamedVarAddress { name: name.to_string(), scope_addr: scope_addr.clone() };
      let value_mapping = (addr.clone(), BuiltinConstant { value: value.clone() });
      let type_mapping = (addr, value_to_type(value.clone()));
      (value_mapping, type_mapping)
    })
    .collect();

  let (func_defs, functions): (HashMap<_, _>, HashMap<_, _>) = FUNCTIONS
    .iter()
    .map(|StdLibFn { name, func }| {
      let addr = NamedVarAddress { name: name.to_string(), scope_addr: scope_addr.clone() };
      let value_mapping = (addr.clone(), BuiltinFn { value: func.clone() });
      let typ = OrganicType::Function(Arc::new(func.clone()));
      let type_mapping = (addr, typ);
      (value_mapping, type_mapping)
    })
    .collect();

  let (note_defs, notes): (HashMap<_, _>, HashMap<_, _>) = NOTES
    .iter()
    .map(|(name, note)| {
      let addr = NamedVarAddress { name: name.clone(), scope_addr: scope_addr.clone() };
      let value_mapping = (addr.clone(), BuiltinNote { note: note.clone() });
      let type_mapping = (addr, OrganicType::Number);
      (value_mapping, type_mapping)
    })
    .collect();

  let vars: HashMap<_, _> = constants.into_iter().chain(functions).chain(notes).collect();
  let bindings: HashMap<_, _> = vars.keys().map(|addr| (addr.name.clone(), addr.clone())).collect();
  let defs = const_defs.into_iter().chain(func_defs).chain(note_defs).collect();

  BuiltIns { bindings, defs, vars }
});

fn value_to_type(value: ConstantValue) -> OrganicType {
  match value {
    CV::AudioEffect => OT::AudioEffect,
    CV::Boolean(_) => OT::Boolean,
    CV::List(inner_type) => OT::List(Box::new(value_to_type(*inner_type))),
    CV::Number(_) => OT::Number,
    CV::RandomArg => OT::RandomArg,
    CV::RoundArg => OT::RoundArg,
    CV::SequenceArg => OT::SequenceArg,
    CV::String(_) => OT::String,
  }
}

pub(super) static INITIAL_SCOPE_ADDRESS: &ScopeAddress = &ScopeAddress { n: 0 };

static FUNCTIONS: LazyLock<[StdLibFn; 32]> = LazyLock::new(|| {
  [
    StdLibFn {
      name: "absolute",
      func: Function { params: vec![PI(own("value"), OT::Number, false)], return_type: OT::Number },
    },
    StdLibFn {
      name: "all",
      func: Function {
        params: vec![PI(own("values"), OT::List(Box::new(OT::Boolean)), false)],
        return_type: OT::Boolean,
      },
    },
    StdLibFn {
      name: "all-pass",
      func: Function {
        params: vec![
          PI(own("feedback"), OT::Number, false),
          PI(own("delay"), OT::Number, false),
          PI(own("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "any",
      func: Function {
        params: vec![PI(own("values"), OT::List(Box::new(OT::Boolean)), false)],
        return_type: OT::Boolean,
      },
    },
    StdLibFn {
      name: "comb",
      func: Function {
        params: vec![
          PI(own("feedback"), OT::Number, false),
          PI(own("delay"), OT::Number, false),
          PI(own("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "delay",
      func: Function {
        params: vec![
          PI(own("feedback"), OT::Number, false),
          PI(own("delay"), OT::Number, false),
          PI(own("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "effect-group",
      func: Function {
        params: vec![
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), false),
          PI(own("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "granulate",
      func: Function {
        params: vec![
          PI(
            own("shape"),
            OT::Function(Arc::new(Function {
              params: vec![PI(own("value"), OT::Number, false)],
              return_type: OT::Number,
            })),
            true,
          ),
          PI(own("length"), OT::Number, true),
          PI(own("grains"), OT::Number, true),
          PI(own("sample"), OT::String, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "group",
      func: Function {
        params: vec![
          PI(own("sources"), OT::List(Box::new(OT::Number)), false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "hold",
      func: Function {
        params: vec![
          PI(own("length"), OT::Number, false),
          PI(own("value"), OT::Generic("Input".to_string()), false),
        ],
        return_type: OT::Generic("Input".to_string()),
      },
    },
    StdLibFn {
      name: "if",
      func: Function {
        params: vec![
          PI(own("is-false"), OT::Generic("Input".to_string()), false),
          PI(own("is-true"), OT::Generic("Input".to_string()), false),
          PI(own("condition"), OT::Boolean, false),
        ],
        return_type: OT::Generic("Input".to_string()),
      },
    },
    StdLibFn {
      name: "lfo",
      func: Function {
        params: vec![
          PI(own("length"), OT::Number, false),
          PI(own("to"), OT::Number, false),
          PI(own("from"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "limit",
      func: Function {
        params: vec![
          PI(own("max"), OT::Number, false),
          PI(own("min"), OT::Number, false),
          PI(own("value"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "low-pass",
      func: Function { params: vec![PI(own("threshold"), OT::Number, false)], return_type: OT::Number },
    },
    StdLibFn {
      name: "max",
      func: Function {
        params: vec![PI(own("values"), OT::List(Box::new(OT::Number)), false)],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "min",
      func: Function {
        params: vec![PI(own("values"), OT::List(Box::new(OT::Number)), false)],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "noise",
      func: Function {
        params: vec![
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "none",
      func: Function {
        params: vec![PI(own("values"), OT::List(Box::new(OT::Boolean)), false)],
        return_type: OT::Boolean,
      },
    },
    StdLibFn {
      name: "oscillator",
      func: Function {
        params: vec![
          PI(
            own("waveform"),
            OT::Function(Arc::new(Function {
              params: vec![PI(own("phase"), OT::Number, false)],
              return_type: OT::Number,
            })),
            false,
          ),
          PI(own("frequency"), OT::Number, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "random",
      func: Function {
        params: vec![
          PI(own("type"), OT::RandomArg, true),
          PI(own("length"), OT::Number, false),
          PI(own("to"), OT::Number, false),
          PI(own("from"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "repeat",
      func: Function {
        params: vec![
          PI(own("repeats"), OT::Number, true),
          PI(own("value"), OT::Generic("Input".to_string()), false),
        ],
        return_type: OT::Generic("Input".to_string()),
      },
    },
    StdLibFn {
      name: "reverb",
      func: Function {
        params: vec![PI(own("length"), OT::Number, false), PI(own("mix"), OT::Number, true)],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "round",
      func: Function {
        params: vec![
          PI(own("direction"), OT::RoundArg, true),
          PI(own("step"), OT::Number, true),
          PI(own("value"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sample",
      func: Function {
        params: vec![
          PI(own("file"), OT::String, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "saw",
      func: Function {
        params: vec![
          PI(own("frequency"), OT::Number, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sequence",
      func: Function {
        params: vec![
          PI(own("order"), OT::SequenceArg, true),
          PI(own("values"), OT::List(Box::new(OT::Generic("Input".to_string()))), false),
        ],
        return_type: OT::Generic("Input".to_string()),
      },
    },
    StdLibFn {
      name: "sine",
      func: Function {
        params: vec![
          PI(own("frequency"), OT::Number, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "square",
      func: Function {
        params: vec![
          PI(own("frequency"), OT::Number, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sweep",
      func: Function {
        params: vec![
          PI(own("length"), OT::Number, false),
          PI(own("to"), OT::Number, false),
          PI(own("from"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn { name: "time", func: Function { params: vec![], return_type: OT::Number } },
    StdLibFn {
      name: "triangle",
      func: Function {
        params: vec![
          PI(own("frequency"), OT::Number, false),
          PI(own("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(own("pan"), OT::Number, true),
          PI(own("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "trigger",
      func: Function {
        params: vec![
          PI(own("value"), OT::Generic("Input".to_string()), false),
          PI(own("condition"), OT::Boolean, false),
        ],
        return_type: OT::Generic("Input".to_string()),
      },
    },
  ]
});

static CONSTANTS: &[Constant] = &[
  Constant { name: "backward", value: CV::SequenceArg },
  Constant { name: "down", value: CV::RoundArg },
  Constant { name: "e", value: CV::Number(consts::E) },
  Constant { name: "false", value: CV::Boolean(false) },
  Constant { name: "forward", value: CV::SequenceArg },
  Constant { name: "linear", value: CV::RandomArg },
  Constant { name: "nearest", value: CV::RoundArg },
  Constant { name: "pi", value: CV::Number(consts::PI) },
  Constant { name: "shuffle", value: CV::SequenceArg },
  Constant { name: "step", value: CV::RandomArg },
  Constant { name: "tau", value: CV::Number(consts::TAU) },
  Constant { name: "true", value: CV::Boolean(true) },
  Constant { name: "up", value: CV::RoundArg },
];

// e.g. `as5` is an A# on the 5th octave
static NOTES: LazyLock<HashMap<String, Note>> = LazyLock::new(|| {
  PitchClass::iter()
    .flat_map(|pitch_class| Accidental::iter().map(move |accidental| (pitch_class.clone(), accidental)))
    .flat_map(|(pitch, acci)| {
      (0..=9).map(move |octave| Note { pitch_class: pitch.clone(), accidental: acci.clone(), octave })
    })
    .map(|note| {
      let Note { pitch_class, accidental, octave } = note.clone();

      let pitch = match pitch_class {
        PitchClass::A => 'a',
        PitchClass::B => 'b',
        PitchClass::C => 'c',
        PitchClass::D => 'd',
        PitchClass::E => 'e',
        PitchClass::F => 'f',
        PitchClass::G => 'g',
      };

      let acci = match accidental {
        Accidental::Flat => "f",
        Accidental::Natural => "",
        Accidental::Sharp => "s",
      };

      (format!("{pitch}{acci}{octave}"), note)
    })
    .collect()
});

fn own(value: &str) -> String {
  value.to_owned()
}
