use std::fmt::Write;

use crate::parser::ast::Expr::Function;
use crate::parser::ast::Statement::{self, FunctionCall, VariableDecl};
use crate::parser::ast::{
  Arg, Expr, Formal, FuncCall, FuncLiteral, Include, Module, Operator, Symbol, VarDecl,
};

pub(super) fn miniformat(ast: &Module, avail_width: u32) -> String {
  let Module { includes, statements } = ast;

  let mut output = String::new();

  for include in includes {
    output.push_str(&render_include(include));
    output.push('\n');
  }

  if !includes.is_empty() {
    output.push('\n');
  }

  output.push_str(&render_statements(statements, avail_width));

  output
}

fn render_include(include: &Include) -> String {
  format!("include({})", render_string(&include.path))
}

fn render_statements(statements: &[Statement], avail_width: u32) -> String {
  let mut buffer = String::new();
  let mut gave_last_one_space = false;

  for (i, statement) in statements.iter().enumerate() {
    let rendered = render_statement(statement, avail_width);
    let was_multiline = rendered.lines().count() > 1;

    if was_multiline && !gave_last_one_space && i > 0 {
      buffer.push('\n');
    }
    buffer.push_str(&rendered);

    if statements.get(i + 1).is_some() {
      buffer.push('\n');
      if was_multiline {
        buffer.push('\n');
        gave_last_one_space = true;
      } else {
        gave_last_one_space = false;
      }
    }
  }

  buffer
}

fn render_statement(statement: &Statement, avail_width: u32) -> String {
  match statement {
    FunctionCall(call) => render_func_call(call.as_ref(), avail_width),
    VariableDecl(decl) => render_var_decl(decl, avail_width),
  }
}

fn render_func_call(call: &FuncCall, avail_width: u32) -> String {
  let FuncCall { func, args } = call;
  let name = render_symbol(func);
  lay_out_variadic(&format!("{name}("), ",", ")", args, render_arg, avail_width)
}

fn render_arg(arg: &Arg, avail_width: u32) -> String {
  let Arg { name, value } = arg;

  let arg_name = render_symbol(name);
  let prefix = format!("{arg_name}: ");
  let arg_value = render_expr(value, avail_width.saturating_sub(prefix.len().as_u32_risky()));

  format!("{prefix}{arg_value}")
}

fn render_var_decl(decl: &VarDecl, avail_width: u32) -> String {
  let VarDecl { name, init } = decl;
  if let Function { .. } = init {
    render_expr(init, avail_width)
  } else {
    let var_name = render_symbol(name);
    let prefix = format!("{var_name} = ");
    let var_value = render_expr(init, avail_width.saturating_sub(prefix.len().as_u32_risky()));
    format!("{prefix}{var_value}")
  }
}

fn render_expr(expr: &Expr, avail_width: u32) -> String {
  match expr {
    Expr::Call { call, .. } => render_func_call(call, avail_width),
    Expr::Function { value, .. } => render_func_def(value, avail_width),
    Expr::Grouping { value, .. } => format!("({})", render_expr(value, avail_width.saturating_sub(2))),
    Expr::List { values, .. } => lay_out_variadic("[", ",", "]", values, render_expr, avail_width),
    Expr::LValue { name, .. } => name.name.clone(),
    Expr::Negated { value, .. } => format!("-{}", render_expr(value.as_ref(), avail_width.saturating_sub(1))),
    Expr::Number { value, .. } => value.to_string(),
    Expr::Op { left, operator, right, .. } => {
      format!(
        "{} {} {}",
        render_expr(left, avail_width),
        render_operator(operator),
        render_expr(right, avail_width)
      )
    },
    Expr::String { value, .. } => render_string(value),
  }
}

fn render_operator(operator: &Operator) -> String {
  match operator {
    Operator::Plus => "+",
    Operator::Minus => "-",
    Operator::Times => "*",
    Operator::Divide => "/",
    Operator::Equals => "==",
    Operator::LessThan => "<",
    Operator::LessOrEquals => "<=",
    Operator::GreaterThan => ">",
    Operator::GreaterOrEquals => ">=",
  }
  .to_string()
}

fn render_func_def(func: &FuncLiteral, avail_width: u32) -> String {
  let FuncLiteral { name, formals, body } = func;

  let func_body = render_statements(body, avail_width);
  let indented_body = func_body.lines().map(|line| format!("    {line}")).collect::<Vec<_>>().join("\n");

  let func_name = render_symbol(name);

  let func_top =
    lay_out_variadic(&format!("{func_name}("), ",", ") = {", formals, render_formal, avail_width);

  format!(
    "{func_top}
{indented_body}
}}"
  )
}

fn render_formal(formal: &Formal, avail_width: u32) -> String {
  let Formal { name, default } = formal;

  let formal_name = render_symbol(name);
  let prefix = format!("{formal_name}: ");
  let formal_value = render_expr(default, avail_width);

  format!("{prefix}{formal_value}")
}

fn lay_out_variadic<T, F: Fn(&T, u32) -> String>(
  prefix: &str, sep: &str, suffix: &str, xs: &[T], renderer: F, avail_width: u32,
) -> String {
  let mut one_liner = prefix.to_string();

  for (i, x) in xs.iter().enumerate() {
    let overhead = ((one_liner.len() + sep.len() + suffix.len()).as_u32_risky()) + 1;
    let rendered_x = renderer(x, avail_width.saturating_sub(overhead));
    let _ = write!(one_liner, "{rendered_x}");

    if xs.get(i + 1).is_some() {
      let _ = write!(one_liner, "{sep} ");
    }

    if one_liner.len().as_u32_risky() > (avail_width.saturating_sub(suffix.len().as_u32_risky())) {
      break;
    }
  }
  let _ = write!(one_liner, "{suffix}");

  if one_liner.len().as_u32_risky() <= avail_width {
    one_liner
  } else {
    layout_variadic_one_per_line(prefix, sep, suffix, xs, renderer, avail_width)
  }
}

fn layout_variadic_one_per_line<T, F: Fn(&T, u32) -> String>(
  prefix: &str, sep: &str, suffix: &str, xs: &[T], renderer: F, avail_width: u32,
) -> String {
  let mut multiline = prefix.to_string();

  for (i, x) in xs.iter().enumerate() {
    let rendered_x = renderer(x, avail_width);
    let indented = rendered_x.lines().map(|line| format!("    {line}")).collect::<Vec<_>>().join("\n");
    let _ = write!(multiline, "\n{indented}");

    if xs.get(i + 1).is_some() {
      let _ = write!(multiline, "{sep}");
    }
  }
  let _ = write!(multiline, "\n{suffix}");

  multiline
}

fn render_string(str: &str) -> String {
  let escaped = str.replace('\\', "\\\\").replace('"', "\\\"");
  format!("\"{escaped}\"")
}

fn render_symbol(symbol: &Symbol) -> String {
  symbol.name.clone()
}

trait U32Risky {
  fn as_u32_risky(&self) -> u32;
}

impl U32Risky for usize {
  #[allow(clippy::cast_possible_truncation)]
  fn as_u32_risky(&self) -> u32 {
    *self as u32
  }
}
