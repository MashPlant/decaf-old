use super::loc::*;
use super::config::*;
use std::fmt;

pub trait IError {
  fn get_msg(&self) -> String;
}

pub struct Error {
  pub loc: Loc,
  pub error: Box<IError>,
}

impl Error {
  pub fn new<E: IError + 'static>(loc: Loc, error: E) -> Error {
    Error {
      loc,
      error: Box::new(error),
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.loc {
      NO_LOC => write!(f, "*** Error: {}", self.error.get_msg()),
      loc => write!(f, "*** Error at {}: {}", loc, &self.error.get_msg()),
    }
  }
}

macro_rules! make_error {
  ($self_: ident, $($name: ident => $($field: ident : $type_: ty),* => $value: expr),*) => {
    $(pub struct $name {
      $(pub $field : $type_),*
    }
    impl IError for $name {
      fn get_msg(&$self_) -> String {
        $value
      }
    })*
  };
}

make_error!(self,
  UnterminatedStr => string: String => format!("unterminated string constant {}", self.string),
  NewlineInStr => string: String => format!("illegal newline in string constant {}", self.string),
  IntTooLarge => string: String => format!("integer literal {} is too large", self.string),
  UnrecognizedChar => ch: char => format!("unrecognized character '{}'", self.ch),
  ConflictDeclaration => earlier: Loc, name: &'static str => format!("declaration of '{}' here conflicts with earlier declaration at {}", self.name, self.earlier),
  NoSuchClass => name: &'static str => format!("class '{}' not found", self.name),
  CyclicInheritance => => "illegal class inheritance (should be a cyclic)".to_owned(),
  SealedInheritance => => "illegal class inheritance from sealed class".to_owned(),
  NoMainClass => => format!("no legal Main class named '{}' was found", MAIN_CLASS),
  VoidArrayElement => => "array element type must be non-void known type".to_owned(),
  VoidVar => name: &'static str => format!("cannot declare identifier '{}' as void type", self.name),
  OverrideVar => name: &'static str => format!("overriding variable is not allowed for var '{}'", self.name),
  BadOverride => method: &'static str, parent: &'static str => format!("overriding method '{}' doesn't match the type signature in class '{}'", self.method, self.parent),
  IncompatibleUnary => op: &'static str, r_t: String => format!("incompatible operand: {} {}", self.op, self.r_t),
  TestNotBool => => "test expression must have bool type".to_owned(),
  IncompatibleBinary => l_t: String, op: &'static str, r_t: String => format!("incompatible operands: {} {} {}", self.l_t, self.op, self.r_t),
  BreakOutOfLoop => => "'break' is only allowed inside a loop".to_owned(),
  UndeclaredVar => name: &'static str => format!("undeclared variable '{}'", self.name),
  RefInStatic => field: &'static str, method: &'static str => format!("can not reference a non-static field '{}' from static method '{}'", self.field, self.method),
  BadFieldAccess => name: &'static str, owner_t: String => format!("cannot access field '{}' from '{}'", self.name, self.owner_t),
  PrivateFieldAccess => name: &'static str, owner_t: String => format!("field '{}' of '{}' not accessible here", self.name, self.owner_t),
  NoSuchField => name: &'static str, owner_t: String => format!("field '{}' not found in '{}'", self.name, self.owner_t),
  LengthWithArgument => count: i32 => format!("function 'length' expects 0 argument(s) but {} given", self.count),
  BadLength => => "'length' can only be applied to arrays".to_owned(),
  NotMethod => name: &'static str, owner_t: String => format!("'{}' is not a method in class '{}'", self.name, self.owner_t),
  WrongArgc => name: &'static str, expect: i32, actual: i32 => format!("function '{}' expects {} argument(s) but {} given", self.name, self.expect, self.actual),
  WrongArgType => loc: i32, arg_t: String, param_t: String => format!("incompatible argument {}: {} given, {} expected", self.loc, self.arg_t, self.param_t),
  ThisInStatic => => "can not use this in static function".to_owned(),
  NotObject => type_: String => format!("{} is not a class type", self.type_),
  BadPrintArg => loc: i32, type_: String => format!("incompatible argument {}: {} given, int/bool/string expected", self.loc, self.type_),
  WrongReturnType => ret_t: String, expect_t: String => format!("incompatible return: {} given, {} expected", self.ret_t, self.expect_t),
  BadNewArrayLen => => "new array length must be an integer".to_owned(),
  NotArray => => "[] can only be applied to arrays".to_owned(),
  ArrayIndexNotInt => => "array subscript must be an integer".to_owned(),
  ArrayRepeatNotInt => => "array repeats time type must be int type".to_owned(),
  BadArrayOp => => "Array Operation on non-array type".to_owned(),
  DefaultMismatch => elem_t: String, dft_t: String => format!("Array has Element type {} but default has type {}", self.elem_t, self.dft_t),
  ForeachMismatch => elem_t: String, def_t: String => format!("Array has Element type {} but Foreach wants type {}", self.elem_t, self.def_t),
  ConcatMismatch => l_t: String, r_t: String => format!("concat {} with {}", self.l_t, self.r_t),
  SCopyNotClass => which: &'static str, type_: String => format!("incompatible argument {}: {} given, class expected", self.which, self.type_),
  SCopyMismatch => dst_t: String, src_t: String => format!("incompatible dst type: {} and src type: {}", self.dst_t, self.src_t),
  NotLValue => op: &'static str => format!("operator {} can only be applied to lvalue", self.op)
);