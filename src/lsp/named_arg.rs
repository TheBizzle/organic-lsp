use crate::core::address::NamedVarAddress;

use crate::analyzer::function::{Function, ParamInfo};

use crate::lsp::pretty_type::pretty_type;

#[must_use]
pub(super) fn describe_named_arg(arg_name: &str, _func_addr: &NamedVarAddress, func: &Function) -> String {
  let param_opt = func.params.iter().find(|ParamInfo(name, _, _)| name == arg_name);
  if let Some(ParamInfo(_, typ, has_default)) = param_opt {
    let optionality = has_default.then_some(" (optional)").unwrap_or("");
    format!(
      "```scala
{arg_name}: {}{optionality}
```",
      pretty_type(typ)
    )
  } else {
    "Unknown parameter".to_string()
  }
}
