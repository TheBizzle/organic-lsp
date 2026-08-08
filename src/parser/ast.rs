use ordered_float::NotNan;

use crate::lexer::token::Token;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
  pub name: String,
  pub token: Token,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
  Call {
    call: FuncCall,
    token: Token,
    start: Token,
    end: Token,
  },
  Function {
    value: FuncLiteral,
    token: Token,
    start: Token,
    end: Token,
  },
  Grouping {
    value: Box<Self>,
    token: Token,
    start: Token,
    end: Token,
  },
  List {
    values: Vec<Self>,
    token: Token,
    start: Token,
    end: Token,
  },
  LValue {
    name: Symbol,
    token: Token,
    start: Token,
    end: Token,
  },
  Negated {
    value: Box<Self>,
    token: Token,
    start: Token,
    end: Token,
  },
  Number {
    value: NotNan<f64>,
    token: Token,
    start: Token,
    end: Token,
  },
  Op {
    left: Box<Self>,
    operator: Operator,
    right: Box<Self>,
    token: Token,
    start: Token,
    end: Token,
  },
  String {
    value: String,
    token: Token,
    start: Token,
    end: Token,
  },
}

#[derive(Debug, PartialEq)]
pub struct Arg {
  pub name: Symbol,
  pub value: Expr,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operator {
  Plus,
  Minus,
  Times,
  Divide,
  Equals,
  LessThan,
  LessOrEquals,
  GreaterThan,
  GreaterOrEquals,
}

pub struct Module {
  pub includes: Vec<Include>,
  pub statements: Vec<Statement>,
}

pub struct Include {
  pub path: String,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Statement {
  FunctionCall(Box<FuncCall>),
  VariableDecl(VarDecl),
}

#[derive(Debug, PartialEq)]
pub struct FuncCall {
  pub func: Symbol,
  pub args: Vec<Arg>,
}

#[derive(Debug, PartialEq)]
pub struct FuncLiteral {
  pub name: Symbol,
  pub formals: Vec<Formal>,
  pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub struct Formal {
  pub name: Symbol,
  pub default: Expr,
}

#[derive(Debug, PartialEq)]
pub struct VarDecl {
  pub name: Symbol,
  pub init: Expr,
}

impl Expr {
  #[must_use]
  pub fn get_token(&self) -> Token {
    match self {
      Self::Call { token, .. }
      | Self::Function { token, .. }
      | Self::Grouping { token, .. }
      | Self::List { token, .. }
      | Self::LValue { token, .. }
      | Self::Negated { token, .. }
      | Self::Number { token, .. }
      | Self::Op { token, .. }
      | Self::String { token, .. } => token.clone(),
    }
  }

  #[must_use]
  pub fn get_start(&self) -> Token {
    match self {
      Self::Call { start, .. }
      | Self::Function { start, .. }
      | Self::Grouping { start, .. }
      | Self::List { start, .. }
      | Self::LValue { start, .. }
      | Self::Negated { start, .. }
      | Self::Number { start, .. }
      | Self::Op { start, .. }
      | Self::String { start, .. } => start.clone(),
    }
  }

  #[must_use]
  pub fn get_end(&self) -> Token {
    match self {
      Self::Call { end, .. }
      | Self::Function { end, .. }
      | Self::Grouping { end, .. }
      | Self::List { end, .. }
      | Self::LValue { end, .. }
      | Self::Negated { end, .. }
      | Self::Number { end, .. }
      | Self::Op { end, .. }
      | Self::String { end, .. } => end.clone(),
    }
  }
}
