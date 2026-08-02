use crate::core::address::NamedVarAddress;

use crate::analyzer::function::{Function, ParamInfo};

use crate::lsp::builtins::lookup_builtin_arg;
use crate::lsp::pretty_type::pretty_type;

#[must_use]
pub(super) fn describe_named_arg(arg_name: &str, func_addr: &NamedVarAddress, func: &Function) -> String {
  let param_opt = func.params.iter().find(|ParamInfo(name, _, _)| name == arg_name);
  if let Some(ParamInfo(_, typ, has_default)) = param_opt {
    let optionality = has_default.then_some(" (optional)").unwrap_or("");
    let main_str = format!(
      "```scala
{arg_name}: {}{optionality}
```",
      pretty_type(typ)
    );

    if func_addr.scope_addr.n == 0
      && let Some(info) = lookup_builtin_arg(func_addr.name.as_str(), arg_name)
    {
      format!(
        "{main_str}
---
{info}"
      )
    } else {
      main_str
    }
  } else {
    "Unknown parameter".to_string()
  }
}
