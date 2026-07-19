use std::borrow::Cow;
use std::collections::HashMap;
use std::f64::consts;
use std::sync::{Arc, LazyLock};

use strum::IntoEnumIterator;

use crate::analyzer::address::{NamedVarAddress, ScopeAddress};
use crate::analyzer::function::{Function, ParamInfo as PI};
use crate::analyzer::organic_type::OrganicType;
use crate::analyzer::value::TermDefn::{self, BuiltinConstant, BuiltinFn, BuiltinNote};
use crate::analyzer::value::{Accidental, ConstantValue, Note, PitchClass};

use ConstantValue as CV;
use OrganicType as OT;

pub struct Constant {
  name: &'static str,
  value: ConstantValue,
}

pub struct StdLibFn {
  name: &'static str,
  func: Function<'static>,
}

#[must_use]
pub fn initial_state() -> (
  HashMap<String, NamedVarAddress>,
  HashMap<NamedVarAddress, OrganicType<'static>>,
  HashMap<NamedVarAddress, TermDefn<'static>>,
) {
  let (bindings, vars, defs) = &*INITIAL_STATE;
  (bindings.clone(), vars.clone(), defs.clone())
}

#[allow(clippy::type_complexity)]
static INITIAL_STATE: LazyLock<(
  HashMap<String, NamedVarAddress>,
  HashMap<NamedVarAddress, OrganicType<'static>>,
  HashMap<NamedVarAddress, TermDefn<'static>>,
)> = LazyLock::new(|| {
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

  (bindings, vars, defs)
});

