use std::collections::HashMap;
use std::sync::LazyLock;

#[allow(dead_code)]
pub struct BuiltIn {
  name: &'static str,
  description: &'static str,
  parameters: HashMap<&'static str, &'static str>,
}

#[must_use]
pub(super) fn lookup_builtin_arg(func_name: &str, param_name: &str) -> Option<&'static str> {
  DOCS.get(func_name)?.parameters.get(param_name).copied()
}

// TODO: FILL_INs
static DOCS: LazyLock<HashMap<&'static str, BuiltIn>> = LazyLock::new(|| {
  vec![
    ("absolute", "FILL_IN", HashMap::from([("value", "FILL_IN")])),
    ("all", "FILL_IN", HashMap::from([("values", "FILL_IN")])),
    (
      "all-pass",
      "FILL_IN",
      HashMap::from([("feedback", "FILL_IN"), ("delay", "FILL_IN"), ("mix", "FILL_IN")]),
    ),
    ("any", "FILL_IN", HashMap::from([("values", "FILL_IN")])),
    ("comb", "FILL_IN", HashMap::from([("feedback", "FILL_IN"), ("delay", "FILL_IN"), ("mix", "FILL_IN")])),
    ("delay", "FILL_IN", HashMap::from([("feedback", "FILL_IN"), ("delay", "FILL_IN"), ("mix", "FILL_IN")])),
    ("effect-group", "FILL_IN", HashMap::from([("effects", "FILL_IN"), ("mix", "FILL_IN")])),
    (
      "granulate",
      "FILL_IN",
      HashMap::from([
        ("shape", "FILL_IN"),
        ("length", "FILL_IN"),
        ("grains", "FILL_IN"),
        ("sample", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    (
      "group",
      "FILL_IN",
      HashMap::from([
        ("sources", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    ("hold", "FILL_IN", HashMap::from([("length", "FILL_IN"), ("value", "FILL_IN")])),
    (
      "if",
      "FILL_IN",
      HashMap::from([("is-false", "FILL_IN"), ("is-true", "FILL_IN"), ("condition", "FILL_IN")]),
    ),
    ("lfo", "FILL_IN", HashMap::from([("length", "FILL_IN"), ("to", "FILL_IN"), ("from", "FILL_IN")])),
    ("limit", "FILL_IN", HashMap::from([("max", "FILL_IN"), ("min", "FILL_IN"), ("value", "FILL_IN")])),
    ("low-pass", "FILL_IN", HashMap::from([("threshold", "FILL_IN")])),
    ("max", "FILL_IN", HashMap::from([("values", "FILL_IN")])),
    ("min", "FILL_IN", HashMap::from([("values", "FILL_IN")])),
    ("noise", "FILL_IN", HashMap::from([("effects", "FILL_IN"), ("pan", "FILL_IN"), ("volume", "FILL_IN")])),
    ("none", "FILL_IN", HashMap::from([("values", "FILL_IN")])),
    (
      "oscillator",
      "FILL_IN",
      HashMap::from([
        ("waveform", "FILL_IN"),
        ("frequency", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    (
      "random",
      "FILL_IN",
      HashMap::from([("type", "FILL_IN"), ("length", "FILL_IN"), ("to", "FILL_IN"), ("from", "FILL_IN")]),
    ),
    ("repeat", "FILL_IN", HashMap::from([("repeats", "FILL_IN"), ("value", "FILL_IN")])),
    ("reverb", "FILL_IN", HashMap::from([("length", "FILL_IN"), ("mix", "FILL_IN")])),
    (
      "round",
      "FILL_IN",
      HashMap::from([("direction", "FILL_IN"), ("step", "FILL_IN"), ("value", "FILL_IN")]),
    ),
    (
      "sample",
      "FILL_IN",
      HashMap::from([("file", "FILL_IN"), ("effects", "FILL_IN"), ("pan", "FILL_IN"), ("volume", "FILL_IN")]),
    ),
    (
      "saw",
      "FILL_IN",
      HashMap::from([
        ("frequency", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    ("sequence", "FILL_IN", HashMap::from([("order", "FILL_IN"), ("values", "FILL_IN")])),
    (
      "sine",
      "FILL_IN",
      HashMap::from([
        ("frequency", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    (
      "square",
      "FILL_IN",
      HashMap::from([
        ("frequency", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    ("sweep", "FILL_IN", HashMap::from([("length", "FILL_IN"), ("to", "FILL_IN"), ("from", "FILL_IN")])),
    ("time", "FILL_IN", HashMap::from([])),
    (
      "triangle",
      "FILL_IN",
      HashMap::from([
        ("frequency", "FILL_IN"),
        ("effects", "FILL_IN"),
        ("pan", "FILL_IN"),
        ("volume", "FILL_IN"),
      ]),
    ),
    ("trigger", "FILL_IN", HashMap::from([("value", "FILL_IN"), ("condition", "FILL_IN")])),
  ]
  .into_iter()
  .map(|(name, description, parameters)| (name, BuiltIn { name, description, parameters }))
  .collect()
});
