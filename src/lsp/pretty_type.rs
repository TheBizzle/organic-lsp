use crate::analyzer::function::{Function, ParamInfo};
use crate::analyzer::organic_type::OrganicType as OT;

pub(super) fn pretty_type(typ: &OT) -> String {
  match typ {
    OT::AudioEffect => "effect".to_string(),
    OT::Boolean => "true/false".to_string(),
    OT::Function(func) => pretty_func(func.as_ref()),
    OT::Generic(name) => format!("<{name}>"),
    OT::List(subtype) => format!("[{}]", pretty_type(subtype)),
    OT::Number => "number".to_string(),
    OT::RandomArg => "random-type".to_string(),
    OT::RoundArg => "round-direction".to_string(),
    OT::SequenceArg => "sequence-order".to_string(),
    OT::String => "text".to_string(),
    OT::Unknown => "???".to_string(),
  }
}

fn pretty_func(func: &Function) -> String {
  let Function { params, return_type } = func;

  let params_str = format!(
    "({})",
    params
      .iter()
      .map(|ParamInfo(name, typ, has_def)| {
        let type_str = pretty_type(typ);
        let pairing = format!("{name}: {type_str}");
        if *has_def {
          format!("{{{pairing}}}")
        } else {
          pairing
        }
      })
      .collect::<Vec<_>>()
      .join(", ")
  );

  let return_str = pretty_type(return_type);
  format!("({params_str} => {return_str})")
}