fn value_to_type<'a>(value: ConstantValue) -> OrganicType<'a> {
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

pub static INITIAL_SCOPE_ADDRESS: &ScopeAddress = &ScopeAddress { n: 0 };

static FUNCTIONS: LazyLock<[StdLibFn; 32]> = LazyLock::new(|| {
  [
    StdLibFn {
      name: "absolute",
      func: Function { params: vec![PI(cb("value"), OT::Number, false)], return_type: OT::Number },
    },
    StdLibFn {
      name: "all",
      func: Function {
        params: vec![PI(cb("values"), OT::List(Box::new(OT::Boolean)), false)],
        return_type: OT::Boolean,
      },
    },
    StdLibFn {
      name: "all-pass",
      func: Function {
        params: vec![
          PI(cb("feedback"), OT::Number, false),
          PI(cb("delay"), OT::Number, false),
          PI(cb("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "any",
      func: Function {
        params: vec![PI(cb("values"), OT::List(Box::new(OT::Boolean)), false)],
        return_type: OT::Boolean,
      },
    },
    StdLibFn {
      name: "comb",
      func: Function {
        params: vec![
          PI(cb("feedback"), OT::Number, false),
          PI(cb("delay"), OT::Number, false),
          PI(cb("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "delay",
      func: Function {
        params: vec![
          PI(cb("feedback"), OT::Number, false),
          PI(cb("delay"), OT::Number, false),
          PI(cb("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "effect-group",
      func: Function {
        params: vec![
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), false),
          PI(cb("mix"), OT::Number, true),
        ],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "granulate",
      func: Function {
        params: vec![
          PI(
            cb("shape"),
            OT::Function(Arc::new(Function {
              params: vec![PI(cb("value"), OT::Number, false)],
              return_type: OT::Number,
            })),
            true,
          ),
          PI(cb("length"), OT::Number, true),
          PI(cb("grains"), OT::Number, true),
          PI(cb("sample"), OT::String, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "group",
      func: Function {
        params: vec![
          PI(cb("sources"), OT::List(Box::new(OT::Number)), false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "hold",
      func: Function {
        params: vec![PI(cb("length"), OT::Number, false), PI(cb("value"), OT::Number, false)],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "if",
      func: Function {
        params: vec![
          PI(cb("is-false"), OT::Number, false),
          PI(cb("is-true"), OT::Number, false),
          PI(cb("condition"), OT::Boolean, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "lfo",
      func: Function {
        params: vec![
          PI(cb("length"), OT::Number, false),
          PI(cb("to"), OT::Number, false),
          PI(cb("from"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "limit",
      func: Function {
        params: vec![
          PI(cb("max"), OT::Number, false),
          PI(cb("min"), OT::Number, false),
          PI(cb("value"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "low-pass",
      func: Function { params: vec![PI(cb("threshold"), OT::Number, false)], return_type: OT::Number },
    },
    StdLibFn {
      name: "max",
      func: Function {
        params: vec![PI(cb("values"), OT::List(Box::new(OT::Number)), false)],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "min",
      func: Function {
        params: vec![PI(cb("values"), OT::List(Box::new(OT::Number)), false)],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "noise",
      func: Function {
        params: vec![
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "none",
      func: Function {
        params: vec![PI(cb("values"), OT::List(Box::new(OT::Boolean)), false)],
        return_type: OT::Boolean,
      },
    },
    StdLibFn {
      name: "oscillator",
      func: Function {
        params: vec![
          PI(
            cb("waveform"),
            OT::Function(Arc::new(Function {
              params: vec![PI(cb("phase"), OT::Number, false)],
              return_type: OT::Number,
            })),
            false,
          ),
          PI(cb("frequency"), OT::Number, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "random",
      func: Function {
        params: vec![
          PI(cb("type"), OT::RandomArg, true),
          PI(cb("length"), OT::Number, false),
          PI(cb("to"), OT::Number, false),
          PI(cb("from"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "repeat",
      func: Function {
        params: vec![PI(cb("repeats"), OT::Number, true), PI(cb("value"), OT::Number, false)],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "reverb",
      func: Function {
        params: vec![PI(cb("length"), OT::Number, false), PI(cb("mix"), OT::Number, true)],
        return_type: OT::AudioEffect,
      },
    },
    StdLibFn {
      name: "round",
      func: Function {
        params: vec![
          PI(cb("direction"), OT::RoundArg, true),
          PI(cb("step"), OT::Number, true),
          PI(cb("value"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sample",
      func: Function {
        params: vec![
          PI(cb("file"), OT::String, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "saw",
      func: Function {
        params: vec![
          PI(cb("frequency"), OT::Number, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sequence",
      func: Function {
        params: vec![
          PI(cb("order"), OT::SequenceArg, true),
          PI(cb("values"), OT::List(Box::new(OT::Number)), false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sine",
      func: Function {
        params: vec![
          PI(cb("frequency"), OT::Number, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "square",
      func: Function {
        params: vec![
          PI(cb("frequency"), OT::Number, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "sweep",
      func: Function {
        params: vec![
          PI(cb("length"), OT::Number, false),
          PI(cb("to"), OT::Number, false),
          PI(cb("from"), OT::Number, false),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn { name: "time", func: Function { params: vec![], return_type: OT::Number } },
    StdLibFn {
      name: "triangle",
      func: Function {
        params: vec![
          PI(cb("frequency"), OT::Number, false),
          PI(cb("effects"), OT::List(Box::new(OT::AudioEffect)), true),
          PI(cb("pan"), OT::Number, true),
          PI(cb("volume"), OT::Number, true),
        ],
        return_type: OT::Number,
      },
    },
    StdLibFn {
      name: "trigger",
      func: Function {
        params: vec![PI(cb("value"), OT::Number, false), PI(cb("condition"), OT::Boolean, false)],
        return_type: OT::Number,
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
  Constant { name: "ping-pong", value: CV::SequenceArg },
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
      let start = match (pitch.clone(), acci.clone()) {
        (PitchClass::C, Accidental::Flat) => 1,
        _ => 0,
      };
      (start..=9).map(move |octave| Note { pitch_class: pitch.clone(), accidental: acci.clone(), octave })
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

const fn cb<T: ?Sized + ToOwned>(value: &T) -> Cow<'_, T> {
  Cow::Borrowed(value)
}
