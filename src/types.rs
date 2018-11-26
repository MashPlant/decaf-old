use super::ast::{ClassDef, MethodDef};
use super::print::*;
use super::errors::*;
use super::loc::*;
use super::symbol::Symbol;

use std::default::Default as D;
use std::fmt;

// the struct SemanticType in ast.rs is syntactic type(so it have field `loc`)
#[derive(Debug)]
pub enum SemanticType {
  Error,
  Var,
  Null,
  // int, string, bool, void
  Basic(&'static str),
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

impl Clone for SemanticType {
  fn clone(&self) -> Self {
    match &self {
      SemanticType::Error => SemanticType::Error,
      SemanticType::Var => SemanticType::Var,
      SemanticType::Null => SemanticType::Null,
      SemanticType::Basic(name) => SemanticType::Basic(name),
      SemanticType::Named(name) => SemanticType::Named(name),
      SemanticType::Object(class) => SemanticType::Object(*class),
      SemanticType::Class(class) => SemanticType::Class(*class),
      SemanticType::Array(elem) => SemanticType::Array(elem.clone()),
      SemanticType::Method(method) => SemanticType::Method(*method),
    }
  }
}

pub const ERROR: SemanticType = SemanticType::Error;
pub const VAR: SemanticType = SemanticType::Var;
pub const NULL: SemanticType = SemanticType::Null;
pub const INT: SemanticType = SemanticType::Basic("int");
pub const BOOL: SemanticType = SemanticType::Basic("bool");
pub const VOID: SemanticType = SemanticType::Basic("void");
pub const STRING: SemanticType = SemanticType::Basic("string");

impl D for SemanticType {
  fn default() -> Self {
    ERROR
  }
}

impl fmt::Display for SemanticType {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      SemanticType::Error => write!(f, "error"),
      SemanticType::Var => write!(f, "var"),
      SemanticType::Null => write!(f, "null"),
      SemanticType::Basic(name) => write!(f, "{}", name),
      SemanticType::Named(name) => write!(f, "class : {}", name),
      SemanticType::Object(class) => write!(f, "class : {}", unsafe { &**class }.name),
      SemanticType::Class(class) => write!(f, "class : {}", unsafe { &**class }.name),
      SemanticType::Array(elem) => write!(f, "{}[]", elem),
      SemanticType::Method(method) => {
        let method = unsafe { &**method };
        for parameter in &method.param {
          write!(f, "{}->", parameter.type_.sem);
        }
        write!(f, "{}", method.ret_t.sem)
      }
    }
  }
}

impl SemanticType {
  // a relationship of is-subclass-of
  pub fn extends(&self, rhs: &SemanticType) -> bool {
    match (self, rhs) {
      (SemanticType::Error, _) => true,
      (_, SemanticType::Error) => true,
      (SemanticType::Basic(name1), SemanticType::Basic(name2)) => name1 == name2,
      (SemanticType::Object(class1), SemanticType::Object(class2)) => unsafe { (&**class1).extends(*class2) },
      (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
      (SemanticType::Null, SemanticType::Object(_)) => true,
      _ => false,
    }
  }

  pub fn is_class(&self) -> bool {
    match self {
      SemanticType::Class(_) => true,
      _ => false,
    }
  }

  pub fn is_object(&self) -> bool {
    match self {
      SemanticType::Object(_) => true,
      _ => false,
    }
  }

  pub fn is_method(&self) -> bool {
    match self {
      SemanticType::Method(_) => true,
      _ => false,
    }
  }

  pub fn is_array(&self) -> bool {
    match self {
      SemanticType::Array(_) => true,
      _ => false,
    }
  }

  pub fn get_class(&self) -> &ClassDef {
    unsafe {
      match self {
        SemanticType::Object(class) => &**class,
        SemanticType::Class(class) => &**class,
        _ => panic!("call get_class on non-class & non-object type"),
      }
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

  pub fn print_ast(&self, printer: &mut IndentPrinter) {
    match self {
      SemanticType::Var => printer.print("var"),
      SemanticType::Basic(name) => printer.print(&(name.to_string() + "type")),
      SemanticType::Named(name) => {
        printer.print("classtype");
        printer.print(name);
      }
      SemanticType::Array(elem) => {
        printer.print("arrtype");
        elem.print_ast(printer);
      }
      _ => unreachable!()
    }
  }
}

impl PartialEq for SemanticType {
  fn eq(&self, other: &SemanticType) -> bool {
    // in correct usage, SemanticType::Null won't be compared here
    match (self, other) {
      (SemanticType::Var, SemanticType::Var) => true,
      (SemanticType::Error, SemanticType::Error) => true,
      (SemanticType::Basic(name1), SemanticType::Basic(name2)) => name1 == name2,
      (SemanticType::Object(class1), SemanticType::Object(class2)) => class1 == class2,
      (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
      _ => false,
    }
  }
}

impl Eq for SemanticType {}

pub trait SemanticTypeVisitor {
  fn push_error(&mut self, error: Error);

  fn lookup_class(&self, name: &'static str) -> Option<Symbol>;

  fn semantic_type(&mut self, type_: &mut SemanticType, loc: Loc) {
    unsafe {
      let type_ptr = type_ as *mut SemanticType;
      if match type_ { // work around with borrow check
        SemanticType::Named(name) =>
          if let Some(class_symbol) = self.lookup_class(name) {
            *type_ptr = SemanticType::Object(class_symbol.as_class());
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
            self.push_error(Error::new(loc, VoidArrayElement));
            true
          } else { false }
        }
        _ => false,
      } { *type_ = ERROR; }
    }
  }
}