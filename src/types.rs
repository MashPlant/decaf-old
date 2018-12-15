use super::ast::{ClassDef, MethodDef};
use super::print::*;
use super::errors::*;
use super::loc::*;
use super::symbol::Symbol;
use super::util::*;

use std::default::Default as D;
use std::fmt;

// the struct Type in ast.rs is syntactic type(so it have field `loc`)
#[derive(Debug, Clone)]
pub enum SemanticType {
  Error,
  Var,
  Null,
  Int,
  Bool,
  String,
  Void,
  // only a name, generated while parsing, whether it can become an object depends on type check process
  Named(&'static str),
  // a class object
  Object(*const ClassDef),
  // a class, e.g., the type of `Main` in Main.f()
  Class(*const ClassDef),
  // type [][]...
  Array(Box<SemanticType>),
  // refer to a method, only possible in semantic analysis
  Method(*const MethodDef),
}

pub const ERROR: SemanticType = SemanticType::Error;
pub const VAR: SemanticType = SemanticType::Var;
pub const NULL: SemanticType = SemanticType::Null;
pub const INT: SemanticType = SemanticType::Int;
pub const BOOL: SemanticType = SemanticType::Bool;
pub const VOID: SemanticType = SemanticType::Void;
pub const STRING: SemanticType = SemanticType::String;

impl D for SemanticType {
  fn default() -> Self {
    ERROR
  }
}

impl fmt::Display for SemanticType {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use self::SemanticType::*;
    match self {
      Error => write!(f, "error"),
      Var => write!(f, "unknown"),
      Null => write!(f, "null"),
      Int => write!(f, "int"),
      Bool => write!(f, "bool"),
      String => write!(f, "string"),
      Void => write!(f, "void"),
      Named(name) => write!(f, "class : {}", name),
      Object(class) => write!(f, "class : {}", class.get().name),
      Class(class) => write!(f, "class : {}", class.get().name),
      Array(elem) => write!(f, "{}[]", elem),
      Method(method) => {
        let method = method.get();
        for parameter in &method.param {
          write!(f, "{}->", parameter.type_.sem)?;
        }
        write!(f, "{}", method.ret_t.sem)
      }
    }
  }
}

impl SemanticType {
  pub fn assignable_to(&self, rhs: &SemanticType) -> bool {
    use self::SemanticType::*;
    match (self, rhs) {
      (Error, _) => true,
      (_, Error) => true,
      (Int, Int) => true,
      (Bool, Bool) => true,
      (String, String) => true,
      (Void, Void) => true,
      (SemanticType::Object(class1), SemanticType::Object(class2)) => class1.get().extends(*class2),
      (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
      (SemanticType::Null, SemanticType::Object(_)) => true,
      _ => false,
    }
  }

  pub fn is_class(&self) -> bool {
    if let SemanticType::Class(_) = self { true } else { false }
  }

  pub fn is_object(&self) -> bool {
    if let SemanticType::Object(_) = self { true } else { false }
  }

  pub fn is_method(&self) -> bool {
    if let SemanticType::Method(_) = self { true } else { false }
  }

  pub fn is_array(&self) -> bool {
    if let SemanticType::Array(_) = self { true } else { false }
  }

  pub fn get_class(&self) -> &ClassDef {
    match self {
      SemanticType::Object(class) => class.get(),
      SemanticType::Class(class) => class.get(),
      _ => panic!("call get_class on non-class & non-object type"),
    }
  }

  pub fn error_or(&self, require: &SemanticType) -> bool {
    self == &ERROR || self == require
  }

  pub fn error_or_array(&self) -> bool {
    match self {
      SemanticType::Error | SemanticType::Array(_) => true,
      _ => false,
    }
  }
}

impl ASTData for SemanticType {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    use self::SemanticType::*;
    match self {
      Var => { printer.print("var"); }
      Int => { printer.print("inttype"); }
      Bool => { printer.print("booltype"); }
      String => { printer.print("stringtype"); }
      Void => { printer.print("voidtype"); }
      Named(name) => {
        printer.print("classtype");
        printer.print(name);
      }
      Array(elem) => {
        printer.print("arrtype");
        elem.print_ast(printer);
      }
      _ => unreachable!()
    };
  }
}

impl PartialEq for SemanticType {
  fn eq(&self, other: &SemanticType) -> bool {
    // in correct usage, SemanticType::Null won't be compared here
    use self::SemanticType::*;
    match (self, other) {
      (Var, Var) => true,
      (Error, Error) => true,
      (Int, Int) => true,
      (Bool, Bool) => true,
      (String, String) => true,
      (Void, Void) => true,
      (Object(class1), Object(class2)) => class1 == class2,
      (Array(elem1), Array(elem2)) => elem1 == elem2,
      _ => false,
    }
  }
}

impl Eq for SemanticType {}

pub trait SemanticTypeVisitor {
  fn push_error(&mut self, error: Error);

  fn lookup_class(&self, name: &'static str) -> Option<Symbol>;

  fn semantic_type(&mut self, type_: &mut SemanticType, loc: Loc) {
    let type_ptr = type_ as *mut SemanticType;
    if match type_ { // work around with borrow check
      SemanticType::Named(name) =>
        if let Some(class_symbol) = self.lookup_class(name) {
          *type_ptr.get() = SemanticType::Object(class_symbol.as_class());
          false
        } else {
          self.push_error(Error::new(loc, NoSuchClass { name }));
          true
        }
      SemanticType::Array(elem) => {
        self.semantic_type(elem, loc);
        if elem.as_ref() == &ERROR {
          true
        } else if elem.as_ref() == &VOID {
          self.push_error(Error::new(loc, VoidArrayElement {}));
          true
        } else { false }
      }
      _ => false,
    } { *type_ = ERROR; }
  }
}
